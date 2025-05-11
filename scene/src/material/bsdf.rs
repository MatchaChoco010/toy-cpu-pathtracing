//! マテリアルのSurfaceの反射のBSDFを定義するモジュール。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{SceneId, SurfaceInteraction};

/// BSDFのサンプリングの結果を表す構造体。
pub struct BsdfSample {
    /// 波長ごとにサンプリングしたBSDFの値。
    pub f: SampledSpectrum,
    /// サンプリングした際のPDF。
    pub pdf: f32,
}

/// BSDFのトレイト。
pub trait Bsdf<Id: SceneId>: Send + Sync {
    /// BSDFのサンプリングを行う。
    fn sample(
        &self,
        lambda: SampledWavelengths,
        wi: Vector3<Tangent>,
        wo: Vector3<Tangent>,
        shading_point: SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample>;
}
