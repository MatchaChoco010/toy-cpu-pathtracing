//! BSDFで共通して使用される汎用的な計算関数群。

use math::{ShadingNormalTangent, Vector3};
use spectrum::SampledSpectrum;

/// 球面座標計算
pub fn cos_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z()
}

pub fn cos2_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z() * w.z()
}

pub fn abs_cos_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    w.z().abs()
}

pub fn tan2_theta(w: &Vector3<ShadingNormalTangent>) -> f32 {
    let cos2 = cos2_theta(w);
    if cos2 == 0.0 {
        f32::INFINITY
    } else {
        (1.0 - cos2) / cos2
    }
}

pub fn cos_phi(w: &Vector3<ShadingNormalTangent>) -> f32 {
    let sin_theta = (1.0 - cos2_theta(w)).max(0.0).sqrt();
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x() / sin_theta).clamp(-1.0, 1.0)
    }
}

pub fn sin_phi(w: &Vector3<ShadingNormalTangent>) -> f32 {
    let sin_theta = (1.0 - cos2_theta(w)).max(0.0).sqrt();
    if sin_theta == 0.0 {
        0.0
    } else {
        (w.y() / sin_theta).clamp(-1.0, 1.0)
    }
}

/// ハーフベクトルを計算
pub fn half_vector(
    wo: &Vector3<ShadingNormalTangent>,
    wi: &Vector3<ShadingNormalTangent>,
) -> Option<Vector3<ShadingNormalTangent>> {
    let wm = *wo + *wi;
    if wm.length_squared() == 0.0 {
        None
    } else {
        Some(wm.normalize())
    }
}

/// 反射ベクトルを計算
pub fn reflect(
    wo: &Vector3<ShadingNormalTangent>,
    n: &Vector3<ShadingNormalTangent>,
) -> Vector3<ShadingNormalTangent> {
    *n * (2.0 * wo.dot(n)) - *wo
}

/// 二つのベクトルが同じ半球にあるかチェック
pub fn same_hemisphere(
    w1: &Vector3<ShadingNormalTangent>,
    w2: &Vector3<ShadingNormalTangent>,
) -> bool {
    w1.z() * w2.z() > 0.0
}

/// 極座標を使った単位円盤のサンプリング
pub fn sample_uniform_disk_polar(u: glam::Vec2) -> glam::Vec2 {
    let r = u.x.sqrt();
    let theta = 2.0 * std::f32::consts::PI * u.y;
    glam::Vec2::new(r * theta.cos(), r * theta.sin())
}

/// 誘電体のフレネル反射率を計算する（スペクトル対応）。
///
/// # Arguments
/// - `cos_theta_i` - 入射角のコサイン値
/// - `eta` - 屈折率の比（透過側/入射側、スペクトル依存）
pub fn fresnel_dielectric(cos_theta_i: f32, eta: &SampledSpectrum) -> SampledSpectrum {
    let cos_theta_i = cos_theta_i.clamp(0.0, 1.0);

    // Snellの法則で透過角を計算
    let sin2_theta_i = 1.0 - cos_theta_i * cos_theta_i;
    let sin2_theta_t_spectrum =
        SampledSpectrum::constant(sin2_theta_i) / (eta.clone() * eta.clone());

    // 全反射判定とcos_theta_t計算をSampledSpectrumで処理
    let one = SampledSpectrum::one();
    let cos_theta_t_spectrum = (one - sin2_theta_t_spectrum).clamp(0.0, 1.0).sqrt();

    // フレネル方程式をSampledSpectrumで計算
    let cos_theta_i_spectrum = SampledSpectrum::constant(cos_theta_i);

    let r_parl = (eta.clone() * cos_theta_i_spectrum.clone() - cos_theta_t_spectrum.clone())
        / (eta.clone() * cos_theta_i_spectrum.clone() + cos_theta_t_spectrum.clone());
    let r_perp = (cos_theta_i_spectrum.clone() - eta.clone() * cos_theta_t_spectrum.clone())
        / (cos_theta_i_spectrum + eta.clone() * cos_theta_t_spectrum);

    (r_parl.clone() * r_parl + r_perp.clone() * r_perp) * 0.5
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
        Some(wt.normalize())
    }
}
