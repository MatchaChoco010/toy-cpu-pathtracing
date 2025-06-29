//! 拡散反射（Lambert）マテリアル実装。

use std::sync::Arc;

use math::{Normal, Vector3, VertexNormalTangent};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample, NormalParameter,
    NormalizedLambertBsdf, SpectrumParameter, SurfaceInteraction, SurfaceMaterial,
    evaluate_microfacet_normal_mapping, pdf_microfacet_normal_mapping,
    sample_microfacet_normal_mapping,
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
        uc: f32,
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

        // 幾何学的法線（VertexNormalTangent空間では常にz軸方向）
        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        // Microfacet-based normal mappingを使用してサンプリング
        let base_bsdf_sample = |wo_local: &Vector3<math::ShadingNormalTangent>,
                                uv_sample: glam::Vec2| {
            let bsdf = NormalizedLambertBsdf::new(albedo.clone());
            bsdf.sample(wo_local, uv_sample)
        };

        match sample_microfacet_normal_mapping(
            wo,
            uv,
            uc,
            &normal_map,
            &geometry_normal,
            base_bsdf_sample,
            albedo.clone(),
        ) {
            Some(sample) => sample,
            None => MaterialSample::failed(normal_map),
        }
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

        // 幾何学的法線（VertexNormalTangent空間では常にz軸方向）
        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        let base_bsdf_eval =
            |wo_local: &Vector3<math::ShadingNormalTangent>,
             wi_local: &Vector3<math::ShadingNormalTangent>| {
                let bsdf = NormalizedLambertBsdf::new(albedo.clone());
                bsdf.evaluate(wo_local, wi_local)
            };

        let f = evaluate_microfacet_normal_mapping(
            wo,
            wi,
            &normal_map,
            &geometry_normal,
            base_bsdf_eval,
        );

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // Deterministic evaluation, not stochastic
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
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // 幾何学的法線（VertexNormalTangent空間では常にz軸方向）
        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        // Microfacet-based normal mappingを使用してPDF計算
        let base_bsdf_pdf =
            |wo_local: &Vector3<math::ShadingNormalTangent>,
             wi_local: &Vector3<math::ShadingNormalTangent>| {
                let bsdf = NormalizedLambertBsdf::new(albedo.clone());
                bsdf.pdf(wo_local, wi_local)
            };

        pdf_microfacet_normal_mapping(wo, wi, &normal_map, &geometry_normal, base_bsdf_pdf)
    }

    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        self.albedo.sample(uv).sample(lambda)
    }
}
