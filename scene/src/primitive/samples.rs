//! シーン上の点をサンプルした結果を持つ構造体を定義するモジュール。

use math::{CoordinateSystem, Normal, Point3, Transform, Vector3};
use spectrum::SampledSpectrum;
use util_macros::impl_binary_ops;

use crate::{SceneId, primitive::PrimitiveIndex};

/// サンプルしたジオメトリを特定するための情報を持つ列挙型。
#[derive(Debug, Clone, Copy)]
pub enum InteractGeometryInfo {
    /// サンプルした三角形メッシュの三角形を特定するための情報。
    TriangleMesh {
        /// サンプルした三角形メッシュのインデックス。
        triangle_index: u32,
    },
}

/// シーンをサンプルした結果の情報を持つ列挙型。
pub enum Interaction<Id: SceneId, C: CoordinateSystem> {
    Surface {
        /// サンプルした位置。
        position: Point3<C>,
        /// サンプルした幾何法線。
        normal: Normal<C>,
        /// サンプルしたシェーディング法線。
        shading_normal: Normal<C>,
        /// サンプルしたタンジェントベクトル。
        tangent: Vector3<C>,
        /// サンプルしたUV座標。
        uv: glam::Vec2,
        /// サンプルしたプリミティブのインデックス。
        primitive_index: PrimitiveIndex<Id>,
        /// サンプルしたジオメトリの追加情報。
        geometry_info: InteractGeometryInfo,
    },
    // Medium {
    //     ...
    // },
}
#[impl_binary_ops(Mul)]
fn mul<Id: SceneId, From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Interaction<Id, From>,
) -> Interaction<Id, To> {
    match rhs {
        Interaction::Surface {
            position,
            normal,
            shading_normal,
            tangent,
            uv,
            primitive_index,
            geometry_info,
        } => Interaction::Surface {
            position: lhs * position,
            normal: lhs * normal,
            shading_normal: lhs * shading_normal,
            tangent: lhs * tangent,
            uv: *uv,
            primitive_index: *primitive_index,
            geometry_info: *geometry_info,
        },
    }
}

/// ライト上のサンプルされた放射輝度情報とPDFを持つ構造体。
pub struct LightSampleRadiance<Id: SceneId, C: CoordinateSystem> {
    /// サンプルした放射輝度。
    pub radiance: SampledSpectrum,
    /// サンプルのPDF。
    pub pdf: f32,
    /// シーンをサンプルした結果の情報。
    pub interaction: Interaction<Id, C>,
}

/// ライト上のサンプルされた放射照度情報を持つ構造体。
pub struct LightIrradiance {
    /// 計算した放射照度。
    pub irradiance: SampledSpectrum,
}
