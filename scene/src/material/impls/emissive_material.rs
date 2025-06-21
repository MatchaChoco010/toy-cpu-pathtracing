//! 発光（Emissive）マテリアル実装。

use std::sync::Arc;

use math::{VertexNormalTangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    EmissiveSurfaceMaterial, FloatParameter, Material, SpectrumParameter, SurfaceInteraction,
    SurfaceMaterial, UniformEdf,
};

/// 発光のみを行うEmissiveマテリアル。
/// テクスチャ対応の放射輝度と強度パラメータを持つ。
pub struct EmissiveMaterial {
    /// 放射輝度パラメータ
    radiance: SpectrumParameter,
    /// 強度乗算係数パラメータ
    intensity: FloatParameter,
    /// 内部でEDF計算を行う構造体
    edf: UniformEdf,
}

impl EmissiveMaterial {
    /// 新しいEmissiveMaterialを作成する。
    ///
    /// # Arguments
    /// - `radiance` - 放射輝度パラメータ
    /// - `intensity` - 強度乗算係数パラメータ
    pub fn new(radiance: SpectrumParameter, intensity: FloatParameter) -> Material {
        Arc::new(Self {
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

    fn as_emissive_material(&self) -> Option<&dyn EmissiveSurfaceMaterial> {
        Some(self)
    }
}
impl EmissiveSurfaceMaterial for EmissiveMaterial {
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        _wo: Vector3<VertexNormalTangent>,
        light_sample_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> SampledSpectrum {
        let radiance_spectrum = self.radiance.sample(light_sample_point.uv).sample(lambda);
        let intensity_value = self.intensity.sample(light_sample_point.uv);
        self.edf.radiance(&radiance_spectrum, intensity_value)
    }

    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // 平均強度は定数値として計算（テクスチャの場合は近似）
        let radiance_spectrum = match &self.radiance {
            SpectrumParameter::Constant(spectrum) => spectrum.sample(lambda),
            SpectrumParameter::Texture { .. } => {
                // テクスチャの場合は中央の値で近似
                self.radiance
                    .sample(glam::Vec2::new(0.5, 0.5))
                    .sample(lambda)
            }
        };
        let intensity_value = match &self.intensity {
            FloatParameter::Constant(value) => *value,
            FloatParameter::Texture(_) => {
                // テクスチャの場合は中央の値で近似
                self.intensity.sample(glam::Vec2::new(0.5, 0.5))
            }
        };
        self.edf
            .average_intensity(&radiance_spectrum, intensity_value)
    }
}
