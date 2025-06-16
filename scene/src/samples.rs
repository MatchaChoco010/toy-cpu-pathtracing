//! シーン上の点やマテリアルをサンプルした結果を持つ構造体を定義するモジュール。

use math::{CoordinateSystem, GeometryTangent, Normal, Point3, ShadingTangent, Transform, Vector3};
use spectrum::SampledSpectrum;
use util_macros::impl_binary_ops;

use crate::material::Material;

/// マテリアル評価結果を表す構造体。
#[derive(Debug, Clone)]
pub struct MaterialEvaluationResult {
    /// BSDF値
    pub f: SampledSpectrum,
    /// レイヤー選択PDF（レイヤーマテリアルでの確率的BSDF選択用）
    pub pdf: f32,
    /// 選択されたレイヤーの法線マップ（シェーディング接空間）
    pub normal: Normal<ShadingTangent>,
}

/// NonSpecular方向サンプリング結果。
#[derive(Debug, Clone)]
pub struct NonSpecularDirectionSample {
    pub f: spectrum::SampledSpectrum,
    pub pdf: f32,
    pub wi: math::Vector3<ShadingTangent>,
}

/// Specular方向サンプリング結果。
#[derive(Debug, Clone)]
pub struct SpecularDirectionSample {
    pub f: spectrum::SampledSpectrum,
    pub wi: math::Vector3<ShadingTangent>,
}

/// マテリアルの方向サンプリング結果を表す列挙型。
#[derive(Debug, Clone)]
pub enum MaterialSample {
    NonSpecular {
        sample: Option<NonSpecularDirectionSample>,
        normal: Normal<ShadingTangent>,
    },
    Specular {
        sample: Option<SpecularDirectionSample>,
        normal: Normal<ShadingTangent>,
    },
}
impl MaterialSample {
    /// Specularのサンプリングかどうか。
    pub fn is_specular(&self) -> bool {
        matches!(self, MaterialSample::Specular { .. })
    }

    /// 非Specularのサンプリングかどうか。
    pub fn is_non_specular(&self) -> bool {
        matches!(self, MaterialSample::NonSpecular { .. })
    }

    /// サンプリングが成功したかどうか。
    pub fn is_sampled(&self) -> bool {
        match self {
            MaterialSample::NonSpecular { sample, .. } => sample.is_some(),
            MaterialSample::Specular { sample, .. } => sample.is_some(),
        }
    }
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
pub struct SurfaceInteraction<C: CoordinateSystem> {
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
}
#[impl_binary_ops(Mul)]
fn mul<From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &SurfaceInteraction<From>,
) -> SurfaceInteraction<To> {
    SurfaceInteraction {
        position: lhs * rhs.position,
        normal: lhs * rhs.normal,
        shading_normal: lhs * rhs.shading_normal,
        tangent: lhs * rhs.tangent,
        uv: rhs.uv,
        material: rhs.material.clone(),
    }
}
impl<C: CoordinateSystem> SurfaceInteraction<C> {
    /// ShadingTangent座標系に変換するTransformを取得する。
    pub fn shading_transform(&self) -> Transform<C, ShadingTangent> {
        Transform::from_shading_normal_tangent(&self.shading_normal, &self.tangent)
    }

    /// GeometryTangent座標系に変換するTransformを取得する。
    pub fn geometry_transform(&self) -> Transform<C, GeometryTangent> {
        Transform::from_geometry_normal_tangent(&self.normal, &self.tangent)
    }
}

/// ライト上のサンプルされた放射輝度情報とpdfを持つ構造体。
pub struct AreaLightSampleRadiance<C: CoordinateSystem> {
    /// サンプルした放射輝度。
    pub radiance: SampledSpectrum,
    /// サンプルのpdf。
    pub pdf: f32,
    /// ライト表面の法線（幾何項計算を遅延実行するため）。
    pub light_normal: Normal<C>,
    /// サンプルの方向要素のpdf。
    pub pdf_dir: f32,
    /// シーンをサンプルした結果の情報。
    pub interaction: SurfaceInteraction<C>,
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
pub enum LightIntensity<C: CoordinateSystem> {
    /// 面積光源 からサンプルした放射輝度情報。
    RadianceAreaLight(AreaLightSampleRadiance<C>),
    /// デルタ点光源の放射強度情報。
    IntensityDeltaPointLight(DeltaPointLightIntensity<C>),
    /// デルタ方向光源の放射強度情報。
    IntensityDeltaDirectionalLight(DeltaDirectionalLightIntensity<C>),
}
