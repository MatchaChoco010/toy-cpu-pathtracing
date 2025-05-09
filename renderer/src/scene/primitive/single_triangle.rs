//! 三角形のプリミティブの実装のモジュール。

use glam::{Vec2, Vec3};

use math::{Bounds, Local, Normal, Point3, Ray, Render, Transform, World, intersect_triangle};

use crate::scene::primitive::{Intersection, PrimitiveGeometry, PrimitiveIndex, PrimitiveTrait};
use crate::scene::{GeometryRepository, MaterialId, SceneId};

/// 三角形のプリミティブの構造体。
pub struct SingleTriangle<Id: SceneId> {
    positions: [Point3<Local>; 3],
    normals: [Normal<Local>; 3],
    uvs: [Vec2; 3],
    material_id: MaterialId<Id>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl<Id: SceneId> SingleTriangle<Id> {
    /// 新しい三角形のプリミティブを作成する。
    pub fn new(
        positions: [Point3<Local>; 3],
        normals: [Normal<Local>; 3],
        uvs: [Vec2; 3],
        material_id: MaterialId<Id>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            positions,
            normals,
            uvs,
            material_id,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}

impl<Id: SceneId> PrimitiveTrait for SingleTriangle<Id> {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for SingleTriangle<Id> {
    fn bounds(&self, _geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for position in &self.positions {
            let point = &self.local_to_render * position;
            min = min.min(point.to_vec3());
            max = max.max(point.to_vec3());
        }
        let min = Point3::from(min);
        let max = Point3::from(max);
        Bounds::new(min, max)
    }

    fn material_id(&self) -> MaterialId<Id> {
        self.material_id
    }

    fn intersect(
        &self,
        _primitive_index: PrimitiveIndex<Id>,
        _geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>> {
        let ray = &self.local_to_render.inverse() * ray;
        let hit = match intersect_triangle(&ray, t_max, self.positions) {
            Some(hit) => hit,
            None => return None,
        };

        let shading_normal = Normal::<Local>::from(
            self.normals[0].to_vec3() * hit.barycentric[0]
                + self.normals[1].to_vec3() * hit.barycentric[1]
                + self.normals[2].to_vec3() * hit.barycentric[2],
        );
        let uv = Vec2::new(
            self.uvs[0].x * hit.barycentric[0]
                + self.uvs[1].x * hit.barycentric[1]
                + self.uvs[2].x * hit.barycentric[2],
            self.uvs[0].y * hit.barycentric[0]
                + self.uvs[1].y * hit.barycentric[1]
                + self.uvs[2].y * hit.barycentric[2],
        );

        Some(Intersection {
            t_hit: hit.t_hit,
            interaction: crate::scene::Interaction::Surface {
                position: &self.local_to_render * hit.position,
                normal: &self.local_to_render * hit.normal,
                shading_normal: &self.local_to_render * shading_normal,
                uv,
                primitive_index: _primitive_index,
                geometry_info: super::GeometryInfo::TriangleMesh {
                    triangle_index: 0, // TODO: 三角形のインデックスを取得する
                },
            },
        })
    }
}
