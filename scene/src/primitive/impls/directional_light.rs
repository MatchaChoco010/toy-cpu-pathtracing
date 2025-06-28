//! 指向性ライトのプリミティブの実装のモジュール。

use std::f32::consts::PI;

use math::{Bounds, Local, Render, Transform, Vector3, World};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    DeltaDirectionalLightIntensity, SceneId, SurfaceInteraction,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight,
    },
};

/// 指向性ライトのプリミティブの構造体。
/// 指向性ライトの方向はローカル座標系のZ+軸方向である。
pub struct DirectionalLight {
    intensity: f32,
    spectrum: Spectrum,
    area: Option<f32>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl DirectionalLight {
    /// 指向性ライトの新しいインスタンスを作成する。
    pub fn new(
        intensity: f32,
        spectrum: Spectrum,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            intensity,
            spectrum,
            area: None,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl<Id: SceneId> Primitive<Id> for DirectionalLight {
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

    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>> {
        None
    }

    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>> {
        Some(self)
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for DirectionalLight {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // シーンの断面積の雑な近似としてシーンのバウンディングスフィアの断面の円の面積をつかう。
        // 垂直放射照度intensityと照射した面積を掛け合わせることで放射束phiを計算する。
        let area = self.area.expect("preprocess not called!");
        self.intensity * area * self.spectrum.sample(lambda)
    }

    fn preprocess(&mut self, bounds: &Bounds<Render>) {
        // phiを計算する際に必要なareaを計算する。
        // 垂直放射照度intensityと照射した面積を掛け合わせることで放射束phiが求まる。
        // ここではシーン全体のバウンディングスフィアの断面を照射面積の近似とする。
        let (_center, radius) = bounds.bounding_sphere();
        let area = PI * radius * radius;
        self.area = Some(area);
    }
}
impl<Id: SceneId> PrimitiveDeltaDirectionalLight<Id> for DirectionalLight {
    fn calculate_intensity(
        &self,
        _shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
    ) -> DeltaDirectionalLightIntensity<Render> {
        let direction = (&self.local_to_render * Vector3::new(0.0, 0.0, 1.0)).normalize();
        let intensity = self.intensity * self.spectrum.sample(lambda);
        DeltaDirectionalLightIntensity {
            intensity,
            direction,
        }
    }
}
