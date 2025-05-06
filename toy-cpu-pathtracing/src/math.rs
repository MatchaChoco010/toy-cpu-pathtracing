//! 数学関連のモジュール。
//! NaNを発生させない数学関数や、
//! 座標系を区別するベクトルや点、法線、レイ、変換行列などを定義する。

use std::marker::PhantomData;

use glam::{Mat4, Quat, Vec3};
use macros::impl_binary_ops;

use crate::scene::{SceneId, primitive};

/// 定義域外の値を与えてもNaNを返さないacos関数の実装トレイト。
pub trait SafeAcos {
    fn safe_acos(self) -> f32;
}
impl SafeAcos for f32 {
    #[inline(always)]
    fn safe_acos(self) -> f32 {
        if self < -1.0 {
            -std::f32::consts::PI
        } else if self > 1.0 {
            std::f32::consts::PI
        } else {
            self.acos()
        }
    }
}

/// 座標系のマーカー用トレイト。
pub trait CoordinateSystem: std::fmt::Debug + Clone {}

/// ワールド座標系を表すマーカー構造体。
#[derive(Debug, Clone)]
pub struct World;
impl CoordinateSystem for World {}

/// モデルローカル座標系を表すマーカー構造体。
#[derive(Debug, Clone)]
pub struct Local;
impl CoordinateSystem for Local {}

/// レンダリング座標系を表すマーカー構造体。
///
/// レンダリング座標系はカメラを原点にして座標軸はワールド座標系と平行な座標系。
/// 多くの場合、シーンにはワールド座標の軸と平行な直線が含まれることがあり、特に地面などは軸とズレていないことも多い。
/// そのため、カメラが斜めになったときでもレンダリングに使う座標系では
/// ワールド座標系と軸が平行がそのままの方がバウンディングボックスがタイトになりやすく、多少良いBVHが構築できうる。
#[derive(Debug, Clone)]
pub struct Render;
impl CoordinateSystem for Render {}

/// 座標系Cでの点を表す構造体。
#[derive(Debug, Clone)]
pub struct Point3<C: CoordinateSystem> {
    vec: glam::Vec3,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Point3<C> {
    /// Point3を作成する。
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(glam::Vec3::new(x, y, z))
    }

    /// 2点間の距離を計算する。
    #[inline(always)]
    pub fn distance(&self, other: impl AsRef<Point3<C>>) -> f32 {
        (self.vec - other.as_ref().vec).length()
    }

    /// 2点間の距離の2乗を計算する。
    #[inline(always)]
    pub fn distance_squared(&self, other: impl AsRef<Point3<C>>) -> f32 {
        (self.vec - other.as_ref().vec).length_squared()
    }

    /// この点から他の点へのベクトルを計算する。
    #[inline(always)]
    pub fn vector_to(&self, other: impl AsRef<Point3<C>>) -> Vector3<C> {
        Vector3::from(other.as_ref().vec - self.vec)
    }

    /// Point3をglam::Vec3に変換する。
    #[inline(always)]
    pub fn to_vec3(&self) -> glam::Vec3 {
        self.vec
    }
}
impl<C: CoordinateSystem> From<glam::Vec3> for Point3<C> {
    #[inline(always)]
    fn from(vec: glam::Vec3) -> Self {
        Self {
            vec,
            _marker: PhantomData,
        }
    }
}
impl<C: CoordinateSystem> AsRef<Point3<C>> for Point3<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Point3<C> {
        &self
    }
}

/// 座標系Cでのベクトルを表す構造体。
#[derive(Debug, Clone)]
pub struct Vector3<C: CoordinateSystem> {
    vec: glam::Vec3,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Vector3<C> {
    /// Vector3を作成する。
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(glam::Vec3::new(x, y, z))
    }

    /// ベクトルを正規化する。
    #[inline(always)]
    pub fn normalize(&self) -> Self {
        Self::from(self.vec.normalize())
    }

    /// 内積を計算する。
    #[inline(always)]
    pub fn dot(&self, other: impl AsRef<Vector3<C>>) -> f32 {
        self.vec.dot(other.as_ref().vec)
    }

    /// 外積を計算する。
    #[inline(always)]
    pub fn cross(&self, other: impl AsRef<Vector3<C>>) -> Self {
        Self::from(self.vec.cross(other.as_ref().vec))
    }

    /// ベクトルの長さを計算する。
    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.vec.length()
    }

    /// ベクトルの長さの2乗を計算する。
    #[inline(always)]
    pub fn length_squared(&self) -> f32 {
        self.vec.length_squared()
    }

    /// Vector3をglam::Vec3に変換する。
    #[inline(always)]
    pub fn to_vec3(&self) -> glam::Vec3 {
        self.vec
    }
}
impl<C: CoordinateSystem> From<glam::Vec3> for Vector3<C> {
    #[inline(always)]
    fn from(vec: glam::Vec3) -> Self {
        Self {
            vec,
            _marker: PhantomData,
        }
    }
}
impl<C: CoordinateSystem> AsRef<Vector3<C>> for Vector3<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Vector3<C> {
        &self
    }
}

/// 座標系Cでの法線ベクトルを表す構造体。
#[derive(Debug, Clone)]
pub struct Normal<C: CoordinateSystem> {
    vec: glam::Vec3,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Normal<C> {
    /// Normalを作成する。
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(glam::Vec3::new(x, y, z).normalize())
    }

    /// Normal3をglam::Vec3に変換する。
    #[inline(always)]
    pub fn to_vec3(&self) -> glam::Vec3 {
        self.vec
    }
}
impl<C: CoordinateSystem> From<glam::Vec3> for Normal<C> {
    #[inline(always)]
    fn from(vec: glam::Vec3) -> Self {
        Self {
            vec: vec.normalize(),
            _marker: PhantomData,
        }
    }
}
impl<C: CoordinateSystem> AsRef<Normal<C>> for Normal<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Normal<C> {
        &self
    }
}

/// Ray構造体。
/// dirは座標変換によっては正規化されていない値になりうる。
#[derive(Debug, Clone)]
pub struct Ray<C: CoordinateSystem> {
    pub origin: Point3<C>,
    pub dir: Vector3<C>,
}
impl<C: CoordinateSystem> Ray<C> {
    /// Rayを作成する。
    #[inline(always)]
    pub fn new(origin: impl AsRef<Point3<C>>, dir: impl AsRef<Vector3<C>>) -> Self {
        let origin = origin.as_ref().clone();
        let dir = dir.as_ref().clone();
        Self { origin, dir }
    }
}
impl<C: CoordinateSystem> AsRef<Ray<C>> for Ray<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Ray<C> {
        &self
    }
}

/// BoundsとRayの交差したtの値を表す構造体。
pub struct BoundsIntersection {
    pub t0: f32,
    pub t1: f32,
}

/// Bounds構造体。
#[derive(Debug, Clone)]
pub struct Bounds<C: CoordinateSystem> {
    pub min: Point3<C>,
    pub max: Point3<C>,
}
impl<C: CoordinateSystem> Bounds<C> {
    /// Boundsを作成する。
    #[inline(always)]
    pub fn new(min: impl AsRef<Point3<C>>, max: impl AsRef<Point3<C>>) -> Self {
        let min = min.as_ref().clone();
        let max = max.as_ref().clone();
        Self { min, max }
    }

    /// 交差を判定する。
    pub fn intersect(
        &self,
        ray: impl AsRef<Ray<C>>,
        t_max: f32,
        inv_dir: Vec3,
    ) -> Option<BoundsIntersection> {
        let ray = ray.as_ref();

        let mut t0 = 0.0;
        let mut t1 = t_max;

        for i in 0..3 {
            let mut t_near = (self.min.to_vec3()[i] - ray.origin.to_vec3()[i]) * inv_dir[i];
            let mut t_far = (self.max.to_vec3()[i] - ray.origin.to_vec3()[i]) * inv_dir[i];

            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if t_far < t1 { t_far } else { t1 };

            if t0 > t1 {
                return None;
            }
        }

        Some(BoundsIntersection { t0, t1 })
    }

    /// Boundsの中心を取得する。
    #[inline(always)]
    pub fn center(&self) -> Point3<C> {
        let center = (self.min.to_vec3() + self.max.to_vec3()) * 0.5;
        Point3::from(center)
    }

    /// Boundsの表面積を取得する。
    #[inline(always)]
    pub fn area(&self) -> f32 {
        let d = self.max.to_vec3() - self.min.to_vec3();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    /// Boundsを含むバウンディングスフィアを計算する。
    #[inline(always)]
    pub fn bounding_sphere(&self) -> (Point3<C>, f32) {
        let center = self.center();
        let radius = center.distance(&self.max);
        (center, radius)
    }

    /// Boundsをマージする。
    #[inline(always)]
    pub fn merge(&self, other: impl AsRef<Bounds<C>>) -> Self {
        let other = other.as_ref();
        let min = Point3::from(glam::Vec3::min(self.min.to_vec3(), other.min.to_vec3()));
        let max = Point3::from(glam::Vec3::max(self.max.to_vec3(), other.max.to_vec3()));
        Bounds::new(min, max)
    }
}
impl<C: CoordinateSystem> AsRef<Bounds<C>> for Bounds<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Bounds<C> {
        &self
    }
}

/// ライト上をサンプルするためのコンテキスト。
/// サンプルする際のシェーディング点の情報を持つ。
#[derive(Debug, Clone)]
pub struct LightSampleContext<C: CoordinateSystem> {
    pub position: Vector3<C>,
    pub normal: Normal<C>,
    pub shading_normal: Normal<C>,
}
impl<C: CoordinateSystem> AsRef<LightSampleContext<C>> for LightSampleContext<C> {
    #[inline(always)]
    fn as_ref(&self) -> &LightSampleContext<C> {
        &self
    }
}

/// 座標系の変換を行う行列の構造体。
#[derive(Debug, Clone)]
pub struct Transform<From: CoordinateSystem, To: CoordinateSystem> {
    matrix: Mat4,
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, Mid: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<Mid, To>,
    rhs: &Transform<From, Mid>,
) -> Transform<From, To> {
    let matrix = lhs.matrix * rhs.matrix;
    Transform::from_matrix(matrix)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Point3<From>,
) -> Point3<To> {
    let vec = lhs.matrix.transform_point3(rhs.vec);
    Point3::from(vec)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Vector3<From>,
) -> Vector3<To> {
    let vec = lhs.matrix.transform_vector3(rhs.vec);
    Vector3::from(vec)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Normal<From>,
) -> Normal<To> {
    let matrix = lhs.matrix.inverse().transpose();
    let vec = matrix.transform_vector3(rhs.vec);
    Normal::from(vec)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Ray<From>,
) -> Ray<To> {
    let origin = lhs * &rhs.origin;
    let dir = lhs * &rhs.dir;
    Ray::new(origin, dir)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Bounds<From>,
) -> Bounds<To> {
    let prev_min = lhs * &rhs.min;
    let prev_max = lhs * &rhs.max;
    let min = glam::Vec3::min(prev_min.to_vec3(), prev_max.to_vec3());
    let max = glam::Vec3::max(prev_min.to_vec3(), prev_max.to_vec3());
    Bounds::new(Point3::from(min), Point3::from(max))
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &LightSampleContext<From>,
) -> LightSampleContext<To> {
    let position = lhs * &rhs.position;
    let normal = lhs * &rhs.normal;
    let shading_normal = lhs * &rhs.shading_normal;
    LightSampleContext {
        position,
        normal,
        shading_normal,
    }
}
#[impl_binary_ops(Mul)]
fn mul<Id: SceneId, From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &primitive::Interaction<Id, From>,
) -> primitive::Interaction<Id, To> {
    match rhs {
        primitive::Interaction::Surface {
            position,
            normal,
            shading_normal,
            uv,
            primitive_index,
            geometry_info,
        } => primitive::Interaction::Surface {
            position: lhs * position,
            normal: lhs * normal,
            shading_normal: lhs * shading_normal,
            uv: *uv,
            primitive_index: *primitive_index,
            geometry_info: *geometry_info,
        },
    }
}
#[impl_binary_ops(Mul)]
fn mul<Id: SceneId, From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &primitive::Intersection<Id, From>,
) -> primitive::Intersection<Id, To> {
    primitive::Intersection {
        t_hit: rhs.t_hit,
        interaction: lhs * &rhs.interaction,
    }
}
impl<From: CoordinateSystem, To: CoordinateSystem> Transform<From, To> {
    #[inline(always)]
    fn from_matrix(matrix: Mat4) -> Self {
        Transform {
            matrix,
            _from: PhantomData,
            _to: PhantomData,
        }
    }

    /// 単位行列のTransformを作成する。
    #[inline(always)]
    pub fn identity() -> Self {
        Transform::from_matrix(Mat4::IDENTITY)
    }

    /// 平行移動のTransformを作成する。
    #[inline(always)]
    pub fn translate(translation: Vec3) -> Self {
        let matrix = Mat4::from_translation(translation);
        Transform::from_matrix(matrix)
    }

    /// 回転のTransformを作成する。
    #[inline(always)]
    pub fn rotate(rotation: Quat) -> Self {
        let matrix = Mat4::from_quat(rotation);
        Transform::from_matrix(matrix)
    }

    /// スケールのTransformを作成する。
    #[inline(always)]
    pub fn scale(scale: Vec3) -> Self {
        let matrix = Mat4::from_scale(scale);
        Transform::from_matrix(matrix)
    }

    /// 平行移動、回転、スケールのTransformを作成する。
    #[inline(always)]
    pub fn trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let translation_matrix = Mat4::from_translation(translation);
        let rotation_matrix = Mat4::from_quat(rotation);
        let scale_matrix = Mat4::from_scale(scale);

        let matrix = translation_matrix * rotation_matrix * scale_matrix;

        Transform::from_matrix(matrix)
    }

    /// Transformを逆行列に変換する。
    #[inline(always)]
    pub fn inverse(&self) -> Transform<To, From> {
        let inverse_matrix = self.matrix.inverse();
        Transform::from_matrix(inverse_matrix)
    }
}
