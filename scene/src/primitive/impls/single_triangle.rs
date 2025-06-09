//! 三角形のプリミティブの実装のモジュール。

use std::marker::PhantomData;

use math::{Bounds, Local, Normal, Point3, Ray, Render, Transform, World, intersect_triangle};

use crate::{
    InteractGeometryInfo, Intersection, Material, PrimitiveIndex, SceneId, SurfaceInteraction,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 三角形のプリミティブの構造体。
pub struct SingleTriangle<Id: SceneId> {
    positions: [Point3<Local>; 3],
    normals: [Normal<Local>; 3],
    uvs: [glam::Vec2; 3],
    material: Material,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
    _phantom: PhantomData<Id>,
}
impl<Id: SceneId> SingleTriangle<Id> {
    /// 新しい三角形のプリミティブを作成する。
    pub fn new(
        positions: [Point3<Local>; 3],
        normals: [Normal<Local>; 3],
        uvs: [glam::Vec2; 3],
        material: Material,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        Self {
            positions,
            normals,
            uvs,
            material,
            local_to_world,
            local_to_render: Transform::identity(),
            _phantom: PhantomData,
        }
    }
}
impl<Id: SceneId> Primitive<Id> for SingleTriangle<Id> {
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
        None
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for SingleTriangle<Id> {
    fn bounds(&self, _geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
        for position in &self.positions {
            let point = &self.local_to_render * position;
            min = min.min(point.to_vec3());
            max = max.max(point.to_vec3());
        }
        let min = Point3::from(min);
        let max = Point3::from(max);
        Bounds::new(min, max)
    }

    fn surface_material(&self) -> Material {
        self.material.clone()
    }

    fn intersect(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        _geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>> {
        let wo = -ray.dir;
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
        let uv = glam::Vec2::new(
            self.uvs[0].x * hit.barycentric[0]
                + self.uvs[1].x * hit.barycentric[1]
                + self.uvs[2].x * hit.barycentric[2],
            self.uvs[0].y * hit.barycentric[0]
                + self.uvs[1].y * hit.barycentric[1]
                + self.uvs[2].y * hit.barycentric[2],
        );

        let edge1 = self.positions[0].vector_to(self.positions[1]);
        let edge2 = self.positions[0].vector_to(self.positions[2]);
        let delta_uv1 = self.uvs[1] - self.uvs[0];
        let delta_uv2 = self.uvs[2] - self.uvs[0];
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = r * (edge1 * delta_uv2.y - edge2 * delta_uv1.y);
        let tangent = tangent.normalize();

        Some(Intersection {
            t_hit: hit.t_hit,
            wo,
            interaction: SurfaceInteraction {
                position: &self.local_to_render * hit.position,
                normal: &self.local_to_render * hit.normal,
                shading_normal: &self.local_to_render * shading_normal,
                tangent: &self.local_to_render * tangent,
                uv,
                material: self.material.clone(),
                primitive_index,
                geometry_info: InteractGeometryInfo::None,
            },
        })
    }

    fn intersect_p(
        &self,
        _primitive_index: PrimitiveIndex<Id>,
        _geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> bool {
        let ray = &self.local_to_render.inverse() * ray;
        match intersect_triangle(&ray, t_max, self.positions) {
            Some(_) => true,
            None => false,
        }
    }
}
