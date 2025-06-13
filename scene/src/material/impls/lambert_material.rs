//! 拡散反射（Lambert）マテリアル実装。

use math::{Normal, ShadingTangent, Transform, Vector3};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSurfaceMaterial, Material, MaterialDirectionSample, MaterialEvaluationResult,
    NormalParameter, NormalizedLambertBsdf, SceneId, SpectrumParameter, SurfaceInteraction,
    SurfaceMaterial,
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
    ) -> Option<MaterialDirectionSample> {
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
        let bsdf_result = self.bsdf.sample(&albedo, &wo_normalmap, uv)?;

        // 結果をシェーディングタンジェント空間に変換して返す
        match bsdf_result {
            crate::material::bsdf::BsdfSample::Bsdf { f, pdf, wi } => {
                let wi_shading = &transform_inv * &wi;
                Some(MaterialDirectionSample::Bsdf {
                    f,
                    pdf,
                    wi: wi_shading,
                    normal: normal_map,
                })
            }
            crate::material::bsdf::BsdfSample::Specular { f, wi } => {
                let wi_shading = &transform_inv * &wi;
                Some(MaterialDirectionSample::Specular {
                    f,
                    wi: wi_shading,
                    normal: normal_map,
                })
            }
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

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // BSDF PDF計算（ノーマルマップタンジェント空間で実行）
        self.bsdf.pdf(&wo_normalmap, &wi_normalmap)
    }
}
