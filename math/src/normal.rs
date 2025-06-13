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

    /// x成分を取得する。
    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.vec.x()
    }

    /// y成分を取得する。
    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.vec.y()
    }

    /// z成分を取得する。
    #[inline(always)]
    pub fn z(&self) -> f32 {
        self.vec.z()
    }

    /// 内積を計算する。Vector3やNormalの参照を受け取れる。
    #[inline(always)]
    pub fn dot(&self, other: impl AsRef<Vector3<C>>) -> f32 {
        self.vec.dot(other)
    }

    /// 指定したベクトルをこのNormalに対して正規直交化する。
    #[inline(always)]
    pub fn orthogonalize_vector(&self, vector: &Vector3<C>) -> Vector3<C> {
        // NormalとVectorの線形結合で投影を減算
        let projection_magnitude = self.dot(vector);
        let orthogonal = *vector - self.vec * projection_magnitude;
        orthogonal.normalize()
    }

    /// このNormalに対して適切なタンジェントベクトルを生成する。
    #[inline(always)]
    pub fn generate_tangent(&self) -> Vector3<C> {
        let normal_vec = self.to_vec3();
        // X軸との角度が小さい場合はY軸を使用
        let candidate = if normal_vec.x.abs() > 0.999 {
            glam::Vec3::Y
        } else {
            glam::Vec3::X
        };
        let candidate_vector = Vector3::from(candidate);
        self.orthogonalize_vector(&candidate_vector)
    }

    /// 3つのNormalをbarycentric座標で補間する。
    #[inline(always)]
    pub fn interpolate_barycentric(
        n0: &Normal<C>,
        n1: &Normal<C>,
        n2: &Normal<C>,
        barycentric: [f32; 3],
    ) -> Self {
        let interpolated = n0.to_vec3() * barycentric[0]
            + n1.to_vec3() * barycentric[1]
            + n2.to_vec3() * barycentric[2];
        Normal::from(interpolated)
    }

    /// 外積を計算する。
    #[inline(always)]
    pub fn cross(&self, other: impl AsRef<Vector3<C>>) -> Vector3<C> {
        self.vec.cross(other)
    }

    /// Normal3をglam::Vec3に変換する。
    #[inline(always)]
    pub(crate) fn to_vec3(&self) -> glam::Vec3 {
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
impl<C: CoordinateSystem> From<Normal<C>> for Vector3<C> {
    #[inline(always)]
    fn from(val: Normal<C>) -> Self {
        val.vec
    }
}
impl<C: CoordinateSystem> From<&Normal<C>> for Vector3<C> {
    #[inline(always)]
    fn from(val: &Normal<C>) -> Self {
        val.vec
    }
}
