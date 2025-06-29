//! Microfacet-based Normal Mapping implementation based on
//! "Microfacet-based Normal Mapping for Robust Monte Carlo Path Tracing"
//! by Schüssler et al. (2017)
//! https://jo.dreggn.org/home/2017_normalmap.pdf
//!
//! This module implements Section 6.2 "Choosing a Specular Tangent Facet"
//! with double-scattering calculations for energy conservation and symmetry.
//!
//! # Important Note on Direction Convention
//!
//! **Paper notation** vs **Our implementation**:
//! - Paper: `ωi` = eye direction (camera), `ωo` = light direction
//! - Our code: `wo` = camera direction, `wi` = light direction
//!
//! Therefore, when implementing paper equations:
//! - Paper's `ωi` corresponds to our `wo`
//! - Paper's `ωo` corresponds to our `wi`
//! - Paper's `f(ωi, ωo)` becomes our `f(wo, wi)`
//! - Paper's `⟨ωo, ωg⟩` becomes our `⟨wi, ωg⟩` (light cosine term)

use math::{Normal, ShadingNormalTangent, Transform, Vector3, VertexNormalTangent};
use spectrum::SampledSpectrum;

use crate::{BsdfSample, MaterialSample};

// MicrofacetNormalMappingResult removed - evaluation is deterministic, not stochastic

/// Sample a direction using microfacet-based normal mapping
///
/// # Arguments
/// * `wo` - Outgoing direction in VertexNormalTangent space
/// * `uv` - Random sample for BSDF sampling
/// * `uc` - Random sample for facet selection
/// * `normal_map` - Perturbed normal from normal map
/// * `geometry_normal` - Geometric normal (should be (0,0,1) in VertexNormalTangent space)
/// * `base_bsdf_sample` - Closure to sample the base BSDF in ShadingNormalTangent space
/// * `albedo` - Surface albedo
pub fn sample_microfacet_normal_mapping<F>(
    wo: &Vector3<VertexNormalTangent>,
    uv: glam::Vec2,
    uc: f32,
    normal_map: &Normal<VertexNormalTangent>,
    geometry_normal: &Normal<VertexNormalTangent>,
    base_bsdf_sample: F,
    _albedo: SampledSpectrum,
) -> Option<MaterialSample>
where
    F: Fn(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
{
    // Check if normal map is identity (no change from geometric normal)
    let perturbed_normal = Vector3::from(*normal_map);
    let geometric_normal = Vector3::from(*geometry_normal);

    let dot_pg = perturbed_normal.dot(geometric_normal);
    if (dot_pg - 1.0).abs() < 1e-6 {
        // No normal mapping, use regular BSDF with identity transform
        let transform = Transform::from_normal_map(normal_map);
        let wo_local = &transform * wo;

        let bsdf_result = base_bsdf_sample(&wo_local, uv)?;
        let wi_local = bsdf_result.wi;

        let transform_inv = transform.inverse();
        let wi = &transform_inv * &wi_local;

        return Some(MaterialSample::new(
            bsdf_result.f,
            wi,
            bsdf_result.pdf,
            bsdf_result.sample_type,
            *normal_map,
        ));
    }

    let tangent_normal = compute_tangent_facet_normal(&perturbed_normal, &geometric_normal)?;

    let (lambda_p, _lambda_t) = compute_intersection_probabilities(
        wo,
        &perturbed_normal,
        &tangent_normal,
        &geometric_normal,
    );

    // Sample initial facet based on intersection probabilities
    if uc < lambda_p {
        // Start from perturbed facet
        sample_from_perturbed_facet(
            wo,
            uv,
            &perturbed_normal,
            &tangent_normal,
            &geometric_normal,
            base_bsdf_sample,
            *normal_map,
            lambda_p,
        )
    } else {
        // Start from tangent facet
        sample_from_tangent_facet(
            wo,
            uv,
            &perturbed_normal,
            &tangent_normal,
            &geometric_normal,
            base_bsdf_sample,
            *normal_map,
        )
    }
}

///
/// # Arguments
/// * `wo` - Outgoing direction in VertexNormalTangent space
/// * `wi` - Incoming direction in VertexNormalTangent space
/// * `normal_map` - Perturbed normal from normal map
/// * `geometry_normal` - Geometric normal
/// * `base_bsdf_eval` - Closure to evaluate the base BSDF in ShadingNormalTangent space
pub fn evaluate_microfacet_normal_mapping<F>(
    wo: &Vector3<VertexNormalTangent>,
    wi: &Vector3<VertexNormalTangent>,
    normal_map: &Normal<VertexNormalTangent>,
    geometry_normal: &Normal<VertexNormalTangent>,
    base_bsdf_eval: F,
) -> SampledSpectrum
where
    F: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> SampledSpectrum,
{
    // Check geometric validity
    let wi_cos_geom = geometry_normal.dot(*wi);
    let wo_cos_geom = geometry_normal.dot(*wo);
    if wi_cos_geom.signum() != wo_cos_geom.signum() {
        return SampledSpectrum::zero();
    }

    let perturbed_normal = Vector3::from(*normal_map);
    let geometric_normal = Vector3::from(*geometry_normal);

    let dot_pg = perturbed_normal.dot(geometric_normal);
    if (dot_pg - 1.0).abs() < 1e-6 {
        // No normal mapping, use regular BSDF with identity transform
        let transform = Transform::from_normal_map(normal_map);
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;

        return base_bsdf_eval(&wo_local, &wi_local);
    }

    // Compute tangent facet normal
    let tangent_normal = match compute_tangent_facet_normal(&perturbed_normal, &geometric_normal) {
        Some(tn) => tn,
        None => {
            return SampledSpectrum::zero();
        }
    };

    let (lambda_p, lambda_t) = compute_intersection_probabilities(
        wo,
        &perturbed_normal,
        &tangent_normal,
        &geometric_normal,
    );

    let cosine_weighted_f = evaluate_double_scattering_bsdf(
        wi,
        wo,
        &perturbed_normal,
        &tangent_normal,
        &geometric_normal,
        lambda_p,
        lambda_t,
        base_bsdf_eval,
        normal_map,
    );

    let cos_wi_geom = wi.dot(*geometry_normal);
    if cos_wi_geom > 1e-6 {
        cosine_weighted_f / cos_wi_geom.max(1e-6)
    } else {
        SampledSpectrum::zero()
    }
}

/// Compute PDF for microfacet-based normal mapping
///
/// # Arguments
/// * `wo` - Outgoing direction in VertexNormalTangent space
/// * `wi` - Incoming direction in VertexNormalTangent space
/// * `normal_map` - Perturbed normal from normal map
/// * `geometry_normal` - Geometric normal
/// * `base_bsdf_pdf` - Closure to compute base BSDF PDF in ShadingNormalTangent space
pub fn pdf_microfacet_normal_mapping<F>(
    wo: &Vector3<VertexNormalTangent>,
    wi: &Vector3<VertexNormalTangent>,
    normal_map: &Normal<VertexNormalTangent>,
    geometry_normal: &Normal<VertexNormalTangent>,
    base_bsdf_pdf: F,
) -> f32
where
    F: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> f32,
{
    // Check geometric validity
    let wi_cos_geom = geometry_normal.dot(*wi);
    let wo_cos_geom = geometry_normal.dot(*wo);
    if wi_cos_geom.signum() != wo_cos_geom.signum() {
        return 0.0;
    }

    let perturbed_normal = Vector3::from(*normal_map);
    let geometric_normal = Vector3::from(*geometry_normal);

    let dot_pg = perturbed_normal.dot(geometric_normal);
    if (dot_pg - 1.0).abs() < 1e-6 {
        // No normal mapping, use regular BSDF PDF with identity transform
        let transform = Transform::from_normal_map(normal_map);
        let wo_local = &transform * wo;
        let wi_local = &transform * wi;

        return base_bsdf_pdf(&wo_local, &wi_local);
    }

    // Compute tangent facet normal
    let tangent_normal = match compute_tangent_facet_normal(&perturbed_normal, &geometric_normal) {
        Some(tn) => tn,
        None => return 0.0,
    };

    let (lambda_p, _lambda_t) = compute_intersection_probabilities(
        wo,
        &perturbed_normal,
        &tangent_normal,
        &geometric_normal,
    );

    // Compute microfacet PDF
    compute_microfacet_pdf(
        wo,
        wi,
        &perturbed_normal,
        &tangent_normal,
        &geometric_normal,
        lambda_p,
    )
}

//
// Helper functions
//

/// Compute tangent facet normal ωt from perturbed normal ωp and geometric normal ωg
///
/// Based on Section 4.1: "the other one is perpendicular to the geometric surface
/// and its corresponding micro-normal ωt is tangent to the geometric surface"
///
/// The V-cavity model requires that the average normal equals the geometric normal:
/// D(ωp)/⟨ωp,ωg⟩ + D(ωt) = 1, where D(ωt) is the area of tangent facet
fn compute_tangent_facet_normal(
    perturbed_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
) -> Option<Vector3<VertexNormalTangent>> {
    let dot_pg = perturbed_normal.dot(*geometric_normal);
    if (dot_pg - 1.0).abs() < 1e-6 {
        // ωp and ωg are parallel, no tangent facet needed
        return None;
    }

    // For V-cavity model, ωt must be:
    // 1. Tangent to geometric surface (ωt·ωg = 0)
    // 2. Forms a V-cavity with ωp such that average normal = ωg
    // 3. Perpendicular to the projection of ωp onto the tangent plane

    // Project ωp onto tangent plane of geometric surface
    let projected_p = *perturbed_normal - dot_pg * *geometric_normal;
    let projected_length = projected_p.length();

    if projected_length < 1e-6 {
        // ωp is parallel to ωg, no meaningful tangent direction
        return None;
    }

    // ωt is in tangent plane, opposing the projected direction
    // This creates a V-cavity where the average normal equals ωg
    let tangent_direction = -projected_p / projected_length;

    Some(tangent_direction)
}

/// Compute projected areas and intersection probabilities
/// Paper uses ωi (eye direction) which corresponds to our wo (camera direction)
fn compute_intersection_probabilities(
    paper_eye_dir: &Vector3<VertexNormalTangent>, // ωi in paper (eye direction = our wo)
    perturbed_normal: &Vector3<VertexNormalTangent>,
    tangent_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
) -> (f32, f32) {
    let dot_pg = perturbed_normal.dot(*geometric_normal);
    let dot_eye_p = paper_eye_dir.dot(*perturbed_normal); // ⟨ωi, ωp⟩
    let dot_eye_t = paper_eye_dir.dot(*tangent_normal); // ⟨ωi, ωt⟩

    // Projected areas (equations from Section 4.3)
    // ap(ωi) = ⟨ωi, ωp⟩ / ⟨ωp, ωg⟩
    // at(ωi) = ⟨ωi, ωt⟩√(1 - ⟨ωp, ωg⟩²) / ⟨ωp, ωg⟩
    let a_p = (dot_eye_p / dot_pg).max(0.0);
    let a_t = (dot_eye_t * (1.0 - dot_pg * dot_pg).sqrt() / dot_pg).max(0.0);
    let denom = a_p + a_t;

    if denom.abs() < 1e-6 {
        return (0.5, 0.5);
    }

    // Intersection probabilities based on visible projected areas
    let lambda_p = a_p / denom;
    let lambda_t = a_t / denom;

    (lambda_p, lambda_t)
}

/// Compute masking function G1
fn compute_masking_function(
    w: &Vector3<VertexNormalTangent>,
    m: &Vector3<VertexNormalTangent>,
    perturbed_normal: &Vector3<VertexNormalTangent>,
    tangent_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
) -> f32 {
    let dot_wm = w.dot(*m);
    if dot_wm <= 0.0 {
        return 0.0; // Heaviside function
    }

    let dot_pg = perturbed_normal.dot(*geometric_normal);
    let dot_w_p = w.dot(*perturbed_normal);
    let dot_w_t = w.dot(*tangent_normal);
    let dot_wg = w.dot(*geometric_normal);

    // Projected areas (can be negative for backfacing facets)
    let a_p = (dot_w_p / dot_pg).max(0.0);
    let a_t = (dot_w_t * (1.0 - dot_pg * dot_pg).sqrt() / dot_pg).max(0.0);

    // Total projected area (sum of all facet contributions)
    let total_projected_area = a_p + a_t;
    if total_projected_area.abs() < 1e-6 {
        return 0.0; // Avoid division by zero
    }

    (dot_wg / total_projected_area).min(1.0).max(0.0)
}

/// Sample from perturbed facet (Section 6.2)
fn sample_from_perturbed_facet<F>(
    wo: &Vector3<VertexNormalTangent>,
    uv: glam::Vec2,
    perturbed_normal: &Vector3<VertexNormalTangent>,
    tangent_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
    base_bsdf_sample: F,
    normal_map: Normal<VertexNormalTangent>,
    lambda_p: f32,
) -> Option<MaterialSample>
where
    F: Fn(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
{
    // Create coordinate transform for perturbed facet
    let transform = Transform::from_normal_map(&normal_map);

    // Transform wo to perturbed facet local space
    let wo_local = &transform * wo;

    // Sample BSDF in perturbed facet local space
    let bsdf_result = base_bsdf_sample(&wo_local, uv)?;
    let wi_local = bsdf_result.wi;

    // Transform wi back to VertexNormalTangent space
    let transform_inv = transform.inverse();
    let wi = &transform_inv * &wi_local;

    // Compute masking functions
    let g1_wo = compute_masking_function(
        wo,
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );
    let g1_wi = compute_masking_function(
        &wi,
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );

    let f = bsdf_result.f * g1_wo * g1_wi;

    // PDF needs to be adjusted by the probability of choosing this facet
    let adjusted_pdf = lambda_p * bsdf_result.pdf;

    Some(MaterialSample::new(
        f,
        wi,
        adjusted_pdf,
        bsdf_result.sample_type,
        normal_map,
    ))
}

/// Sample from tangent facet (Section 6.2)
fn sample_from_tangent_facet<F>(
    wo: &Vector3<VertexNormalTangent>,
    uv: glam::Vec2,
    perturbed_normal: &Vector3<VertexNormalTangent>,
    tangent_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
    base_bsdf_sample: F,
    normal_map: Normal<VertexNormalTangent>,
) -> Option<MaterialSample>
where
    F: Fn(&Vector3<ShadingNormalTangent>, glam::Vec2) -> Option<BsdfSample>,
{
    // Create coordinate transform for perturbed facet
    let transform = Transform::from_normal_map(&normal_map);

    // Reflect wo about tangent facet
    let wo_reflected = reflect_vector(wo, tangent_normal);

    // Transform reflected wo to perturbed facet local space
    let wo_reflected_local = &transform * &wo_reflected;

    // Sample from perturbed facet using reflected direction
    let bsdf_result = base_bsdf_sample(&wo_reflected_local, uv)?;
    let wi_local = bsdf_result.wi;

    // Transform wi back to VertexNormalTangent space
    let transform_inv = transform.inverse();
    let wi = &transform_inv * &wi_local;

    // Apply masking and compute contribution
    let g1_wo = compute_masking_function(
        wo,
        tangent_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );
    let g1_wi = compute_masking_function(
        &wi,
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );

    let f = bsdf_result.f * g1_wo * g1_wi;

    Some(MaterialSample::new(
        f,
        wi,
        bsdf_result.pdf,
        bsdf_result.sample_type,
        normal_map,
    ))
}

fn evaluate_double_scattering_bsdf<F>(
    paper_light_dir: &Vector3<VertexNormalTangent>, // ωo in paper (light direction)
    paper_eye_dir: &Vector3<VertexNormalTangent>,   // ωi in paper (eye direction)
    perturbed_normal: &Vector3<VertexNormalTangent>,
    tangent_normal: &Vector3<VertexNormalTangent>,
    geometric_normal: &Vector3<VertexNormalTangent>,
    lambda_p: f32,
    lambda_t: f32,
    base_bsdf_eval: F,
    normal_map: &Normal<VertexNormalTangent>,
) -> SampledSpectrum
where
    F: Fn(&Vector3<ShadingNormalTangent>, &Vector3<ShadingNormalTangent>) -> SampledSpectrum,
{
    // Create coordinate transform for perturbed facet
    let transform = Transform::from_normal_map(normal_map);

    // Transform directions to perturbed facet local space
    // paper_eye_dir (ωi) goes to wo_local for BSDF evaluation
    // paper_light_dir (ωo) goes to wi_local for BSDF evaluation
    let wo_local = &transform * paper_eye_dir; // ωi → wo for BSDF
    let wi_local = &transform * paper_light_dir; // ωo → wi for BSDF

    // f_2(ωi, ωo)⟨ωo, ωg⟩ =
    //   λ_p(ωi) * f_p(ωi, ωo)⟨ωo, ωp⟩ * G_1(ωo, ωp) +
    //   λ_p(ωi) * f_p(ωi, ωo')⟨ωo', ωp⟩ * (1 - G_1(ωo', ωp)) * G_1(ωo, ωt) +
    //   λ_t(ωi) * f_p(ωi', ωo)⟨ωo, ωp⟩ * G_1(ωo, ωp)

    // Term 1: Single scattering ωi → p → ωo
    let cos_light_p = paper_light_dir.dot(*perturbed_normal); // ⟨ωo, ωp⟩
    let g1_light_p = compute_masking_function(
        paper_light_dir, // ωo
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );
    let term1 = lambda_p * base_bsdf_eval(&wo_local, &wi_local) * cos_light_p * g1_light_p;

    // Term 2: Double scattering ωi → p → t → ωo
    // Paper's ωo' = reflect(ωo, ωt) (light direction reflected by tangent)
    let light_reflected_by_tangent = reflect_vector(paper_light_dir, tangent_normal);
    let light_reflected_local = &transform * &light_reflected_by_tangent;
    let cos_light_reflected_p = light_reflected_by_tangent.dot(*perturbed_normal); // ⟨ωo', ωp⟩

    // Masking: G1(ωo, ωt) for tangent facet visibility from light direction
    let g1_light_tangent = compute_masking_function(
        paper_light_dir, // ωo
        tangent_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );

    // Masking: G1(ωo', ωp) for perturbed facet visibility from reflected light direction
    let g1_light_reflected_perturbed = compute_masking_function(
        &light_reflected_by_tangent, // ωo'
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );

    let term2 = lambda_p
        * base_bsdf_eval(&wo_local, &light_reflected_local)  // f_p(ωi, ωo')
        * cos_light_reflected_p  // ⟨ωo', ωp⟩
        * (1.0 - g1_light_reflected_perturbed)  // (1 - G_1(ωo', ωp))
        * g1_light_tangent; // G_1(ωo, ωt)

    // Term 3: Double scattering ωi → t → p → ωo
    // Paper's ωi' = reflect(ωi, ωt) (eye direction reflected by tangent)
    let eye_reflected_by_tangent = reflect_vector(paper_eye_dir, tangent_normal);
    let eye_reflected_local = &transform * &eye_reflected_by_tangent;
    let cos_light_p_term3 = paper_light_dir.dot(*perturbed_normal); // ⟨ωo, ωp⟩

    // Masking: G1(ωo, ωp) for perturbed facet visibility from light direction
    let g1_light_perturbed_term3 = compute_masking_function(
        paper_light_dir, // ωo
        perturbed_normal,
        perturbed_normal,
        tangent_normal,
        geometric_normal,
    );

    let term3 = lambda_t
        * base_bsdf_eval(&eye_reflected_local, &wi_local)  // f_p(ωi', ωo)
        * cos_light_p_term3  // ⟨ωo, ωp⟩
        * g1_light_perturbed_term3; // G_1(ωo, ωp) - Only this G1 as per paper

    term1 + term2 + term3
}

/// Compute microfacet PDF
fn compute_microfacet_pdf(
    _wo: &Vector3<VertexNormalTangent>,
    wi: &Vector3<VertexNormalTangent>,
    perturbed_normal: &Vector3<VertexNormalTangent>,
    _tangent_normal: &Vector3<VertexNormalTangent>,
    _geometric_normal: &Vector3<VertexNormalTangent>,
    lambda_p: f32,
) -> f32 {
    // For Section 6.2 specular tangent facet approach,
    // PDF is primarily determined by the perturbed facet
    let wi_cos_p = wi.dot(*perturbed_normal);
    if wi_cos_p <= 0.0 {
        return 0.0;
    }

    // For Section 6.2 with specular tangent facet:
    // - Perturbed facet contributes normal cosine-weighted PDF
    // - Tangent facet is specular, so it doesn't contribute to PDF in the usual sense
    // The total PDF is weighted by the probability of choosing the perturbed facet
    let pdf_p = compute_cosine_hemisphere_pdf(wi, perturbed_normal);

    // Since tangent facet is specular, the effective PDF is just from perturbed facet
    // weighted by the probability of sampling from it
    lambda_p * pdf_p
}

/// Compute cosine hemisphere PDF
fn compute_cosine_hemisphere_pdf(
    w: &Vector3<VertexNormalTangent>,
    normal: &Vector3<VertexNormalTangent>,
) -> f32 {
    let cos_theta = w.dot(*normal);
    if cos_theta <= 0.0 {
        0.0
    } else {
        cos_theta / std::f32::consts::PI
    }
}

/// Reflect vector about a normal
fn reflect_vector(
    incident: &Vector3<VertexNormalTangent>,
    normal: &Vector3<VertexNormalTangent>,
) -> Vector3<VertexNormalTangent> {
    *incident - 2.0 * incident.dot(*normal) * *normal
}
