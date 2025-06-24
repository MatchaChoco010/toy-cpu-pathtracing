//! 誘電体BSDFの実装。

use math::{ShadingNormalTangent, Vector3};
use spectrum::SampledSpectrum;

use super::{BsdfSample, BsdfSampleType};

/// 球面座標計算
fn cos_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z()
}

fn cos2_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z() * w.z()
}

fn abs_cos_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z().abs()
}

fn sin2_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    (1.0 - cos2_theta(w)).max(0.0)
}

fn tan2_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    sin2_theta(w) / cos2_theta(w)
}

fn cos_phi(w: &Vector3<ShadingNormalTangent>) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x() / sin_theta).clamp(-1.0, 1.0)
    }
}

fn sin_phi(w: &Vector3<ShadingNormalTangent>) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 {
        0.0
    } else {
        (w.y() / sin_theta).clamp(-1.0, 1.0)
    }
}

fn same_hemisphere(w1: &Vector3<ShadingNormalTangent>, w2: &Vector3<ShadingNormalTangent>) -> bool {
    w1.z() * w2.z() > 0.0
}

/// 誘電体のフレネル反射率を計算する。
///
/// # Arguments
/// - `cos_theta_i` - 入射角のコサイン値
/// - `eta` - 屈折率の比（透過側/入射側）
pub fn fresnel_dielectric(cos_theta_i: f32, eta: f32) -> f32 {
    // Snellの法則で透過角を計算
    let sin2_theta_i = 1.0 - cos_theta_i * cos_theta_i;
    let sin2_theta_t = sin2_theta_i / (eta * eta);

    // 全反射の場合
    if sin2_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).max(0.0).sqrt();

    // フレネル方程式
    let r_parl = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);
    let r_perp = (cos_theta_i - eta * cos_theta_t) / (cos_theta_i + eta * cos_theta_t);

    (r_parl * r_parl + r_perp * r_perp) * 0.5
}

/// 屈折方向を計算する。
///
/// # Arguments
/// - `wi` - 入射方向
/// - `n` - 法線方向
/// - `eta` - 屈折率の比（透過側/入射側）
///
/// # Returns
/// - `Some(wt)` - 屈折方向
/// - `None` - 全反射の場合
pub fn refract(
    wi: &Vector3<ShadingNormalTangent>,
    n: &Vector3<ShadingNormalTangent>,
    eta: f32,
) -> Option<Vector3<ShadingNormalTangent>> {
    let cos_theta_i = n.dot(wi);
    let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2_theta_t = sin2_theta_i / (eta * eta);

    // 全反射チェック
    if sin2_theta_t >= 1.0 {
        return None;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).max(0.0).sqrt();
    let wt = -*wi / eta + n * (cos_theta_i / eta - cos_theta_t);
    let wt_length_sq = wt.length_squared();
    if wt_length_sq < 1e-12 {
        None
    } else {
        Some(wt / wt_length_sq.sqrt())
    }
}

/// 反射ベクトルを計算
fn reflect(
    wo: &Vector3<ShadingNormalTangent>,
    n: &Vector3<ShadingNormalTangent>,
) -> Vector3<ShadingNormalTangent> {
    *n * 2.0 * wo.dot(n) - *wo
}

/// 極座標を使った単位円盤のサンプリング
fn sample_uniform_disk_polar(u: glam::Vec2) -> glam::Vec2 {
    let r = u.x.sqrt();
    let theta = 2.0 * std::f32::consts::PI * u.y;
    glam::Vec2::new(r * theta.cos(), r * theta.sin())
}

/// 誘電体のBSDF計算を行う構造体。
/// 完全鏡面とマイクロファセットをサポート。
pub struct DielectricBsdf {
    /// 屈折率
    eta: f32,
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

        let alpha2 =
            (cos_phi(w) * self.alpha_x).powi(2) + (sin_phi(w) * self.alpha_y).powi(2);
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
    /// - `eta` - 屈折率
    /// - `entering` - 入射方向が面の外側に向いているかどうか
    /// - `thin_surface` - Thin surfaceフラグ
    /// - `alpha_x` - X方向のroughness parameter
    /// - `alpha_y` - Y方向のroughness parameter
    pub fn new(eta: f32, entering: bool, thin_surface: bool, alpha_x: f32, alpha_y: f32) -> Self {
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
            self.sample_perfect_specular(wo, glam::Vec2::new(uc, uv.x))
        } else {
            self.sample_rough_dielectric(wo, uv, uc)
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

    /// Rough dielectric BSDFのサンプリングを行う
    fn sample_rough_dielectric(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        u: glam::Vec2,
        uc: f32,
    ) -> Option<BsdfSample> {
        // マイクロファセット法線をサンプリング
        let wm = self.sample_visible_normal(wo, u);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.thin_surface {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta = eta_t / eta_i;

        // フレネル反射率を計算
        let wo_dot_wm = wo.dot(wm);
        let fresnel = fresnel_dielectric(wo_dot_wm.abs(), eta);
        let pr = fresnel;
        let pt = 1.0 - pr;

        if self.thin_surface {
            let (pr, pt) = self.calculate_thin_surface_coefficients(fresnel);

            if uc < pr / (pr + pt) {
                // 反射
                self.sample_rough_reflection(wo, &wm, pr, pr / (pr + pt))
            } else {
                // Thin surface透過
                self.sample_rough_transmission_thin_surface(
                    wo,
                    &wm,
                    pt,
                    pt / (pr + pt),
                    eta,
                )
            }
        } else if uc < pr / (pr + pt) {
            // 反射
            self.sample_rough_reflection(wo, &wm, pr, pr / (pr + pt))
        } else {
            // 透過
            self.sample_rough_transmission(wo, &wm, pt, pt / (pr + pt), eta)
        }
    }

    /// Rough dielectric反射のサンプリング
    fn sample_rough_reflection(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        r: f32,
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

        // BRDF値計算
        let d = self.microfacet_distribution(wm);
        let g = self.masking_shadowing_g(wo, &wi);
        let f_value = d * g * r / (4.0 * abs_cos_theta(&wi) * abs_cos_theta(wo));

        Some(BsdfSample::new(
            SampledSpectrum::constant(f_value),
            wi,
            pdf,
            BsdfSampleType::Glossy,
        ))
    }

    /// Rough dielectric透過のサンプリング
    fn sample_rough_transmission(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
        t: f32,
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

        let mut ft =
            t * d * g * wi.dot(wm).abs() * wo.dot(wm).abs() / (denom * cos_theta_i * cos_theta_o);
        ft /= etap * etap;

        Some(BsdfSample::new(
            SampledSpectrum::constant(ft),
            wi,
            pdf,
            BsdfSampleType::Glossy,
        ))
    }

    /// Rough dielectric thin surface透過のサンプリング
    fn sample_rough_transmission_thin_surface(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        _wm: &Vector3<ShadingNormalTangent>,
        pt: f32,
        prob: f32,
        _eta: f32,
    ) -> Option<BsdfSample> {
        // Thin surfaceの場合は反対方向への透過
        let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());

        // PDF計算（thin surfaceの場合は特別な処理）
        let pdf = prob;

        // BTDF値（thin surfaceの特別な処理）
        let wi_cos_n = abs_cos_theta(&wi);
        let f_value = pt / wi_cos_n;

        Some(BsdfSample::new(
            SampledSpectrum::constant(f_value),
            wi,
            pdf,
            BsdfSampleType::Glossy,
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
    fn sample_perfect_specular(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();

        // 法線方向
        let n = if self.entering {
            Vector3::new(0.0, 0.0, 1.0)
        } else {
            Vector3::new(0.0, 0.0, -1.0)
        };

        // 屈折率を計算
        let (eta_i, eta_t) = if self.thin_surface {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let etap = eta_t / eta_i;

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo_cos_n.abs(), etap);

        if self.thin_surface {
            // Thin surfaceの場合は累積係数を計算
            let (pr, pt) = self.calculate_thin_surface_coefficients(fresnel);

            // 反射か透過かをサンプリング
            if uv.x < pr / (pr + pt) {
                // 反射（thin surface/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                if wo_cos_n.abs() < 1e-6 {
                    return None;
                }
                let f = SampledSpectrum::constant(pr / wo_cos_n.abs());
                Some(BsdfSample::new(
                    f,
                    wi,
                    pr / (pr + pt),
                    BsdfSampleType::Specular,
                ))
            } else {
                // Thin surface: 透過方向は入射方向の反対（wi = -wo）
                let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());
                let wi_cos_n = wi.z();
                if wi_cos_n == 0.0 {
                    return None;
                }

                // Thin surfaceの場合、放射輝度のスケーリングは不要（同じ媒質に戻るため）
                let f = SampledSpectrum::constant(pt / wi_cos_n.abs());
                Some(BsdfSample::new(
                    f,
                    wi,
                    pt / (pr + pt),
                    BsdfSampleType::Specular,
                ))
            }
        } else {
            let pr = fresnel;
            let pt = 1.0 - pr;

            // 反射か透過かをサンプリング
            if uv.x < pr / (pr + pt) {
                // 反射（thin surface/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                if wo_cos_n.abs() < 1e-6 {
                    return None;
                }
                let f = SampledSpectrum::constant(pr / wo_cos_n.abs());
                Some(BsdfSample::new(
                    f,
                    wi,
                    pr / (pr + pt),
                    BsdfSampleType::Specular,
                ))
            } else {
                // 通常の誘電体: Snellの法則による屈折
                if let Some(wt) = refract(wo, &n, etap) {
                    let wt_cos_n = wt.z();
                    if wt_cos_n == 0.0 {
                        return None;
                    }

                    let f = SampledSpectrum::constant(pt / etap.powi(2) / wt_cos_n.abs());
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
            self.evaluate_rough_dielectric(wo, wi)
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
            self.pdf_rough_dielectric(wo, wi)
        }
    }

    /// Rough dielectric BSDFの評価を行う
    fn evaluate_rough_dielectric(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.thin_surface {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta = eta_t / eta_i;

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
            Some(wm) => wm,
            None => return SampledSpectrum::zero(),
        };

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo.dot(wm).abs(), eta);

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        if reflect {
            let d = self.microfacet_distribution(&wm);
            let g = self.masking_shadowing_g(wo, wi);
            let f_value = d * g * fresnel / (4.0 * abs_cos_theta(wi) * abs_cos_theta(wo));
            SampledSpectrum::constant(f_value)
        } else {
            let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
            let d = self.microfacet_distribution(&wm);
            let g = self.masking_shadowing_g(wo, wi);

            let numerator = d * (1.0 - fresnel) * g * wi.dot(wm).abs() * wo.dot(wm).abs();
            let denominator = denom * abs_cos_theta(wi) * abs_cos_theta(wo);

            let mut ft = numerator / denominator;
            ft /= eta * eta;

            SampledSpectrum::constant(ft)
        }
    }

    /// Rough dielectric BSDFのPDFを計算する
    fn pdf_rough_dielectric(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.thin_surface {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else if self.entering {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let eta = eta_t / eta_i;

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
            Some(wm) => wm,
            None => return 0.0,
        };

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo.dot(wm).abs(), eta);
        let pr = fresnel;
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
            let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
            let dwm_dwi = wi.dot(wm).abs() / denom;

            self.visible_normal_distribution(wo, &wm) * dwm_dwi * pt / (pr + pt)
        }
    }
}
