//! 空間の変換を表す構造体を定義するモジュール。

use std::marker::PhantomData;

use util_macros::impl_binary_ops;

use crate::{Bounds, CoordinateSystem, Normal, Point3, Ray, Vector3};

/// 座標系の変換を行う行列の構造体。
#[derive(Debug, Clone)]
pub struct Transform<From: CoordinateSystem, To: CoordinateSystem> {
    matrix: glam::Mat4,
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
    let vec = lhs.matrix.transform_point3(rhs.to_vec3());
    Point3::from(vec)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Vector3<From>,
) -> Vector3<To> {
    let vec = lhs.matrix.transform_vector3(rhs.to_vec3());
    Vector3::from(vec)
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Normal<From>,
) -> Normal<To> {
    let matrix = lhs.matrix.inverse().transpose();
    let vec = matrix.transform_vector3(rhs.to_vec3());
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
    let mut min = glam::Vec3::splat(f32::INFINITY);
    let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
    for vertex in rhs.vertices() {
        let transformed_vertex = lhs * &vertex;
        min = min.min(transformed_vertex.to_vec3());
        max = max.max(transformed_vertex.to_vec3());
    }
    Bounds::new(Point3::from(min), Point3::from(max))
}
impl<From: CoordinateSystem, To: CoordinateSystem> Transform<From, To> {
    #[inline(always)]
    fn from_matrix(matrix: glam::Mat4) -> Self {
        Transform {
            matrix,
            _from: PhantomData,
            _to: PhantomData,
        }
    }

    /// 単位行列のTransformを作成する。
    #[inline(always)]
    pub fn identity() -> Self {
        Transform::from_matrix(glam::Mat4::IDENTITY)
    }

    /// 平行移動のTransformを作成する。
    #[inline(always)]
    pub fn translation(translation: glam::Vec3) -> Self {
        let matrix = glam::Mat4::from_translation(translation);
        Transform::from_matrix(matrix)
    }

    /// 回転のTransformを作成する。
    #[inline(always)]
    pub fn rotation(rotation: glam::Quat) -> Self {
        let matrix = glam::Mat4::from_quat(rotation);
        Transform::from_matrix(matrix)
    }

    /// スケールのTransformを作成する。
    #[inline(always)]
    pub fn scaling(scale: glam::Vec3) -> Self {
        let matrix = glam::Mat4::from_scale(scale);
        Transform::from_matrix(matrix)
    }

    /// 平行移動をかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn translate(&self, translation: glam::Vec3) -> Self {
        let matrix = glam::Mat4::from_translation(translation) * self.matrix;
        Transform::from_matrix(matrix)
    }

    /// 回転をかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn rotate(&self, rotation: glam::Quat) -> Self {
        let matrix = glam::Mat4::from_quat(rotation) * self.matrix;
        Transform::from_matrix(matrix)
    }

    /// スケールをかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn scale(&self, scale: glam::Vec3) -> Self {
        let matrix = glam::Mat4::from_scale(scale) * self.matrix;
        Transform::from_matrix(matrix)
    }

    /// 平行移動、回転、スケールのTransformを作成する。
    #[inline(always)]
    pub fn trs(translation: glam::Vec3, rotation: glam::Quat, scale: glam::Vec3) -> Self {
        let translation_matrix = glam::Mat4::from_translation(translation);
        let rotation_matrix = glam::Mat4::from_quat(rotation);
        let scale_matrix = glam::Mat4::from_scale(scale);

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
