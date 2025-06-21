//! 拡散反射（Lambert）マテリアル実装。

use std::sync::Arc;

use math::{Normal, Transform, Vector3, VertexNormalTangent};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample, NormalParameter,
    NormalizedLambertBsdf, SpectrumParameter, SurfaceInteraction, SurfaceMaterial,
};

/// 拡散反射のみを行うLambertマテリアル。
/// テクスチャ対応の反射率とノーマルマップパラメータを持つ。
pub struct LambertMaterial {
    /// 反射率パラメータ
    albedo: SpectrumParameter,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// 内部でBSDF計算を行う構造体
    bsdf: NormalizedLambertBsdf,
}

impl LambertMaterial {
    /// 新しいLambertMaterialを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率パラメータ
    /// - `normal` - ノーマルマップパラメータ
    pub fn new(albedo: SpectrumParameter, normal: NormalParameter) -> Material {
        Arc::new(Self {
            albedo,
            normal,
            bsdf: NormalizedLambertBsdf::new(),
        })
    }
}
impl SurfaceMaterial for LambertMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for LambertMaterial {
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialSample {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

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

        // BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let bsdf_result = match self.bsdf.sample(&albedo, &wo_normalmap, uv) {
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
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

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

        // BSDF評価（ノーマルマップタンジェント空間で実行）
        let f = self.bsdf.evaluate(&albedo, &wo_normalmap, &wi_normalmap);

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // 単一BSDFなので選択確率は1.0
            normal: normal_map,
        }
    }

    fn pdf(
        &self,
        _lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> f32 {
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

        // BSDF PDF計算（ノーマルマップタンジェント空間で実行）
        self.bsdf.pdf(&wo_normalmap, &wi_normalmap)
    }

    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        self.albedo.sample(uv).sample(lambda)
    }
}
