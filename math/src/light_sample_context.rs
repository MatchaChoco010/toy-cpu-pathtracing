//! 空間上のライトサンプルの際のシェーディング店の情報を持つ構造体を定義するモジュール。

use util_macros::impl_binary_ops;

use crate::{CoordinateSystem, Normal, Point3, Transform};

/// ライト上をサンプルするためのコンテキスト。
/// サンプルする際のシェーディング点の情報を持つ。
#[derive(Debug, Clone)]
pub struct LightSampleContext<C: CoordinateSystem> {
    pub position: Point3<C>,
    pub normal: Normal<C>,
    pub shading_normal: Normal<C>,
}
impl<C: CoordinateSystem> AsRef<LightSampleContext<C>> for LightSampleContext<C> {
    #[inline(always)]
    fn as_ref(&self) -> &LightSampleContext<C> {
        &self
    }
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
