//! Babylon.jsに似たクリアコートの計算を行うシンプルなPBRマテリアル実装。
//! クリアコートで反射しなかった光は、減衰して下層で反射してまた減衰して出ていく。
//! 下層からまた上層で反射して戻るような層の間での反射は考慮しない。

use std::sync::Arc;

use math::{Normal, ShadingNormalTangent, Transform, Vector3, VertexNormalTangent};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    material::bsdf::{GeneralizedSchlickBsdf, NormalizedLambertBsdf, ScatterMode},
    BsdfSurfaceMaterial, FloatParameter, Material, MaterialEvaluationResult, MaterialSample,
    NormalParameter, SpectrumParameter, SurfaceInteraction, SurfaceMaterial,
};

/// シンプルなクリアコート付きPBRマテリアル。
pub struct SimpleClearcoatPbrMaterial {
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
    /// クリアコートの屈折率
    clearcoat_ior: FloatParameter,
    /// クリアコートの粗さ
    clearcoat_roughness: FloatParameter,
    /// クリアコートの色付け
    clearcoat_tint_color: SpectrumParameter,
    /// クリアコートの厚さ（単位: m）
    clearcoat_thickness: FloatParameter,
}

impl SimpleClearcoatPbrMaterial {
    /// 新しいSimpleClearcoatPbrMaterialを作成する。
    ///
    /// # Arguments
    /// - `base_color` - ベースカラー
    /// - `metallic` - 金属性（0.0=非金属、1.0=金属）
    /// - `roughness` - 表面の粗さパラメータ
    /// - `normal` - ノーマルマップパラメータ
    /// - `ior` - 屈折率（非金属部分の計算に使用）
    /// - `clearcoat_ior` - クリアコートの屈折率
    /// - `clearcoat_roughness` - クリアコートの粗さ
    /// - `clearcoat_tint_color` - クリアコートの色付け
    /// - `clearcoat_thickness` - クリアコートの厚さ（単位: m）
    pub fn new(
        base_color: SpectrumParameter,
        metallic: FloatParameter,
        roughness: FloatParameter,
        normal: NormalParameter,
        ior: FloatParameter,
        clearcoat_ior: FloatParameter,
        clearcoat_roughness: FloatParameter,
        clearcoat_tint_color: SpectrumParameter,
        clearcoat_thickness: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            base_color,
            metallic,
            roughness,
            normal,
            ior,
            clearcoat_ior,
            clearcoat_roughness,
            clearcoat_tint_color,
            clearcoat_thickness,
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

    /// Beer-Lambert attenuationを計算する。
    fn compute_attenuation(
        tint_color: &SampledSpectrum,
        thickness: f32,
        cos_theta: f32,
    ) -> SampledSpectrum {
        // Beer-Lambertの式: attenuation = exp(-sigma * L)
        // sigma = -log(tint_color) / 0.001
        // L = thickness / max(cos_theta, 1e-4)
        // tint_colorは1mmあたりの色付けを表しているものとする。

        // sigma = -log(tint_color) / 0.001を計算
        let log_tint = tint_color.log();
        let sigma = -1.0 * log_tint / 0.001;

        let l = thickness / cos_theta.max(1e-4);
        (-1.0 * sigma * l).exp()
    }
}

impl SurfaceMaterial for SimpleClearcoatPbrMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}

impl BsdfSurfaceMaterial for SimpleClearcoatPbrMaterial {
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
        let clearcoat_ior_value = self.clearcoat_ior.sample(shading_point.uv);
        let clearcoat_roughness_value = self.clearcoat_roughness.sample(shading_point.uv);
        let clearcoat_tint_color_spectrum = self
            .clearcoat_tint_color
            .sample(shading_point.uv)
            .sample(lambda);
        let clearcoat_thickness_value = self.clearcoat_thickness.sample(shading_point.uv);

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

        // clearcoat thicknessが0の場合はclearcoatをスキップ
        if clearcoat_thickness_value <= 0.0 {
            return self.sample_base_material(
                &base_color_spectrum,
                metallic_value,
                roughness_value,
                ior_value,
                &wo_normalmap,
                uc,
                uv,
                lambda,
                &transform_inv,
                normal_map,
            );
        }

        // clearcoatの処理
        let clearcoat_alpha = Self::roughness_to_alpha(clearcoat_roughness_value);
        let clearcoat_r0_value = Self::compute_dielectric_r0(clearcoat_ior_value);
        let clearcoat_r0 = SampledSpectrum::constant(clearcoat_r0_value);
        let clearcoat_r90 = SampledSpectrum::constant(1.0);
        let clearcoat_tint = SampledSpectrum::constant(1.0);

        let clearcoat_bsdf = GeneralizedSchlickBsdf::new(
            clearcoat_r0,
            clearcoat_r90,
            5.0,
            clearcoat_tint,
            ScatterMode::R,
            SampledSpectrum::constant(clearcoat_ior_value),
            true,
            false,
            clearcoat_alpha,
            clearcoat_alpha,
        );

        let clearcoat_fresnel = clearcoat_bsdf.fresnel(&wo_normalmap).average();

        if uc < clearcoat_fresnel {
            // clearcoat層をサンプリング
            let uc_adjusted = uc / clearcoat_fresnel;
            match clearcoat_bsdf.sample(&wo_normalmap, uv, uc_adjusted, lambda) {
                Some(bsdf_result) => {
                    let wi_shading = &transform_inv * &bsdf_result.wi;
                    MaterialSample::new(
                        bsdf_result.f,
                        wi_shading,
                        bsdf_result.pdf * clearcoat_fresnel,
                        bsdf_result.sample_type,
                        normal_map,
                    )
                }
                None => MaterialSample::failed(normal_map),
            }
        } else {
            // 下層をサンプリング
            let uc_adjusted = (uc - clearcoat_fresnel) / (1.0 - clearcoat_fresnel);
            let substrate_sample = self.sample_base_material(
                &base_color_spectrum,
                metallic_value,
                roughness_value,
                ior_value,
                &wo_normalmap,
                uc_adjusted,
                uv,
                lambda,
                &transform_inv,
                normal_map,
            );

            if !substrate_sample.is_sampled {
                return substrate_sample;
            }

            // attenuationを計算
            let cos_wo = wo_normalmap.z();
            let cos_wi = substrate_sample.wi.z();
            let attenuation_i = Self::compute_attenuation(
                &clearcoat_tint_color_spectrum,
                clearcoat_thickness_value,
                cos_wo,
            );
            let attenuation_o = Self::compute_attenuation(
                &clearcoat_tint_color_spectrum,
                clearcoat_thickness_value,
                cos_wi,
            );
            let attenuation = attenuation_i * attenuation_o;

            MaterialSample::new(
                substrate_sample.f * attenuation,
                substrate_sample.wi,
                substrate_sample.pdf * (1.0 - clearcoat_fresnel),
                substrate_sample.sample_type,
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
        let clearcoat_ior_value = self.clearcoat_ior.sample(shading_point.uv);
        let clearcoat_roughness_value = self.clearcoat_roughness.sample(shading_point.uv);
        let clearcoat_tint_color_spectrum = self
            .clearcoat_tint_color
            .sample(shading_point.uv)
            .sample(lambda);
        let clearcoat_thickness_value = self.clearcoat_thickness.sample(shading_point.uv);

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

        // clearcoat thicknessが0の場合はclearcoatをスキップ
        if clearcoat_thickness_value <= 0.0 {
            let f = self.evaluate_base_material(
                &base_color_spectrum,
                metallic_value,
                roughness_value,
                ior_value,
                &wo_normalmap,
                &wi_normalmap,
            );
            return MaterialEvaluationResult {
                f,
                pdf: 1.0,
                normal: normal_map,
            };
        }

        // clearcoatの処理
        let clearcoat_alpha = Self::roughness_to_alpha(clearcoat_roughness_value);
        let clearcoat_r0_value = Self::compute_dielectric_r0(clearcoat_ior_value);
        let clearcoat_r0 = SampledSpectrum::constant(clearcoat_r0_value);
        let clearcoat_r90 = SampledSpectrum::constant(1.0);
        let clearcoat_tint = SampledSpectrum::constant(1.0);

        let clearcoat_bsdf = GeneralizedSchlickBsdf::new(
            clearcoat_r0,
            clearcoat_r90,
            5.0,
            clearcoat_tint,
            ScatterMode::R,
            SampledSpectrum::constant(clearcoat_ior_value),
            true,
            false,
            clearcoat_alpha,
            clearcoat_alpha,
        );

        let clearcoat_fresnel = clearcoat_bsdf.fresnel(&wo_normalmap).average();
        let clearcoat_f = clearcoat_bsdf.evaluate(&wo_normalmap, &wi_normalmap);

        // 下層のf値を計算
        let substrate_f = self.evaluate_base_material(
            &base_color_spectrum,
            metallic_value,
            roughness_value,
            ior_value,
            &wo_normalmap,
            &wi_normalmap,
        );

        // attenuationを計算
        let cos_wo = wo_normalmap.z();
        let cos_wi = wi_normalmap.z();
        let attenuation_i = Self::compute_attenuation(
            &clearcoat_tint_color_spectrum,
            clearcoat_thickness_value,
            cos_wo,
        );
        let attenuation_o = Self::compute_attenuation(
            &clearcoat_tint_color_spectrum,
            clearcoat_thickness_value,
            cos_wi,
        );
        let attenuation = attenuation_i * attenuation_o;

        // 全体のf値を計算
        let f =
            clearcoat_f * clearcoat_fresnel + substrate_f * attenuation * (1.0 - clearcoat_fresnel);

        MaterialEvaluationResult {
            f,
            pdf: 1.0,
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
        let base_color_spectrum = self.base_color.sample(shading_point.uv).sample(_lambda);
        let metallic_value = self.metallic.sample(shading_point.uv);
        let roughness_value = self.roughness.sample(shading_point.uv);
        let ior_value = self.ior.sample(shading_point.uv);
        let clearcoat_ior_value = self.clearcoat_ior.sample(shading_point.uv);
        let clearcoat_roughness_value = self.clearcoat_roughness.sample(shading_point.uv);
        let clearcoat_thickness_value = self.clearcoat_thickness.sample(shading_point.uv);

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

        // clearcoat thicknessが0の場合はclearcoatをスキップ
        if clearcoat_thickness_value <= 0.0 {
            return self.pdf_base_material(
                &base_color_spectrum,
                metallic_value,
                roughness_value,
                ior_value,
                &wo_normalmap,
                &wi_normalmap,
            );
        }

        // clearcoatの処理
        let clearcoat_alpha = Self::roughness_to_alpha(clearcoat_roughness_value);
        let clearcoat_r0_value = Self::compute_dielectric_r0(clearcoat_ior_value);
        let clearcoat_r0 = SampledSpectrum::constant(clearcoat_r0_value);
        let clearcoat_r90 = SampledSpectrum::constant(1.0);
        let clearcoat_tint = SampledSpectrum::constant(1.0);

        let clearcoat_bsdf = GeneralizedSchlickBsdf::new(
            clearcoat_r0,
            clearcoat_r90,
            5.0,
            clearcoat_tint,
            ScatterMode::R,
            SampledSpectrum::constant(clearcoat_ior_value),
            true,
            false,
            clearcoat_alpha,
            clearcoat_alpha,
        );

        let clearcoat_fresnel = clearcoat_bsdf.fresnel(&wo_normalmap).average();
        let clearcoat_pdf = clearcoat_bsdf.pdf(&wo_normalmap, &wi_normalmap);

        // 下層のPDFを計算
        let substrate_pdf = self.pdf_base_material(
            &base_color_spectrum,
            metallic_value,
            roughness_value,
            ior_value,
            &wo_normalmap,
            &wi_normalmap,
        );

        // 全体のPDFを計算
        clearcoat_pdf * clearcoat_fresnel + substrate_pdf * (1.0 - clearcoat_fresnel)
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

impl SimpleClearcoatPbrMaterial {
    /// ベースマテリアルでサンプリングを行う
    fn sample_base_material(
        &self,
        base_color_spectrum: &SampledSpectrum,
        metallic_value: f32,
        roughness_value: f32,
        ior_value: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        transform_inv: &Transform<ShadingNormalTangent, VertexNormalTangent>,
        normal_map: Normal<VertexNormalTangent>,
    ) -> MaterialSample {
        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        if metallic_value >= 1.0 {
            // 完全金属
            self.sample_metallic(
                base_color_spectrum,
                alpha,
                wo_normalmap,
                uv,
                lambda,
                transform_inv,
                normal_map,
            )
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.sample_dielectric(
                base_color_spectrum,
                ior_value,
                alpha,
                wo_normalmap,
                uc,
                uv,
                lambda,
                transform_inv,
                normal_map,
            )
        } else {
            // 金属と非金属をミックス
            self.sample_mixed(
                base_color_spectrum,
                metallic_value,
                ior_value,
                alpha,
                wo_normalmap,
                uc,
                uv,
                lambda,
                transform_inv,
                normal_map,
            )
        }
    }

    /// ベースマテリアルでevaluateを行う
    fn evaluate_base_material(
        &self,
        base_color_spectrum: &SampledSpectrum,
        metallic_value: f32,
        roughness_value: f32,
        ior_value: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> SampledSpectrum {
        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        if metallic_value >= 1.0 {
            // 完全金属
            self.evaluate_metallic(base_color_spectrum, alpha, wo_normalmap, wi_normalmap)
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.evaluate_dielectric(
                base_color_spectrum,
                ior_value,
                alpha,
                wo_normalmap,
                wi_normalmap,
            )
        } else {
            // 金属と非金属をミックス
            let metallic_f =
                self.evaluate_metallic(base_color_spectrum, alpha, wo_normalmap, wi_normalmap);
            let dielectric_f = self.evaluate_dielectric(
                base_color_spectrum,
                ior_value,
                alpha,
                wo_normalmap,
                wi_normalmap,
            );
            metallic_f * metallic_value + dielectric_f * (1.0 - metallic_value)
        }
    }

    /// ベースマテリアルでpdfを計算する
    fn pdf_base_material(
        &self,
        base_color_spectrum: &SampledSpectrum,
        metallic_value: f32,
        roughness_value: f32,
        ior_value: f32,
        wo_normalmap: &Vector3<math::ShadingNormalTangent>,
        wi_normalmap: &Vector3<math::ShadingNormalTangent>,
    ) -> f32 {
        // roughnessからalpha値を計算
        let alpha = Self::roughness_to_alpha(roughness_value);

        if metallic_value >= 1.0 {
            // 完全金属
            self.pdf_metallic(alpha, wo_normalmap, wi_normalmap)
        } else if metallic_value <= 0.0 {
            // 完全非金属
            self.pdf_dielectric(
                base_color_spectrum,
                ior_value,
                alpha,
                wo_normalmap,
                wi_normalmap,
            )
        } else {
            // 金属と非金属をミックス
            let metallic_pdf = self.pdf_metallic(alpha, wo_normalmap, wi_normalmap);
            let dielectric_pdf = self.pdf_dielectric(
                base_color_spectrum,
                ior_value,
                alpha,
                wo_normalmap,
                wi_normalmap,
            );
            metallic_pdf * metallic_value + dielectric_pdf * (1.0 - metallic_value)
        }
    }

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

        let fresnel = generalized_schlick.fresnel(wo_normalmap).average();

        if uc < fresnel {
            // Specular反射をサンプリング
            let uc = uc / fresnel;
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
            let lambert_bsdf = NormalizedLambertBsdf::new(base_color.clone());
            match lambert_bsdf.sample(wo_normalmap, uv) {
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
        if uc <= metallic {
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

        let lambert_bsdf = NormalizedLambertBsdf::new(base_color.clone());
        let lambert_reflection = lambert_bsdf.evaluate(wo_normalmap, wi_normalmap);

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
        base_color: &SampledSpectrum,
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

        let lambert_bsdf = NormalizedLambertBsdf::new(base_color.clone());
        let lambert_pdf = lambert_bsdf.pdf(wo_normalmap, wi_normalmap);

        fresnel * direct_pdf + (1.0 - fresnel) * lambert_pdf
    }
}
