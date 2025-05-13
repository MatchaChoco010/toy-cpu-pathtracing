//! 放射面を含む三角形メッシュのプリミティブの実装のモジュール。

use std::sync::Arc;

use math::{Bounds, Local, Normal, Point3, Ray, Render, Transform, Vector3, World};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    GeometryIndex, InteractGeometryInfo, Intersection, LightSampleRadiance, PrimitiveIndex,
    SceneId, SurfaceInteraction, SurfaceMaterial,
    geometry::{GeometryRepository, impls::TriangleMesh},
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaLight, PrimitiveGeometry,
        PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 放射面を含む三角形メッシュのプリミティブの構造体。
pub struct EmissiveTriangleMesh<Id: SceneId> {
    geometry_index: GeometryIndex<Id>,
    material: Arc<SurfaceMaterial<Id>>,
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
        material: Arc<SurfaceMaterial<Id>>,
        local_to_world: Transform<Local, World>,
    ) -> Self {
        // ワールド空間での面積などを計算しておく。

        // 三角形の面積のリスト。
        let mut area_list = vec![];
        for index in triangle_mesh.indices.chunks(3) {
            let p0 = triangle_mesh.positions[index[0] as usize];
            let p1 = triangle_mesh.positions[index[1] as usize];
            let p2 = triangle_mesh.positions[index[2] as usize];
            let area = ((p0.vector_to(p1)).cross(p0.vector_to(p2))).length() / 2.0;
            area_list.push(area);
        }

        // 面積の総和と面積の重み付きでサンプリングするための累積確率のテーブルを作成する。
        let mut area_sum = 0.0;
        let mut area_table = vec![];
        for area in &area_list {
            area_sum += area;
            area_table.push(area_sum);
        }
        for i in 0..area_table.len() {
            area_table[i] /= area_sum;
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
                        material: self.material.clone(),
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
        // 面積の総和を使って、放射面のスペクトル放射束を計算する。
        self.material
            .edf
            .as_ref()
            .unwrap()
            .average_intensity(lambda)
            * self.area_sum
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EmissiveTriangleMesh<Id> {
    fn sample_radiance(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        shading_point: &SurfaceInteraction<Id, Render>,
        lambda: &SampledWavelengths,
        s: f32,
        uv: glam::Vec2,
    ) -> LightSampleRadiance<Id, Render> {
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

        // 三角形のサンプリングの選択確率を取得する。
        let probability = if index == 0 {
            self.area_table[0]
        } else {
            self.area_table[index] - self.area_table[index - 1]
        };

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
        let p0 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3] as usize];
        let p1 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3 + 1] as usize];
        let p2 = &self.local_to_render
            * triangle_mesh.positions[triangle_mesh.indices[index * 3 + 2] as usize];
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
        let n0 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3] as usize];
        let n1 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3 + 1] as usize];
        let n2 = &self.local_to_render
            * triangle_mesh.normals[triangle_mesh.indices[index * 3 + 2] as usize];
        let shading_normal = Normal::from(
            n0.to_vec3() * barycentric[0]
                + n1.to_vec3() * barycentric[1]
                + n2.to_vec3() * barycentric[2],
        );

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
            // 適当な方向をtangentとして使用する。
            if shading_normal.to_vec3().x.abs() > 0.999 {
                // X軸方向が法線に近い場合はY軸方向を使用する。
                Vector3::from(glam::Vec3::Y)
            } else {
                // X軸方向を使用する。
                Vector3::from(glam::Vec3::X)
            }
        } else {
            // Tangentが指定されている場合は、Tangentをそのまま使用する。
            &self.local_to_render * triangle_mesh.tangents[triangle_mesh.indices[index] as usize]
        };

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

        let pdf = 1.0 / self.area_list[index] * probability;

        LightSampleRadiance {
            radiance,
            pdf,
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

    fn pdf_light_sample(&self, interaction: &SurfaceInteraction<Id, Render>) -> f32 {
        // interactionした位置の三角形のジオメトリインデックスを確認する。
        let geometry_index = match interaction.geometry_info {
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
impl<Id: SceneId> PrimitiveAreaLight<Id> for EmissiveTriangleMesh<Id> {
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
