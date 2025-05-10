//! 三角形メッシュのプリミティブの実装のモジュール。

use math::{Bounds, Local, Ray, Render, Transform, World};

use crate::scene::{
    Geometry, GeometryIndex, GeometryRepository, MaterialId, SceneId,
    primitive::{
        GeometryInfo, Interaction, Intersection, PrimitiveGeometry, PrimitiveIndex, PrimitiveTrait,
    },
};

/// 三角形メッシュのプリミティブの構造体。
pub struct TriangleMesh<Id: SceneId> {
    geometry_index: GeometryIndex<Id>,
    material_id: MaterialId<Id>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl<Id: SceneId> TriangleMesh<Id> {
    /// 新しい三角形メッシュのプリミティブを作成する。
    pub fn new(
        geometry_index: GeometryIndex<Id>,
        material_id: MaterialId<Id>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            geometry_index,
            material_id,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl<Id: SceneId> PrimitiveTrait for TriangleMesh<Id> {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for TriangleMesh<Id> {
    fn bounds(&self, geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let geometry = geometry_repository.get(self.geometry_index);
        let triangle_mesh = match geometry {
            Geometry::TriangleMesh(triangle_mesh) => triangle_mesh,
        };
        &self.local_to_render * &triangle_mesh.bounds()
    }

    fn material_id(&self) -> MaterialId<Id> {
        self.material_id
    }

    fn build_geometry_bvh(&mut self, geometry_repository: &mut GeometryRepository<Id>) {
        let geometry = geometry_repository.get_mut(self.geometry_index);
        let triangle_mesh = match geometry {
            Geometry::TriangleMesh(triangle_mesh) => triangle_mesh,
        };
        triangle_mesh.build_bvh();
    }

    fn intersect(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>> {
        let geometry = geometry_repository.get(self.geometry_index);
        let triangle_mesh = match geometry {
            Geometry::TriangleMesh(triangle_mesh) => triangle_mesh,
        };
        let ray = self.local_to_render.inverse() * ray;
        let intersect = triangle_mesh.intersect(&ray, t_max);
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
                        geometry_info: GeometryInfo::TriangleMesh {
                            triangle_index: intersection.triangle_index,
                        },
                    },
                }
        })
    }
}
