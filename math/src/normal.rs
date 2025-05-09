//! 空間上の法線ベクトルを表す構造体定義するモジュール。

use std::marker::PhantomData;

use crate::coordinate_system::CoordinateSystem;

/// 座標系Cでの法線ベクトルを表す構造体。
#[derive(Debug, Clone, Copy)]
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
