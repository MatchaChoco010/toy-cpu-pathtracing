//! 金属マテリアル実装。

use std::sync::Arc;

use math::{Normal, Transform, Vector3, VertexNormalTangent};
use spectrum::{SampledWavelengths, presets};

use crate::{
    BsdfSurfaceMaterial, FloatParameter, Material, MaterialEvaluationResult, MaterialSample,
    NormalParameter, SurfaceInteraction, SurfaceMaterial, material::bsdf::{ConductorBsdf, fresnel_complex},
};

/// 金属の種類を表す列挙型。
#[derive(Debug, Clone, Copy)]
pub enum MetalType {
    /// 金
    Gold,
    /// 銀
    Silver,
    /// 銅
    Copper,
    /// アルミニウム
    Aluminum,
    /// 真鍮
    Brass,
}

/// 金属マテリアル。
/// roughnessパラメータに応じて完全鏡面反射またはマイクロファセット反射を行う。
pub struct MetalMaterial {
    /// 金属の種類
    metal_type: MetalType,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// 表面の粗さパラメータ
    roughness: FloatParameter,
}

impl MetalMaterial {
    /// 新しいMetalMaterialを作成する。
    ///
    /// # Arguments
    /// - `metal_type` - 金属の種類
    /// - `normal` - ノーマルマップパラメータ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面反射）
    pub fn new(
        metal_type: MetalType,
        normal: NormalParameter,
        roughness: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            metal_type,
            normal,
            roughness,
        })
    }

    /// 新しいMetalMaterialを作成する（roughnessパラメータ付き）。
    ///
    /// # Arguments
    /// - `metal_type` - 金属の種類
    /// - `normal` - ノーマルマップパラメータ
    /// - `roughness` - 表面の粗さパラメータ
    pub fn new_with_roughness(
        metal_type: MetalType,
        normal: NormalParameter,
        roughness: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            metal_type,
            normal,
            roughness,
        })
    }

    /// roughnessからalpha値を計算する。
    /// pbrt-v4のissue #479に従い、より知覚的に均一なroughness^2を使用。
    /// 参考: https://github.com/mmp/pbrt-v4/issues/479
    fn roughness_to_alpha(roughness: f32) -> f32 {
        roughness * roughness
    }

    /// 金属の屈折率（実部）を取得する。
    fn get_eta(&self, lambda: &SampledWavelengths) -> spectrum::SampledSpectrum {
        let spectrum = match self.metal_type {
            MetalType::Gold => presets::au_eta(),
            MetalType::Silver => presets::ag_eta(),
            MetalType::Copper => presets::cu_eta(),
            MetalType::Aluminum => presets::al_eta(),
            MetalType::Brass => presets::cu_zn_eta(),
        };
        spectrum.sample(lambda)
    }

    /// 金属の消散係数（虚部）を取得する。
    fn get_k(&self, lambda: &SampledWavelengths) -> spectrum::SampledSpectrum {
        let spectrum = match self.metal_type {
            MetalType::Gold => presets::au_k(),
            MetalType::Silver => presets::ag_k(),
            MetalType::Copper => presets::cu_k(),
            MetalType::Aluminum => presets::al_k(),
            MetalType::Brass => presets::cu_zn_k(),
        };
        spectrum.sample(lambda)
    }
}
impl SurfaceMaterial for MetalMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for MetalMaterial {
    fn sample(
        &self,
        _uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialSample {
        // 金属の光学特性を取得
        let eta = self.get_eta(lambda);
        let k = self.get_k(lambda);

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

        // roughnessパラメータをサンプリングしてalpha値に変換
        let roughness_value = self.roughness.sample(shading_point.uv);
        let alpha = Self::roughness_to_alpha(roughness_value);

        // 導体BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let conductor_bsdf = ConductorBsdf::new(eta, k, alpha, alpha);
        let bsdf_result = match conductor_bsdf.sample(&wo_normalmap, uv) {
            Some(result) => result,
            None => {
                // BSDFサンプリング失敗の場合
                return MaterialSample::failed(normal_map);
            }
        };

        // 結果をシェーディングタンジェント空間に変換して返す
        let wi_shading = &transform_inv * &bsdf_result.wi;

        // 幾何学的制約チェック: wiとwoが幾何法線に対して同じ側にあるかチェック
        let geometry_normal = shading_point.normal;
        let wi_cos_geometric = geometry_normal.dot(wi_shading);
        let wo_cos_geometric = geometry_normal.dot(wo);

        if wi_cos_geometric.signum() != wo_cos_geometric.signum() {
            // 不透明マテリアルなので表面貫通サンプルは無効
            return MaterialSample::failed(normal_map);
        }

        MaterialSample::new(
            bsdf_result.f,
            wi_shading,
            bsdf_result.pdf,
            bsdf_result.sample_type,
            normal_map,
        )
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialEvaluationResult {
        // 金属の光学特性を取得
        let eta = self.get_eta(lambda);
        let k = self.get_k(lambda);

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

        // 幾何学的制約チェック: wiとwoが幾何法線に対して同じ側にあるかチェック
        let geometry_normal = shading_point.normal;
        let wi_cos_geometric = geometry_normal.dot(wi);
        let wo_cos_geometric = geometry_normal.dot(wo);
        if wi_cos_geometric.signum() != wo_cos_geometric.signum() {
            // 不透明マテリアルなので表面貫通は寄与0
            return MaterialEvaluationResult {
                f: spectrum::SampledSpectrum::zero(),
                pdf: 1.0,
                normal: normal_map,
            };
        }

        // roughnessパラメータをサンプリングしてalpha値に変換
        let roughness_value = self.roughness.sample(shading_point.uv);
        let alpha = Self::roughness_to_alpha(roughness_value);

        // 導体BSDF評価（ノーマルマップタンジェント空間で実行）
        let conductor_bsdf = ConductorBsdf::new(eta, k, alpha, alpha);
        let f = conductor_bsdf.evaluate(&wo_normalmap, &wi_normalmap);

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // 単一BSDFなので選択確率は1.0
            normal: normal_map,
        }
    }

    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> f32 {
        // 金属の光学特性を取得
        let eta = self.get_eta(lambda);
        let k = self.get_k(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // 幾何学的制約チェック: wiとwoが幾何法線に対して同じ側にあるかチェック
        let geometry_normal = shading_point.normal;
        let wi_cos_geometric = geometry_normal.dot(wi);
        let wo_cos_geometric = geometry_normal.dot(wo);
        if wi_cos_geometric.signum() != wo_cos_geometric.signum() {
            // 不透明マテリアルなので表面貫通のPDFは0
            return 0.0;
        }

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // roughnessパラメータをサンプリングしてalpha値に変換
        let roughness_value = self.roughness.sample(shading_point.uv);
        let alpha = Self::roughness_to_alpha(roughness_value);

        // 導体BSDF PDF計算（ノーマルマップタンジェント空間で実行）
        let conductor_bsdf = ConductorBsdf::new(eta, k, alpha, alpha);
        conductor_bsdf.pdf(&wo_normalmap, &wi_normalmap)
    }

    fn sample_albedo_spectrum(
        &self,
        _uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        // 金属の場合、アルベドは垂直入射でのFresnel反射率に近似
        let eta = self.get_eta(lambda);
        let k = self.get_k(lambda);

        // Fresnel反射率を計算（垂直入射）
        fresnel_complex(1.0, &eta, &k)
    }
}
