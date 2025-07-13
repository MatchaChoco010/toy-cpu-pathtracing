//! 誘電体BSDFの実装。

use math::{ShadingNormalTangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::material::{
    bsdf::{BsdfSample, BsdfSampleType},
    common::{
        abs_cos_theta, cos_phi, cos_theta, cos2_theta, fresnel_dielectric, reflect, refract,
        same_hemisphere, sample_uniform_disk_polar, sin_phi, tan2_theta,
    },
};

/// 誘電体のBSDF計算を行う構造体。
/// 完全鏡面とマイクロファセットをサポート。
pub struct DielectricBsdf {
    /// 屈折率（スペクトル依存）
    eta: SampledSpectrum,
    /// 入射方向が面の外側に向いているかどうか
    entering: bool,
    /// Thin surfaceフラグ
    thin_surface: bool,
    /// X方向のroughness parameter (α_x)
    alpha_x: f32,
    /// Y方向のroughness parameter (α_y)
    alpha_y: f32,
}
impl DielectricBsdf {
    /// 表面が事実上滑らかかどうかを判定する。
    fn effectively_smooth(&self) -> bool {
        self.alpha_x.max(self.alpha_y) < 1e-3
    }

    /// Trowbridge-Reitz分布関数 D(ωm)を計算する。
    fn microfacet_distribution(&self, wm: &Vector3<ShadingNormalTangent>) -> f32 {
        let tan2_theta = tan2_theta(wm);
        if !tan2_theta.is_finite() {
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
    /// DielectricBsdfを作成する。
    /// alpha_x, alpha_yが0に限りなく近い場合は完全鏡面、それ以外はマイクロファセット。
    ///
    /// # Arguments
    /// - `eta` - 屈折率（スペクトル依存）
    /// - `entering` - 入射方向が面の外側に向いているかどうか
    /// - `thin_surface` - Thin surfaceフラグ
    /// - `alpha_x` - X方向のroughness parameter
    /// - `alpha_y` - Y方向のroughness parameter
    pub fn new(
        eta: SampledSpectrum,
        entering: bool,
        thin_surface: bool,
        alpha_x: f32,
        alpha_y: f32,
    ) -> Self {
        // 屈折率の妥当性チェック：0の場合は1.0にフォールバック
        let eta = if eta.value(0) == 0.0 {
            SampledSpectrum::constant(1.0)
        } else {
            eta
        };

        Self {
            eta,
            entering,
            thin_surface,
            alpha_x,
            alpha_y,
        }
    }

    /// BSDF方向サンプリングを行う。
    /// 完全鏡面反射・透過とマイクロファセットをサポート。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `uv` - ランダムサンプル
    /// - `uc` - 反射/透過選択用の追加ランダム値
    /// - `lambda` - 波長サンプリング情報
    pub fn sample(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
        uc: f32,
        wavelengths: &mut SampledWavelengths,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        if self.effectively_smooth() {
            self.sample_specular(wo, glam::Vec2::new(uc, uv.x), wavelengths)
        } else {
            self.sample_microfacet(wo, uv, uc, wavelengths)
        }
    }

    ///
    /// # Arguments
    /// - `wo` - 出射方向
    /// - `wi` - 入射方向
    /// - `eta` - 屈折率の比
    ///
    /// # Returns
    /// - `Some(wm)` - Generalized half vector
    /// - `None` - 計算できない場合（grazing angle等）
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

    /// Microfacet dielectric BSDFのサンプリングを行う
    fn sample_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        u: glam::Vec2,
        uc: f32,
        wavelengths: &mut SampledWavelengths,
    ) -> Option<BsdfSample> {
        // マイクロファセット法線をサンプリング
        let wm = self.sample_visible_normal(wo, u);

        // 屈折率を計算
        let eta_spectrum = if self.thin_surface {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else {
            SampledSpectrum::one() / self.eta.clone() // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta_scalar = eta_spectrum.value(0);

        let wo_dot_wm = wo.dot(wm);
        let fresnel = fresnel_dielectric(wo_dot_wm.abs(), &eta_spectrum);
        let pr = fresnel.average();
        let pt = 1.0 - pr;

        if self.thin_surface {
            let avg_fresnel = fresnel.average();
            let (pr, pt) = self.calculate_thin_surface_coefficients(avg_fresnel);

            if uc < pr / (pr + pt) {
                // 反射
                self.sample_microfacet_reflection(wo, &wm, fresnel, pr / (pr + pt))
            } else {
                // Thin surface透過
                self.sample_specular_transmission_thin_surface(
                    wo,
                    &wm,
                    SampledSpectrum::one() - fresnel,
                    pt / (pr + pt),
                    eta_scalar,
                )
            }
        } else if uc < pr / (pr + pt) {
            // 反射
            self.sample_microfacet_reflection(wo, &wm, fresnel, pr / (pr + pt))
        } else {
            // 透過（屈折時のみ波長を制限）
            if !self.eta.is_constant() {
                wavelengths.terminate_secondary();
            }
            self.sample_microfacet_transmission(
                wo,
                &wm,
                SampledSpectrum::one() - fresnel,
                pt / (pr + pt),
                eta_scalar,
            )
        }
    }

    /// Rough dielectric反射のサンプリング
    fn sample_microfacet_reflection(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        fresnel: SampledSpectrum,
        prob: f32,
    ) -> Option<BsdfSample> {
        let wi = reflect(wo, wm);

        if !same_hemisphere(wo, &wi) {
            return None;
        }

        // PDF計算
        let cos_theta_dot = wo.dot(wm).abs();
        if cos_theta_dot < 1e-6 {
            return None;
        }
        let pdf = self.visible_normal_distribution(wo, wm) / (4.0 * cos_theta_dot) * prob;

        // BRDF値計算（cosine項を含む）
        let d = self.microfacet_distribution(wm);
        let g = self.masking_shadowing_g(wo, &wi);
        let f_value = fresnel * d * g * abs_cos_theta(&wi) / (4.0 * abs_cos_theta(wo));

        Some(BsdfSample::new(
            f_value,
            wi,
            pdf,
            BsdfSampleType::GlossyReflection,
        ))
    }

    /// Rough dielectric透過のサンプリング
    fn sample_microfacet_transmission(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        transmission: SampledSpectrum,
        prob: f32,
        etap: f32,
    ) -> Option<BsdfSample> {
        let wm_refract = if self.entering { wm } else { &-*wm };
        let wi = refract(wo, wm_refract, etap)?;

        if same_hemisphere(wo, &wi) || wi.z().abs() == 0.0 {
            return None;
        }

        let denom = (wi.dot(wm) + wo.dot(wm) / etap).powi(2);
        let dwm_dwi = wi.dot(wm).abs() / denom;

        // PDF計算
        let pdf = self.visible_normal_distribution(wo, wm) * dwm_dwi * prob;

        let d = self.microfacet_distribution(wm);
        let g = self.masking_shadowing_g(wo, &wi);
        let cos_theta_i = abs_cos_theta(&wi);
        let cos_theta_o = abs_cos_theta(wo);

        let ft = transmission * d * g * wi.dot(wm).abs() * wo.dot(wm).abs() * cos_theta_i
            / (denom * cos_theta_o * etap * etap);

        Some(BsdfSample::new(
            ft,
            wi,
            pdf,
            BsdfSampleType::GlossyTransmission,
        ))
    }

    /// Rough dielectric thin surface透過のサンプリング
    fn sample_specular_transmission_thin_surface(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        _wm: &Vector3<ShadingNormalTangent>,
        transmission: SampledSpectrum,
        prob: f32,
        _eta: f32,
    ) -> Option<BsdfSample> {
        // Thin surfaceの場合は反対方向への透過
        let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());

        // PDF計算（thin surfaceの場合は特別な処理）
        let pdf = prob;

        // BTDF値（thin surfaceの特別な処理）
        let f_value = transmission;

        Some(BsdfSample::new(
            f_value,
            wi,
            pdf,
            BsdfSampleType::GlossyTransmission,
        ))
    }

    /// Thin surfaceの累積反射・透過係数を計算する。
    ///
    /// # Arguments
    /// - `fresnel` - 通常のフレネル反射率
    ///
    /// # Returns
    /// - `(cumulative_reflection, cumulative_transmission)` - 累積反射率と累積透過率
    fn calculate_thin_surface_coefficients(&self, fresnel: f32) -> (f32, f32) {
        // Geometric seriesを使った累積反射率の計算
        // R' = R + (T²R) / (1 - R²)
        let r = fresnel;
        let t = 1.0 - r;
        let r_squared = r * r;

        let r = if r_squared > 1.0 {
            1.0
        } else {
            r + (t * t * r) / (1.0 - r_squared)
        };

        (r, t)
    }

    /// 完全鏡面反射・透過サンプリング。
    fn sample_specular(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
        wavelengths: &mut SampledWavelengths,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();

        // 法線方向
        let n = if self.entering {
            Vector3::new(0.0, 0.0, 1.0)
        } else {
            Vector3::new(0.0, 0.0, -1.0)
        };

        // 屈折率を計算
        let eta_spectrum = if self.thin_surface {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else {
            SampledSpectrum::one() / self.eta.clone() // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let etap_scalar = eta_spectrum.value(0);

        let fresnel = fresnel_dielectric(wo_cos_n.abs(), &eta_spectrum);

        if self.thin_surface {
            // Thin surfaceの場合は累積係数を計算
            let avg_fresnel = fresnel.average();
            let (pr, pt) = self.calculate_thin_surface_coefficients(avg_fresnel);

            // 反射か透過かをサンプリング
            if uv.x < pr / (pr + pt) {
                // 反射（thin surface/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                if wo_cos_n.abs() < 1e-6 {
                    return None;
                }
                let f = fresnel;
                Some(BsdfSample::new(
                    f,
                    wi,
                    pr / (pr + pt),
                    BsdfSampleType::SpecularReflection,
                ))
            } else {
                // Thin surface: 透過方向は入射方向の反対（wi = -wo）
                let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());
                let wi_cos_n = wi.z();
                if wi_cos_n == 0.0 {
                    return None;
                }

                // Thin surfaceの場合、放射輝度のスケーリングは不要（同じ媒質に戻るため）
                let transmission = SampledSpectrum::one() - fresnel;
                let f = transmission;
                Some(BsdfSample::new(
                    f,
                    wi,
                    pt / (pr + pt),
                    BsdfSampleType::SpecularTransmission,
                ))
            }
        } else {
            let avg_fresnel = fresnel.average();
            let pr = avg_fresnel;
            let pt = 1.0 - pr;

            // 反射か透過かをサンプリング
            if uv.x < pr / (pr + pt) {
                // 反射（thin surface/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                if wo_cos_n.abs() < 1e-6 {
                    return None;
                }
                let f = fresnel;
                Some(BsdfSample::new(
                    f,
                    wi,
                    pr / (pr + pt),
                    BsdfSampleType::SpecularReflection,
                ))
            } else {
                // 通常の誘電体: Snellの法則による屈折（波長制限）
                if !self.eta.is_constant() {
                    wavelengths.terminate_secondary();
                }

                if let Some(wt) = refract(wo, &n, etap_scalar) {
                    let wt_cos_n = wt.z();
                    if wt_cos_n == 0.0 {
                        return None;
                    }

                    let transmission = SampledSpectrum::one() - fresnel;
                    let f = transmission / etap_scalar.powi(2);
                    Some(BsdfSample::new(
                        f,
                        wt,
                        pt / (pr + pt),
                        BsdfSampleType::SpecularTransmission,
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// BSDF値を評価する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    /// マイクロファセットの場合は実際のBSDF値を計算する。
    pub fn evaluate(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        if self.effectively_smooth() {
            SampledSpectrum::zero() // 完全鏡面の場合は0
        } else {
            self.evaluate_microfacet(wo, wi)
        }
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    /// マイクロファセットの場合は実際のPDF値を計算する。
    pub fn pdf(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        if self.effectively_smooth() {
            0.0 // 完全鏡面の場合は0
        } else {
            self.pdf_microfacet(wo, wi)
        }
    }

    /// Microfacet dielectric BSDFの評価を行う
    fn evaluate_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let eta_spectrum = if self.thin_surface {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else {
            SampledSpectrum::one() / self.eta.clone() // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta_scalar = eta_spectrum.value(0);

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta_scalar) {
            Some(wm) => wm,
            None => return SampledSpectrum::zero(),
        };

        let fresnel = fresnel_dielectric(wo.dot(wm).abs(), &eta_spectrum);

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        if reflect {
            let d = self.microfacet_distribution(&wm);
            let g = self.masking_shadowing_g(wo, wi);

            fresnel * d * g / (4.0 * abs_cos_theta(wo))
        } else {
            let denom = (wi.dot(wm) + wo.dot(wm) / eta_scalar).powi(2);
            let d = self.microfacet_distribution(&wm);
            let g = self.masking_shadowing_g(wo, wi);
            let transmission = SampledSpectrum::one() - fresnel;

            transmission * d * g * wi.dot(wm).abs() * wo.dot(wm).abs()
                / (denom * abs_cos_theta(wo) * eta_scalar * eta_scalar)
        }
    }

    /// Microfacet dielectric BSDFのPDFを計算する
    fn pdf_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let eta_spectrum = if self.thin_surface {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            self.eta.clone() // 空気(1.0) → 誘電体(n): eta = n
        } else {
            SampledSpectrum::one() / self.eta.clone() // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta_scalar = eta_spectrum.value(0);

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta_scalar) {
            Some(wm) => wm,
            None => return 0.0,
        };

        let fresnel = fresnel_dielectric(wo.dot(wm).abs(), &eta_spectrum);
        let pr = fresnel.average();
        let pt = 1.0 - pr;

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        if reflect {
            // 反射PDF
            self.visible_normal_distribution(wo, &wm) / (4.0 * wo.dot(wm).abs()) * pr / (pr + pt)
        } else if self.thin_surface {
            // Thin surface透過PDF
            pt / (pr + pt)
        } else {
            let denom = (wi.dot(wm) + wo.dot(wm) / eta_scalar).powi(2);
            let dwm_dwi = wi.dot(wm).abs() / denom;

            self.visible_normal_distribution(wo, &wm) * dwm_dwi * pt / (pr + pt)
        }
    }
}
