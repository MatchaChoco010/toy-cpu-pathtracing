//! 拡散反射（Lambert）マテリアル実装。

use math::{Normal, ShadingTangent, Transform, Vector3};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample,
    NonSpecularDirectionSample, NormalParameter, NormalizedLambertBsdf, SceneId, SpectrumParameter,
    SurfaceInteraction, SurfaceMaterial, material::bsdf::BsdfSample,
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
        std::sync::Arc::new(Self {
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
}
impl<Id: SceneId> BsdfSurfaceMaterial<Id> for LambertMaterial {
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
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
                return MaterialSample::NonSpecular {
                    sample: None,
                    normal: normal_map,
                };
            }
        };

        // 結果をシェーディングタンジェント空間に変換して返す
        match bsdf_result {
            BsdfSample::Bsdf { f, pdf, wi } => {
                let wi_shading = &transform_inv * &wi;

                // 幾何学的制約チェック: wiとwoが幾何法線に対して同じ側にあるかチェック
                let geometry_normal = shading_point.normal;
                let wi_cos_geometric = geometry_normal.dot(wi_shading);
                let wo_cos_geometric = geometry_normal.dot(wo);
                let sample = if wi_cos_geometric.signum() != wo_cos_geometric.signum() {
                    // 不透明マテリアルなので表面貫通サンプルは無効
                    None
                } else {
                    Some(NonSpecularDirectionSample {
                        f,
                        pdf,
                        wi: wi_shading,
                    })
                };

                MaterialSample::NonSpecular {
                    sample,
                    normal: normal_map,
                }
            }
            _ => unreachable!("Lambert material should not produce specular samples"),
        }
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
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
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
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
