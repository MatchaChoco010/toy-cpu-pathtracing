//! マテリアルのSurfaceの発光のEDFを定義するモジュール。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{SceneId, SurfaceInteraction};

/// EDFのトレイト。
pub trait Edf<Id: SceneId>: Send + Sync {
    /// EDFの放射輝度の計算を行う。
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        emissive_point: SurfaceInteraction<Id, Tangent>,
        wo: Vector3<Tangent>,
    ) -> Option<SampledSpectrum>;

    /// 平均放射発散度を取得する。
    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum;
}
