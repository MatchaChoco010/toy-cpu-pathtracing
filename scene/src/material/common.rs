//! BSDFで共通して使用される汎用的な計算関数群。

use math::{ShadingNormalTangent, Vector3};

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
