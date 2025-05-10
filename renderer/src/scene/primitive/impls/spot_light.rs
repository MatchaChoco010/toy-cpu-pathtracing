//! スポットライトのプリミティブの実装のモジュール。

use math::{LightSampleContext, Local, Render, Transform, World};
use spectrum::SampledWavelengths;

use crate::scene::{
    SceneId,
    primitive::{
        LightIrradiance, Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
        PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// スポットライトのプリミティブの構造体。
/// スポットライトの方向はローカル座標系のZ+軸方向である。
pub struct SpotLight {
    angle: f32,
    intensity: f32,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl SpotLight {
    /// 新しいスポットライトのプリミティブを作成する。
    pub fn new(intensity: f32, angle: f32, local_to_world: Transform<Local, World>) -> Self {
        Self {
            angle,
            intensity,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl<Id: SceneId> Primitive<Id> for SpotLight {
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
impl<Id: SceneId> PrimitiveLight<Id> for SpotLight {
    fn phi(&self, lambda: &SampledWavelengths) -> f32 {
        todo!()
    }
}
impl<Id: SceneId> PrimitiveDeltaLight<Id> for SpotLight {
    fn calculate_irradiance(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
    ) -> LightIrradiance {
        todo!()
    }
}
