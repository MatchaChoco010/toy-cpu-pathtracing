//! 空間上のレイを表す構造体を定義するモジュール。

use crate::{CoordinateSystem, Normal, Point3, Vector3};

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
        let origin = *origin.as_ref();
        let dir = *dir.as_ref();
        Self { origin, dir }
    }

    /// Rayの原点を少しだけdirの方向に移動させたRayを返す。
    #[inline(always)]
    pub fn move_forward(&self, distance: f32) -> Self {
        let origin = self.origin + self.dir * distance;
        Self::new(origin, self.dir)
    }
}
impl<C: CoordinateSystem> AsRef<Ray<C>> for Ray<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Ray<C> {
        self
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
        .vector_to(ps[1])
        .cross(ps[0].vector_to(ps[2]))
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
    assert!(det.is_finite());

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
        const EPSILON: f32 = f32::EPSILON * 0.5;
        (n as f32 * EPSILON) / (1.0 - n as f32 * EPSILON)
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
            .vector_to(ps[1])
            .cross(ps[0].vector_to(ps[2]))
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
