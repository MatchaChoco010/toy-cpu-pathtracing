//! 発光（Emissive）マテリアル実装。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    EmissiveSurfaceMaterial, Material, SceneId, SurfaceInteraction, SurfaceMaterial, UniformEdf,
};

/// 発光のみを行うEmissiveマテリアル。
/// 単一のRGBパラメータから放射輝度を設定する。
pub struct EmissiveMaterial {
    /// 放射輝度スペクトル
    radiance: Spectrum,
    /// 強度乗算係数
    intensity: f32,
    /// 内部でEDF計算を行う構造体
    edf: UniformEdf,
}

impl EmissiveMaterial {
    /// 新しいEmissiveMaterialを作成する。
    ///
    /// # Arguments
    /// - `radiance` - 放射輝度スペクトル
    /// - `intensity` - 強度乗算係数
    pub fn new(radiance: Spectrum, intensity: f32) -> Material {
        std::sync::Arc::new(Self {
            radiance,
            intensity,
            edf: UniformEdf::new(),
        })
    }
}
impl SurfaceMaterial for EmissiveMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl<Id: SceneId> EmissiveSurfaceMaterial<Id> for EmissiveMaterial {
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        _wo: Vector3<Tangent>,
        _light_sample_point: &SurfaceInteraction<Id, Tangent>,
    ) -> SampledSpectrum {
        // テクスチャ座標uvと出射方向woは将来の実装のため保持
        let radiance_spectrum = self.radiance.sample(lambda);
        self.edf.radiance(&radiance_spectrum, self.intensity)
    }

    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        let radiance_spectrum = self.radiance.sample(lambda);
        self.edf
            .average_intensity(&radiance_spectrum, self.intensity)
    }
}
