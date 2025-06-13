//! 空間の変換を表す構造体を定義するモジュール。

use std::marker::PhantomData;

use util_macros::impl_binary_ops;

use crate::{
    Bounds, CoordinateSystem, GeometryTangent, Normal, NormalMapTangent, Point3, Ray,
    ShadingTangent, Vector3,
};

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
    let origin = lhs * rhs.origin;
    let dir = lhs * rhs.dir;
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
        let transformed_vertex = lhs * vertex;
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
    pub fn from_translate(translate: glam::Vec3) -> Self {
        let matrix = glam::Mat4::from_translation(translate);
        Transform::from_matrix(matrix)
    }

    /// Point3を使用して平行移動のTransformを作成する。
    #[inline(always)]
    pub fn translate_to<C: CoordinateSystem>(point: &Point3<C>) -> Self {
        Self::from_translate(point.to_vec3())
    }

    /// Point3を使用して逆方向の平行移動Transformを作成する。
    #[inline(always)]
    pub fn translate_from<C: CoordinateSystem>(point: &Point3<C>) -> Self {
        Self::from_translate(-point.to_vec3())
    }

    /// 回転のTransformを作成する。
    #[inline(always)]
    pub fn from_rotate(rotate: glam::Quat) -> Self {
        let matrix = glam::Mat4::from_quat(rotate);
        Transform::from_matrix(matrix)
    }

    /// スケールのTransformを作成する。
    #[inline(always)]
    pub fn from_scale(scale: glam::Vec3) -> Self {
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
impl<C: CoordinateSystem> Transform<C, GeometryTangent> {
    /// 任意の座標系CからGeometryTangent座標系への変換Transformを作成する。
    /// 幾何法線がZ軸になり、tangentの方向にX軸が向くような座標系に変換するTransform。
    pub fn from_geometry_normal_tangent(
        normal: &Normal<C>,
        tangent: &Vector3<C>,
    ) -> Transform<C, GeometryTangent> {
        let normal = normal.to_vec3().normalize();
        let bitangent = tangent.to_vec3().cross(normal).normalize();
        let tangent = bitangent.cross(normal);
        let matrix = glam::Mat4::from_cols(
            tangent.extend(0.0),
            bitangent.extend(0.0),
            normal.extend(0.0),
            glam::vec4(0.0, 0.0, 0.0, 1.0),
        );
        Transform::from_matrix(matrix.inverse())
    }
}

impl<C: CoordinateSystem> Transform<C, ShadingTangent> {
    /// 任意の座標系CからShadingTangent座標系への変換Transformを作成する。
    /// シェーディング法線がZ軸になり、tangentの方向にX軸が向くような座標系に変換するTransform。
    pub fn from_shading_normal_tangent(
        shading_normal: &Normal<C>,
        tangent: &Vector3<C>,
    ) -> Transform<C, ShadingTangent> {
        let shading_normal = shading_normal.to_vec3().normalize();
        let bitangent = tangent.to_vec3().cross(shading_normal).normalize();
        let tangent = bitangent.cross(shading_normal);
        let matrix = glam::Mat4::from_cols(
            tangent.extend(0.0),
            bitangent.extend(0.0),
            shading_normal.extend(0.0),
            glam::vec4(0.0, 0.0, 0.0, 1.0),
        );
        Transform::from_matrix(matrix.inverse())
    }
}
impl Transform<ShadingTangent, NormalMapTangent> {
    /// ノーマルマップの法線からShadingTangent空間からNormalMapTangent空間への基底変換Transformを作成する。
    ///
    /// 標準的なシェーディングタンジェント空間（Z+が法線）から、法線マップで変更されたタンジェント空間への変換。
    ///
    /// # Arguments
    /// - `normal_map_normal` - ノーマルマップから取得したシェーディングタンジェント空間での法線
    ///
    /// # Returns
    /// シェーディングタンジェント空間からノーマルマップタンジェント空間への変換Transform
    pub fn from_normal_map(
        normal_map_normal: &Normal<ShadingTangent>,
    ) -> Transform<ShadingTangent, NormalMapTangent> {
        let perturbed_normal = normal_map_normal.to_vec3().normalize();

        // 摂動された法線がZ軸とほぼ同じ場合は変換不要
        if (perturbed_normal - glam::Vec3::Z).length() < 1e-6 {
            return Transform::identity();
        }

        // 新しい基底を構築：perturbed_normalをZ軸とする
        let new_z = perturbed_normal;

        // X軸の候補を選択（Z軸と平行でない方向）
        let candidate_x = if new_z.dot(glam::Vec3::X).abs() < 0.9 {
            glam::Vec3::X
        } else {
            glam::Vec3::Y
        };

        // グラム・シュミット法で正規直交基底を構築
        let new_x = (candidate_x - new_z.dot(candidate_x) * new_z).normalize();
        let new_y = new_z.cross(new_x).normalize();

        // 新しい基底行列を構築（列ベクトルとして配置）
        let matrix = glam::Mat4::from_cols(
            new_x.extend(0.0),
            new_y.extend(0.0),
            new_z.extend(0.0),
            glam::vec4(0.0, 0.0, 0.0, 1.0),
        );

        Transform::from_matrix(matrix.inverse())
    }
}
