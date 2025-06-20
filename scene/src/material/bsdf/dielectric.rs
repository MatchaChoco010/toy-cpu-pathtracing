//! 誘電体BSDFの実装。

use math::{NormalMapTangent, Vector3};
use spectrum::SampledSpectrum;

use super::{BsdfSample, BsdfSampleType};

/// 球面座標ヘルパー関数
#[inline]
fn cos_theta(w: &Vector3<NormalMapTangent>) -> f32 {
    w.z()
}

#[inline]
fn cos2_theta(w: &Vector3<NormalMapTangent>) -> f32 {
    w.z() * w.z()
}

#[inline]
fn abs_cos_theta(w: &Vector3<NormalMapTangent>) -> f32 {
    w.z().abs()
}

#[inline]
fn sin2_theta(w: &Vector3<NormalMapTangent>) -> f32 {
    (1.0 - cos2_theta(w)).max(0.0)
}

#[inline]
fn tan2_theta(w: &Vector3<NormalMapTangent>) -> f32 {
    sin2_theta(w) / cos2_theta(w)
}

#[inline]
fn cos_phi(w: &Vector3<NormalMapTangent>) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x() / sin_theta).clamp(-1.0, 1.0)
    }
}

#[inline]
fn sin_phi(w: &Vector3<NormalMapTangent>) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 {
        0.0
    } else {
        (w.y() / sin_theta).clamp(-1.0, 1.0)
    }
}

#[inline]
fn same_hemisphere(w1: &Vector3<NormalMapTangent>, w2: &Vector3<NormalMapTangent>) -> bool {
    w1.z() * w2.z() > 0.0
}

#[inline]
fn reflect(
    wo: &Vector3<NormalMapTangent>,
    n: &Vector3<NormalMapTangent>,
) -> Vector3<NormalMapTangent> {
    *wo - *n * 2.0 * wo.dot(n)
}

/// 均等分布円盤サンプリング (Polar座標版)
fn sample_uniform_disk_polar(u: glam::Vec2) -> glam::Vec2 {
    let r = u.x.sqrt();
    let theta = 2.0 * std::f32::consts::PI * u.y;
    glam::Vec2::new(r * theta.cos(), r * theta.sin())
}

/// Trowbridge-Reitz (GGX) マイクロファセット分布
/// pbrt-v4のTrowbridgeReitzDistributionに忠実に実装
#[derive(Debug, Clone)]
pub struct TrowbridgeReitzDistribution {
    /// X軸方向の粗さパラメータ
    alpha_x: f32,
    /// Y軸方向の粗さパラメータ
    alpha_y: f32,
}

impl TrowbridgeReitzDistribution {
    /// 新しいTrowbridge-Reitz分布を作成
    pub fn new(alpha_x: f32, alpha_y: f32) -> Self {
        Self { alpha_x, alpha_y }
    }

    /// 等方性分布を作成
    pub fn isotropic(alpha: f32) -> Self {
        Self::new(alpha, alpha)
    }

    /// マイクロファセット分布関数D(ωm)を計算
    /// pbrt-v4 Equation (9.16)に基づく
    pub fn d(&self, wm: &Vector3<NormalMapTangent>) -> f32 {
        let tan2_theta = tan2_theta(wm);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wm) * cos2_theta(wm);
        if cos4_theta == 0.0 {
            return 0.0;
        }

        let cos_phi = cos_phi(wm);
        let sin_phi = sin_phi(wm);

        let e = tan2_theta
            * ((cos_phi / self.alpha_x) * (cos_phi / self.alpha_x)
                + (sin_phi / self.alpha_y) * (sin_phi / self.alpha_y));

        1.0 / (std::f32::consts::PI
            * self.alpha_x
            * self.alpha_y
            * cos4_theta
            * (1.0 + e)
            * (1.0 + e))
    }

    /// マスキング関数G1(ω)を計算
    /// pbrt-v4 Equation (9.19)に基づく
    pub fn g1(&self, w: &Vector3<NormalMapTangent>) -> f32 {
        1.0 / (1.0 + self.lambda(w))
    }

    /// 双方向マスキング-シャドウイング関数G(ωo, ωi)を計算
    /// pbrt-v4のSmith's approximationに基づく
    pub fn g(&self, wo: &Vector3<NormalMapTangent>, wi: &Vector3<NormalMapTangent>) -> f32 {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    /// Λ(ω)関数を計算 (Smith's approximation用)
    /// pbrt-v4のTrowbridge-Reitz分布用の実装
    fn lambda(&self, w: &Vector3<NormalMapTangent>) -> f32 {
        let abs_tan_theta = tan2_theta(w).sqrt().abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        let cos_phi = cos_phi(w);
        let sin_phi = sin_phi(w);
        let alpha = (cos_phi * self.alpha_x).powi(2) + (sin_phi * self.alpha_y).powi(2);
        let alpha = alpha.sqrt();

        let a = 1.0 / (alpha * abs_tan_theta);
        if a >= 1.6 {
            0.0
        } else {
            (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
        }
    }

    /// 可視法線分布D_ω(ωm)を計算
    /// pbrt-v4 Equation (9.23)に基づく
    pub fn d_visible(&self, w: &Vector3<NormalMapTangent>, wm: &Vector3<NormalMapTangent>) -> f32 {
        self.g1(w) / abs_cos_theta(w) * self.d(wm) * w.dot(wm).abs()
    }

    /// 可視法線分布からマイクロファセット法線をサンプリング
    /// pbrt-v4のellipsoid projection methodに基づく
    pub fn sample_wm(
        &self,
        w: &Vector3<NormalMapTangent>,
        u: glam::Vec2,
    ) -> Vector3<NormalMapTangent> {
        // Transform w to hemispherical configuration
        let wh: Vector3<NormalMapTangent> =
            Vector3::new(self.alpha_x * w.x(), self.alpha_y * w.y(), w.z()).normalize();

        let wh = if wh.z() < 0.0 { -wh } else { wh };

        // Find orthonormal basis for visible normal sampling
        let t1: Vector3<NormalMapTangent> = if wh.z() < 0.99999 {
            Vector3::new(0.0, 0.0, 1.0).cross(&wh).normalize()
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let t2 = wh.cross(&t1);

        // Generate uniformly distributed points on the unit disk
        let p = sample_uniform_disk_polar(u);

        // Warp hemispherical projection for visible normal sampling
        let h = (1.0 - p.x * p.x).sqrt();
        let py = ((1.0 + wh.z()) / 2.0).max(h) * (1.0 - u.y) + h * u.y;

        // Reproject to hemisphere and transform normal to ellipsoid configuration
        let pz = (1.0 - p.x * p.x - py * py).max(0.0).sqrt();
        let nh = t1 * p.x + t2 * py + wh * pz;

        Vector3::<NormalMapTangent>::new(
            self.alpha_x * nh.x(),
            self.alpha_y * nh.y(),
            nh.z().max(1e-6),
        )
        .normalize()
    }

    /// サンプリングPDFを計算
    pub fn pdf(&self, w: &Vector3<NormalMapTangent>, wm: &Vector3<NormalMapTangent>) -> f32 {
        self.d_visible(w, wm)
    }

    /// 事実上滑らかとみなせるかどうかを判定
    pub fn effectively_smooth(&self) -> bool {
        self.alpha_x.max(self.alpha_y) < 1e-3
    }
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
    wi: &Vector3<NormalMapTangent>,
    n: &Vector3<NormalMapTangent>,
    eta: f32,
) -> Option<Vector3<NormalMapTangent>> {
    let cos_theta_i = n.dot(wi);
    let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2_theta_t = sin2_theta_i / (eta * eta);

    // 全反射チェック
    if sin2_theta_t >= 1.0 {
        return None;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).max(0.0).sqrt();
    let wt = -*wi / eta + n * (cos_theta_i / eta - cos_theta_t);
    Some(wt.normalize())
}

/// 誘電体のBSDF計算を行う構造体。
/// 完全鏡面とマイクロファセットをサポート。
pub struct DielectricBsdf {
    /// 屈折率
    eta: f32,
    /// 入射方向が面の外側に向いているかどうか
    entering: bool,
    /// Thin filmフラグ
    thin_film: bool,
    /// マイクロファセット分布（Noneの場合は完全鏡面）
    microfacet_distribution: Option<TrowbridgeReitzDistribution>,
}
impl DielectricBsdf {
    /// 完全鏡面用のDielectricBsdfを作成する。
    ///
    /// # Arguments
    /// - `eta` - 屈折率（スペクトル依存）
    /// - `thin_film` - Thin filmフラグ
    pub fn new(eta: f32, entering: bool, thin_film: bool) -> Self {
        Self {
            eta,
            entering,
            thin_film,
            microfacet_distribution: None,
        }
    }

    /// マイクロファセット用のDielectricBsdfを作成する。
    ///
    /// # Arguments
    /// - `eta` - 屈折率
    /// - `entering` - 入射方向が面の外側に向いているかどうか
    /// - `thin_film` - Thin filmフラグ
    /// - `roughness` - 表面粗さパラメータ（0.0で完全鏡面）
    pub fn new_with_roughness(eta: f32, entering: bool, thin_film: bool, roughness: f32) -> Self {
        let microfacet_distribution = if roughness > 1e-3 {
            Some(TrowbridgeReitzDistribution::isotropic(roughness))
        } else {
            None
        };

        Self {
            eta,
            entering,
            thin_film,
            microfacet_distribution,
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
        wo: &Vector3<NormalMapTangent>,
        uv: glam::Vec2,
        uc: f32,
    ) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        match &self.microfacet_distribution {
            Some(distrib) => self.sample_rough_dielectric(wo, uv, uc, distrib),
            None => self.sample_perfect_specular(wo, glam::Vec2::new(uc, uv.x)),
        }
    }

    /// Generalized half vectorを計算する（pbrt-v4 Equation 9.34に基づく）
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
        wo: &Vector3<NormalMapTangent>,
        wi: &Vector3<NormalMapTangent>,
        eta: f32,
    ) -> Option<Vector3<NormalMapTangent>> {
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

    /// Rough dielectric BSDFのサンプリングを行う（pbrt-v4に基づく）
    fn sample_rough_dielectric(
        &self,
        wo: &Vector3<NormalMapTangent>,
        u: glam::Vec2,
        uc: f32,
        distrib: &TrowbridgeReitzDistribution,
    ) -> Option<BsdfSample> {
        // マイクロファセット法線をサンプリング
        let wm = distrib.sample_wm(wo, u);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.entering {
            (1.0, self.eta)
        } else {
            (self.eta, 1.0)
        };
        let eta = eta_t / eta_i;

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo.dot(&wm).abs(), eta);
        let r = fresnel;
        let t = 1.0 - r;

        if self.thin_film {
            let (pr, pt) = self.calculate_thin_film_coefficients(fresnel);

            if uc < pr / (pr + pt) {
                // 反射
                self.sample_rough_reflection(wo, &wm, distrib, pr, pr + pt)
            } else {
                // Thin film透過
                self.sample_rough_transmission_thin_film(wo, &wm, distrib, pt, pr + pt, eta)
            }
        } else {
            if uc < r / (r + t) {
                // 反射
                self.sample_rough_reflection(wo, &wm, distrib, r, r + t)
            } else {
                // 透過
                self.sample_rough_transmission(wo, &wm, distrib, t, r + t, eta)
            }
        }
    }

    /// Rough dielectric反射のサンプリング
    fn sample_rough_reflection(
        &self,
        wo: &Vector3<NormalMapTangent>,
        wm: &Vector3<NormalMapTangent>,
        distrib: &TrowbridgeReitzDistribution,
        r: f32,
        total_prob: f32,
    ) -> Option<BsdfSample> {
        let wi = reflect(wo, wm);

        if !same_hemisphere(wo, &wi) {
            return None;
        }

        // PDF計算
        let pdf = distrib.pdf(wo, wm) / (4.0 * wo.dot(wm).abs()) * r / total_prob;

        // BRDF値計算（pbrt-v4 rough conductor BRDFと同様）
        let d = distrib.d(wm);
        let g = distrib.g(wo, &wi);
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
        wo: &Vector3<NormalMapTangent>,
        wm: &Vector3<NormalMapTangent>,
        distrib: &TrowbridgeReitzDistribution,
        t: f32,
        total_prob: f32,
        eta: f32,
    ) -> Option<BsdfSample> {
        // 屈折方向を計算
        let wi = refract(wo, wm, eta)?;

        if same_hemisphere(wo, &wi) {
            return None;
        }

        // Jacobian計算（pbrt-v4 Equation 9.36）
        let denom = (wi.dot(wm) + wo.dot(wm) / eta).powi(2);
        let dwm_dwi = wi.dot(wm).abs() / denom;

        // PDF計算
        let pdf = distrib.pdf(wo, wm) * dwm_dwi * t / total_prob;

        // BTDF値計算（pbrt-v4 Equation 9.40）
        let d = distrib.d(wm);
        let g = distrib.g(wo, &wi);
        let cos_theta_i = abs_cos_theta(&wi);
        let cos_theta_o = abs_cos_theta(wo);

        let numerator = d * t * g * wi.dot(wm).abs() * wo.dot(wm).abs();
        let denominator = denom * cos_theta_i * cos_theta_o;

        let mut ft = numerator / denominator;

        // Transport mode補正（radiance mode時はη²で割る）
        ft /= eta * eta;

        Some(BsdfSample::new(
            SampledSpectrum::constant(ft),
            wi,
            pdf,
            BsdfSampleType::Glossy,
        ))
    }

    /// Rough dielectric thin film透過のサンプリング
    fn sample_rough_transmission_thin_film(
        &self,
        wo: &Vector3<NormalMapTangent>,
        _wm: &Vector3<NormalMapTangent>,
        _distrib: &TrowbridgeReitzDistribution,
        pt: f32,
        total_prob: f32,
        _eta: f32,
    ) -> Option<BsdfSample> {
        // Thin filmの場合は反対方向への透過
        let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());

        // PDF計算（thin filmの場合は特別な処理）
        let pdf = pt / total_prob;

        // BTDF値（thin filmの特別な処理）
        let wi_cos_n = abs_cos_theta(&wi);
        let f_value = pt / wi_cos_n;

        Some(BsdfSample::new(
            SampledSpectrum::constant(f_value),
            wi,
            pdf,
            BsdfSampleType::Glossy,
        ))
    }

    /// Thin filmの累積反射・透過係数を計算する。
    ///
    /// # Arguments
    /// - `fresnel` - 通常のフレネル反射率
    ///
    /// # Returns
    /// - `(cumulative_reflection, cumulative_transmission)` - 累積反射率と累積透過率
    fn calculate_thin_film_coefficients(&self, fresnel: f32) -> (f32, f32) {
        // Geometric seriesを使った累積反射率の計算
        // R' = R + (T²R) / (1 - R²)
        let r = fresnel;
        let t = 1.0 - r;
        let r_squared = r * r;

        let cumulative_reflection = if r_squared >= 1.0 {
            1.0 // 全反射の場合
        } else {
            r + (t * t * r) / (1.0 - r_squared)
        };

        let cumulative_transmission = 1.0 - cumulative_reflection;

        (cumulative_reflection, cumulative_transmission)
    }

    /// 完全鏡面反射・透過サンプリング。
    fn sample_perfect_specular(
        &self,
        wo: &Vector3<NormalMapTangent>,
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
        let (eta_i, eta_t) = if self.entering {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let etap = eta_t / eta_i;

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo_cos_n.abs(), etap);

        if self.thin_film {
            // Thin filmの場合は累積係数を計算
            let (pr, pt) = self.calculate_thin_film_coefficients(fresnel);

            // 反射か透過かをサンプリング
            if uv.x < pr / (pr + pt) {
                // 反射（thin film/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
                let f = SampledSpectrum::constant(pr / wo_cos_n.abs());
                Some(BsdfSample::new(
                    f,
                    wi,
                    pr / (pr + pt),
                    BsdfSampleType::Specular,
                ))
            } else {
                // Thin film: 透過方向は入射方向の反対（wi = -wo）
                let wi = Vector3::new(-wo.x(), -wo.y(), -wo.z());
                let wi_cos_n = wi.z();
                if wi_cos_n == 0.0 {
                    return None;
                }

                // Thin filmの場合、放射輝度のスケーリングは不要（同じ媒質に戻るため）
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
                // 反射（thin film/通常誘電体ともに同じ鏡面反射方向）
                let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
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
                    // let wt_cos_n = wt.z();
                    let wt_cos_n = n.dot(&wt);
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
        wo: &Vector3<NormalMapTangent>,
        wi: &Vector3<NormalMapTangent>,
    ) -> SampledSpectrum {
        match &self.microfacet_distribution {
            Some(distrib) => self.evaluate_rough_dielectric(wo, wi, distrib),
            None => SampledSpectrum::zero(), // 完全鏡面の場合は0
        }
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    /// マイクロファセットの場合は実際のPDF値を計算する。
    pub fn pdf(&self, wo: &Vector3<NormalMapTangent>, wi: &Vector3<NormalMapTangent>) -> f32 {
        match &self.microfacet_distribution {
            Some(distrib) => self.pdf_rough_dielectric(wo, wi, distrib),
            None => 0.0, // 完全鏡面の場合は0
        }
    }

    /// Rough dielectric BSDFの評価を行う（pbrt-v4に基づく）
    fn evaluate_rough_dielectric(
        &self,
        wo: &Vector3<NormalMapTangent>,
        wi: &Vector3<NormalMapTangent>,
        distrib: &TrowbridgeReitzDistribution,
    ) -> SampledSpectrum {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.entering {
            (1.0, self.eta)
        } else {
            (self.eta, 1.0)
        };
        let eta = eta_t / eta_i;

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
            Some(wm) => wm,
            None => return SampledSpectrum::zero(),
        };

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo.dot(&wm).abs(), eta);

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        if reflect {
            // 反射BRDF（pbrt-v4 rough conductor BRDFと同様）
            let d = distrib.d(&wm);
            let g = distrib.g(wo, wi);
            let f_value = d * g * fresnel / (4.0 * abs_cos_theta(wi) * abs_cos_theta(wo));
            SampledSpectrum::constant(f_value)
        } else {
            // 透過BTDF（pbrt-v4 Equation 9.40）
            let etap = if cos_theta_o > 0.0 { eta } else { 1.0 / eta };

            let denom = (wi.dot(&wm) + wo.dot(&wm) / etap).powi(2);
            let d = distrib.d(&wm);
            let g = distrib.g(wo, wi);

            let numerator = d * (1.0 - fresnel) * g * wi.dot(&wm).abs() * wo.dot(&wm).abs();
            let denominator = denom * abs_cos_theta(wi) * abs_cos_theta(wo);

            let mut ft = numerator / denominator;

            // Transport mode補正（radiance mode時はη²で割る）
            ft /= etap * etap;

            SampledSpectrum::constant(ft)
        }
    }

    /// Rough dielectric BSDFのPDFを計算する（pbrt-v4に基づく）
    fn pdf_rough_dielectric(
        &self,
        wo: &Vector3<NormalMapTangent>,
        wi: &Vector3<NormalMapTangent>,
        distrib: &TrowbridgeReitzDistribution,
    ) -> f32 {
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);

        // 屈折率を計算
        let (eta_i, eta_t) = if self.entering {
            (1.0, self.eta)
        } else {
            (self.eta, 1.0)
        };
        let eta = eta_t / eta_i;

        // Generalized half vectorを計算
        let wm = match self.compute_generalized_half_vector(wo, wi, eta) {
            Some(wm) => wm,
            None => return 0.0,
        };

        // フレネル反射率を計算
        let fresnel = fresnel_dielectric(wo.dot(&wm).abs(), eta);

        // 反射か透過かを判定
        let reflect = cos_theta_i * cos_theta_o > 0.0;

        let r = if self.thin_film {
            let (pr, _pt) = self.calculate_thin_film_coefficients(fresnel);
            pr
        } else {
            fresnel
        };

        let t = if self.thin_film {
            let (_pr, pt) = self.calculate_thin_film_coefficients(fresnel);
            pt
        } else {
            1.0 - fresnel
        };

        if reflect {
            // 反射PDF
            distrib.pdf(wo, &wm) / (4.0 * wo.dot(&wm).abs()) * r / (r + t)
        } else {
            if self.thin_film {
                // Thin film透過PDF
                t / (r + t)
            } else {
                // 通常の透過PDF（pbrt-v4 Equation 9.37）
                let etap = if cos_theta_o > 0.0 { eta } else { 1.0 / eta };
                let denom = (wi.dot(&wm) + wo.dot(&wm) / etap).powi(2);
                let dwm_dwi = wi.dot(&wm).abs() / denom;

                distrib.pdf(wo, &wm) * dwm_dwi * t / (r + t)
            }
        }
    }

    /// フレネル反射率を計算する。
    /// Thin filmの場合は累積反射率を返す。
    ///
    /// # Arguments
    /// - `cos_theta_i` - 入射角のコサイン値
    pub fn fresnel(&self, cos_theta_i: f32) -> f32 {
        let (eta_i, eta_t) = if cos_theta_i >= 0.0 {
            (1.0, self.eta) // 空気(1.0) → 誘電体(n): eta = n
        } else {
            (self.eta, 1.0) // 誘電体(n) → 空気(1.0): eta = 1/n
        };
        let etap = eta_t / eta_i;
        let base_fresnel = fresnel_dielectric(cos_theta_i.abs(), etap);

        if self.thin_film {
            let (cumulative_reflection, _) = self.calculate_thin_film_coefficients(base_fresnel);
            cumulative_reflection
        } else {
            base_fresnel
        }
    }
}
