//! マテリアルのSurfaceの発光のEDFを定義するモジュール。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{SceneId, SurfaceInteraction};

mod uniform;

pub use uniform::Uniform;

/// EDFのトレイト。
pub trait Edf<Id: SceneId>: Send + Sync {
    /// EDFの放射輝度の計算を行う。
    ///
    /// # Arguments
    /// - `lambda` - サンプリングする波長。
    /// - `emissive_point` - 発光点上の情報。
    /// - `wo` - 出射方向。
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        emissive_point: SurfaceInteraction<Id, Tangent>,
        wo: Vector3<Tangent>,
    ) -> Option<SampledSpectrum>;

    /// 指定された波長に対する平均放射発散度を取得する。
    /// uv座標やwoに応じて放射輝度が変わるEDFの場合に、その平均の放射輝度を返す。
    ///
    /// # Arguments
    /// - `lambda` - サンプリングする波長。
    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum;
}
