//! プラスチックマテリアル実装。

use std::sync::Arc;

use math::{Normal, Transform, Vector3, VertexNormalTangent};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::SpectrumParameter;
use crate::{
    BsdfSurfaceMaterial, FloatParameter, Material, MaterialEvaluationResult, MaterialSample,
    NormalParameter, SurfaceInteraction, SurfaceMaterial, material::bsdf::DielectricBsdf,
};
use spectrum::ConstantSpectrum;

/// プラスチックマテリアル。
/// roughnessパラメータに応じて完全鏡面反射・透過またはマイクロファセット反射・透過を行う定数屈折率の誘電体マテリアル。
pub struct PlasticMaterial {
    /// 屈折率（定数値）
    eta: f32,
    /// 色
    color: SpectrumParameter,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// Thin Surfaceフラグ
    thin_surface: bool,
    /// 表面の粗さパラメータ
    roughness: FloatParameter,
}
impl PlasticMaterial {
    /// 新しいPlasticMaterialを作成する。
    /// roughnessが0に限りなく近い場合は完全鏡面、それ以外はマイクロファセット。
    ///
    /// # Arguments
    /// - `eta` - 屈折率（定数値）
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_surface` - Thin Surfaceフラグ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面）
    pub fn new(
        eta: f32,
        color: SpectrumParameter,
        normal: NormalParameter,
        thin_surface: bool,
        roughness: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            eta,
            color,
            normal,
            thin_surface,
            roughness,
        })
    }

    /// 一般的なプラスチック用のPlasticMaterialを作成する（屈折率 1.5）。
    ///
    /// # Arguments
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_surface` - Thin Surfaceフラグ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面）
    pub fn new_generic(
        normal: NormalParameter,
        thin_surface: bool,
        roughness: FloatParameter,
    ) -> Material {
        Self::new(
            1.5,
            SpectrumParameter::Constant(ConstantSpectrum::new(1.0)),
            normal,
            thin_surface,
            roughness,
        )
    }

    /// アクリル用のPlasticMaterialを作成する（屈折率 1.49）。
    ///
    /// # Arguments
    /// - `color` - 色パラメータ
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_surface` - Thin Surfaceフラグ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面）
    pub fn new_acrylic(
        color: SpectrumParameter,
        normal: NormalParameter,
        thin_surface: bool,
        roughness: FloatParameter,
    ) -> Material {
        Self::new(1.49, color, normal, thin_surface, roughness)
    }

    /// ポリカーボネート用のPlasticMaterialを作成する（屈折率 1.58）。
    ///
    /// # Arguments
    /// - `color` - 色パラメータ
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_surface` - Thin Surfaceフラグ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面）
    pub fn new_polycarbonate(
        color: SpectrumParameter,
        normal: NormalParameter,
        thin_surface: bool,
        roughness: FloatParameter,
    ) -> Material {
        Self::new(1.58, color, normal, thin_surface, roughness)
    }

    /// 定数屈折率をスペクトラムに変換する。
    fn get_eta(&self, _lambda: &SampledWavelengths) -> SampledSpectrum {
        SampledSpectrum::constant(self.eta)
    }
}

impl SurfaceMaterial for PlasticMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for PlasticMaterial {
    fn sample(
        &self,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialSample {
        let eta = self.get_eta(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);
        let transform_inv = transform.inverse();

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;

        // roughnessパラメータをサンプリング
        let roughness_value = self.roughness.sample(shading_point.uv);

        // 誘電体BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(
            eta,
            entering,
            self.thin_surface,
            roughness_value,
            roughness_value,
        );
        let mut bsdf_result = match dielectric_bsdf.sample(&wo_normalmap, uv, uc, lambda) {
            Some(result) => result,
            None => {
                // BSDFサンプリング失敗の場合
                return MaterialSample::failed();
            }
        };

        // 透過の場合、カラーフィルタを適用
        if bsdf_result.wi.dot(wo_normalmap) < 0.0 {
            let color_spectrum = self.color.sample(uv).sample(lambda);
            bsdf_result.f *= color_spectrum;
        }

        // 結果をシェーディングタンジェント空間に変換して返す
        let wi_shading = &transform_inv * &bsdf_result.wi;

        MaterialSample::new(
            bsdf_result.f,
            wi_shading,
            bsdf_result.pdf,
            bsdf_result.sample_type,
        )
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialEvaluationResult {
        // プラスチックの光学特性を取得
        let eta = self.get_eta(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // roughnessパラメータをサンプリング
        let roughness_value = self.roughness.sample(shading_point.uv);

        // 誘電体BSDF評価（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(
            eta,
            entering,
            self.thin_surface,
            roughness_value,
            roughness_value,
        );
        let mut f = dielectric_bsdf.evaluate(&wo_normalmap, &wi_normalmap);

        // 透過の場合、カラーフィルタを適用
        if wi_normalmap.dot(wo_normalmap) < 0.0 {
            let color_spectrum = self.color.sample(shading_point.uv).sample(lambda);
            f *= color_spectrum;
        }

        MaterialEvaluationResult { f, pdf: 1.0 }
    }

    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> f32 {
        // プラスチックの光学特性を取得
        let eta = self.get_eta(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // roughnessパラメータをサンプリング
        let roughness_value = self.roughness.sample(shading_point.uv);

        // 誘電体BSDF PDF計算（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(
            eta,
            entering,
            self.thin_surface,
            roughness_value,
            roughness_value,
        );
        dielectric_bsdf.pdf(&wo_normalmap, &wi_normalmap)
    }

    fn sample_albedo_spectrum(
        &self,
        _uv: glam::Vec2,
        _lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        // プラスチックの場合、アルベドは1.0（損失なし）
        SampledSpectrum::constant(1.0)
    }
}
