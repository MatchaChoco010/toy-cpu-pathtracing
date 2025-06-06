//! 一様EDFを定義するモジュール。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{Edf, SceneId, SurfaceInteraction};

/// 一様EDFを表す構造体。
pub struct Uniform {
    /// 放射輝度スペクトル。
    pub radiance: Spectrum,
    /// 強度。
    pub intensity: f32,
}
impl Uniform {
    /// 新しいUniformなEDFを作成する。
    ///
    /// # Arguments
    /// - `radiance` - 放射輝度スペクトル
    /// - `intensity` - 強度
    pub fn new(radiance: Spectrum, intensity: f32) -> Box<Self> {
        Box::new(Self {
            radiance,
            intensity,
        })
    }
}
impl<Id: SceneId> Edf<Id> for Uniform {
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        _emissive_point: SurfaceInteraction<Id, Tangent>,
        _wo: Vector3<Tangent>,
    ) -> Option<SampledSpectrum> {
        Some(self.radiance.sample(lambda) * self.intensity)
    }

    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        self.radiance.sample(lambda) * self.intensity
    }
}
