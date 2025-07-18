//! 放射面を含む三角形のプリミティブの実装のモジュール。

use std::marker::PhantomData;

use math::{Bounds, Local, Normal, Point3, Ray, Render, Transform, World, intersect_triangle};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    AreaLightSampleRadiance, InteractGeometryInfo, Intersection, Material, PrimitiveIndex, SceneId,
    SurfaceInteraction,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight,
    },
};

/// 放射面を含む三角形のプリミティブの構造体。
pub struct EmissiveSingleTriangle<Id: SceneId> {
    positions: [Point3<Local>; 3],
    normals: [Normal<Local>; 3],
    uvs: [glam::Vec2; 3],
    material: Material,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
    area: f32,
    _phantom: PhantomData<Id>,
}
impl<Id: SceneId> EmissiveSingleTriangle<Id> {
    /// 新しい放射面を含む三角形のプリミティブを作成する。
    pub fn new(
        positions: [Point3<Local>; 3],
        normals: [Normal<Local>; 3],
        uvs: [glam::Vec2; 3],
        material: Material,
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
            _phantom: PhantomData,
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
        let transformed_positions: Vec<_> = self
            .positions
            .iter()
            .map(|pos| &self.local_to_render * pos)
            .collect();
        let (min, max) = Point3::min_max_from_points(&transformed_positions);
        Bounds::new(min, max)
    }

    fn surface_material(&self) -> Material {
        self.material.clone()
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
        let hit = intersect_triangle(&ray, t_max, self.positions)?;

        let shading_normal = Normal::interpolate_barycentric(
            &self.normals[0],
            &self.normals[1],
            &self.normals[2],
            hit.barycentric,
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
            primitive_index: _primitive_index,
            geometry_info: InteractGeometryInfo::TriangleMesh {
                triangle_index: 0, // TODO: 三角形のインデックスを取得する
            },
            interaction: SurfaceInteraction {
                position: &self.local_to_render * hit.position,
                normal: &self.local_to_render * hit.normal,
                shading_normal: &self.local_to_render * shading_normal,
                tangent: &self.local_to_render * tangent,
                uv,
                material: self.material.clone(),
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
        intersect_triangle(&ray, t_max, self.positions).is_some()
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for EmissiveSingleTriangle<Id> {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // マテリアルの放射発散度の平均値に三角形の面積を掛けて全体の放射束とする。
        self.material
            .as_emissive_material()
            .unwrap()
            .average_intensity(lambda)
            * self.area
    }
}
impl<Id: SceneId> PrimitiveAreaLight<Id> for EmissiveSingleTriangle<Id> {
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
        _s: f32,
        uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render> {
        // サンプリングする点のbarycentric座標を計算する。
        let b0;
        let b1;
        if uv[0] < uv[1] {
            b0 = uv[0] / 2.0;
            b1 = uv[1] - b0;
        } else {
            b1 = uv[1] / 2.0;
            b0 = uv[0] - b1;
        }
        let b2 = 1.0 - b0 - b1;
        let barycentric = [b0, b1, b2];

        // サンプリングする点のRender空間での位置を計算する。
        let p0 = &self.local_to_render * self.positions[0];
        let p1 = &self.local_to_render * self.positions[1];
        let p2 = &self.local_to_render * self.positions[2];
        let p = Point3::interpolate_barycentric(&p0, &p1, &p2, barycentric);

        // サンプリングする点のRender空間での幾何法線を計算する。
        let normal = p0
            .vector_to(p1)
            .cross(p0.vector_to(p2))
            .normalize()
            .to_normal();

        // サンプリングする点のRender空間でのShading法線を計算する。
        let n0 = &self.local_to_render * self.normals[0];
        let n1 = &self.local_to_render * self.normals[1];
        let n2 = &self.local_to_render * self.normals[2];
        let shading_normal = Normal::interpolate_barycentric(&n0, &n1, &n2, barycentric);

        // サンプリングする点のRender空間でのTangentを計算する。
        let edge1 = p0.vector_to(p1);
        let edge2 = p0.vector_to(p2);
        let delta_uv1 = self.uvs[1] - self.uvs[0];
        let delta_uv2 = self.uvs[2] - self.uvs[0];
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = r * (edge1 * delta_uv2.y - edge2 * delta_uv1.y);
        let tangent = tangent.normalize();

        // tangentを再度正規直交化する。
        let tangent = shading_normal.orthogonalize_vector(&tangent);

        // Render空間からサンプルした点のTangent空間への変換Transformを計算する。
        let render_to_tangent = Transform::from_shading_normal_tangent(&shading_normal, &tangent);

        // シェーディングポイントに対する光源の方向ベクトルを計算する。
        let wi = shading_point.position.vector_to(p).normalize();

        // サンプリングした光源上の点のSurfaceInteractionを作成する。
        let light_sample_point = SurfaceInteraction {
            position: p,
            normal,
            shading_normal,
            tangent,
            uv,
            material: self.material.clone(),
        };

        // マテリアルから放射輝度を取得する。
        let radiance = self.material.as_emissive_material().unwrap().radiance(
            lambda,
            &render_to_tangent * -wi,
            &(render_to_tangent * light_sample_point),
        );

        // 面積を計算して一様サンプリングしたときのpdfを計算する。
        let pdf = 1.0 / self.area;

        // 方向要素のpdfを計算する。
        let distance = p.distance(shading_point.position);
        let pdf_dir = pdf * (distance * distance) / (normal.dot(-wi).abs()).max(1e-8);

        AreaLightSampleRadiance {
            radiance,
            pdf,
            light_normal: normal,
            pdf_dir,
            interaction: SurfaceInteraction {
                position: p,
                normal,
                shading_normal,
                tangent,
                uv,
                material: self.material.clone(),
            },
        }
    }

    fn intersect_radiance(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        interaction: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        // 交差した光源上の点のTangent空間への変換Transformを計算する。
        let render_to_tangent = interaction.shading_transform();

        // サンプリングした点のTangent空間での方向を計算する。
        let wo = interaction
            .position
            .vector_to(shading_point.position)
            .normalize();

        self.material.as_emissive_material().unwrap().radiance(
            lambda,
            &render_to_tangent * wo,
            &(render_to_tangent * interaction),
        )
    }

    fn pdf_light_sample(&self, _intersection: &Intersection<Id, Render>) -> f32 {
        1.0 / self.area
    }
}
