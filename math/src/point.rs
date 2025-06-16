//! 空間上の点を表す構造体を定義するモジュール。

use std::marker::PhantomData;

use util_macros::impl_binary_ops;

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

    /// x成分を取得する。
    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.vec.x
    }

    /// y成分を取得する。
    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.vec.y
    }

    /// z成分を取得する。
    #[inline(always)]
    pub fn z(&self) -> f32 {
        self.vec.z
    }

    /// 指定した軸の値を取得する。
    #[inline(always)]
    pub fn axis(&self, axis: usize) -> f32 {
        match axis {
            0 => self.vec.x,
            1 => self.vec.y,
            2 => self.vec.z,
            _ => panic!("Invalid axis: {}", axis),
        }
    }

    /// 複数のPoint3のmin/maxを計算する。
    #[inline(always)]
    pub fn min_max_from_points(points: &[Point3<C>]) -> (Point3<C>, Point3<C>) {
        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
        for point in points {
            min = min.min(point.vec);
            max = max.max(point.vec);
        }
        (Point3::from(min), Point3::from(max))
    }

    /// 3つのPoint3をbarycentric座標で補間する。
    #[inline(always)]
    pub fn interpolate_barycentric(
        p0: &Point3<C>,
        p1: &Point3<C>,
        p2: &Point3<C>,
        barycentric: [f32; 3],
    ) -> Self {
        let interpolated = p0.to_vec3() * barycentric[0]
            + p1.to_vec3() * barycentric[1]
            + p2.to_vec3() * barycentric[2];
        Point3::from(interpolated)
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

    /// この点を指定したベクトルで平行移動する。
    #[inline(always)]
    pub fn translate(&self, translation: impl AsRef<Vector3<C>>) -> Self {
        Point3::from(self.vec + translation.as_ref().to_vec3())
    }

    /// Point3をglam::Vec3に変換する。
    #[inline(always)]
    pub(crate) fn to_vec3(&self) -> glam::Vec3 {
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
        self
    }
}
#[impl_binary_ops(Add)]
fn add<C: CoordinateSystem>(lhs: &Point3<C>, rhs: &Vector3<C>) -> Point3<C> {
    Point3::from(lhs.vec + rhs.to_vec3())
}
#[impl_binary_ops(Sub)]
fn sub<C: CoordinateSystem>(lhs: &Point3<C>, rhs: &Vector3<C>) -> Point3<C> {
    Point3::from(lhs.vec - rhs.to_vec3())
}
