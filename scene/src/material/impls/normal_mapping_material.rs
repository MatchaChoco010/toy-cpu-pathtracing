//! 拡散反射（Lambert）マテリアル実装。

use std::sync::Arc;

use rand::prelude::*;

use math::{Normal, ShadingNormalTangent, Transform, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::material::common::reflect;
use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample, NormalParameter,
    SurfaceInteraction, SurfaceMaterial,
};

/// 下層マテリアルに法線マップを適用するマテリアル。
pub struct NormalMappingMaterial {
    /// ベースマテリアル
    base_material: Material,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// ノーマルマップ強度
    normal_strength: f32,
    /// MicrofacetNormalMappingを行うかどうか
    enable_microfacet: bool,
}

impl NormalMappingMaterial {
    /// 新しいNormalMappingMaterialを作成する。
    ///
    /// # Arguments
    /// - `base_material` - ベースマテリアル
    /// - `normal` - ノーマルマップパラメータ
    /// - `normal_strength` - ノーマルマップの強度
    /// - `enable_microfacet` - MicrofacetNormalMappingを有効にするかどうか
    pub fn new(
        base_material: Material,
        normal: NormalParameter,
        normal_strength: f32,
        enable_microfacet: bool,
    ) -> Material {
        Arc::new(Self {
            base_material,
            normal,
            normal_strength,
            enable_microfacet,
        })
    }
}
impl SurfaceMaterial for NormalMappingMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for NormalMappingMaterial {
    fn sample(
        &self,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> MaterialSample {
        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let normal_map = Normal::new(
            normal_map.x() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.y() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.z() * self.normal_strength + 1.0 * (1.0 - self.normal_strength),
        );

        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        // let transform = Transform::from_normal_map(&normal_map);
        // // let transform = Transform::from_normal_map(&normal_map).inverse().inverse();
        // let wg = &transform * geometry_normal;
        // let wp = &transform * normal_map;
        // let wo = &transform * wo;
        // let shading_point = &transform * shading_point;

        let Some(base_material) = self.base_material.as_bsdf_material() else {
            panic!("Base material must implement BsdfSurfaceMaterial");
        };

        if self.enable_microfacet {
            let mut sample = Self::sample_microfacet_normal_mapping(
                uc,
                uv,
                lambda,
                &wo,
                &shading_point,
                &normal_map,
                base_material,
            );

            if !sample.is_sampled {
                return MaterialSample::failed(geometry_normal);
            }

            sample.f *= 1.0 / sample.wi.z().max(1e-8);

            // sample.wi = transform.inverse() * sample.wi;
            // sample.normal = geometry_normal;
            // sample.normal = normal_map;
            // sample.f *= 1.0 / sample.wi.dot(sample.normal).abs();
            // sample.f *= sample.wi.dot(normal_map).max(0.0) / sample.wi.dot(sample.normal).abs();

            // sample.f *= if sample.wi.dot(geometry_normal) < 0.0 {
            //     normal_map.dot(-sample.wi).max(0.0) / geometry_normal.dot(-sample.wi).max(1e-8)
            // } else {
            //     normal_map.dot(sample.wi).max(0.0) / geometry_normal.dot(sample.wi).max(1e-8)
            // };

            sample
        } else {
            let transform = Transform::from_normal_map(&normal_map);
            let wo_local = &transform * wo;

            let sample = base_material.sample(uc, uv, lambda, &wo_local, &shading_point);

            if !sample.is_sampled {
                return MaterialSample::failed(geometry_normal);
            }

            let wi = transform.inverse() * sample.wi;

            if wi.z().signum() != wo.z().signum() {
                return MaterialSample::failed(geometry_normal);
            }

            let wi = transform.inverse() * sample.wi;
            MaterialSample::new(sample.f, wi, sample.pdf, sample.sample_type, normal_map)

            // sample.normal = geometry_normal;
            // sample.normal = normal_map;
            // sample.f *= sample.wi.dot(normal_map).abs() / sample.wi.dot(sample.normal).abs();

            // sample.f *= if sample.wi.dot(geometry_normal) < 0.0 {
            //     normal_map.dot(-sample.wi).max(0.0) / geometry_normal.dot(-sample.wi).max(1e-8)
            // } else {
            //     normal_map.dot(sample.wi).max(0.0) / geometry_normal.dot(sample.wi).max(1e-8)
            // };

            // sample
        }

        // if !material_sample.is_sampled {
        //     return MaterialSample::failed(geometry_normal);
        // }

        // if wo.dot(wg).signum() != material_sample.wi.dot(wg).signum() {
        //     // 同じ半球内でない場合は無効
        //     return MaterialSample::failed(normal_map);
        // }

        // let wi = transform.inverse() * material_sample.wi;
        // let normal = normal_map;

        // MaterialSample::new(
        //     material_sample.f,
        //     wi,
        //     material_sample.pdf,
        //     material_sample.sample_type,
        //     normal,
        // )

        // if !self.enable_microfacet {
        //     return base_material.sample(uc, uv, lambda, &wo, &shading_point);
        // }

        // Self::sample_microfacet_normal_mapping(
        //     uc,
        //     uv,
        //     lambda,
        //     &wo,
        //     &shading_point,
        //     normal_map,
        //     base_material,
        // )
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> MaterialEvaluationResult {
        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let normal_map = Normal::new(
            normal_map.x() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.y() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.z() * self.normal_strength + 1.0 * (1.0 - self.normal_strength),
        );

        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        let transform = Transform::from_normal_map(&normal_map);
        // let transform = Transform::from_normal_map(&normal_map).inverse();
        let wg = &transform * geometry_normal;
        let wp = &transform * normal_map;
        let mut wo = &transform * wo;
        let mut wi = &transform * wi;
        let shading_point = transform * shading_point;

        let Some(base_material) = self.base_material.as_bsdf_material() else {
            panic!("Base material must implement BsdfSurfaceMaterial");
        };

        // if !self.enable_microfacet {
        // return base_material.evaluate(lambda, &wo, &wi, &shading_point);
        let mut result = base_material.evaluate(lambda, &wo, &wi, &shading_point);
        result.normal = normal_map;
        return result;
        // }

        // if wg.dot(wo).signum() != wg.dot(wi).signum() {
        //     // 同じ半球内でない場合は無効
        //     return MaterialEvaluationResult {
        //         f: SampledSpectrum::zero(),
        //         pdf: 1.0,
        //         normal: geometry_normal,
        //     };
        // }

        // let mut weight = 2.0;
        // let mut rng = rand::rng();
        // if rng.random::<f32>() < 0.5 {
        //     let tmp = wo;
        //     wo = wi;
        //     wi = tmp;
        //     weight *= if wg.dot(wo) < 0.0 {
        //         wg.dot(&-wo).max(0.0) / wg.dot(&-wi).max(1e-8)
        //     } else {
        //         wg.dot(wo).max(0.0) / wg.dot(wi).max(1e-8)
        //     };
        // }

        // let eval_result = Self::evaluate_microfacet_normal_mapping(
        //     lambda,
        //     &wo,
        //     &wi,
        //     &shading_point,
        //     &wp,
        //     &wg,
        //     base_material,
        // );
        // MaterialEvaluationResult {
        //     f: eval_result.f * weight,
        //     pdf: eval_result.pdf * weight,
        //     normal: eval_result.normal,
        // }
    }

    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    ) -> f32 {
        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        let normal_map = Normal::new(
            normal_map.x() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.y() * self.normal_strength + 0.0 * (1.0 - self.normal_strength),
            normal_map.z() * self.normal_strength + 1.0 * (1.0 - self.normal_strength),
        );

        let geometry_normal = Normal::new(0.0, 0.0, 1.0);

        let transform = Transform::from_normal_map(&normal_map);
        let wg = &transform * geometry_normal;
        let wo = &transform * wo;
        let wi = &transform * wi;
        let shading_point = transform * shading_point;

        let Some(base_material) = self.base_material.as_bsdf_material() else {
            panic!("Base material must implement BsdfSurfaceMaterial");
        };

        // if !self.enable_microfacet {
        return base_material.pdf(lambda, &wo, &wi, &shading_point);
        // }

        // Self::pdf_microfacet_normal_mapping(
        //     lambda,
        //     &wo,
        //     &wi,
        //     &shading_point,
        //     &normal_map,
        //     &wg,
        //     base_material,
        // )
    }

    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        let Some(base_material) = self.base_material.as_bsdf_material() else {
            panic!("Base material must implement BsdfSurfaceMaterial");
        };
        base_material.sample_albedo_spectrum(uv, lambda)
    }
}
impl NormalMappingMaterial {
    // fn wt(
    //     wp: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    // ) -> Normal<ShadingNormalTangent> {
    //     let projection_magnitude = wg.dot(&wp);
    //     let wp_vec: &Vector3<_> = wp.as_ref();
    //     let wg_vec: &Vector3<_> = wg.as_ref();
    //     let orthogonal = wp_vec - projection_magnitude * wg_vec;
    //     if orthogonal.length_squared() < 1e-8 {
    //         dbg!(wp, wg);
    //         Normal::new(1.0, 0.0, 0.0)
    //     } else {
    //         (-orthogonal).normalize().to_normal()
    //     }
    // }
    fn wt(wp: &Normal<ShadingNormalTangent>) -> Normal<ShadingNormalTangent> {
        Vector3::new(-wp.x(), -wp.y(), 0.0).to_normal()
    }

    fn a_p(
        wi: &Vector3<ShadingNormalTangent>,
        wp: &Normal<ShadingNormalTangent>,
        wg: &Normal<ShadingNormalTangent>,
    ) -> f32 {
        (wi.dot(wp) / wp.dot(wg).abs()).max(0.0)
    }

    fn a_t(
        wi: &Vector3<ShadingNormalTangent>,
        wp: &Normal<ShadingNormalTangent>,
        wt: &Normal<ShadingNormalTangent>,
        wg: &Normal<ShadingNormalTangent>,
    ) -> f32 {
        let d = wp.dot(wg);
        (wi.dot(wt) * (1.0 - d.powi(2)).sqrt()).max(0.0) / d.abs()
    }

    // fn lambda_p(
    //     wi: &Vector3<ShadingNormalTangent>,
    //     wp: &Normal<ShadingNormalTangent>,
    //     wt: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    // ) -> f32 {
    //     Self::a_p(wi, wp, wg) / (Self::a_p(wi, wp, wg) + Self::a_t(wi, wp, wt, wg))
    // }

    // fn lambda_p(
    //     wi: &Vector3<ShadingNormalTangent>,
    //     wp: &Normal<ShadingNormalTangent>,
    //     wt: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    // ) -> f32 {
    //     let i_dot_p = wp.dot(wi).max(0.0);
    //     i_dot_p / (i_dot_p + wi.dot(wt).max(0.0) * (1.0 - wp.dot(wg).powi(2)).sqrt())
    // }

    fn lambda_p(wp: &Normal<ShadingNormalTangent>, wi: &Vector3<ShadingNormalTangent>) -> f32 {
        let i_dot_p = wp.dot(wi).max(0.0);
        i_dot_p / (i_dot_p + wi.dot(Self::wt(wp)).max(0.0) * (1.0 - wp.z().powi(2)).sqrt())
    }

    fn g1(
        wi: &Vector3<ShadingNormalTangent>,
        wm: &Normal<ShadingNormalTangent>,
        wp: &Normal<ShadingNormalTangent>,
        wt: &Normal<ShadingNormalTangent>,
        wg: &Normal<ShadingNormalTangent>,
    ) -> f32 {
        let dot_wi_wm = wi.dot(wm);
        if dot_wi_wm <= 0.0 {
            return 0.0; // Heaviside function
        }

        (wi.dot(wg) / (Self::a_p(wi, wp, wg) + Self::a_t(wi, wp, wt, wg))).clamp(0.0, 1.0)
    }

    // fn g1(
    //     wi: &Vector3<ShadingNormalTangent>,
    //     wm: &Normal<ShadingNormalTangent>,
    //     wp: &Normal<ShadingNormalTangent>,
    //     wt: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    // ) -> f32 {
    //     (wi.dot(wg).max(0.0) * wp.dot(wg).max(0.0)
    //         / (wi.dot(wp).max(0.0) + wi.dot(wt).max(0.0) * (1.0 - wp.dot(wg).powi(2)).sqrt()))
    //     .min(1.0)
    // }

    // fn g1(wp: &Normal<ShadingNormalTangent>, w: &Vector3<ShadingNormalTangent>) -> f32 {
    //     (w.z().max(0.0) * wp.z().max(0.0)
    //         / (w.dot(wp).max(0.0) + w.dot(Self::wt(wp)).max(0.0) * (1.0 - wp.z().powi(2)).sqrt()))
    //     .min(1.0)
    // }

    fn sample_microfacet_normal_mapping(
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<ShadingNormalTangent>,
        shading_point: &SurfaceInteraction<ShadingNormalTangent>,
        wp: &Normal<ShadingNormalTangent>,
        base_material: &dyn BsdfSurfaceMaterial,
    ) -> MaterialSample {
        if wp.z() <= 0.0 || (wp.x().abs() < 1e-6 && wp.y().abs() < 1e-6) {
            return base_material.sample(uc, uv, lambda, wo, shading_point);
        }

        // Use additional random numbers for Russian roulette - we need to reserve uc for different purpose
        let mut rng_state = (uc * 1000000.0) as u32;
        let mut next_random = || {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            (rng_state as f32) / (u32::MAX as f32)
        };

        let transform_wp = Transform::from_normal_map(wp);
        let shading_point_wp = &transform_wp * shading_point;

        let wt = Self::wt(wp);
        assert!(wt.dot(wp) < 0.0, "{:?}, {:?}", wt, wp);
        let wg = Normal::new(0.0, 0.0, 1.0);

        let mut i = 0;
        let mut sample_type;
        let mut wr = -*wo;
        let mut weight = SampledSpectrum::one();

        let max_scattering_iterations = 100;

        if next_random() > Self::lambda_p(wp, wo) {
            wr = -reflect(&wr, &wt);
            // let wr_vec: Vector3<_> = *wr.as_ref();
            // let wt_vec: Vector3<_> = *wt.as_ref();
            // let wr_vec = wr_vec - 2.0 * wr_vec.dot(wt_vec) * wt_vec;
            // wr = wr_vec.normalize();
            i += 1;
        }

        loop {
            if i > max_scattering_iterations {
                return MaterialSample::failed(wg);
            }

            let wo_local = &transform_wp * -wr;
            let uv = glam::vec2(next_random(), next_random());
            let material_sample =
                base_material.sample(uc, uv, lambda, &wo_local, &shading_point_wp);
            if !material_sample.is_sampled {
                return MaterialSample::failed(wg);
            }

            // let cos_theta = wp.dot(material_sample.wi).max(0.0);
            let cos_theta = material_sample.wi.z().max(0.0);
            wr = transform_wp.inverse() * material_sample.wi;
            // wr = material_sample.wi;
            // let cos_theta = wp.dot(wr).max(0.0);
            weight *= material_sample.f * cos_theta / material_sample.pdf;
            sample_type = material_sample.sample_type;
            if weight.is_zero() {
                return MaterialSample::failed(wg);
            }

            // let g = Self::g1(&wr, wp, wp, wt, wg);
            // let g = Self::g1(wp, &wr);
            // let g = Self::g1(wp, &(&transform_wp * &wr));
            // let g = Self::g1(&(&transform_wp * wp), &(&transform_wp * &wr));
            // let g = Self::g1(&wr, wp, wp, &wt, &wg);
            let g = Self::g1(
                // &(&transform_wp * &wr),
                // &(&transform_wp * wp),
                // &(&transform_wp * wp),
                // &(&transform_wp * &wt),
                // &(&transform_wp * &wg),
                &wr, //
                &wp, //
                &wp, //
                &wt, //
                &wg, //
            );
            i += 1;

            if next_random() < g {
                // if true {
                break;
            }
            if i > max_scattering_iterations {
                return MaterialSample::failed(wg);
            }

            wr = -reflect(&wr, &wt);
            // {
            //     let wr_vec: Vector3<_> = *wr.as_ref();
            //     let wt_vec: Vector3<_> = *wt.as_ref();
            //     let wr_vec = wr_vec - 2.0 * wr_vec.dot(wt_vec) * wt_vec;
            //     wr = wr_vec.normalize();
            // }

            // let g = Self::g1(wp, &wr);
            // let g = Self::g1(wp, &(&transform_wp * &wr));
            // let g = Self::g1(&(&transform_wp * wp), &(&transform_wp * &wr));
            // let g = Self::g1(&-wr, wp, wp, wt, wg);
            // let g = Self::g1(&wr, wp, wp, &wt, &wg);
            // let g = Self::g1(&wr, &wt, wp, &wt, &wg);
            let g = Self::g1(
                // &(&transform_wp * &wr),
                // &(&transform_wp * &wt),
                // &(&transform_wp * wp),
                // &(&transform_wp * &wt),
                // &(&transform_wp * &wg),
                &wr, //
                &wt, //
                &wp, //&wp,
                &wt, // &wg,
                &wg, //
            );
            i += 1;

            if next_random() < g {
                break;
            }
        }

        // let f = weight / wg.dot(wr).abs().max(1e-6);
        let f = weight;
        let wi = wr;
        let pdf = 1.0;

        if wo.z().signum() != wi.z().signum() {
            // Geometric validity check failed
            return MaterialSample::failed(wg);
        }

        MaterialSample::new(f, wi, pdf, sample_type, wg)
    }

    // fn evaluate_microfacet_normal_mapping(
    //     lambda: &SampledWavelengths,
    //     wo: &Vector3<ShadingNormalTangent>,
    //     wi: &Vector3<ShadingNormalTangent>,
    //     shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    //     wp: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    //     base_material: &dyn BsdfSurfaceMaterial,
    // ) -> MaterialEvaluationResult {
    //     let mut rng = rand::rng();
    //     // let uc = rng.random();

    //     if (wp.dot(wg) - 1.0).abs() < 1e-8 {
    //         return base_material.evaluate(lambda, wo, wi, shading_point);
    //     }

    //     let wt = &Self::wt(wp, wg);

    //     let mut i = 0;
    //     let mut f = SampledSpectrum::zero();
    //     let mut wr = -*wo;
    //     let mut weight = SampledSpectrum::one();

    //     let max_scattering_iterations = 100;

    //     let mut single_scattering = true;
    //     let mut pdf_wo = 1.0;

    //     if rng.random::<f32>() > Self::lambda_p(wo, wp, wt, wg) {
    //         wr = -reflect(&wr, &wt);
    //         i += 1;
    //     }

    //     loop {
    //         let eval_result = base_material.evaluate(lambda, &-wr, wi, shading_point);
    //         // let mut contribution = eval_result.f / eval_result.pdf;
    //         let mut contribution = eval_result.f * wp.dot(wi).max(0.0) / eval_result.pdf;
    //         if single_scattering {
    //             contribution *= 0.5;
    //         } else {
    //             let mut pdf_wi = eval_result.pdf;
    //             pdf_wi /= wg.dot(wi);
    //             contribution *= pdf_wo / (pdf_wo + pdf_wi);
    //         }
    //         contribution *= Self::g1(wi, wp, wp, wt, wg) * &weight;
    //         f += contribution;

    //         i += 1;
    //         if i > max_scattering_iterations {
    //             break;
    //         }

    //         let wi_reflected = -reflect(wi, wt);
    //         let eval_result = base_material.evaluate(lambda, &-wr, &wi_reflected, shading_point);
    //         // let mut contribution = eval_result.f / eval_result.pdf;
    //         let mut contribution = eval_result.f * wp.dot(wi_reflected) / eval_result.pdf;
    //         if single_scattering {
    //             contribution *= 0.5;
    //         } else {
    //             let mut pdf_wi = eval_result.pdf;
    //             pdf_wi /= wg.dot(wi_reflected);
    //             contribution *= pdf_wo / (pdf_wo + pdf_wi);
    //         }
    //         contribution *= (1.0 - Self::g1(&wi_reflected, wp, wp, wt, wg))
    //             * Self::g1(wi, wt, wp, wt, wg)
    //             * &weight;
    //         f += contribution;

    //         let uc = rng.random::<f32>();
    //         let uv = glam::vec2(rng.random::<f32>(), rng.random::<f32>());
    //         let mut lambda = lambda.clone();
    //         let sample_result = base_material.sample(uc, uv, &mut lambda, &-wr, shading_point);
    //         if !sample_result.is_sampled || sample_result.f.is_zero() {
    //             break;
    //         }
    //         if single_scattering {
    //             pdf_wo = sample_result.pdf;
    //             pdf_wo /= wg.dot(sample_result.wi);
    //             single_scattering = false;
    //         }
    //         weight *= sample_result.f * sample_result.wi.dot(wp).max(0.0) / sample_result.pdf;
    //         wr = sample_result.wi;

    //         let g = Self::g1(&wr, wp, wp, wt, wg);

    //         if rng.random::<f32>() < g {
    //             break;
    //         }

    //         // wr = -reflect(-wr, wt);
    //         let wr_vec: Vector3<_> = *wr.as_ref();
    //         let wt_vec: Vector3<_> = *wt.as_ref();
    //         let wr_vec = wr_vec + 2.0 * (-wr_vec).dot(wt_vec) * wt_vec;
    //         wr = wr_vec.normalize();

    //         // let g = Self::g1(&wr, wt, wp, wt, wg);
    //         let g = Self::g1(&wr, wp, wp, wt, wg);

    //         if rng.random::<f32>() < g {
    //             break;
    //         }

    //         i += 1;
    //         if i > max_scattering_iterations {
    //             break;
    //         }
    //     }

    //     MaterialEvaluationResult {
    //         f: f / wi.dot(wg).abs().max(1e-8),
    //         // f: f,
    //         pdf: 1.0,
    //         normal: *wg,
    //     }
    // }

    // fn pdf_microfacet_normal_mapping(
    //     lambda: &SampledWavelengths,
    //     wo: &Vector3<ShadingNormalTangent>,
    //     wi: &Vector3<ShadingNormalTangent>,
    //     shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    //     wp: &Normal<ShadingNormalTangent>,
    //     wg: &Normal<ShadingNormalTangent>,
    //     base_material: &dyn BsdfSurfaceMaterial,
    // ) -> f32 {
    //     if (wp.z() - 1.0).abs() < 1e-6 {
    //         return base_material.pdf(lambda, wo, wi, shading_point);
    //     }

    //     if wi.z().signum() != wo.z().signum() {
    //         return 0.0;
    //     }

    //     let mut pdf = 0.0;

    //     let wt = &Self::wt(wp, wg);

    //     let probability_wp = Self::lambda_p(wi, wp, wt, wg);

    //     if probability_wp > 0.0 {
    //         pdf += probability_wp * base_material.pdf(lambda, &wo, &wi, shading_point);
    //     }

    //     if probability_wp > 0.0 && wi.dot(wt) > 1e-6 {
    //         let wi_reflected = reflect(wi, &wt);
    //         pdf += probability_wp
    //             * base_material.pdf(lambda, &wo, &wi_reflected, shading_point)
    //             * (1.0 - Self::g1(&wi_reflected, wp, wp, wt, wg));
    //     }

    //     if probability_wp < 1.0 && wo.dot(wt) > 1e-6 {
    //         let wo_reflected = -reflect(&wo, &wt);
    //         pdf += (1.0 - probability_wp)
    //             * base_material.pdf(lambda, &wo_reflected, &wi, shading_point);
    //     }

    //     pdf
    // }
}
