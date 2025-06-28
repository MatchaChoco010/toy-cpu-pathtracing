//! 放射面を含む三角形メッシュのプリミティブの実装のモジュール。

use math::{Bounds, Local, Normal, Point3, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    AreaLightSampleRadiance, GeometryIndex, InteractGeometryInfo, Intersection, Material,
    PrimitiveIndex, SceneId, SurfaceInteraction,
    geometry::{GeometryRepository, impls::TriangleMesh},
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 放射面を含む三角形メッシュのプリミティブの構造体。
pub struct EmissiveTriangleMesh<Id: SceneId> {
    geometry_index: GeometryIndex<Id>,
    material: Material,
    area_list: Vec<f32>,
    area_sum: f32,
    area_table: Vec<f32>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl<Id: SceneId> EmissiveTriangleMesh<Id> {
    /// 新しい放射面を含む三角形メッシュのプリミティブを作成する。
    pub fn new(
        triangle_mesh: &TriangleMesh<Id>,
        geometry_index: GeometryIndex<Id>,
        material: Material,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        // ワールド空間での面積などを計算しておく。

        // 三角形の面積のリスト。
        let mut area_list = vec![];
        for index in triangle_mesh.indices.chunks(3) {
            let p0 = &local_to_world * triangle_mesh.positions[index[0] as usize];
            let p1 = &local_to_world * triangle_mesh.positions[index[1] as usize];
            let p2 = &local_to_world * triangle_mesh.positions[index[2] as usize];
            let e0 = p1.vector_to(p0);
            let e1 = p2.vector_to(p0);
            let area = (e0.cross(e1)).length() * 0.5;
            area_list.push(area);
        }

        // 面積の総和と面積の重み付きでサンプリングするための累積確率のテーブルを作成する。
        let mut area_sum = 0.0;
        let mut area_table = vec![];
        for area in &area_list {
            area_sum += area;
            area_table.push(area_sum);
        }
        for item in &mut area_table {
            *item /= area_sum;
        }

        Self {
            geometry_index,
            material,
            area_list,
            area_sum,
            area_table,
            local_to_world,
            local_to_render: Transform::identity(),
        }
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
impl<Id: SceneId> PrimitiveGeometry<Id> for EmissiveTriangleMesh<Id> {
    fn bounds(&self, geometry_repository: &GeometryRepository<Id>) -> Bounds<Render> {
        let geometry = geometry_repository.get(self.geometry_index);
        &self.local_to_render * &geometry.bounds()
    }

    fn surface_material(&self) -> Material {
        self.material.clone()
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
                    wo: -ray.dir,
                    primitive_index,
                    geometry_info: InteractGeometryInfo::TriangleMesh {
                        triangle_index: intersection.index,
                    },
                    interaction: SurfaceInteraction {
                        position: intersection.position,
                        normal: intersection.normal,
                        shading_normal: intersection.shading_normal,
                        tangent: intersection.tangent,
                        uv: intersection.uv,
                        material: self.material.clone(),
                    },
                }
        })
    }

    fn intersect_p(
        &self,
        _primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> bool {
        let geometry = geometry_repository.get(self.geometry_index);
        let ray = self.local_to_render.inverse() * ray;
        geometry.intersect_p(&ray, t_max)
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for EmissiveTriangleMesh<Id> {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        // 面積の総和を使って、放射面のスペクトル放射束を計算する。
        self.material
            .as_emissive_material()
            .unwrap()
            .average_intensity(lambda)
            * self.area_sum
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EmissiveTriangleMesh<Id> {
    fn sample_radiance(
        &self,
        geometry_repository: &GeometryRepository<Id>,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
        s: f32,
        uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render> {
        // sを使って三角形をarea_tableからサンプリングする.
        let mut index = 0;
        for (i, area) in self.area_table.iter().enumerate() {
            if s < *area {
                index = i;
                break;
            }
        }
        let geometry = geometry_repository.get(self.geometry_index);
        let triangle_mesh = geometry
            .as_any()
            .downcast_ref::<TriangleMesh<Id>>()
            .unwrap();

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
        let p0 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3] as usize];
        let p1 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3 + 1] as usize];
        let p2 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3 + 2] as usize];
        let p = Point3::interpolate_barycentric(&p0, &p1, &p2, barycentric);

        // サンプリングする点のRender空間での幾何法線を計算する。
        let normal = p0
            .vector_to(p1)
            .cross(p0.vector_to(p2))
            .normalize()
            .to_normal();

        // サンプリングする点のRender空間でのShading法線を計算する。
        let n0 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3] as usize];
        let n1 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3 + 1] as usize];
        let n2 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3 + 2] as usize];
        let shading_normal = Normal::interpolate_barycentric(&n0, &n1, &n2, barycentric);

        // サンプリングする点のUV座標を計算する。
        let uvs = if triangle_mesh.uvs.is_empty() {
            let uv = glam::Vec2::new(0.0, 0.0);
            [uv, uv, uv]
        } else {
            [
                triangle_mesh.uvs[triangle_mesh.indices[index * 3] as usize],
                triangle_mesh.uvs[triangle_mesh.indices[index * 3 + 1] as usize],
                triangle_mesh.uvs[triangle_mesh.indices[index * 3 + 2] as usize],
            ]
        };
        let uv = uvs[0] * barycentric[0] + uvs[1] * barycentric[1] + uvs[2] * barycentric[2];

        // サンプリングする点のRender空間でのTangentを計算する。
        let tangent = if triangle_mesh.tangents.is_empty() {
            shading_normal.generate_tangent()
        } else {
            // Tangentが指定されている場合は、正規直交化する
            let raw_tangent = &self.local_to_render
                * triangle_mesh.tangents[triangle_mesh.indices[index] as usize];
            shading_normal.orthogonalize_vector(&raw_tangent)
        };

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

        // マテリアルのedfから放射輝度を取得する。
        let radiance = self.material.as_emissive_material().unwrap().radiance(
            lambda,
            &render_to_tangent * -wi,
            &(render_to_tangent * light_sample_point),
        );

        // pdfを計算する。
        // 三角形から一様に取得するので三角形の面積で割ったものをpdfとする。
        // 三角形の選択確率もpdfに組み込む。
        // let pdf = 1.0 / self.area_list[index];
        // let probability = self.area_list[index] / self.area_sum;
        // let pdf = pdf * probability;
        let pdf = 1.0 / self.area_sum;

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
}
impl<Id: SceneId> PrimitiveAreaLight<Id> for EmissiveTriangleMesh<Id> {
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

        // マテリアルのedfから放射輝度を取得する。

        self.material.as_emissive_material().unwrap().radiance(
            lambda,
            &render_to_tangent * wo,
            &(render_to_tangent * interaction),
        )
    }

    fn pdf_light_sample(&self, intersection: &Intersection<Id, Render>) -> f32 {
        // interactionした位置の三角形のジオメトリインデックスを確認する。
        let geometry_index = match intersection.geometry_info {
            InteractGeometryInfo::TriangleMesh { triangle_index } => triangle_index,
            _ => panic!("Invalid geometry info"),
        } as usize;

        // 三角形のサンプリングの選択確率を取得する。
        let probability = if geometry_index == 0 {
            self.area_table[0]
        } else {
            self.area_table[geometry_index] - self.area_table[geometry_index - 1]
        };

        // interactionした位置の三角形の面積を取得する。
        let area = self.area_list[geometry_index as usize];

        // pdfを計算する。
        1.0 / area * probability
    }
}
