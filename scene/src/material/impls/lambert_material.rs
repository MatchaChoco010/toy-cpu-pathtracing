//! 拡散反射（Lambert）マテリアル実装。

use math::{Normal, Tangent, Vector3};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSample, BsdfSurfaceMaterial, Material, MaterialEvaluationResult, NormalParameter,
    NormalizedLambertBsdf, SceneId, SpectrumParameter, SurfaceInteraction, SurfaceMaterial,
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
        wo: &Vector3<Tangent>,
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample> {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // BSDFサンプリング（法線パラメータ付き）
        self.bsdf.sample(&albedo, wo, uv, &normal)
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> MaterialEvaluationResult {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // BSDF評価（純粋なBSDF値のみ）
        let f = self.bsdf.evaluate(&albedo, wo, wi, &normal);

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // 単一BSDFなので選択確率は1.0
            normal,
        }
    }

    fn pdf(
        &self,
        _lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> f32 {
        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // BSDF PDF計算（法線パラメータ付き）
        self.bsdf.pdf(wo, wi, &normal)
    }
}
