//! 放射面を含む三角形メッシュのプリミティブの実装のモジュール。

use math::{Bounds, LightSampleContext, Local, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    GeometryIndex, InteractGeometryInfo, Interaction, Intersection, LightSampleRadiance,
    MaterialId, PrimitiveIndex, SceneId,
    geometry::GeometryRepository,
    primitive::{
        PrimitiveRepository,
        traits::{
            Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
            PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
        },
    },
};

/// 放射面を含む三角形メッシュのプリミティブの構造体。
pub struct EmissiveTriangleMesh<Id: SceneId> {
    geometry_index: GeometryIndex<Id>,
    material_id: MaterialId<Id>,
    area_list: Vec<f32>,
    area_sum: f32,
    area_table: Vec<f32>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl<Id: SceneId> EmissiveTriangleMesh<Id> {
    /// 新しい放射面を含む三角形メッシュのプリミティブを作成する。
    pub fn new(
        geometry_repository: &GeometryRepository<Id>,
        primitive_repository: &PrimitiveRepository<Id>,
        geometry_index: GeometryIndex<Id>,
        material_id: MaterialId<Id>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        todo!()
        // シーンからジオメトリを引いて、その三角形の面積を収集する
        // ワールド座標でのboundsを計算する
        // Self {
        //     geometry_index,
        //     material_id,
        //     area_table,
        //     area_sum,
        //     transform,
        // }
    }
}
impl<Id: SceneId> Primitive<Id> for EmissiveTriangleMesh<Id> {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }

    fn as_geometry(&self) -> Option<&dyn PrimitiveGeometry<Id>> {
        Some(self)
    }

    fn as_geometry_mut(&mut self) -> Option<&mut dyn PrimitiveGeometry<Id>> {
        Some(self)
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

    fn as_delta_light(&self) -> Option<&dyn PrimitiveDeltaLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        Some(self)
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for EmissiveTriangleMesh<Id> {
    fn bounds(&self, geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let geometry = geometry_repository.get(self.geometry_index);
        &self.local_to_render * &geometry.bounds()
    }

    fn material_id(&self) -> MaterialId<Id> {
        self.material_id
    }

    fn build_geometry_bvh(&mut self, geometry_repository: &mut GeometryRepository<Id>) {
        let geometry = geometry_repository.get_mut(self.geometry_index);
        geometry.build_bvh();
    }

    fn intersect(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>> {
        let geometry = geometry_repository.get(self.geometry_index);
        let ray = self.local_to_render.inverse() * ray;
        let intersect = geometry.intersect(&ray, t_max);
        intersect.map(|intersection| {
            &self.local_to_render
                * Intersection {
                    t_hit: intersection.t_hit,
                    interaction: Interaction::Surface {
                        position: intersection.position,
                        normal: intersection.normal,
                        shading_normal: intersection.shading_normal,
                        uv: intersection.uv,
                        primitive_index,
                        geometry_info: InteractGeometryInfo::TriangleMesh {
                            triangle_index: intersection.index,
                        },
                    },
                }
        })
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for EmissiveTriangleMesh<Id> {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        todo!()
        // area_sumとmaterialのエネルギーを使って、光源としての総エネルギーを計算する
    }

    fn preprocess(&mut self, _scene_bounds: &Bounds<Render>) {
        todo!()
        // 累積した確率のarea_tableを計算する
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EmissiveTriangleMesh<Id> {
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        // material_repository: &MaterialRepository<Id>,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
        _s: f32,
        _uv: glam::Vec2,
    ) -> LightSampleRadiance<Id, Render> {
        // sを使って三角形をarea_tableからサンプリングする
        // uvを使って三角形から位置要サンプリングして、object_to_renderを使ってレンダリング空間に変換する
        todo!()
    }

    fn pdf_light_sample(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _interaction: &Interaction<Id, Render>,
    ) -> f32 {
        todo!()
    }
}
impl<Id: SceneId> PrimitiveAreaLight<Id> for EmissiveTriangleMesh<Id> {
    fn intersect_radiance(
        &self,
        // material_repository: &MaterialRepository<Id>,
        _interaction: &Interaction<Id, Render>,
        _lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        todo!()
    }
}
