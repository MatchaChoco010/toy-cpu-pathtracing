//! 空間上の点を表す構造体を定義するモジュール。

use std::marker::PhantomData;

use crate::{CoordinateSystem, Vector3};

/// 座標系Cでの点を表す構造体。
#[derive(Debug, Clone, Copy)]
pub struct Point3<C: CoordinateSystem> {
    vec: glam::Vec3,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Point3<C> {
    pub const ZERO: Self = Self {
        vec: glam::Vec3::ZERO,
        _marker: PhantomData,
    };

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
