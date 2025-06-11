//! 空間上の法線ベクトルを表す構造体定義するモジュール。

use std::marker::PhantomData;

use crate::{CoordinateSystem, Vector3};

/// 座標系Cでの法線ベクトルを表す構造体。
#[derive(Debug, Clone, Copy)]
pub struct Normal<C: CoordinateSystem> {
    vec: Vector3<C>,
    _marker: PhantomData<C>,
}
impl<C: CoordinateSystem> Normal<C> {
    /// Normalを作成する。
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from(Vector3::new(x, y, z).normalize())
    }

    /// 内積を計算する。
    #[inline(always)]
    pub fn dot(&self, other: impl AsRef<Vector3<C>>) -> f32 {
        self.vec.dot(other)
    }

    /// 外積を計算する。
    #[inline(always)]
    pub fn cross(&self, other: impl AsRef<Vector3<C>>) -> Vector3<C> {
        self.vec.cross(other)
    }

    /// Normal3をglam::Vec3に変換する。
    #[inline(always)]
    pub fn to_vec3(&self) -> glam::Vec3 {
        self.vec.to_vec3()
    }
}
impl<C: CoordinateSystem> From<glam::Vec3> for Normal<C> {
    #[inline(always)]
    fn from(vec: glam::Vec3) -> Self {
        Self {
            vec: Vector3::from(vec).normalize(),
            _marker: PhantomData,
        }
    }
}
impl<C: CoordinateSystem> From<Vector3<C>> for Normal<C> {
    #[inline(always)]
    fn from(vec: Vector3<C>) -> Self {
        Self::from(vec.to_vec3())
    }
}
impl<C: CoordinateSystem> AsRef<Normal<C>> for Normal<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Normal<C> {
        self
    }
}
impl<C: CoordinateSystem> AsRef<Vector3<C>> for Normal<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Vector3<C> {
        &self.vec
    }
}
