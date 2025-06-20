//! 誘電体BSDFの実装。

use math::{NormalMapTangent, Vector3};
use spectrum::SampledSpectrum;

use super::{BsdfSample, BsdfSampleType};

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

/// 誘電体の純粋なBSDF計算を行う構造体。
/// 完全鏡面のみサポート。
pub struct DielectricBsdf {
    /// 屈折率
    eta: f32,
    /// 入射方向が面の外側に向いているかどうか
    entering: bool,
    /// Thin filmフラグ
    thin_film: bool,
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
        }
    }

    /// BSDF方向サンプリングを行う。
    /// 完全鏡面反射・透過のみサポート。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `uv` - ランダムサンプル
    pub fn sample(&self, wo: &Vector3<NormalMapTangent>, uv: glam::Vec2) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        self.sample_perfect_specular(wo, uv)
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
    pub fn evaluate(
        &self,
        _wo: &Vector3<NormalMapTangent>,
        _wi: &Vector3<NormalMapTangent>,
    ) -> SampledSpectrum {
        // 完全鏡面の場合、evaluate()は常に0を返す（デルタ関数のため）
        SampledSpectrum::zero()
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    pub fn pdf(&self, _wo: &Vector3<NormalMapTangent>, _wi: &Vector3<NormalMapTangent>) -> f32 {
        // 完全鏡面の場合、PDF()は常に0を返す（デルタ関数のため）
        0.0
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
