//! スポットライトのプリミティブの実装のモジュール。

use std::f32::consts::PI;

use math::{LightSampleContext, Local, Point3, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    LightIrradiance, SceneId,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
        PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// スポットライトのプリミティブの構造体。
/// スポットライトの方向はローカル座標系のZ+軸方向である。
pub struct SpotLight {
    angle_inner: f32,
    angle_outer: f32,
    intensity: f32,
    spectrum: Box<dyn Spectrum>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl SpotLight {
    /// 新しいスポットライトのプリミティブを作成する。
    pub fn new(
        angle_inner: f32,
        angle_outer: f32,
        intensity: f32,
        spectrum: Box<dyn Spectrum>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            angle_inner,
            angle_outer,
            intensity,
            spectrum,
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
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // スポットライトの放射束を計算する。
        // angleが0からangle_innerの間でのフォールオフの積分値は(1.0 - cos(angle_inner))。
        // angleがangle_innerからangle_outerの間でsmoothstepのフォールオフ積分値は
        // (cos(angle_inner) - cos(angle_outer)) / 2。
        // それらの積分値の和を0からPIまでのライトの積分2 * PI * sampleに掛け合わせる。
        self.intensity
            * self.spectrum.sample(lambda)
            * 2.0
            * PI
            * ((1.0 - self.angle_inner.cos())
                + (self.angle_inner.cos() - self.angle_outer.cos()) / 2.0)
    }
}
impl<Id: SceneId> PrimitiveDeltaLight<Id> for SpotLight {
    fn calculate_irradiance(
        &self,
        light_sample_context: &LightSampleContext<Render>,
        lambda: &SampledWavelengths,
    ) -> LightIrradiance {
        // Render空間でのライトの方向とcos成分を計算する。
        let position = &self.local_to_render * Point3::ZERO;
        let distance_vec = light_sample_context.position.vector_to(position);
        let wi = distance_vec.normalize();
        let cos_theta = wi.dot(light_sample_context.normal);

        // angle_innerとangle_outerの間でcos成分でsmoothstep補間した値をスポットライトの減衰とする。
        fn smoothstep(a: f32, b: f32, t: f32) -> f32 {
            let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        }
        let falloff = smoothstep(self.angle_outer, self.angle_inner, cos_theta);

        // 放射照度を計算する。
        let irradiance = self.intensity * self.spectrum.sample(lambda) * cos_theta * falloff;
        LightIrradiance { irradiance }
    }
}
