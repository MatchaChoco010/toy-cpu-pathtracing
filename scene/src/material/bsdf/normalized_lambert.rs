//! 正規化ランバート反射のBSDFを定義するモジュール。

use std::f32::consts::PI;

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{Bsdf, BsdfSample, SceneId, SurfaceInteraction};

/// Y軸方向を法線方向として、半球状のコサイン充填サンプリングを行う。
fn sample_cosine_hemisphere(uv: glam::Vec2) -> Vector3<Tangent> {
    let r = uv.x.sqrt();
    let theta = 2.0 * PI * uv.y;
    Vector3::new(r * theta.cos(), (1.0 - uv.x).sqrt(), r * theta.sin())
}

/// 正規化ランバート反射のBSDFを表す構造体。
pub struct NormalizedLambert {
    /// 反射スペクトル
    pub rho: Spectrum,
}
impl NormalizedLambert {
    /// 新しいNormalizedLambertを作成する。
    ///
    /// # Arguments
    /// - `rho` - 反射スペクトル
    pub fn new(rho: Spectrum) -> Box<Self> {
        Box::new(Self { rho })
    }
}
impl<Id: SceneId> Bsdf<Id> for NormalizedLambert {
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        _shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample> {
        if wo.y() == 0.0 {
            // woが完全に接戦方向の場合はBsdfをサンプリングしない。
            return None;
        }

        // ランダムな方向をサンプリングする。
        let wi = sample_cosine_hemisphere(uv);
        let wi = if wo.y() < 0.0 {
            // woが法線と逆の方向の場合はwiを反転する。
            Vector3::new(wi.x(), -wi.y(), wi.z())
        } else {
            wi
        };

        if wi.y() == 0.0 {
            // wiが完全に接戦方向の場合はBsdfをサンプリングしない。
            return None;
        }

        // BSDFの値を計算する。
        let f = self.rho.sample(&lambda) / PI;

        // pdfを計算する。
        let cos_theta = wi.y().abs();
        let pdf = cos_theta / PI;

        Some(BsdfSample::Bsdf { f, pdf, wi })
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        _shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> SampledSpectrum {
        if wo.y() == 0.0 || wi.y() == 0.0 {
            // woまたはwiが完全に接戦方向の場合はBSDFを評価しない。
            return SampledSpectrum::zero();
        }

        if wo.y().signum() != wi.y().signum() {
            // woとwiが逆方向の場合はBSDFを評価しない。
            return SampledSpectrum::zero();
        }

        // BSDFの値を計算する。
        let f = self.rho.sample(&lambda) / PI;

        f
    }
}
