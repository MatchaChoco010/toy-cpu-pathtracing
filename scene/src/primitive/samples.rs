//! シーン上の点をサンプルした結果を持つ構造体を定義するモジュール。

use std::sync::Arc;

use math::{CoordinateSystem, Normal, Point3, Transform, Vector3};
use spectrum::SampledSpectrum;
use util_macros::impl_binary_ops;

use crate::SurfaceMaterial;
use crate::{SceneId, primitive::PrimitiveIndex};

/// サンプルしたジオメトリを特定するための情報を持つ列挙型。
#[derive(Debug, Clone, Copy)]
pub enum InteractGeometryInfo {
    /// ジオメトリにインデックスが必要ない場合。
    None,
    /// サンプルした三角形メッシュの三角形を特定するための情報。
    TriangleMesh {
        /// サンプルした三角形メッシュのインデックス。
        triangle_index: u32,
    },
}

/// シーンの表面をサンプルした結果の情報を持つ構造体。
pub struct SurfaceInteraction<Id: SceneId, C: CoordinateSystem> {
    /// サンプルした位置。
    pub position: Point3<C>,
    /// サンプルした幾何法線。
    pub normal: Normal<C>,
    /// サンプルしたシェーディング法線。
    pub shading_normal: Normal<C>,
    /// サンプルしたタンジェントベクトル。
    pub tangent: Vector3<C>,
    /// サンプルしたUV座標。
    pub uv: glam::Vec2,
    /// マテリアル。
    pub material: Arc<SurfaceMaterial<Id>>,
    /// サンプルしたプリミティブのインデックス。
    pub primitive_index: PrimitiveIndex<Id>,
    /// サンプルしたジオメトリの追加情報。
    pub geometry_info: InteractGeometryInfo,
}
#[impl_binary_ops(Mul)]
fn mul<Id: SceneId, From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &SurfaceInteraction<Id, From>,
) -> SurfaceInteraction<Id, To> {
    SurfaceInteraction {
        position: lhs * rhs.position,
        normal: lhs * rhs.normal,
        shading_normal: lhs * rhs.shading_normal,
        tangent: lhs * rhs.tangent,
        uv: rhs.uv,
        material: rhs.material.clone(),
        primitive_index: rhs.primitive_index,
        geometry_info: rhs.geometry_info,
    }
}

/// ライト上のサンプルされた放射輝度情報とpdfを持つ構造体。
pub struct AreaLightSampleRadiance<Id: SceneId, C: CoordinateSystem> {
    /// サンプルした放射輝度。
    pub radiance: SampledSpectrum,
    /// サンプルのpdf。
    pub pdf: f32,
    /// 幾何項。
    pub g: f32,
    /// シーンをサンプルした結果の情報。
    pub interaction: SurfaceInteraction<Id, C>,
}

/// ライトからの放射照度情報を持つ構造体。
pub struct DeltaPointLightIrradiance<C: CoordinateSystem> {
    /// 計算した放射照度。
    pub irradiance: SampledSpectrum,
    /// 光源の位置。
    pub position: Point3<C>,
}

/// ライトからの放射照度情報を持つ構造体。
pub struct DeltaDirectionalLightLightIrradiance<C: CoordinateSystem> {
    /// 計算した放射照度。
    pub irradiance: SampledSpectrum,
    /// 光源の方向。
    pub direction: Vector3<C>,
}

/// ライトのサンプル結果を持つ列挙子。
pub enum LightIntensity<Id: SceneId, C: CoordinateSystem> {
    /// 面積光源 からサンプルした放射輝度情報。
    RadianceAreaLight(AreaLightSampleRadiance<Id, C>),
    /// デルタ点光源の放射照度情報。
    IrradianceDeltaPointLight(DeltaPointLightIrradiance<C>),
    /// デルタ方向光源の放射照度情報。
    IrradianceDeltaDirectionalLight(DeltaDirectionalLightLightIrradiance<C>),
}
