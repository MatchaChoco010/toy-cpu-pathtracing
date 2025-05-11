//! 空間での方向を表すベクトルを定義するモジュール

use std::marker::PhantomData;

use util_macros::impl_binary_ops;

use crate::CoordinateSystem;

/// 座標系Cでのベクトルを表す構造体。
#[derive(Debug, Clone, Copy)]
pub struct Vector3<C: CoordinateSystem> {
    vec: glam::Vec3,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Vector3<C> {
    pub const ZERO: Self = Self {
        vec: glam::Vec3::ZERO,
        _marker: PhantomData,
    };

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
#[impl_binary_ops(Mul)]
fn mul<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &f32) -> Vector3<C> {
    Vector3::from(lhs.vec * rhs)
}
#[impl_binary_ops(Mul)]
fn mul<C: CoordinateSystem>(lhs: &f32, rhs: &Vector3<C>) -> Vector3<C> {
    Vector3::from(lhs * rhs.vec)
}
#[impl_binary_ops(Add)]
fn add<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &Vector3<C>) -> Vector3<C> {
    Vector3::from(lhs.vec + rhs.vec)
}
#[impl_binary_ops(Sub)]
fn sub<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &Vector3<C>) -> Vector3<C> {
    Vector3::from(lhs.vec - rhs.vec)
}
#[impl_binary_ops(Mul)]
fn mul<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &Vector3<C>) -> Vector3<C> {
    Vector3::from(lhs.vec / rhs.vec)
}
#[impl_binary_ops(Div)]
fn div<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &f32) -> Vector3<C> {
    Vector3::from(lhs.vec / rhs)
}
#[impl_binary_ops(Div)]
fn div<C: CoordinateSystem>(lhs: &Vector3<C>, rhs: &Vector3<C>) -> Vector3<C> {
    Vector3::from(lhs.vec / rhs.vec)
}
