//! シンプルなPBRマテリアル実装。

use std::sync::Arc;

use math::{Normal, ShadingNormalTangent, Transform, Vector3, VertexNormalTangent};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    BsdfSurfaceMaterial, FloatParameter, Material, MaterialEvaluationResult, MaterialSample,
    NormalParameter, SpectrumParameter, SurfaceInteraction, SurfaceMaterial,
    material::bsdf::{GeneralizedSchlickBsdf, NormalizedLambertBsdf, ScatterMode},
};

/// シンプルなPBRマテリアル。
/// metellicパラメータで金属と非金属の結果をミックスして返す。
pub struct SimplePbrMaterial {
    /// ベースカラー
    base_color: SpectrumParameter,
    /// 金属性（0.0=非金属、1.0=金属）
    metallic: FloatParameter,
    /// 表面の粗さパラメータ
    roughness: FloatParameter,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// 屈折率（非金属部分の計算に使用）
    ior: FloatParameter,
}

impl SimplePbrMaterial {
    /// 新しいSimplePbrMaterialを作成する。
    ///
    /// # Arguments
    /// - `base_color` - ベースカラー
    /// - `metallic` - 金属性（0.0=非金属、1.0=金属）
    /// - `roughness` - 表面の粗さパラメータ
    /// - `normal` - ノーマルマップパラメータ
    /// - `ior` - 屈折率（非金属部分の計算に使用）
    pub fn new(
        base_color: SpectrumParameter,
        metallic: FloatParameter,
        roughness: FloatParameter,
        normal: NormalParameter,
        ior: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            base_color,
            metallic,
            roughness,
            normal,
            ior,
        })
    }

    /// roughnessからalpha値を計算する。
    fn roughness_to_alpha(roughness: f32) -> f32 {
        roughness * roughness
    }

    /// 非金属のr0反射率を計算する。
    /// r0 = ((n - 1) / (n + 1))^2
    fn compute_dielectric_r0(ior: f32) -> f32 {
        let r = (ior - 1.0) / (ior + 1.0);
        r * r
    }
}

impl SurfaceMaterial for SimplePbrMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}

impl BsdfSurfaceMaterial for SimplePbrMaterial {
    fn sample(
        &self,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialSample {
        // パラメータをサンプリング
        let base_color_spectrum = self.base_color.sample(shading_point.uv).sample(lambda);
        let metallic_value = self.metallic.sample(shading_point.uv);
        let roughness_value = self.roughness.sample(shading_point.uv);
        let ior_value = self.ior.sample(shading_point.uv);

        // 法線マップから法線を取得
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);
        let transform_inv = transform.inverse();

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;

        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        if metallic_value >= 1.0 {
            // 完全金属
            self.sample_metallic(
                &base_color_spectrum,
                alpha,
                &wo_normalmap,
                uv,
                lambda,
                &transform_inv,
                normal_map,
            )
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.sample_dielectric(
                &base_color_spectrum,
                ior_value,
                alpha,
                &wo_normalmap,
                uc,
                uv,
                lambda,
                &transform_inv,
                normal_map,
            )
        } else {
            // 金属と非金属をミックス
            self.sample_mixed(
                &base_color_spectrum,
                metallic_value,
                ior_value,
                alpha,
                &wo_normalmap,
                uc,
                uv,
                lambda,
                &transform_inv,
                normal_map,
            )
        }
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialEvaluationResult {
        // パラメータをサンプリング
        let base_color_spectrum = self.base_color.sample(shading_point.uv).sample(lambda);
        let metallic_value = self.metallic.sample(shading_point.uv);
        let roughness_value = self.roughness.sample(shading_point.uv);
        let ior_value = self.ior.sample(shading_point.uv);

        // 法線マップから法線を取得
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        let f = if metallic_value >= 1.0 {
            // 完全金属
            self.evaluate_metallic(&base_color_spectrum, alpha, &wo_normalmap, &wi_normalmap)
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.evaluate_dielectric(
                &base_color_spectrum,
                ior_value,
                alpha,
                &wo_normalmap,
                &wi_normalmap,
            )
        } else {
            // 金属と非金属をミックス
            let metallic_f =
                self.evaluate_metallic(&base_color_spectrum, alpha, &wo_normalmap, &wi_normalmap);
            let dielectric_f = self.evaluate_dielectric(
                &base_color_spectrum,
                ior_value,
                alpha,
                &wo_normalmap,
                &wi_normalmap,
            );
            metallic_f * metallic_value + dielectric_f * (1.0 - metallic_value)
        };

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // evaluateは決定論的なレイヤー評価
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
        // パラメータをサンプリング
        let metallic_value = self.metallic.sample(shading_point.uv);
        let roughness_value = self.roughness.sample(shading_point.uv);
        let ior_value = self.ior.sample(shading_point.uv);

        // 法線マップから法線を取得
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        if metallic_value >= 1.0 {
            // 完全金属
            self.pdf_metallic(alpha, &wo_normalmap, &wi_normalmap)
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.pdf_dielectric(ior_value, alpha, &wo_normalmap, &wi_normalmap)
        } else {
            // 金属と非金属をミックス
            let metallic_pdf = self.pdf_metallic(alpha, &wo_normalmap, &wi_normalmap);
            let dielectric_pdf =
                self.pdf_dielectric(ior_value, alpha, &wo_normalmap, &wi_normalmap);
            metallic_pdf * metallic_value + dielectric_pdf * (1.0 - metallic_value)
        }
    }

    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        // ベースカラーをアルベドとして返す
        self.base_color.sample(uv).sample(lambda)
    }
}

impl SimplePbrMaterial {
    /// 金属マテリアルのサンプリング
    fn sample_metallic(
        &self,
        base_color: &SampledSpectrum,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        transform_inv: &Transform<ShadingNormalTangent, VertexNormalTangent>,
        normal_map: Normal<VertexNormalTangent>,
    ) -> MaterialSample {
        // 金属用GeneralizedSchlickBsdf
        let r0 = base_color.clone();
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(1.0), // eta（反射のみなので使用されない）
            true,                           // entering
            false,                          // thin_surface
            alpha,
            alpha,
        );

        match generalized_schlick.sample(wo_normalmap, uv, 0.0, lambda) {
            Some(bsdf_result) => {
                let wi_shading = transform_inv * &bsdf_result.wi;
                MaterialSample::new(
                    bsdf_result.f,
                    wi_shading,
                    bsdf_result.pdf,
                    bsdf_result.sample_type,
                    normal_map,
                )
            }
            None => MaterialSample::failed(normal_map),
        }
    }

    /// 非金属マテリアルのサンプリング（GeneralizedSchlick + Lambertレイヤー）
    fn sample_dielectric(
        &self,
        base_color: &SampledSpectrum,
        ior: f32,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        transform_inv: &Transform<ShadingNormalTangent, VertexNormalTangent>,
        normal_map: Normal<VertexNormalTangent>,
    ) -> MaterialSample {
        // 非金属用GeneralizedSchlickBsdf（反射・透過）
        let r0_value = Self::compute_dielectric_r0(ior);
        let r0 = SampledSpectrum::constant(r0_value);
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(ior),
            true,
            false,
            alpha,
            alpha,
        );

        let fresnel = generalized_schlick.fresnel(&wo_normalmap).average();

        if uc < fresnel {
            // Specular反射をサンプリング
            let uc = if fresnel == 1.0 {
                uc
            } else {
                (uc - fresnel) / (1.0 - fresnel)
            };
            match generalized_schlick.sample(wo_normalmap, uv, uc, lambda) {
                Some(bsdf_result) => {
                    let wi_shading = transform_inv * &bsdf_result.wi;
                    MaterialSample::new(
                        bsdf_result.f,
                        wi_shading,
                        bsdf_result.pdf * fresnel,
                        bsdf_result.sample_type,
                        normal_map,
                    )
                }
                None => MaterialSample::failed(normal_map),
            }
        } else {
            // Diffuse反射をサンプリング
            let lambert_bsdf = NormalizedLambertBsdf::new();
            match lambert_bsdf.sample(base_color, wo_normalmap, uv) {
                Some(bsdf_result) => {
                    let wi_shading = transform_inv * &bsdf_result.wi;
                    MaterialSample::new(
                        bsdf_result.f * (1.0 - fresnel),
                        wi_shading,
                        bsdf_result.pdf * (1.0 - fresnel),
                        bsdf_result.sample_type,
                        normal_map,
                    )
                }
                None => MaterialSample::failed(normal_map),
            }
        }
    }

    /// 金属と非金属の混合サンプリング
    fn sample_mixed(
        &self,
        base_color: &SampledSpectrum,
        metallic: f32,
        ior: f32,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        transform_inv: &Transform<ShadingNormalTangent, VertexNormalTangent>,
        normal_map: Normal<VertexNormalTangent>,
    ) -> MaterialSample {
        if uc < metallic {
            // 金属として扱う
            self.sample_metallic(
                base_color,
                alpha,
                wo_normalmap,
                uv,
                lambda,
                transform_inv,
                normal_map,
            )
        } else {
            // 非金属として扱う
            self.sample_dielectric(
                base_color,
                ior,
                alpha,
                wo_normalmap,
                (uc - metallic) / (1.0 - metallic),
                uv,
                lambda,
                transform_inv,
                normal_map,
            )
        }
    }

    /// 金属マテリアルの評価
    fn evaluate_metallic(
        &self,
        base_color: &SampledSpectrum,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let r0 = base_color.clone();
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(1.0),
            true,
            false,
            alpha,
            alpha,
        );

        generalized_schlick.evaluate(wo_normalmap, wi_normalmap)
    }

    /// 非金属マテリアルの評価（GeneralizedSchlick + Lambertレイヤー）
    fn evaluate_dielectric(
        &self,
        base_color: &SampledSpectrum,
        ior: f32,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let r0_value = Self::compute_dielectric_r0(ior);
        let r0 = SampledSpectrum::constant(r0_value);
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(ior),
            true,
            false,
            alpha,
            alpha,
        );

        let direct_reflection = generalized_schlick.evaluate(wo_normalmap, wi_normalmap);

        let fresnel = generalized_schlick.fresnel(wo_normalmap).average();

        let lambert_bsdf = NormalizedLambertBsdf::new();
        let lambert_reflection = lambert_bsdf.evaluate(base_color, wo_normalmap, wi_normalmap);

        direct_reflection + (1.0 - fresnel) * lambert_reflection
    }

    /// 金属マテリアルのPDF
    fn pdf_metallic(
        &self,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> f32 {
        let r0 = SampledSpectrum::constant(1.0);
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(1.0),
            true,
            false,
            alpha,
            alpha,
        );

        generalized_schlick.pdf(wo_normalmap, wi_normalmap)
    }

    /// 非金属マテリアルのPDF（GeneralizedSchlick + Lambertレイヤー）
    fn pdf_dielectric(
        &self,
        ior: f32,
        alpha: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> f32 {
        let r0_value = Self::compute_dielectric_r0(ior);
        let r0 = SampledSpectrum::constant(r0_value);
        let r90 = SampledSpectrum::constant(1.0);
        let tint = SampledSpectrum::constant(1.0);

        let generalized_schlick = GeneralizedSchlickBsdf::new(
            r0,
            r90,
            5.0,
            tint,
            ScatterMode::R,
            SampledSpectrum::constant(ior),
            true,
            false,
            alpha,
            alpha,
        );

        let direct_pdf = generalized_schlick.pdf(wo_normalmap, wi_normalmap);

        let fresnel = generalized_schlick.fresnel(wo_normalmap).average();

        let lambert_bsdf = NormalizedLambertBsdf::new();
        let lambert_pdf = lambert_bsdf.pdf(wo_normalmap, wi_normalmap);

        fresnel * direct_pdf + (1.0 - fresnel) * lambert_pdf
    }
}
