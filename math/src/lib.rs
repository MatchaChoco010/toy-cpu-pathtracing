//! 数学関連のモジュール。
//! NaNを発生させない数学関数や、
//! 座標系を区別するベクトルや点、法線、レイ、変換行列などを定義する。

use std::marker::PhantomData;

use glam::{Mat4, Quat, Vec3};
use util_macros::impl_binary_ops;

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

/// コンパイル時の3乗根関数。
pub const fn cube_root(x: f32) -> f32 {
    // 0.0の場合は0.0を返す。
    if x == 0.0 {
        return 0.0;
    }

    // ビットを使って3乗根の初期近似値を生成
    let x_bits = x.abs().to_bits();
    let approx_bits = x_bits / 3 + 709921077; // ≒ (127 << 23) / 3
    let mut guess = f32::from_bits(approx_bits);

    // ニュートン法で3乗根を計算
    let mut i = 0;
    while i < 20 {
        guess = (2.0 * guess + x / (guess * guess)) / 3.0;
        i += 1;
    }

    // 符号を元に戻す
    guess.copysign(x.signum())
}

/// コンパイル時の平方根関数。
pub const fn square_root(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }

    // ビット演算による初期近似（Quick and Dirty法）
    let x_bits = x.to_bits();
    let approx_bits = (x_bits >> 1) + 0x1fc00000;
    let mut guess = f32::from_bits(approx_bits);

    // ニュートン–ラフソン反復（精度重視）
    let mut i = 0;
    while i < 10 {
        guess = 0.5 * (guess + x / guess);
        i += 1;
    }

    guess
}

/// glam::Mat3とglam::Mat3の積をコンパイル時に計算する関数。
pub const fn mat3_mul_mat3(m1: glam::Mat3, m2: glam::Mat3) -> glam::Mat3 {
    glam::Mat3::from_cols(
        glam::vec3(
            m1.x_axis.x * m2.x_axis.x + m1.y_axis.x * m2.x_axis.y + m1.z_axis.x * m2.x_axis.z,
            m1.x_axis.y * m2.x_axis.x + m1.y_axis.y * m2.x_axis.y + m1.z_axis.y * m2.x_axis.z,
            m1.x_axis.z * m2.x_axis.x + m1.y_axis.z * m2.x_axis.y + m1.z_axis.z * m2.x_axis.z,
        ),
        glam::vec3(
            m1.x_axis.x * m2.y_axis.x + m1.y_axis.x * m2.y_axis.y + m1.z_axis.x * m2.y_axis.z,
            m1.x_axis.y * m2.y_axis.x + m1.y_axis.y * m2.y_axis.y + m1.z_axis.y * m2.y_axis.z,
            m1.x_axis.z * m2.y_axis.x + m1.y_axis.z * m2.y_axis.y + m1.z_axis.z * m2.y_axis.z,
        ),
        glam::vec3(
            m1.x_axis.x * m2.z_axis.x + m1.y_axis.x * m2.z_axis.y + m1.z_axis.x * m2.z_axis.z,
            m1.x_axis.y * m2.z_axis.x + m1.y_axis.y * m2.z_axis.y + m1.z_axis.y * m2.z_axis.z,
            m1.x_axis.z * m2.z_axis.x + m1.y_axis.z * m2.z_axis.y + m1.z_axis.z * m2.z_axis.z,
        ),
    )
}

/// glam::Mat3とglam::Vec3の積をコンパイル時に計算する関数。
pub const fn mat3_mul_vec3(m: glam::Mat3, v: glam::Vec3) -> glam::Vec3 {
    glam::vec3(
        m.x_axis.x * v.x + m.y_axis.x * v.y + m.z_axis.x * v.z,
        m.x_axis.y * v.x + m.y_axis.y * v.y + m.z_axis.y * v.z,
        m.x_axis.z * v.x + m.y_axis.z * v.y + m.z_axis.z * v.z,
    )
}

/// glam::Mat3の逆行列をコンパイル時に計算する関数。
pub const fn mat3_inverse(m: glam::Mat3) -> glam::Mat3 {
    let det = m.x_axis.x * (m.y_axis.y * m.z_axis.z - m.z_axis.y * m.y_axis.z)
        - m.y_axis.x * (m.x_axis.y * m.z_axis.z - m.z_axis.y * m.x_axis.z)
        + m.z_axis.x * (m.x_axis.y * m.y_axis.z - m.y_axis.y * m.x_axis.z);
    let inv_det = 1.0 / det;

    glam::Mat3::from_cols(
        glam::vec3(
            (m.y_axis.y * m.z_axis.z - m.z_axis.y * m.y_axis.z) * inv_det,
            -(m.x_axis.y * m.z_axis.z - m.z_axis.y * m.x_axis.z) * inv_det,
            (m.x_axis.y * m.y_axis.z - m.y_axis.y * m.x_axis.z) * inv_det,
        ),
        glam::vec3(
            -(m.y_axis.x * m.z_axis.z - m.z_axis.x * m.y_axis.z) * inv_det,
            (m.x_axis.x * m.z_axis.z - m.z_axis.x * m.x_axis.z) * inv_det,
            -(m.x_axis.x * m.y_axis.z - m.y_axis.x * m.x_axis.z) * inv_det,
        ),
        glam::vec3(
            (m.y_axis.x * m.z_axis.y - m.z_axis.x * m.y_axis.y) * inv_det,
            -(m.x_axis.x * m.z_axis.y - m.z_axis.x * m.x_axis.y) * inv_det,
            (m.x_axis.x * m.y_axis.y - m.y_axis.x * m.x_axis.y) * inv_det,
        ),
    )
}

/// glam::Mat3のLUP分解（LU分解+ピボット行列）をコンパイル時に計算する関数。
pub const fn lup_decompose(m: glam::Mat3, epsilon: f32) -> (glam::Mat3, glam::Mat3, [usize; 3]) {
    let mut a = m.to_cols_array(); // [f32; 9]、列優先（column-major）
    let mut p = [0, 1, 2];
    let mut l = [0.0; 9];
    let mut u = [0.0; 9];

    let mut i = 0;
    while i < 3 {
        // ピボット選択
        let mut max_row = i;
        let mut max_val = a[i + 0 * 3].abs();

        let mut k = i + 1;
        while k < 3 {
            let v = a[i + k * 3].abs();
            if v > max_val {
                max_val = v;
                max_row = k;
            }
            k += 1;
        }

        // 特異行列の検出
        if max_val < epsilon {
            panic!("Matrix is singular or nearly singular");
        }

        // 行交換
        if max_row != i {
            let tmp = p[i];
            p[i] = p[max_row];
            p[max_row] = tmp;

            let mut j = 0;
            while j < 3 {
                let idx1 = j + i * 3;
                let idx2 = j + max_row * 3;
                let temp = a[idx1];
                a[idx1] = a[idx2];
                a[idx2] = temp;
                j += 1;
            }
        }

        // LU 分解処理
        let mut j = 0;
        while j < 3 {
            if j < i {
                l[j + i * 3] = 0.0;
            } else if j == i {
                l[j + i * 3] = 1.0;
            } else {
                l[j + i * 3] = a[i + j * 3] / a[i + i * 3];
            }
            u[i + j * 3] = if j < i { 0.0 } else { a[i + j * 3] };
            j += 1;
        }

        let mut j = i + 1;
        while j < 3 {
            let factor = a[i + j * 3] / a[i + i * 3];
            let mut k = i;
            while k < 3 {
                a[k + j * 3] -= factor * a[k + i * 3];
                k += 1;
            }
            j += 1;
        }

        i += 1;
    }

    (
        glam::Mat3::from_cols_array(&l),
        glam::Mat3::from_cols_array(&u),
        p,
    )
}

/// LUP分解の結果を使って連立方程式Ax = bの解xを解く関数。
pub const fn lup_solve(l: glam::Mat3, u: glam::Mat3, p: [usize; 3], b: glam::Vec3) -> glam::Vec3 {
    let l = l.to_cols_array();
    let u = u.to_cols_array();
    let b = b.to_array();

    // P * b の適用
    let pb0 = b[p[0]];
    let pb1 = b[p[1]];
    let pb2 = b[p[2]];

    // 前進代入：L * y = P * b
    // L は単位対角、下三角のみ意味がある
    let y0 = pb0;
    let y1 = pb1 - l[3] * y0; // l[3] = L[1][0]
    let y2 = pb2 - l[6] * y0 - l[7] * y1; // l[6] = L[2][0], l[7] = L[2][1]

    // 後退代入：U * x = y
    let x2 = y2 / u[8]; // u[8] = U[2][2]
    let x1 = (y1 - u[5] * x2) / u[4]; // u[4] = U[1][1], u[5] = U[1][2]
    let x0 = (y0 - u[1] * x1 - u[2] * x2) / u[0]; // u[0] = U[0][0], u[1]=U[0][1], u[2]=U[0][2]

    Vec3::new(x0, x1, x2)
}

/// 座標系のマーカー用トレイト。
pub trait CoordinateSystem: std::fmt::Debug + Clone + Copy {}

/// ワールド座標系を表すマーカー構造体。
#[derive(Debug, Clone, Copy)]
pub struct World;
impl CoordinateSystem for World {}

/// モデルローカル座標系を表すマーカー構造体。
#[derive(Debug, Clone, Copy)]
pub struct Local;
impl CoordinateSystem for Local {}

/// レンダリング座標系を表すマーカー構造体。
///
/// レンダリング座標系はカメラを原点にして座標軸はワールド座標系と平行な座標系。
/// 多くの場合、シーンにはワールド座標の軸と平行な直線が含まれることがあり、特に地面などは軸とズレていないことも多い。
/// そのため、カメラが斜めになったときでもレンダリングに使う座標系では
/// ワールド座標系と軸が平行がそのままの方がバウンディングボックスがタイトになりやすく、多少良いBVHが構築できうる。
#[derive(Debug, Clone, Copy)]
pub struct Render;
impl CoordinateSystem for Render {}

/// 座標系Cでの点を表す構造体。
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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

    /// 頂点を取得する。
    #[inline(always)]
    pub fn vertices(&self) -> [Point3<C>; 8] {
        let min = self.min.to_vec3();
        let max = self.max.to_vec3();
        [
            Point3::from(glam::Vec3::new(min.x, min.y, min.z)),
            Point3::from(glam::Vec3::new(max.x, min.y, min.z)),
            Point3::from(glam::Vec3::new(min.x, max.y, min.z)),
            Point3::from(glam::Vec3::new(max.x, max.y, min.z)),
            Point3::from(glam::Vec3::new(min.x, min.y, max.z)),
            Point3::from(glam::Vec3::new(max.x, min.y, max.z)),
            Point3::from(glam::Vec3::new(min.x, max.y, max.z)),
            Point3::from(glam::Vec3::new(max.x, max.y, max.z)),
        ]
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
    let mut min = glam::Vec3::splat(f32::INFINITY);
    let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
    for vertex in rhs.vertices() {
        let transformed_vertex = lhs * &vertex;
        min = min.min(transformed_vertex.to_vec3());
        max = max.max(transformed_vertex.to_vec3());
    }
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
    pub fn translation(translation: Vec3) -> Self {
        let matrix = Mat4::from_translation(translation);
        Transform::from_matrix(matrix)
    }

    /// 回転のTransformを作成する。
    #[inline(always)]
    pub fn rotation(rotation: Quat) -> Self {
        let matrix = Mat4::from_quat(rotation);
        Transform::from_matrix(matrix)
    }

    /// スケールのTransformを作成する。
    #[inline(always)]
    pub fn scaling(scale: Vec3) -> Self {
        let matrix = Mat4::from_scale(scale);
        Transform::from_matrix(matrix)
    }

    /// 平行移動をかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn translate(&self, translation: Vec3) -> Self {
        let matrix = Mat4::from_translation(translation) * self.matrix;
        Transform::from_matrix(matrix)
    }

    /// 回転をかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn rotate(&self, rotation: Quat) -> Self {
        let matrix = Mat4::from_quat(rotation) * self.matrix;
        Transform::from_matrix(matrix)
    }

    /// スケールをかけ合わせた新しいTransformを作成する。
    #[inline(always)]
    pub fn scale(&self, scale: Vec3) -> Self {
        let matrix = Mat4::from_scale(scale) * self.matrix;
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

/// 三角形の交差を表す構造体。
pub struct TriangleIntersection<C: CoordinateSystem> {
    pub t_hit: f32,
    pub position: Point3<C>,
    pub normal: Normal<C>,
    pub barycentric: [f32; 3],
}

/// 三角形とレイの交差を判定する関数。
pub fn intersect_triangle<C: CoordinateSystem>(
    ray: &Ray<C>,
    t_max: f32,
    ps: [Point3<C>; 3],
) -> Option<TriangleIntersection<C>> {
    // degenerateしていたらhitしない。
    if ps[0]
        .vector_to(&ps[1])
        .cross(&ps[0].vector_to(&ps[2]))
        .length_squared()
        == 0.0
    {
        return None;
    }

    // レイの原点からの頂点へのベクトルを得る。
    let p0_o = ps[0].to_vec3() - ray.origin.to_vec3();
    let p1_o = ps[1].to_vec3() - ray.origin.to_vec3();
    let p2_o = ps[2].to_vec3() - ray.origin.to_vec3();

    // レイの方向の最大の成分がzになるように座標系を並び替える。
    let d = ray.dir.to_vec3();
    let kz = d.abs().max_position();
    let kx = (kz + 1) % 3;
    let ky = (kx + 1) % 3;
    let d = glam::vec3(d[kx], d[ky], d[kz]);
    let mut p0_o = glam::vec3(p0_o[kx], p0_o[ky], p0_o[kz]);
    let mut p1_o = glam::vec3(p1_o[kx], p1_o[ky], p1_o[kz]);
    let mut p2_o = glam::vec3(p2_o[kx], p2_o[ky], p2_o[kz]);

    // xyに関するシアー変形を行う。
    let s_x = -d.x / d.z;
    let s_y = -d.y / d.z;
    let s_z = 1.0 / d.z;
    p0_o.x += s_x * p0_o.z;
    p0_o.y += s_y * p0_o.z;
    p1_o.x += s_x * p1_o.z;
    p1_o.y += s_y * p1_o.z;
    p2_o.x += s_x * p2_o.z;
    p2_o.y += s_y * p2_o.z;

    // エッジ関数の係数を求める。
    let mut e0 = p2_o.x * p1_o.y - p2_o.y * p1_o.x;
    let mut e1 = p0_o.x * p2_o.y - p0_o.y * p2_o.x;
    let mut e2 = p1_o.x * p0_o.y - p1_o.y * p0_o.x;

    // 精度が足りないときはf64にフォールバックする。
    if e0 == 0.0 || e1 == 0.0 || e2 == 0.0 {
        let p2tx_p1ty: f64 = p2_o.x as f64 * p1_o.y as f64;
        let p2ty_p1tx: f64 = p2_o.y as f64 * p1_o.x as f64;
        e0 = (p2tx_p1ty - p2ty_p1tx) as f32;
        let p0tx_p2ty: f64 = p0_o.x as f64 * p2_o.y as f64;
        let p0ty_p2tx: f64 = p0_o.y as f64 * p2_o.x as f64;
        e1 = (p0tx_p2ty - p0ty_p2tx) as f32;
        let p1tx_p0ty: f64 = p1_o.x as f64 * p0_o.y as f64;
        let p1ty_p0tx: f64 = p1_o.y as f64 * p0_o.x as f64;
        e2 = (p1tx_p0ty - p1ty_p0tx) as f32;
    }

    // エッジの係数の符号が異なる場合は、rayと三角形は交差しない。
    if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
        return None;
    }

    // 行列式が0.0の場合は、rayと三角形は交差しない。
    let det = e0 + e1 + e2;
    if det == 0.0 {
        return None;
    }

    // zのシアー変形を適用する。
    p0_o.z *= s_z;
    p1_o.z *= s_z;
    p2_o.z *= s_z;

    // スケールしたヒット距離を計算し0からt_maxの範囲内かを比較する。
    let t_scaled = e0 * p0_o.z + e1 * p1_o.z + e2 * p2_o.z;
    if det < 0.0 && (t_scaled >= 0.0 || t_scaled < t_max * det) {
        return None;
    } else if det > 0.0 && (t_scaled <= 0.0 || t_scaled > t_max * det) {
        return None;
    }

    // barycentric座標を計算する。
    let inv_det = 1.0 / det;
    let barycentric = [e0 * inv_det, e1 * inv_det, e2 * inv_det];

    // t_hitを計算する。
    let t_hit = t_scaled * inv_det;
    assert!(!t_hit.is_nan());

    // t_hitがゼロより大きいかを保守的にチェックする。
    const fn gamma(n: usize) -> f32 {
        const EPSILON: f32 = std::f32::EPSILON * 0.5;
        (n as f32 * EPSILON) / ((1 - n) as f32 * EPSILON)
    }
    // t_hitのエラー幅のdelta_zの項を計算する。
    let max_zt = glam::vec3(p0_o.z, p1_o.z, p2_o.z).abs().max_element();
    let delta_z = gamma(3) * max_zt;
    // t_hitのエラー幅のdelta_x, delta_yの項を計算する。
    let max_xt = glam::vec3(p0_o.x, p1_o.x, p2_o.x).abs().max_element();
    let max_yt = glam::vec3(p0_o.y, p1_o.y, p2_o.y).abs().max_element();
    let delta_x = gamma(5) * max_xt;
    let delta_y = gamma(5) * max_yt;
    // t_hitのエラー幅のdelta_eの項を計算する。
    let delta_e = 2.0 * (gamma(2) * max_xt * max_yt + delta_y * max_xt + delta_x * max_yt);
    // t_hitのエラー幅のdelta_tの項を計算する。
    let max_e = glam::vec3(e0, e1, e2).abs().max_element();
    let delta_t =
        3.0 * (gamma(3) * max_e * max_zt + delta_e * max_zt + delta_z * max_e) * inv_det.abs();
    // t_hitのエラー幅よりt_hit小さければヒットしないものとして扱う。
    if t_hit < delta_t {
        return None;
    }

    // 交差した位置を求める。
    let position = Point3::from(
        ps[0].to_vec3() * barycentric[0]
            + ps[1].to_vec3() * barycentric[1]
            + ps[2].to_vec3() * barycentric[2],
    );

    // 幾何法線を求める。
    let normal = Normal::from(
        ps[0]
            .vector_to(&ps[1])
            .cross(ps[0].vector_to(&ps[2]))
            .normalize()
            .to_vec3(),
    );

    Some(TriangleIntersection {
        t_hit,
        position,
        normal,
        barycentric,
    })
}
