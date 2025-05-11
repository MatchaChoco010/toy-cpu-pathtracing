//! 三角形メッシュのプリミティブの実装のモジュール。

use std::sync::Arc;

use math::{Bounds, Local, Ray, Render, Transform, World};

use crate::{
    GeometryIndex, InteractGeometryInfo, Intersection, PrimitiveIndex, SceneId, SurfaceInteraction,
    SurfaceMaterial,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
        PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 三角形メッシュのプリミティブの構造体。
pub struct TriangleMesh<Id: SceneId> {
    geometry_index: GeometryIndex<Id>,
    material: Arc<SurfaceMaterial<Id>>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl<Id: SceneId> TriangleMesh<Id> {
    /// 新しい三角形メッシュのプリミティブを作成する。
    pub fn new(
        geometry_index: GeometryIndex<Id>,
        material: Arc<SurfaceMaterial<Id>>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            geometry_index,
            material,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl<Id: SceneId> Primitive<Id> for TriangleMesh<Id> {
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
        None
    }

    fn as_light_mut(&mut self) -> Option<&mut dyn PrimitiveLight<Id>> {
        None
    }

    fn as_non_delta_light(&self) -> Option<&dyn PrimitiveNonDeltaLight<Id>> {
        None
    }

    fn as_delta_light(&self) -> Option<&dyn PrimitiveDeltaLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for TriangleMesh<Id> {
    fn bounds(&self, geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let geometry = geometry_repository.get(self.geometry_index);
        &self.local_to_render * geometry.bounds()
    }

    fn surface_material(&self) -> &SurfaceMaterial<Id> {
        &self.material
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
                    interaction: SurfaceInteraction {
                        position: intersection.position,
                        normal: intersection.normal,
                        shading_normal: intersection.shading_normal,
                        tangent: intersection.tangent,
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
