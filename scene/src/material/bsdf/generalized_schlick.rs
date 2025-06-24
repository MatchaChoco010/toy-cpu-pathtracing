//! Adobe Fresnel Modelの一般化されたSchlick BSDF実装。

use math::{ShadingNormalTangent, Vector3};
use spectrum::SampledSpectrum;

use crate::material::{
    bsdf::{BsdfSample, BsdfSampleType, ScatterMode},
    common::{
        abs_cos_theta, cos_phi, cos_theta, cos2_theta, fresnel_dielectric, half_vector, reflect,
        refract, same_hemisphere, sample_uniform_disk_polar, sin_phi, tan2_theta,
    },
};

/// Adobe Fresnel Modelの一般化されたSchlick BSDFを実装する構造体。
/// 従来のSchlickモデルを拡張し、金属の斜め角での「ディップ」を制御できる。
pub struct GeneralizedSchlickBsdf {
    /// 0度での反射率（スペクトル依存）
    r0: SampledSpectrum,
    /// 90度での反射率（スペクトル依存）
    r90: SampledSpectrum,
    /// 補間指数（通常は5.0）
    exponent: f32,
    /// ティントパラメータ（1.0で白、0.0で最大ディップ）
    tint: SampledSpectrum,
    /// 散乱モード（反射のみ or 反射+透過）
    scatter_mode: ScatterMode,
    /// 屈折率（透過時に使用、ScatterMode::RTの場合のみ）
    eta: SampledSpectrum,
    /// Thin surfaceフラグ
    thin_surface: bool,
    /// X方向のroughness parameter (α_x)
    alpha_x: f32,
    /// Y方向のroughness parameter (α_y)
    alpha_y: f32,
}

impl GeneralizedSchlickBsdf {
    /// GeneralizedSchlickBsdfを作成する。
    ///
    /// # Arguments
    /// - `r0` - 0度での反射率（スペクトル依存）
    /// - `r90` - 90度での反射率（スペクトル依存）
    /// - `exponent` - 補間指数（通常は5.0）
    /// - `tint` - ティントパラメータ（1.0で白、0.0で最大ディップ）
    /// - `scatter_mode` - 散乱モード（ScatterMode::R or ScatterMode::RT）
    /// - `eta` - 屈折率（透過時に使用、ScatterMode::RTの場合のみ）
    /// - `thin_surface` - Thin surfaceフラグ
    /// - `alpha_x` - X方向のroughness parameter
    /// - `alpha_y` - Y方向のroughness parameter
    pub fn new(
        r0: SampledSpectrum,
        r90: SampledSpectrum,
        exponent: f32,
        tint: SampledSpectrum,
        scatter_mode: ScatterMode,
        eta: SampledSpectrum,
        thin_surface: bool,
        alpha_x: f32,
        alpha_y: f32,
    ) -> Self {
        Self {
            r0,
            r90,
            exponent,
            tint,
            scatter_mode,
            eta,
            thin_surface,
            alpha_x,
            alpha_y,
        }
    }

    /// 表面が事実上滑らかかどうかを判定する。
    fn effectively_smooth(&self) -> bool {
        self.alpha_x.max(self.alpha_y) < 1e-3
    }

    /// 一般化されたSchlick Fresnelモデルを計算する。
    /// Adobe Fresnel Modelの式：
    /// F(θ) ≈ r₀ + (r₉₀ - r₀)(1 - cos θ)^α - a cos θ (1 - cos θ)^6
    ///
    /// # Arguments
    /// - `cos_theta` - 入射角のコサイン値
    fn generalized_schlick_fresnel(&self, cos_theta: f32) -> SampledSpectrum {
        let cos_theta = cos_theta.clamp(0.0, 1.0);
        let one_minus_cos = 1.0 - cos_theta;

        // θ_max ≈ 82度（cos θ_max = 1/7）
        const COS_THETA_MAX: f32 = 1.0 / 7.0;
        const ONE_MINUS_COS_THETA_MAX: f32 = 1.0 - COS_THETA_MAX;

        // 基本のSchlickモデル: r₀ + (r₉₀ - r₀)(1 - cos θ)^α
        let base_fresnel = self.r0.clone()
            + (self.r90.clone() - self.r0.clone()) * one_minus_cos.powf(self.exponent);

        // パラメータaの計算：
        // a = [r₀ + (r₉₀ - r₀)(1 - cos θ_max)^α](1 - t) / [cos θ_max (1 - cos θ_max)^6]
        let fresnel_at_max = self.r0.clone()
            + (self.r90.clone() - self.r0.clone()) * ONE_MINUS_COS_THETA_MAX.powf(self.exponent);
        let a = fresnel_at_max * (SampledSpectrum::one() - self.tint.clone())
            / (COS_THETA_MAX * ONE_MINUS_COS_THETA_MAX.powi(6));

        // Lazanyi項: a cos θ (1 - cos θ)^6
        let lazanyi_term = a * cos_theta * one_minus_cos.powi(6);

        // 最終的なフレネル反射率
        base_fresnel - lazanyi_term
    }

    /// Trowbridge-Reitz分布関数 D(ωm)を計算する。
    fn microfacet_distribution(&self, wm: &Vector3<ShadingNormalTangent>) -> f32 {
        let tan2_theta = tan2_theta(wm);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wm).powi(2);
        let e = tan2_theta
            * (cos_phi(wm).powi(2) / self.alpha_x.powi(2)
                + sin_phi(wm).powi(2) / self.alpha_y.powi(2));

        1.0 / (std::f32::consts::PI * self.alpha_x * self.alpha_y * cos4_theta * (1.0 + e).powi(2))
    }

    /// Lambda関数を計算する。
    fn lambda(&self, w: &Vector3<ShadingNormalTangent>) -> f32 {
        let tan2_theta = tan2_theta(w);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let alpha2 = (cos_phi(w) * self.alpha_x).powi(2) + (sin_phi(w) * self.alpha_y).powi(2);
        ((1.0 + alpha2 * tan2_theta).sqrt() - 1.0) / 2.0
    }

    /// 単方向マスキング関数 G1(ω)を計算する。
    fn masking_g1(&self, w: &Vector3<ShadingNormalTangent>) -> f32 {
        1.0 / (1.0 + self.lambda(w))
    }

    /// 双方向マスキング・シャドウイング関数 G(ωo, ωi)を計算する。
    fn masking_shadowing_g(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    /// 可視法線分布 D_ω(ωm)を計算する。
    fn visible_normal_distribution(
        &self,
        w: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        let cos_theta_w = w.z().abs();
        if cos_theta_w == 0.0 {
            return 0.0;
        }
        self.masking_g1(w) / cos_theta_w * self.microfacet_distribution(wm) * w.dot(wm).abs()
    }

    /// 可視法線をサンプリングする。
    fn sample_visible_normal(
        &self,
        w: &Vector3<ShadingNormalTangent>,
        u: glam::Vec2,
    ) -> Vector3<ShadingNormalTangent> {
        // wを半球構成に変換
        let mut wh: Vector3<ShadingNormalTangent> =
            Vector3::new(self.alpha_x * w.x(), self.alpha_y * w.y(), w.z()).normalize();
        if wh.z() < 0.0 {
            wh = -wh;
        }

        // 可視法線サンプリング用の直交基底を見つける
        let t1 = if wh.z() < 0.99999 {
            Vector3::new(0.0, 0.0, 1.0).cross(wh).normalize()
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let t2 = wh.cross(t1);

        // 単位円盤上に均等分布点を生成
        let p = sample_uniform_disk_polar(u);

        // 半球投影を可視法線サンプリング用にワープ
        let h = (1.0 - p.x * p.x).max(0.0).sqrt();
        let lerp_factor = (1.0 + wh.z()) / 2.0;
        let p_y = h * (1.0 - lerp_factor) + p.y * lerp_factor;

        // 半球に再投影し、法線を楕円体構成に変換
        let pz = (1.0 - p.x * p.x - p_y * p_y).max(0.0).sqrt();
        let nh = t1 * p.x + t2 * p_y + wh * pz;

        Vector3::new(
            self.alpha_x * nh.x(),
            self.alpha_y * nh.y(),
            (1e-6_f32).max(nh.z()),
        )
        .normalize()
    }

    /// BSDF方向サンプリングを行う。
    /// 表面の粗さに応じて完全鏡面またはマイクロファセットサンプリングを使用。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `uv` - ランダムサンプル
    /// - `uc` - 反射/透過選択用の追加ランダム値
    pub fn sample(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
        uc: f32,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        if self.effectively_smooth() {
            // 完全鏡面反射/透過
            self.sample_perfect_specular(wo, uc)
        } else {
            // マイクロファセットサンプリング
            self.sample_microfacet(wo, uv, uc)
        }
    }

    /// 完全鏡面反射/透過サンプリング。
    fn sample_perfect_specular(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uc: f32,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();

        // フレネル反射率を計算
        let fresnel = self.generalized_schlick_fresnel(wo_cos_n.abs());

        match self.scatter_mode {
            ScatterMode::R => {
                // 反射のみ
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                let wi_cos_n = wi.z();

                if wi_cos_n == 0.0 {
                    return None;
                }

                // BSDF値: F / |cos(theta_i)|
                let f = fresnel / wi_cos_n.abs();

                Some(BsdfSample::new(f, wi, 1.0, BsdfSampleType::Specular))
            }
            ScatterMode::RT => {
                // 反射と透過
                // フレネル反射率の平均値を使用して反射/透過を決定
                let avg_fresnel = fresnel.average();
                let pr = avg_fresnel;
                let pt = 1.0 - pr;

                if uc < pr / (pr + pt) {
                    // 反射
                    let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                    let wi_cos_n = wi.z();

                    if wi_cos_n == 0.0 {
                        return None;
                    }

                    let f = fresnel * (pr / (pr + pt)) / wi_cos_n.abs();
                    Some(BsdfSample::new(
                        f,
                        wi,
                        pr / (pr + pt),
                        BsdfSampleType::Specular,
                    ))
                } else if self.thin_surface {
                    // Thin surface: 反対方向への透過
                    let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());
                    let wi_cos_n = wi.z();

                    if wi_cos_n == 0.0 {
                        return None;
                    }

                    let transmission = SampledSpectrum::one() - fresnel;
                    let f = transmission * (pt / (pr + pt)) / wi_cos_n.abs();
                    Some(BsdfSample::new(
                        f,
                        wi,
                        pt / (pr + pt),
                        BsdfSampleType::Specular,
                    ))
                } else {
                    // 通常の誘電体：Snellの法則による屈折
                    let eta_val = self.eta.value(0);
                    let eta = eta_val;
                    let n = Vector3::new(0.0, 0.0, 1.0);

                    if let Some(wt) = refract(wo, &n, eta) {
                        let wt_cos_n = wt.z();
                        if wt_cos_n == 0.0 {
                            return None;
                        }

                        let transmission = SampledSpectrum::one() - fresnel;
                        let f = transmission * (pt / (pr + pt)) / (eta * eta * wt_cos_n.abs());
                        Some(BsdfSample::new(
                            f,
                            wt,
                            pt / (pr + pt),
                            BsdfSampleType::Specular,
                        ))
                    } else {
                        None
                    }
                }
            }
        }
    }

    /// マイクロファセットサンプリング。
    fn sample_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
        uc: f32,
    ) -> Option<BsdfSample> {
        // 可視法線をサンプリング
        let wm = self.sample_visible_normal(wo, uv);

        // フレネル反射率を計算
        let wo_dot_wm = wo.dot(wm);
        let fresnel = self.generalized_schlick_fresnel(wo_dot_wm.abs());

        match self.scatter_mode {
            ScatterMode::R => {
                // 反射のみ
                self.sample_microfacet_reflection(wo, &wm, fresnel)
            }
            ScatterMode::RT => {
                // 反射と透過
                let avg_fresnel = fresnel.average();
                let pr = avg_fresnel;
                let pt = 1.0 - pr;

                if uc < pr / (pr + pt) {
                    // 反射
                    self.sample_microfacet_reflection(wo, &wm, fresnel * (pr / (pr + pt)))
                } else {
                    // 透過
                    self.sample_microfacet_transmission(
                        wo,
                        &wm,
                        (SampledSpectrum::one() - fresnel) * (pt / (pr + pt)),
                        pt / (pr + pt),
                    )
                }
            }
        }
    }

    /// マイクロファセット反射サンプリング。
    fn sample_microfacet_reflection(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        fresnel: SampledSpectrum,
    ) -> Option<BsdfSample> {
        // 鏡面反射方向を計算
        let wi = reflect(wo, wm);

        // 同じ半球にあるかチェック
        if !same_hemisphere(wo, &wi) {
            return None;
        }

        // PDF計算
        let cos_theta_dot = wo.dot(wm).abs();
        if cos_theta_dot < 1e-6 {
            return None;
        }
        let pdf = self.visible_normal_distribution(wo, wm) / (4.0 * cos_theta_dot);

        // BRDF値計算
        let d = self.microfacet_distribution(wm);
        let g = self.masking_shadowing_g(wo, &wi);
        let cos_theta_i = wi.z().abs();
        let cos_theta_o = wo.z().abs();

        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return None;
        }

        let f_value = fresnel * d * g / (4.0 * cos_theta_i * cos_theta_o);

        Some(BsdfSample::new(f_value, wi, pdf, BsdfSampleType::Glossy))
    }

    /// マイクロファセット透過サンプリング。
    fn sample_microfacet_transmission(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        transmission: SampledSpectrum,
        prob: f32,
    ) -> Option<BsdfSample> {
        if self.thin_surface {
            // Thin surface: 反対方向への透過
            let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());
            let wi_cos_n = wi.z();

            if wi_cos_n == 0.0 {
                return None;
            }

            // PDF計算（簡単な透過の場合）
            let pdf = prob;

            // BTDF値
            let f_value = transmission / wi_cos_n.abs();

            Some(BsdfSample::new(f_value, wi, pdf, BsdfSampleType::Glossy))
        } else {
            // 通常の誘電体：Snellの法則による屈折
            let eta_val = self.eta.value(0);
            let eta = eta_val; // 空気から材料への屈折

            let wm_refract = wm;
            let wi = refract(wo, wm_refract, eta)?;

            if same_hemisphere(wo, &wi) || wi.z().abs() == 0.0 {
                return None;
            }

            // Generalized half vectorを使用してPDF計算
            let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
            let dwm_dwi = wi.dot(wm).abs() / denom;

            let pdf = self.visible_normal_distribution(wo, wm) * dwm_dwi * prob;

            // マイクロファセットBTDF値計算
            let d = self.microfacet_distribution(wm);
            let g = self.masking_shadowing_g(wo, &wi);
            let cos_theta_i = abs_cos_theta(&wi);
            let cos_theta_o = abs_cos_theta(wo);

            let ft = transmission * d * g * wi.dot(wm).abs() * wo.dot(wm).abs()
                / (denom * cos_theta_i * cos_theta_o * eta * eta);

            Some(BsdfSample::new(ft, wi, pdf, BsdfSampleType::Glossy))
        }
    }

    /// BSDF値を評価する。
    /// 完全鏡面の場合は0、マイクロファセットの場合は実際のBSDF値を返す。
    pub fn evaluate(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        if self.effectively_smooth() {
            // 完全鏡面反射の場合、evaluate()は常に0を返す（デルタ関数のため）
            SampledSpectrum::zero()
        } else {
            // マイクロファセットの場合、実際のBSDF値を評価
            self.evaluate_microfacet(wo, wi)
        }
    }

    /// マイクロファセットBSDFを評価する。
    fn evaluate_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let cos_theta_o = wo.z().abs();
        let cos_theta_i = wi.z().abs();

        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return SampledSpectrum::zero();
        }

        // 反射か透過かを判定
        let is_reflection = same_hemisphere(wo, wi);
        let is_transmission = !is_reflection;

        match self.scatter_mode {
            ScatterMode::R => {
                if is_transmission {
                    return SampledSpectrum::zero();
                }
                // 反射のみ評価
                self.evaluate_reflection(wo, wi)
            }
            ScatterMode::RT => {
                if is_reflection {
                    // 反射評価
                    self.evaluate_reflection(wo, wi)
                } else {
                    // 透過評価
                    self.evaluate_transmission(wo, wi)
                }
            }
        }
    }

    /// 反射BRDFを評価する。
    fn evaluate_reflection(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        // ハーフベクトルを計算
        let wm = match half_vector(wo, wi) {
            Some(wm) => wm,
            None => return SampledSpectrum::zero(),
        };

        // フレネル反射率を計算
        let fresnel = self.generalized_schlick_fresnel(wo.dot(wm).abs());

        // マイクロファセットBRDF: D(ωm) * F(ωo·ωm) * G(ωo, ωi) / (4 * cos θi * cos θo)
        let d = self.microfacet_distribution(&wm);
        let g = self.masking_shadowing_g(wo, wi);
        let cos_theta_i = wi.z().abs();
        let cos_theta_o = wo.z().abs();

        fresnel * d * g / (4.0 * cos_theta_i * cos_theta_o)
    }

    /// 透過BTDFを評価する。
    fn evaluate_transmission(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        if self.thin_surface {
            // Thin surface: 反対方向の透過のみをサポート
            if wo.x() != -wi.x() || wo.y() != -wi.y() || wo.z() != -wi.z() {
                return SampledSpectrum::zero();
            }

            // フレネル透過率を計算
            let fresnel = self.generalized_schlick_fresnel(wo.z().abs());
            let transmission = SampledSpectrum::one() - fresnel;

            // 簡単なBTDF値
            transmission / wi.z().abs()
        } else {
            // 通常の誘電体：適切な屈折BTDF
            let eta_val = self.eta.value(0);
            let eta = eta_val;

            // Generalized half vectorを計算
            let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
                Some(wm) => wm,
                None => return SampledSpectrum::zero(),
            };

            // フレネル透過率を計算
            let fresnel_dielectric_val = fresnel_dielectric(wo.dot(wm).abs(), eta);
            let transmission = 1.0 - fresnel_dielectric_val;

            // マイクロファセットBTDF
            let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
            let d = self.microfacet_distribution(&wm);
            let g = self.masking_shadowing_g(wo, wi);

            let numerator = d * transmission * g * wi.dot(wm).abs() * wo.dot(wm).abs();
            let denominator = denom * abs_cos_theta(wi) * abs_cos_theta(wo);

            let ft = numerator / denominator / (eta * eta);

            SampledSpectrum::constant(ft)
        }
    }

    /// Generalized half vectorを計算する。
    fn compute_generalized_half_vector(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        eta: f32,
    ) -> Option<Vector3<ShadingNormalTangent>> {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        let etap = if !reflect {
            if cos_theta_o > 0.0 { eta } else { 1.0 / eta }
        } else {
            1.0
        };

        // Generalized half vector計算
        let wm = *wi * etap + *wo;

        if cos_theta_i == 0.0 || cos_theta_o == 0.0 || wm.length_squared() == 0.0 {
            return None;
        }

        let wm = wm.normalize();

        // 適切な向きに調整
        let wm = if wm.z() < 0.0 { -wm } else { wm };

        // backfacing microfacetをチェック
        if wm.dot(wi) * cos_theta_i < 0.0 || wm.dot(wo) * cos_theta_o < 0.0 {
            return None;
        }

        Some(wm)
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は0、マイクロファセットの場合は実際のPDF値を返す。
    pub fn pdf(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        if self.effectively_smooth() {
            // 完全鏡面反射の場合、PDF()は常に0を返す（デルタ関数のため）
            0.0
        } else {
            // マイクロファセットの場合、実際のPDF値を計算
            self.pdf_microfacet(wo, wi)
        }
    }

    /// マイクロファセットPDFを計算する。
    fn pdf_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        // 反射か透過かを判定
        let is_reflection = same_hemisphere(wo, wi);
        let is_transmission = !is_reflection;

        match self.scatter_mode {
            ScatterMode::R => {
                if is_transmission {
                    return 0.0;
                }
                // 反射PDFのみ
                self.pdf_reflection(wo, wi)
            }
            ScatterMode::RT => {
                if is_reflection {
                    // 反射PDF
                    let pdf_refl = self.pdf_reflection(wo, wi);

                    // フレネル反射率の平均値で重み付け
                    let fresnel = self.generalized_schlick_fresnel(wo.z().abs());
                    let avg_fresnel = fresnel.average();
                    let pr = avg_fresnel;
                    let pt = 1.0 - pr;

                    pdf_refl * pr / (pr + pt)
                } else {
                    // 透過PDF
                    let pdf_trans = self.pdf_transmission(wo, wi);

                    // フレネル透過率の平均値で重み付け
                    let fresnel = self.generalized_schlick_fresnel(wo.z().abs());
                    let avg_fresnel = fresnel.average();
                    let pr = avg_fresnel;
                    let pt = 1.0 - pr;

                    pdf_trans * pt / (pr + pt)
                }
            }
        }
    }

    /// 反射PDFを計算する。
    fn pdf_reflection(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        // ハーフベクトルを計算
        let wm = match half_vector(wo, wi) {
            Some(wm) => wm,
            None => return 0.0,
        };

        // 可視法線分布のPDF
        let visible_normal_pdf = self.visible_normal_distribution(wo, &wm);

        // ヤコビアン変換: dωm/dωi = 1/(4|ωo·ωm|)
        let jacobian = 4.0 * (wo.dot(wm)).abs();
        if jacobian == 0.0 {
            return 0.0;
        }

        visible_normal_pdf / jacobian
    }

    /// 透過PDFを計算する。
    fn pdf_transmission(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        if self.thin_surface {
            // Thin surface: 反対方向のみ
            if wo.x() != -wi.x() || wo.y() != -wi.y() || wo.z() != -wi.z() {
                return 0.0;
            }

            // 単純な確率密度（thin surfaceスタイル）
            1.0
        } else {
            // 通常の誘電体：適切な屈折PDF
            let eta_val = self.eta.value(0);
            let eta = eta_val;

            // Generalized half vectorを計算
            let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
                Some(wm) => wm,
                None => return 0.0,
            };

            let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
            let dwm_dwi = wi.dot(wm).abs() / denom;

            self.visible_normal_distribution(wo, &wm) * dwm_dwi
        }
    }
}
