//! 放射面を含む三角形のプリミティブの実装のモジュール。

use std::sync::Arc;

use math::{
    Bounds, Local, Normal, Point3, Ray, Render, Transform, Vector3, World, intersect_triangle,
};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    AreaLightSampleRadiance, InteractGeometryInfo, Intersection, PrimitiveIndex, SceneId,
    SurfaceInteraction, SurfaceMaterial,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 放射面を含む三角形のプリミティブの構造体。
pub struct EmissiveSingleTriangle<Id: SceneId> {
    positions: [Point3<Local>; 3],
    normals: [Normal<Local>; 3],
    uvs: [glam::Vec2; 3],
    material: Arc<SurfaceMaterial<Id>>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
    area: f32,
}
impl<Id: SceneId> EmissiveSingleTriangle<Id> {
    /// 新しい放射面を含む三角形のプリミティブを作成する。
    pub fn new(
        positions: [Point3<Local>; 3],
        normals: [Normal<Local>; 3],
        uvs: [glam::Vec2; 3],
        material: Arc<SurfaceMaterial<Id>>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        // ワールド空間での面積を計算しておく。
        let p0 = &local_to_world * positions[0];
        let p1 = &local_to_world * positions[1];
        let p2 = &local_to_world * positions[2];
        let e0 = p0.vector_to(p1);
        let e1 = p0.vector_to(p2);
        let area = e0.cross(e1).length() * 0.5;

        Self {
            positions,
            normals,
            uvs,
            material,
            local_to_world,
            local_to_render: Transform::identity(),
            area,
        }
    }
}
impl<Id: SceneId> Primitive<Id> for EmissiveSingleTriangle<Id> {
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

    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>> {
        None
    }

    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        Some(self)
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        None
    }
}
impl<Id: SceneId> PrimitiveGeometry<Id> for EmissiveSingleTriangle<Id> {
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

    fn surface_material(&self) -> &SurfaceMaterial<Id> {
        &self.material
    }

    fn intersect(
        &self,
        _primitive_index: PrimitiveIndex<Id>,
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
                primitive_index: _primitive_index,
                geometry_info: InteractGeometryInfo::TriangleMesh {
                    triangle_index: 0, // TODO: 三角形のインデックスを取得する
                },
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
impl<Id: SceneId> PrimitiveLight<Id> for EmissiveSingleTriangle<Id> {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // マテリアルの放射発散度の平均値に三角形の面積を掛けて全体の放射束とする。
        self.material
            .edf
            .as_ref()
            .unwrap()
            .average_intensity(lambda)
            * self.area
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EmissiveSingleTriangle<Id> {
    fn sample_radiance(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        _geometry_repository: &GeometryRepository<Id>,
        shading_point: &SurfaceInteraction<Id, Render>,
        lambda: &SampledWavelengths,
        _s: f32,
        uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Id, Render> {
        // サンプリングする点のbarycentric座標を計算する。
        let b0;
        let b1;
        if uv[0] < uv[1] {
            b0 = uv[0] / 2.0;
            b1 = uv[1] - b0;
        } else {
            b1 = uv[0] / 2.0;
            b0 = uv[1] - b1;
        }
        let b2 = 1.0 - b0 - b1;
        let barycentric = [b0, b1, b2];

        // サンプリングする点のRender空間での位置を計算する。
        let p0 = &self.local_to_render * self.positions[0];
        let p1 = &self.local_to_render * self.positions[1];
        let p2 = &self.local_to_render * self.positions[2];
        let p = Point3::from(
            p0.to_vec3() * barycentric[0]
                + p1.to_vec3() * barycentric[1]
                + p2.to_vec3() * barycentric[2],
        );

        // サンプリングする点のRender空間での幾何法線を計算する。
        let normal = Normal::from(
            p1.vector_to(p0)
                .cross(p2.vector_to(p0))
                .normalize()
                .to_vec3(),
        );

        // サンプリングする点のRender空間でのShading法線を計算する。
        let n0 = &self.local_to_render * self.normals[0];
        let n1 = &self.local_to_render * self.normals[1];
        let n2 = &self.local_to_render * self.normals[2];
        let shading_normal = Normal::from(
            n0.to_vec3() * barycentric[0]
                + n1.to_vec3() * barycentric[1]
                + n2.to_vec3() * barycentric[2],
        );

        // サンプリングする点のRender空間でのTangentを計算する。
        let edge1 = p0.vector_to(p1);
        let edge2 = p0.vector_to(p2);
        let delta_uv1 = self.uvs[1] - self.uvs[0];
        let delta_uv2 = self.uvs[2] - self.uvs[0];
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = r * (edge1 * delta_uv2.y - edge2 * delta_uv1.y);
        let tangent = tangent.normalize();

        // tangentを再度正規直行化する。
        let tangent = Vector3::from(
            tangent.to_vec3()
                - shading_normal.to_vec3().dot(tangent.to_vec3()) * shading_normal.to_vec3(),
        )
        .normalize();

        // Render空間からサンプルした点のTangent空間への変換Transformを計算する。
        let render_to_tangent = Transform::from_shading_normal_tangent(&shading_normal, &tangent);

        // サンプリングする光の出力方向を計算する。
        let wo = p.vector_to(shading_point.position).normalize();

        // マテリアルから放射輝度を取得する。
        let radiance = self
            .material
            .edf
            .as_ref()
            .unwrap()
            .radiance(
                lambda,
                &render_to_tangent * shading_point,
                &render_to_tangent * wo,
            )
            .unwrap_or(SampledSpectrum::zero());

        // 面積を計算して一様サンプリングしたときのpdfを計算する。
        let pdf = 1.0 / self.area;

        // 幾何項を計算する。
        let distance = p.distance(shading_point.position);
        let g = shading_point.normal.dot(-wo).abs() * normal.dot(wo).abs() / (distance * distance);

        // 方向要素のpdfを計算する。
        let pdf_dir = pdf * (distance * distance) / normal.dot(wo).abs();

        AreaLightSampleRadiance {
            radiance,
            pdf,
            g,
            pdf_dir,
            interaction: SurfaceInteraction {
                position: p,
                normal,
                shading_normal,
                tangent,
                uv,
                material: self.material.clone(),
                primitive_index,
                geometry_info: InteractGeometryInfo::None,
            },
        }
    }

    fn pdf_light_sample(&self, _interaction: &SurfaceInteraction<Id, Render>) -> f32 {
        // 一様サンプリングしたときのpdfを計算する。
        1.0 / self.area
    }
}
impl<Id: SceneId> PrimitiveAreaLight<Id> for EmissiveSingleTriangle<Id> {
    fn intersect_radiance(
        &self,
        shading_point: &SurfaceInteraction<Id, Render>,
        interaction: &SurfaceInteraction<Id, Render>,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        // 交差した光源上の点のTangent空間への変換Transformを計算する。
        let render_to_tangent = Transform::from_shading_normal_tangent(
            &interaction.shading_normal,
            &interaction.tangent,
        );

        // サンプリングした点のTangent空間での方向を計算する。
        let wo = interaction
            .position
            .vector_to(shading_point.position)
            .normalize();

        let radiance = self
            .material
            .edf
            .as_ref()
            .unwrap()
            .radiance(
                lambda,
                &render_to_tangent * interaction,
                &render_to_tangent * wo,
            )
            .unwrap_or(SampledSpectrum::zero());

        radiance
    }
}
