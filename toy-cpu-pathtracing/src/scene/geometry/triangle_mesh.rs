//! 三角形メッシュのジオメトリのモジュール。
//!
//! 外部に公開する三角形メッシュのジオメトリの構造体の他、BVH構築のための中間データ構造なども含む。

use std::marker::PhantomData;

use glam::{Vec2, Vec3};

use crate::math::{Bounds, Local, Normal, Point3, Ray, intersect_triangle};
use crate::scene::SceneId;
use crate::scene::bvh::{Bvh, BvhItem, BvhItemData, HitInfo};

/// 三角形メッシュとレイの交差の情報。
pub struct Intersection {
    /// サンプルした位置。
    pub position: Point3<Local>,
    /// サンプルした幾何法線。
    pub normal: Normal<Local>,
    /// サンプルしたシェーディング座標。
    pub shading_normal: Normal<Local>,
    /// サンプルしたUV座標。
    pub uv: Vec2,
    /// サンプルした三角形のインデックス。
    pub triangle_index: u32,
    /// ヒットした距離。
    pub t_hit: f32,
}

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
            &data.positions[data.indices[self.triangle_index as usize * 3] as usize],
            &data.positions[data.indices[self.triangle_index as usize * 3 + 1] as usize],
            &data.positions[data.indices[self.triangle_index as usize * 3 + 2] as usize],
        ];
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for position in &positions {
            let point = position.to_vec3();
            min = min.min(point);
            max = max.max(point);
        }
        let min = Point3::from(min);
        let max = Point3::from(max);
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
            let uv = Vec2::new(0.0, 0.0);
            [uv, uv, uv]
        } else {
            [
                data.uvs[data.indices[self.triangle_index as usize * 3] as usize],
                data.uvs[data.indices[self.triangle_index as usize * 3 + 1] as usize],
                data.uvs[data.indices[self.triangle_index as usize * 3 + 2] as usize],
            ]
        };

        // 三角形の交差判定を行う
        let hit = match intersect_triangle(ray, t_max, positions) {
            Some(hit) => hit,
            None => return None,
        };

        // 交差していれば、交差点の情報を計算する
        let shading_normal = Normal::from(
            shading_normals[0].to_vec3() * hit.barycentric[0]
                + shading_normals[1].to_vec3() * hit.barycentric[1]
                + shading_normals[2].to_vec3() * hit.barycentric[2],
        );
        let uv =
            uvs[0] * hit.barycentric[0] + uvs[1] * hit.barycentric[1] + uvs[2] * hit.barycentric[2];

        Some(HitInfo {
            t_hit: hit.t_hit,
            intersection: Intersection {
                position: hit.position,
                normal: hit.normal,
                shading_normal,
                uv,
                triangle_index: self.triangle_index,
                t_hit: hit.t_hit,
            },
        })
    }
}

/// 三角形メッシュのジオメトリの構造体。
pub struct TriangleMesh<Id: SceneId> {
    positions: Vec<Point3<Local>>,
    normals: Vec<Normal<Local>>,
    uvs: Vec<Vec2>,
    indices: Vec<u32>,
    bvh: Option<Bvh<Local, Triangle<Id>>>,
    bounds: Bounds<Local>,
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
            uvs.extend(mesh.texcoords.chunks(2).map(|uv| Vec2::new(uv[0], uv[1])));
            indices.extend(mesh.indices.iter().map(|i| *i as u32));
        }

        // バウンディングボックスを計算する。
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for position in &positions {
            min = min.min(position.to_vec3());
            max = max.max(position.to_vec3());
        }
        let min = Point3::from(min);
        let max = Point3::from(max);
        let bounds = Bounds::new(min, max);

        Self {
            positions,
            normals,
            uvs,
            indices,
            bvh: None,
            bounds,
        }
    }

    /// BVHを構築する。
    pub fn build_bvh(&mut self) {
        // すでにBVHが構築されている場合は何もしない
        if self.bvh.is_some() {
            return;
        }

        let bvh = Bvh::build(self);
        self.bvh = Some(bvh);
    }

    /// 三角形メッシュのバウンディングボックスを返す。
    pub fn bounds(&self) -> Bounds<Local> {
        self.bounds.clone()
    }

    /// 交差判定を行う。
    pub fn intersect(&self, ray: &Ray<Local>, t_max: f32) -> Option<Intersection> {
        if let Some(bvh) = &self.bvh {
            bvh.intersect(self, ray, t_max)
        } else {
            panic!("BVH is not built");
        }
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
