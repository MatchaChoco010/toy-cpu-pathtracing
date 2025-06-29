//! 三角形メッシュのジオメトリのモジュール。
//!
//! 外部に公開する三角形メッシュのジオメトリの構造体の他、BVH構築のための中間データ構造なども含む。

use std::any::Any;
use std::marker::PhantomData;

use math::{Bounds, Local, Normal, Point3, Ray, Vector3, intersect_triangle};

use crate::{
    Geometry, SceneId,
    bvh::{Bvh, BvhItem, BvhItemData, HitInfo},
    geometry::Intersection,
};

/// BVHの要素の三角形の構造体。
#[derive(Debug, Clone, Copy)]
struct Triangle<Id: SceneId> {
    triangle_index: u32,
    _id: PhantomData<Id>,
}
impl<Id: SceneId> BvhItem<Local> for Triangle<Id> {
    type Data<'a>
        = TriangleMesh<Id>
    where
        Id: 'a;
    type Intersection = Intersection;

    fn bounds<'a>(&self, data: &Self::Data<'a>) -> Bounds<Local>
    where
        Id: 'a,
    {
        let positions = [
            data.positions[data.indices[self.triangle_index as usize * 3] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 1] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 2] as usize],
        ];
        let (min, max) = Point3::min_max_from_points(&positions);
        Bounds::new(min, max)
    }

    fn intersect<'a>(
        &self,
        data: &Self::Data<'a>,
        ray: &Ray<Local>,
        t_max: f32,
    ) -> Option<HitInfo<Intersection>>
    where
        Id: 'a,
    {
        // ジオメトリデータを取得
        let positions = [
            data.positions[data.indices[self.triangle_index as usize * 3] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 1] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 2] as usize],
        ];
        let shading_normals = [
            data.normals[data.indices[self.triangle_index as usize * 3] as usize],
            data.normals[data.indices[self.triangle_index as usize * 3 + 1] as usize],
            data.normals[data.indices[self.triangle_index as usize * 3 + 2] as usize],
        ];
        let uvs = if data.uvs.is_empty() {
            let uv = glam::Vec2::new(0.0, 0.0);
            [uv, uv, uv]
        } else {
            [
                data.uvs[data.indices[self.triangle_index as usize * 3] as usize],
                data.uvs[data.indices[self.triangle_index as usize * 3 + 1] as usize],
                data.uvs[data.indices[self.triangle_index as usize * 3 + 2] as usize],
            ]
        };

        // 三角形の交差判定を行う
        let hit = intersect_triangle(ray, t_max, positions)?;

        // 交差していれば、交差点の情報を計算する

        // shading_normalを頂点法線のbarycentric補間で計算する
        let shading_normal = Normal::interpolate_barycentric(
            &shading_normals[0],
            &shading_normals[1],
            &shading_normals[2],
            hit.barycentric,
        );

        // uvを頂点uvのbarycentric補間で計算する
        let uv =
            uvs[0] * hit.barycentric[0] + uvs[1] * hit.barycentric[1] + uvs[2] * hit.barycentric[2];

        // Tangentを計算する。
        let tangent = if data.tangents.is_empty() {
            shading_normal.generate_tangent()
        } else {
            // 既存のtangentを正規直交化する
            shading_normal.orthogonalize_vector(&data.tangents[self.triangle_index as usize])
        };

        Some(HitInfo {
            t_hit: hit.t_hit,
            intersection: Intersection {
                position: hit.position,
                normal: hit.normal,
                shading_normal,
                tangent,
                uv,
                index: self.triangle_index,
                t_hit: hit.t_hit,
            },
        })
    }

    fn intersect_p<'a>(&self, data: &Self::Data<'a>, ray: &Ray<Local>, t_max: f32) -> bool
    where
        Id: 'a,
    {
        // ジオメトリデータを取得
        let positions = [
            data.positions[data.indices[self.triangle_index as usize * 3] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 1] as usize],
            data.positions[data.indices[self.triangle_index as usize * 3 + 2] as usize],
        ];

        // 三角形の交差判定を行う
        intersect_triangle(ray, t_max, positions).is_some()
    }
}

/// 三角形メッシュのジオメトリの構造体。
#[derive(Debug)]
pub struct TriangleMesh<Id: SceneId> {
    pub positions: Vec<Point3<Local>>,
    pub normals: Vec<Normal<Local>>,
    pub tangents: Vec<Vector3<Local>>,
    pub uvs: Vec<glam::Vec2>,
    pub indices: Vec<u32>,
    pub bounds: Bounds<Local>,
    bvh: Option<Bvh<Local, Triangle<Id>>>,
}
impl<Id: SceneId> TriangleMesh<Id> {
    /// objファイルを読み込み新しい三角形メッシュを作成する。
    pub fn load_obj(path: &str) -> Self {
        // モデルを読み込む。
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            },
        )
        .unwrap();

        // モデルのデータを格納する。
        let mut positions = vec![];
        let mut normals = vec![];
        let mut tangents = vec![];
        let mut uvs = vec![];
        let mut indices = vec![];

        for model in models {
            let mesh = model.mesh;
            positions.extend(
                mesh.positions
                    .chunks(3)
                    .map(|p| Point3::new(p[0], p[1], p[2])),
            );
            normals.extend(
                mesh.normals
                    .chunks(3)
                    .map(|n| Normal::new(n[0], n[1], n[2])),
            );
            uvs.extend(
                mesh.texcoords
                    .chunks(2)
                    .map(|uv| glam::Vec2::new(uv[0], uv[1])),
            );

            indices.extend(mesh.indices.iter().copied());

            if !uvs.is_empty() {
                for i in indices.chunks(3) {
                    let p0 = positions[i[0] as usize];
                    let p1 = positions[i[1] as usize];
                    let p2 = positions[i[2] as usize];
                    let edge1 = p0.vector_to(p1);
                    let edge2 = p0.vector_to(p2);

                    let uv0 = uvs[i[0] as usize];
                    let uv1 = uvs[i[1] as usize];
                    let uv2 = uvs[i[2] as usize];
                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    let denominator = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;
                    let r = 1.0 / denominator;
                    let mut tangent = r * (edge1 * delta_uv2.y - edge2 * delta_uv1.y);

                    fn fallback_tangent(
                        edge1: Vector3<Local>,
                        edge2: Vector3<Local>,
                    ) -> Vector3<Local> {
                        let cross_product = edge1.cross(edge2);
                        if cross_product.length_squared() < 1e-12 {
                            // normalも計算出来ない縮退した三角形ではデフォルトのタンジェントを使用
                            Vector3::new(1.0, 0.0, 0.0)
                        } else {
                            let normal = cross_product.normalize().to_normal();
                            normal.generate_tangent()
                        }
                    }

                    if denominator.abs() < 1e-6 {
                        // UVが重なっている場合などは、
                        // 法線を使用してタンジェントを生成する
                        tangent = fallback_tangent(edge1, edge2);
                    } else {
                        tangent = tangent.normalize();
                        if tangent.is_nan() {
                            // NaNが発生した場合はフォールバックする
                            tangent = fallback_tangent(edge1, edge2);
                        }
                    }
                    tangents.push(tangent);
                }
            }
        }

        // バウンディングボックスを計算する。
        let (min, max) = Point3::min_max_from_points(&positions);
        let bounds = Bounds::new(min, max);

        Self {
            positions,
            normals,
            tangents,
            uvs,
            indices,
            bounds,
            bvh: None,
        }
    }
}
impl<Id: SceneId> Geometry<Id> for TriangleMesh<Id> {
    fn build_bvh(&mut self) {
        // すでにBVHが構築されている場合は何もしない
        if self.bvh.is_some() {
            return;
        }

        let bvh = Bvh::build(self);
        self.bvh = Some(bvh);
    }

    fn bounds(&self) -> Bounds<Local> {
        self.bounds.clone()
    }

    fn intersect(&self, ray: &Ray<Local>, t_max: f32) -> Option<Intersection> {
        if let Some(bvh) = &self.bvh {
            bvh.intersect(self, ray, t_max)
        } else {
            panic!("BVH is not built");
        }
    }

    fn intersect_p(&self, ray: &Ray<Local>, t_max: f32) -> bool {
        if let Some(bvh) = &self.bvh {
            bvh.intersect_p(self, ray, t_max)
        } else {
            panic!("BVH is not built");
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl<Id: SceneId> BvhItemData<Triangle<Id>> for TriangleMesh<Id> {
    fn item_list(&self) -> impl Iterator<Item = Triangle<Id>> {
        (0..(self.indices.len() / 3) as u32).map(move |i| Triangle {
            triangle_index: i,
            _id: PhantomData,
        })
    }
}
