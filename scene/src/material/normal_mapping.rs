//! Microfacet-based Normal Mapping implementation based on
//! "Microfacet-based Normal Mapping for Robust Monte Carlo Path Tracing"
//! by Schüssler et al. (2017)
//! https://jo.dreggn.org/home/2017_normalmap.pdf
//!
//! This module implements Section 6.2 "Choosing a Specular Tangent Facet"
//! with random walk multi-scattering calculations.

use math::{Normal, Transform, Vector3, ShadingNormalTangent};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{BsdfSample, BsdfSurfaceMaterial, MaterialSample, SurfaceInteraction};

/// Sample a direction using microfacet-based normal mapping
pub fn sample_microfacet_normal_mapping(
    uc: f32,
    uv: glam::Vec2,
    lambda: &mut SampledWavelengths,
    wo: &Vector3<ShadingNormalTangent>,
    shading_point: &SurfaceInteraction<ShadingNormalTangent>,
    wp: &Normal<ShadingNormalTangent>,
    base_material: &dyn BsdfSurfaceMaterial,
) -> MaterialSample {
    if (wp.z() - 1.0).abs() < 1e-6 {
        return base_material.sample(uc, uv, lambda, wo, shading_point);
    }

    // Use additional random numbers for Russian roulette - we need to reserve uc for different purpose
    let mut rng_state = (uc * 1000000.0) as u32;
    let mut next_random = || {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (rng_state as f32) / (u32::MAX as f32)
    };

    let wg = Normal::new(0.0, 0.0, 1.0);
    let wt = wt(wp);

    let mut i = 0;
    let mut sample_type;
    let mut wr = -*wo;
    let mut weight = SampledSpectrum::one();

    if next_random() > lambda_p(wp, wo) {
        wr = reflect_vector(&wr, &wt);
        i += 1;
    }

    let max_scattering_iterations = 2;

    loop {
        if i > max_scattering_iterations {
            return MaterialSample::failed(wg);
        }

        let uv = glam::vec2(next_random(), next_random());
        let wi = -wr;

        let material_sample = base_material.sample(uc, uv, lambda, &wi, shading_point);
        if !material_sample.is_sampled {
            return MaterialSample::failed(wg);
        }

        weight *= material_sample.f / material_sample.pdf;
        sample_type = material_sample.sample_type;
        if weight.is_zero() {
            return MaterialSample::failed(wg);
        }
        wr = material_sample.wi;
        let g1_ = g1(wp, &wr);
        i += 1;

        if next_random() < g1_ {
            // if true {
            break;
        }
        if i > max_scattering_iterations {
            return MaterialSample::failed(*wp);
        }
        wr = reflect_vector(&wr, &wt);
        let g1_ = g1(wp, &wr);
        i += 1;

        if next_random() <= g1_ {
            // if next_random() < g1(&wt(wp), &wr) {
            break;
        }
    }

    let f = weight;
    let wi = wr;
    let pdf = 1.0;

    if wo.z().signum() != wi.z().signum() {
        // Geometric validity check failed
        return MaterialSample::failed(wg);
    }

    MaterialSample::new(f, wi, pdf, sample_type, wg)
}

/// Evaluate microfacet-based normal mapping
///
/// # Arguments
/// * `wo` - Camera Direction
/// * `wi` - Light Direction
/// * `normal_map` - Perturbed normal from normal map
/// * `uc` - Random sample for facet selection
/// * `base_bsdf_sample` - Closure to sample the base BSDF
/// * `base_bsdf_eval` - Closure to evaluate the base BSDF
pub fn evaluate_microfacet_normal_mapping<S, E>(
    wo: &Vector3<ShadingNormalTangent>,
    wi: &Vector3<ShadingNormalTangent>,
    normal_map: &Normal<ShadingNormalTangent>,
    uc: f32,
    base_bsdf_sample: S,
    base_bsdf_eval: E,
) -> SampledSpectrum
where
    S: Fn(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
    E: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> SampledSpectrum,
{
    // Check geometric validity
    let wi_cos_geom = wi.z();
    let wo_cos_geom = wo.z();
    if wi_cos_geom.signum() != wo_cos_geom.signum() {
        return SampledSpectrum::zero();
    }

    if (normal_map.z() - 1.0).abs() < 1e-6 {
        // No normal mapping, use regular BSDF with identity transform
        let transform = Transform::from_normal_map(&Normal::new(0.0, 0.0, 1.0));
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;

        return base_bsdf_eval(&wo_local, &wi_local);
    }

    evaluate_multi_scattering_bsdf(wi, wo, normal_map, uc, base_bsdf_sample, base_bsdf_eval)
}

/// Compute PDF for microfacet-based normal mapping
///
/// # Arguments
/// * `wo` - Eye direction in VertexNormalTangent space
/// * `wi` - Light direction in VertexNormalTangent space
/// * `normal_map` - Perturbed normal from normal map
/// * `geometry_normal` - Geometric normal
/// * `base_bsdf_pdf` - Closure to compute base BSDF PDF in VertexNormalTangent space
pub fn pdf_microfacet_normal_mapping<F>(
    wo: &Vector3<ShadingNormalTangent>,
    wi: &Vector3<ShadingNormalTangent>,
    normal_map: &Normal<ShadingNormalTangent>,
    base_bsdf_pdf: F,
) -> f32
where
    F: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> f32,
{
    // Check geometric validity
    let wi_cos_geom = wi.z();
    let wo_cos_geom = wo.z();
    if wi_cos_geom.signum() != wo_cos_geom.signum() {
        return 0.0;
    }

    if (normal_map.z() - 1.0).abs() < 1e-6 {
        // No normal mapping, use regular BSDF PDF with identity transform
        let transform = Transform::from_normal_map(normal_map);
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;

        return base_bsdf_pdf(&wo_local, &wi_local);
    }
    // Use double scattering pdf for estimate pdf instead of full multi scatterd pdf
    compute_double_scattering_pdf(
        wi, // Light direction
        wo, // Eye direction
        normal_map,
        base_bsdf_pdf,
    )
}

fn wt(wp: &Normal<ShadingNormalTangent>) -> Normal<ShadingNormalTangent> {
    // let dot_wp_wg = wp.dot(*wg);
    // if (dot_wp_wg - 1.0).abs() < 1e-6 {
    //     return None;
    // }

    // // Use Normal::orthogonalize_vector to compute tangent direction
    // let tangent_vec = wg.orthogonalize_vector(wp);
    // if tangent_vec.length() < 1e-6 {
    //     return None;
    // }

    // Some((-tangent_vec).to_normal())
    let t = Vector3::new(-wp.x(), -wp.y(), 0.0);
    if t.length_squared() == 0.0 {
        return Normal::new(0.0, 0.0, 1.0);
    }
    t.to_normal()
}

fn lambda_p(wp: &Normal<ShadingNormalTangent>, wi: &Vector3<ShadingNormalTangent>) -> f32 {
    // let dot_wp_wg = wp.dot(*wg);
    // let dot_wi_wp = wi.dot(wp);
    // let dot_wi_wt = wi.dot(wt);

    // let a_p = dot_wi_wp / dot_wp_wg;
    // let a_t = dot_wi_wt * (1.0 - dot_wp_wg * dot_wp_wg).sqrt() / dot_wp_wg;

    // let a_p = a_p.max(0.0);
    // let a_t = a_t.max(0.0);

    // let denom = a_p + a_t;

    // if denom.abs() < 1e-8 {
    //     return 0.5;
    // }

    // a_p / denom

    // let wi = -*wi;

    let i_dot_p = wp.dot(wi).max(0.0);
    if i_dot_p == 0.0 {
        return 0.0;
    }

    let sin_theta_wp = (1.0 - wp.z() * wp.z()).max(0.0).sqrt();

    i_dot_p / (i_dot_p + wi.dot(wt(wp)).max(0.0) * sin_theta_wp)
}

fn g1(wp: &Normal<ShadingNormalTangent>, w: &Vector3<ShadingNormalTangent>) -> f32 {
    // let dot_wm = wi.dot(m);

    // // Heaviside function
    // if dot_wm <= 0.0 {
    //     return 0.0;
    // }

    // let dot_wp_wg = wp.dot(*wg);
    // let dot_wi_wp = wi.dot(wp);
    // let dot_wi_wt = wi.dot(wt);
    // let dot_wi_wg = wi.dot(wg);

    // let a_p = dot_wi_wp / dot_wp_wg;
    // let a_t = dot_wi_wt * (1.0 - dot_wp_wg * dot_wp_wg).sqrt() / dot_wp_wg;

    // let a_p = a_p.max(0.0);
    // let a_t = a_t.max(0.0);

    // let denom = a_p + a_t;
    // if denom.abs() < 1e-8 {
    //     return 0.0;
    // }

    // (dot_wi_wg / denom).min(1.0).max(0.0)

    // return (wi.z().max(0.0) * wm.z().max(0.0)
    //     / (wi.dot(wm).max(0.0) + wi.dot(wt).max(0.0) * (1.0 - wm.z() * wm.z()).sqrt()))
    // .min(1.0);

    // let w = -*w;

    // let dot_w_wm = w.dot(wp);

    // // Heaviside function
    // if dot_w_wm <= 0.0 {
    //     return 0.0;
    // }

    // let dot_wp_wg = wp.z();
    // let dot_w_wp = w.dot(wp);
    // let dot_w_wt = w.dot(wt(wp));
    // let dot_w_wg = w.z();

    // let a_p = dot_w_wp / dot_wp_wg;
    // let a_t = dot_w_wt * (1.0 - dot_wp_wg * dot_wp_wg).sqrt() / dot_wp_wg;

    // let a_p = a_p.max(0.0);
    // let a_t = a_t.max(0.0);

    // let denom = a_p + a_t;
    // if denom.abs() < 1e-8 {
    //     return 0.0;
    // }

    // (dot_w_wg / denom).min(1.0).max(0.0)

    // return (wi.z().max(0.0) * wm.z().max(0.0)
    //     / (wi.dot(wm).max(0.0) + wi.dot(wt).max(0.0) * (1.0 - wm.z() * wm.z()).sqrt()))
    // .min(1.0);

    let num = wp.z().max(0.0) * w.z().max(0.0);
    if num == 0.0 {
        return 0.0;
    }

    let sin_theta_wp = (1.0 - wp.z() * wp.z()).max(0.0).sqrt();
    let denom = w.dot(wp).max(0.0) + w.dot(wt(wp)).max(0.0) * sin_theta_wp;

    if denom > 0.0 {
        (num / denom).min(1.0)
    } else {
        0.0
    }
}

fn sample_multi_scattering<S, P>(
    wi: &Vector3<ShadingNormalTangent>, // Eye direction
    wp: &Normal<ShadingNormalTangent>,  // Perturbed normal
    uc: f32,
    mut base_bsdf_sample: S,
    base_bsdf_pdf: P,
) -> MaterialSample
where
    S: FnMut(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
    P: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> f32,
{
    // Use additional random numbers for Russian roulette - we need to reserve uc for different purpose
    let mut rng_state = (uc * 1000000.0) as u32;
    let mut next_random = || {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (rng_state as f32) / (u32::MAX as f32)
    };

    // Create coordinate transform for perturbed facet
    let transform_wp = Transform::from_normal_map(wp);
    let transform_wp_inv = transform_wp.inverse();

    // let mut i = 0;
    // let mut sample_type = super::BsdfSampleType::SpecularReflection;
    // let mut wr = *wi;
    // let mut weight = SampledSpectrum::one();

    // let mut d = next_random() < lambda_p(wp, wi);

    // if !d {
    //     wr = reflect_vector(&-wr, &wt(wp));
    //     // weight *= g1(&wr, wt, wp, wt, wg);
    //     d = !d;
    //     i += 1;
    // }

    // loop {
    //     let wm = if d { wp } else { &wt(wp) };

    //     if d {
    //         let wr_local = &transform * wr;
    //         let Some(bsdf_result) =
    //             base_bsdf_sample(&-wr_local, glam::vec2(next_random(), next_random()))
    //         else {
    //             return MaterialSample::failed(*wp);
    //         };
    //         wr = &transform_inv * &bsdf_result.wi;
    //         sample_type = bsdf_result.sample_type;
    //         weight = bsdf_result.f * wr.dot(*wp) / bsdf_result.pdf;

    //         if next_random() < g1(wm, &wr) {
    //             break;
    //         }
    //     } else {
    //         wr = reflect_vector(&wr, &wt(wp));

    //         if next_random() < g1(wm, &wr) {
    //             break;
    //         }

    //         wr = -wr;
    //     }

    //     d = !d; // Switch between paths

    //     if weight.is_zero() {
    //         break; // Stop if BSDF value becomes zero
    //     }

    //     i += 1;
    //     if i > 100 {
    //         break;
    //     }
    // }

    // let f = weight / wr.z().abs().max(1e-6);
    // let wo = wr;
    // let pdf = 1.0;

    // if wo.z().signum() != wi.z().signum() {
    //     // Geometric validity check failed
    //     return MaterialSample::failed(*wp);
    // }

    // // let pdf = compute_double_scattering_pdf(&wo, wi, wp, base_bsdf_pdf);

    // // if pdf == 0.0 {
    // //     return MaterialSample::failed(*wp);
    // // }

    // MaterialSample::new(f, wo, pdf, sample_type, Normal::new(0.0, 0.0, 1.0))

    // ==============================================================

    // let wg = Normal::new(0.0, 0.0, 1.0);
    // let transform = math::Transform::from_normal_map(&wp);
    // let transform_inv = transform.inverse();
    // let wo_local = &transform * wi;
    // let uv = glam::vec2(next_random(), next_random());
    // let Some(bsdf_sample) = base_bsdf_sample(&wo_local, uv) else {
    //     // let Some(bsdf_sample) = bsdf.sample(&wo_local, uv) else {
    //     return MaterialSample::failed(wg);
    // };
    // let wo = transform_inv * bsdf_sample.wi;
    // if wo.z().signum() != wi.z().signum() {
    //     return MaterialSample::failed(wg);
    // }
    // let weight = bsdf_sample.f / bsdf_sample.pdf;
    // return MaterialSample::new(weight, wo, 1.0, bsdf_sample.sample_type, wg);

    let wt = wt(wp);
    // let transform_wt = Transform::from_normal_map(&wt);
    // let transform_wt_inv = transform_wt.inverse();

    let mut i = 0;
    let mut sample_type;
    let mut wr = -*wi;
    let mut weight = SampledSpectrum::one();

    if next_random() > lambda_p(wp, wi) {
        wr = reflect_vector(&wr, &wt);
        i += 1;
    }

    let max_scattering_iterations = 2;

    loop {
        if i > max_scattering_iterations {
            return MaterialSample::failed(*wp);
        }

        let uv = glam::vec2(next_random(), next_random());
        let wi_local = &transform_wp * -wr;
        let Some(bsdf_result) = base_bsdf_sample(&wi_local, uv) else {
            return MaterialSample::failed(*wp);
        };
        weight *= bsdf_result.f / bsdf_result.pdf;
        // wr = &transform_wp_inv * &bsdf_result.wi;
        // weight *= bsdf_result.f * wp.dot(&wr).max(0.0) / bsdf_result.pdf;
        // weight *= bsdf_result.f / bsdf_result.pdf;
        sample_type = bsdf_result.sample_type;
        if weight.is_zero() {
            return MaterialSample::failed(*wp);
        }
        wr = &transform_wp_inv * &bsdf_result.wi;
        let g1_ = g1(wp, &wr);
        i += 1;

        if next_random() < g1_ {
            // if true {
            break;
        }
        if i > max_scattering_iterations {
            return MaterialSample::failed(*wp);
        }
        wr = reflect_vector(&wr, &wt);
        let g1_ = g1(wp, &wr);
        i += 1;

        if next_random() <= g1_ {
            // if next_random() < g1(&wt(wp), &wr) {
            break;
        }

        // return MaterialSample::failed(*wp);

        // wr = -wr;

        // if weight.is_zero() {
        //     return MaterialSample::failed(*wp);
        // }

        // if i > 100 {
        //     return MaterialSample::failed(*wp);
        // }
    }

    let f = weight;
    let wo = wr;
    let pdf = 1.0;

    if wo.z().signum() != wi.z().signum() {
        // Geometric validity check failed
        return MaterialSample::failed(*wp);
    }

    // let pdf = compute_double_scattering_pdf(&wo, wi, wp, base_bsdf_pdf);

    // if pdf == 0.0 {
    //     return MaterialSample::failed(*wp);
    // }

    MaterialSample::new(f, wo, pdf, sample_type, Normal::new(0.0, 0.0, 1.0))

    // ==============================================================

    // if uc < lambda_p {
    //     let wi_local = &transform * wi;
    //     let Some(bsdf_result) = base_bsdf_sample(&wi_local, uv) else {
    //         return MaterialSample::failed(*wp);
    //     };
    //     let wo_local = bsdf_result.wi;
    //     let transform_inv = transform.inverse();
    //     wo = &transform_inv * &wo_local;
    //     sample_type = bsdf_result.sample_type;

    //     // f = bsdf_result.f;
    //     let dot_wo_wp = wo.dot(*wp);
    //     f = bsdf_result.f * dot_wo_wp;

    //     let g1_wo_wp = compute_masking_function(&wo, wp, wp, wt, wg);
    //     if next_random() > g1_wo_wp {
    //         wo = reflect_vector(&wo, wt);
    //         let g1_wo_wp = compute_masking_function(&wo, wp, wp, wt, wg);
    //         f *= g1_wo_wp;
    //     }
    // } else {
    //     let wi_reflected = reflect_vector(wi, wt);
    //     let wi_local = &transform * &wi_reflected;
    //     let Some(bsdf_result) = base_bsdf_sample(&wi_local, uv) else {
    //         return MaterialSample::failed(*wp);
    //     };
    //     let wo_local = bsdf_result.wi;
    //     let transform_inv = transform.inverse();
    //     wo = &transform_inv * &wo_local;
    //     sample_type = bsdf_result.sample_type;
    //     // f = bsdf_result.f;
    //     let dot_wo_wp = wo.dot(*wp);
    //     f = bsdf_result.f * dot_wo_wp;
    // }

    // let mut wo;
    // let mut f; // Initialize wo to wi

    // if uc < lambda_p {
    //     let wi_local = &transform * wi;
    //     let Some(bsdf_result) =
    //         base_bsdf_sample(&wi_local, glam::vec2(next_random(), next_random()))
    //     else {
    //         return MaterialSample::failed(*wp);
    //     };
    //     let wo_local = bsdf_result.wi;
    //     let transform_inv = transform.inverse();
    //     wo = &transform_inv * &wo_local;
    //     sample_type = bsdf_result.sample_type;

    //     let g1_wo_wp = g1(&wo, wp, wp, wt, wg);

    //     if next_random() < g1_wo_wp {
    //         // Path (a): i→p→o
    //         let dot_wo_wp = wo.dot(*wp);
    //         f = &bsdf_result.f * dot_wo_wp;
    //     } else {
    //         // let g1_wo_wt = compute_masking_function(&wo, wt, wp, wt, wg);

    //         // Path (b): i→p→t→o
    //         let wo_prime = wo;
    //         wo = reflect_vector(&wo, wt);

    //         // let g1_wo_wt = compute_masking_function(&wo, wt, wp, wt, wg);

    //         let g1_wo_wp = g1(&wo, wp, wp, wt, wg);

    //         let dot_wo_prime_wp = wo_prime.dot(*wp);
    //         // f = &bsdf_result.f * dot_wo_prime_wp * g1_wo_wt;
    //         f = &bsdf_result.f * dot_wo_prime_wp * g1_wo_wp;
    //         // f = &bsdf_result.f * dot_wo_prime_wp;
    //         // f = SampledSpectrum::zero();
    //     }
    // } else {
    //     let wi_prime = reflect_vector(wi, wt);

    //     let wi_local = &transform * &wi_prime; // Convert to outgoing for BSDF
    //     let Some(bsdf_result) =
    //         base_bsdf_sample(&wi_local, glam::vec2(next_random(), next_random()))
    //     else {
    //         return MaterialSample::failed(*wp);
    //     };
    //     let wo_local = bsdf_result.wi;
    //     let transform_inv = transform.inverse();
    //     wo = &transform_inv * &wo_local;
    //     sample_type = bsdf_result.sample_type;

    //     // Path (c): i→t→p→o
    //     let g1_wo_wp = g1(&wo, wp, wp, wt, wg);
    //     let dot_wo_wp = wo.dot(*wp);
    //     // f = bsdf_result.f * dot_wo_wp;
    //     f = bsdf_result.f * dot_wo_wp * g1_wo_wp;
    // }

    // if wg.dot(wo).signum() != wg.dot(*wi).signum() {
    //     // Geometric validity check failed
    //     return MaterialSample::failed(*wp);
    // }

    // let pdf = compute_double_scattering_pdf(&wo, wi, wp, wt, wg, lambda_p, base_bsdf_pdf);

    // // Convert to pure BSDF: f = cosine_weighted_f / cos_geom
    // f /= wg.dot(wo).abs().max(1e-6);

    // return MaterialSample::new(f, wo, pdf, sample_type, *wg);
}

/// Random-Walk BRDF evaluation using specular tangent facet
fn evaluate_multi_scattering_bsdf<S, E>(
    wo: &Vector3<ShadingNormalTangent>, // Light direction
    wi: &Vector3<ShadingNormalTangent>, // Eye direction
    wp: &Normal<ShadingNormalTangent>,  // Perturbed normal
    uc: f32,                           // Random sample
    base_bsdf_sample: S,
    base_bsdf_eval: E,
) -> SampledSpectrum
where
    S: Fn(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
    E: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> SampledSpectrum,
{
    // Random number generator state
    let mut rng_state = (uc * 1000000.0) as u32;
    let mut next_random = || {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (rng_state as f32) / (u32::MAX as f32)
    };

    // Create coordinate transform for perturbed facet
    let transform = Transform::from_normal_map(wp);
    let transform_inv = transform.inverse();

    let wo_local = &transform * wo;

    let wt = wt(wp);
    let wo_reflected = reflect_vector(wo, &wt);
    let wo_reflected_local = &transform * &wo_reflected;

    let mut i = 0;
    let mut f = SampledSpectrum::zero();
    let mut wr = *wi;
    let mut weight = SampledSpectrum::one();

    let shadowing = g1(wp, wo);
    let not_shadowed_wp_mirror = 1.0 - g1(wp, &wo_reflected);

    if next_random() > lambda_p(wp, wi) {
        wr = reflect_vector(&wr, &wt);
        i += 1;
    }

    loop {
        let wr_local = &transform * &wr;

        f += &weight * base_bsdf_eval(&wr_local, &wo_local) * wo.dot(&wp) * shadowing;
        i += 1;

        f += &weight
            * base_bsdf_eval(&wr_local, &wo_reflected_local)
            * wo_reflected.dot(&wp)
            * shadowing
            * not_shadowed_wp_mirror;
        i += 1;

        let Some(bsdf_result) =
            base_bsdf_sample(&wr_local, glam::vec2(next_random(), next_random()))
        else {
            break;
        };
        wr = &transform_inv * bsdf_result.wi;
        weight *= bsdf_result.f * wr.dot(&wp) / bsdf_result.pdf;
        // weight *= bsdf_result.f / bsdf_result.pdf;

        if weight.is_zero() {
            break;
        }

        if next_random() < g1(wp, &wr) {
            // if true {
            break;
        }

        wr = reflect_vector(&wr, &wt);
        i += 1;
        if next_random() < g1(wp, &wr) {
            break;
        }

        if i > 100 {
            f = SampledSpectrum::zero();
            break;
        }
    }

    f / wo.z().abs().max(1e-6)

    // let wo_local = &transform * wo;
    // let wi_local = &transform * wi;

    // // Term 1: Single scattering wi → p → wo
    // let term1 = {
    //     let dot_wo_wp = wo.dot(*wp); // ⟨wo, wp⟩
    //     let g1_wo_wp = g1(wp, wo);
    //     lambda_p(wp, wi) * base_bsdf_eval(&wi_local, &wo_local) * dot_wo_wp * g1_wo_wp
    // };

    // // Term 2: Double scattering wi → p → t → wo
    // let term2 = {
    //     let wo_prime = wo;
    //     let wo = reflect_vector(wo_prime, &wt(wp));
    //     let g1_wo_prime_wp = g1(&wp, &wo_prime);
    //     let g1_wo_wt = g1(&wp, &wo);
    //     let wo_local = &transform * &wo;
    //     let dot_wo_prime_wp = wo_prime.dot(*wp);
    //     lambda_p(wp, wi)
    //         * base_bsdf_eval(&wi_local, &wo_local)
    //         * dot_wo_prime_wp
    //         * (1.0 - g1_wo_prime_wp)
    //         * g1_wo_wt
    // };

    // // Term 3: Double scattering wi → t → p → wo
    // let term3 = {
    //     let wi_prime = reflect_vector(wi, &wt(wp));
    //     let wi_prime_local = &transform * &wi_prime;
    //     let dot_wo_wp = wo.dot(*wp);
    //     let g1_wo_wp = g1(wp, wo);
    //     (1.0 - lambda_p(wp, wi)) * base_bsdf_eval(&wi_prime_local, &wo_local) * dot_wo_wp * g1_wo_wp
    //     // lambda_t * base_bsdf_eval(&wi_prime_local, &wo_local) * dot_wo_wp
    // };

    // (term1 + term2 + term3) / wo.z().abs().max(1e-6)
    // // (term2) / wg.dot(wo).abs().max(1e-6)
    // // fallback_term
}

fn compute_double_scattering_pdf<F>(
    wo: &Vector3<ShadingNormalTangent>, // Light direction
    wi: &Vector3<ShadingNormalTangent>, // Eye direction
    wp: &Normal<ShadingNormalTangent>,  // Perturbed normal
    base_bsdf_pdf: F,
) -> f32
where
    F: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> f32,
{
    // // Create coordinate transform for perturbed facet
    // let transform = Transform::from_normal_map(wp);

    // let wi_local = &transform * wi;
    // let wo_local = &transform * wo;

    // let mut pdf = 0.0;

    // if lambda_p > 0.0 {
    //     pdf += lambda_p * base_bsdf_pdf(&wi_local, &wo_local);

    //     if wo.dot(wt) > 1e-6 {
    //         let wo_reflected = reflect_vector(wo, wt);
    //         let wo_reflected_local = &transform * &wo_reflected;
    //         pdf += lambda_p
    //             * base_bsdf_pdf(&wi_local, &wo_reflected_local)
    //             * (1.0 - g1(&wo_reflected, wp, wp, wt, wg));
    //     }
    // }

    // if lambda_p < 1.0 && wi.dot(wt) > 1e-6 {
    //     let wi_reflected = reflect_vector(wi, wt);
    //     let wi_reflected_local = &transform * &wi_reflected;
    //     pdf += (1.0 - lambda_p) * base_bsdf_pdf(&wi_reflected_local, &wo_local);
    // }

    // pdf *= g1(wo, wp, wp, wt, wg);

    // pdf

    if wi.z().signum() != wo.z().signum() {
        return 0.0;
    }

    let transform = Transform::from_normal_map(wp);
    let mut pdf = 0.0;

    let wi_local = &transform * wi;
    let wo_local = &transform * wo;

    let wt = wt(wp);

    let probability_wp = lambda_p(wp, wi);

    if probability_wp > 0.0 {
        pdf += probability_wp * base_bsdf_pdf(&wi_local, &wo_local);
    }

    if probability_wp > 0.0 && wo.dot(wt) > 1e-6 {
        let wo_reflected = reflect_vector(wo, &wt);
        let wo_reflected_local = &transform * &wo_reflected;
        pdf += probability_wp
            * base_bsdf_pdf(&wi_local, &wo_reflected_local)
            * (1.0 - g1(wp, &wo_reflected));
    }

    if probability_wp < 1.0 && wi.dot(wt) > 1e-6 {
        let wi_reflected = reflect_vector(&wi, &wt);
        let wi_reflected_local = &transform * wi_reflected;
        pdf += (1.0 - probability_wp) * base_bsdf_pdf(&wi_reflected_local, &wo_local);
    }

    // assert!(pdf != 0.0);

    pdf
}

/// Reflect vector about a normal
fn reflect_vector(
    incident: &Vector3<ShadingNormalTangent>,
    normal: &Normal<ShadingNormalTangent>,
) -> Vector3<ShadingNormalTangent> {
    *incident - 2.0 * incident.dot(normal) * Vector3::from(*normal)
}
