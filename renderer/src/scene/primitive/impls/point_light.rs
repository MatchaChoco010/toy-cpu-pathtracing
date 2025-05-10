//! 点光源のプリミティブの実装のモジュール。

use math::{LightSampleContext, Local, Render, Transform, World};
use spectrum::SampledWavelengths;

use crate::scene::{
    SceneId,
    primitive::{
        LightIrradiance, Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
        PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 点光源のプリミティブの構造体。
pub struct PointLight {
    phi: f32,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl PointLight {
    /// 新しい点光源のプリミティブを作成する。
    pub fn new(intensity: f32, local_to_world: Transform<Local, World>) -> Self {
        Self {
            phi: intensity,
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

    fn as_delta_light(&self) -> Option<&dyn PrimitiveDeltaLight<Id>> {
        Some(self)
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for PointLight {
    fn phi(&self, lambda: &SampledWavelengths) -> f32 {
        self.phi
    }
}
impl<Id: SceneId> PrimitiveDeltaLight<Id> for PointLight {
    fn calculate_irradiance(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
    ) -> LightIrradiance {
        todo!()
    }
}
