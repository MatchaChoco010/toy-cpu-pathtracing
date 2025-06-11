//! シーン上の点やBSDFをサンプルした結果を持つ構造体を定義するモジュール。

use math::{CoordinateSystem, Normal, Point3, Tangent, Transform, Vector3};
use spectrum::SampledSpectrum;
use util_macros::impl_binary_ops;

use crate::material::Material;
use crate::{SceneId, primitive::PrimitiveIndex};

/// マテリアル評価結果を表す構造体。
#[derive(Debug, Clone)]
pub struct MaterialEvaluationResult {
    /// BSDF値
    pub f: SampledSpectrum,
    /// レイヤー選択PDF（レイヤーマテリアルでの確率的BSDF選択用）
    pub pdf: f32,
    /// 選択されたレイヤーの法線マップ（接空間）
    pub normal: Normal<Tangent>,
}

// Bsdfのサンプリング結果を表す列挙型。
#[derive(Debug, Clone)]
pub enum BsdfSample {
    Bsdf {
        f: spectrum::SampledSpectrum,
        pdf: f32,
        wi: math::Vector3<math::Tangent>,
        normal: Normal<Tangent>,
    },
    Specular {
        f: spectrum::SampledSpectrum,
        wi: math::Vector3<math::Tangent>,
        normal: Normal<Tangent>,
    },
}

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
    pub material: Material,
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
    /// ライト表面の法線（幾何項計算を遅延実行するため）。
    pub light_normal: Normal<C>,
    /// サンプルの方向要素のpdf。
    pub pdf_dir: f32,
    /// シーンをサンプルした結果の情報。
    pub interaction: SurfaceInteraction<Id, C>,
}

/// ライトからの放射強度情報を持つ構造体。
pub struct DeltaPointLightIntensity<C: CoordinateSystem> {
    /// 計算した放射強度。
    pub intensity: SampledSpectrum,
    /// 光源の位置。
    pub position: Point3<C>,
}

/// ライトからの放射強度情報を持つ構造体。
pub struct DeltaDirectionalLightIntensity<C: CoordinateSystem> {
    /// 計算した放射強度。
    pub intensity: SampledSpectrum,
    /// 光源の方向。
    pub direction: Vector3<C>,
}

/// ライトのサンプル結果を持つ列挙子。
pub enum LightIntensity<Id: SceneId, C: CoordinateSystem> {
    /// 面積光源 からサンプルした放射輝度情報。
    RadianceAreaLight(AreaLightSampleRadiance<Id, C>),
    /// デルタ点光源の放射強度情報。
    IntensityDeltaPointLight(DeltaPointLightIntensity<C>),
    /// デルタ方向光源の放射強度情報。
    IntensityDeltaDirectionalLight(DeltaDirectionalLightIntensity<C>),
}
