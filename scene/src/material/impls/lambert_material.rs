//! 拡散反射（Lambert）マテリアル実装。

use std::sync::Arc;

use math::{Normal, ShadingNormalTangent, Vector3};
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
}

impl LambertMaterial {
    /// 新しいLambertMaterialを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率パラメータ
    /// - `normal` - ノーマルマップパラメータ
    pub fn new(albedo: SpectrumParameter, normal: NormalParameter) -> Material {
        Arc::new(Self { albedo, normal })
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
        _uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> MaterialSample {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let bsdf = NormalizedLambertBsdf::new(albedo.clone());
        let transform = math::Transform::from_normal_map(&normal_map);
        let wo_local = &transform * wo;
        let Some(bsdf_result) = bsdf.sample(&wo_local, uv) else {
            return MaterialSample::failed(normal_map);
        };

        MaterialSample::new(
            bsdf_result.f,
            transform.inverse() * bsdf_result.wi,
            bsdf_result.pdf,
            bsdf_result.sample_type,
            normal_map,
        )
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> MaterialEvaluationResult {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let bsdf = NormalizedLambertBsdf::new(albedo.clone());
        let transform = math::Transform::from_normal_map(&normal_map);
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;
        let f = bsdf.evaluate(&wo_local, &wi_local);

        MaterialEvaluationResult {
            f,
            pdf: 1.0,
            normal: normal_map,
        }
    }

    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> f32 {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let bsdf = NormalizedLambertBsdf::new(albedo.clone());
        let transform = math::Transform::from_normal_map(&normal_map);
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;

        bsdf.pdf(&wo_local, &wi_local)
    }

    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        self.albedo.sample(uv).sample(lambda)
    }
}
