//! 点光源のプリミティブの実装のモジュール。

use std::f32::consts::PI;

use math::{Local, Point3, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    DeltaPointLightIntensity, SceneId, SurfaceInteraction,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 点光源のプリミティブの構造体。
pub struct PointLight {
    intensity: f32,
    spectrum: Spectrum,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl PointLight {
    /// 新しい点光源のプリミティブを作成する。
    pub fn new(
        intensity: f32,
        spectrum: Spectrum,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            intensity,
            spectrum,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl<Id: SceneId> Primitive<Id> for PointLight {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }

    fn as_geometry(&self) -> Option<&dyn PrimitiveGeometry<Id>> {
        None
    }

    fn as_geometry_mut(&mut self) -> Option<&mut dyn PrimitiveGeometry<Id>> {
        None
    }

    fn as_light(&self) -> Option<&dyn PrimitiveLight<Id>> {
        Some(self)
    }

    fn as_light_mut(&mut self) -> Option<&mut dyn PrimitiveLight<Id>> {
        Some(self)
    }

    fn as_non_delta_light(&self) -> Option<&dyn PrimitiveNonDeltaLight<Id>> {
        None
    }

    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>> {
        Some(self)
    }

    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for PointLight {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // 放射強度を全天球で積分する。
        4.0 * PI * self.intensity * self.spectrum.sample(lambda)
    }
}
impl<Id: SceneId> PrimitiveDeltaPointLight<Id> for PointLight {
    fn calculate_intensity(
        &self,
        _shading_point: &SurfaceInteraction<Id, Render>,
        lambda: &SampledWavelengths,
    ) -> DeltaPointLightIntensity<Render> {
        let position = &self.local_to_render * Point3::ZERO;
        let intensity = self.intensity * self.spectrum.sample(lambda);
        DeltaPointLightIntensity {
            intensity,
            position,
        }
    }
}
