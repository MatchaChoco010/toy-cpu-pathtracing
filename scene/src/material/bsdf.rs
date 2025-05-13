//! マテリアルのSurfaceの反射のBSDFを定義するモジュール。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{SceneId, SurfaceInteraction};

mod normalized_lambert;

pub use normalized_lambert::NormalizedLambert;

/// BSDFのサンプリングの結果を表す列挙子。
pub enum BsdfSample {
    Bsdf {
        /// 波長ごとにサンプリングしたBSDFの値。
        f: SampledSpectrum,
        /// サンプリングした際のPDF。
        pdf: f32,
        /// サンプリングした方向。
        wi: Vector3<Tangent>,
    },
    Specular {
        /// 波長ごとにサンプリングした反射率/透過率の値。
        f: SampledSpectrum,
        /// サンプリングした方向。
        wi: Vector3<Tangent>,
    },
}

/// BSDFのトレイト。
pub trait Bsdf<Id: SceneId>: Send + Sync {
    /// BSDFのサンプリングを行う。
    ///
    /// # Arguments
    /// - `uv` - サンプリングの乱数。
    /// - `lambda` - サンプリングする波長。
    /// - `wo` - 出射方向。
    /// - `shading_point` - シェーディング点の情報。
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample>;
}
