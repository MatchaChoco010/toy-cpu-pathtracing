//! 環境ライトのプリミティブの実装のモジュール。

use std::path::Path;

use math::{Local, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths, SpectrumTrait};

use crate::{
    AreaLightSampleRadiance, Intersection, SceneId, SurfaceInteraction,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 環境ライトのプリミティブの構造体。
pub struct EnvironmentLight {
    intensity: f32,
    integrated_spectrum: Box<dyn SpectrumTrait>,
    // texture:
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl EnvironmentLight {
    /// 新しい環境ライトのプリミティブを作成する。
    pub fn new(
        _intensity: f32,
        _path: impl AsRef<Path>,
        _transform: Transform<Local, World>,
    ) -> Self {
        todo!()
        // テクスチャを読んで、テクスチャのpdfとかも作る
        // Self {
        //     intensity,
        //     phi,
        //     transform,
        // }
    }
}
impl<Id: SceneId> Primitive<Id> for EnvironmentLight {
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
        Some(self)
    }

    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>> {
        None
    }

    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        Some(self)
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for EnvironmentLight {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        self.intensity * self.integrated_spectrum.sample(lambda)
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EnvironmentLight {
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        _shading_point: &SurfaceInteraction<Render>,
        _lambda: &SampledWavelengths,
        _s: f32,
        _uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render> {
        todo!()
    }

    fn pdf_light_sample(&self, _interaction: &Intersection<Id, Render>) -> f32 {
        todo!()
    }
}
impl<Id: SceneId> PrimitiveInfiniteLight<Id> for EnvironmentLight {
    fn direction_radiance(
        &self,
        _ray: &Ray<Render>,
        _lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        todo!()
    }
}
