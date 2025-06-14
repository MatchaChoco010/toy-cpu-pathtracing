//! レンダラー間で共通して使用される関数を提供するモジュール。

use math::{Ray, Render, ShadingTangent, Transform};
use scene::{
    AreaLightSampleRadiance, BsdfSurfaceMaterial, DeltaDirectionalLightIntensity,
    DeltaPointLightIntensity, SceneId, SurfaceInteraction,
};
use spectrum::SampledSpectrum;

use crate::renderer::NeeResult;

const RAY_FORWARD_EPSILON: f32 = 1e-4;

/// pdfのバランスヒューリスティックを計算する関数。
pub fn balance_heuristic(pdf_a: f32, pdf_b: f32) -> f32 {
    if pdf_a == 0.0 && pdf_b == 0.0 {
        return 0.0;
    }
    pdf_a / (pdf_a + pdf_b)
}

/// デルタ点光源の評価。
pub fn evaluate_delta_point_light<Id: SceneId>(
    scene: &scene::Scene<Id>,
    shading_point: &SurfaceInteraction<Id, Render>,
    intensity: &DeltaPointLightIntensity<Render>,
    bsdf: &dyn BsdfSurfaceMaterial<Id>,
    lambda: &spectrum::SampledWavelengths,
    wo: &math::Vector3<Render>,
    render_to_tangent: &Transform<Render, ShadingTangent>,
    light_probability: f32,
) -> SampledSpectrum {
    // シャドウレイを飛ばして可視性を確認
    let distance_vector = shading_point.position.vector_to(intensity.position);
    let shadow_ray = Ray::new(shading_point.position, distance_vector.normalize());
    let shadow_ray = shadow_ray.move_forward(RAY_FORWARD_EPSILON);
    let t = distance_vector.length() - 2.0 * RAY_FORWARD_EPSILON;
    let visible = !scene.intersect_p(&shadow_ray, t);

    if visible {
        let wo = render_to_tangent * wo;
        let wi = render_to_tangent * distance_vector.normalize();
        let shading_point = render_to_tangent * shading_point;
        let material_result = bsdf.evaluate(lambda, &wo, &wi, &shading_point);

        // Normal mappingされた表面法線に対するcos項
        let distance_squared = distance_vector.length_squared();
        let cos_theta = material_result.normal.dot(&wi).abs();

        material_result.f * &intensity.intensity * cos_theta
            / (distance_squared * light_probability)
    } else {
        SampledSpectrum::zero()
    }
}

/// デルタ方向光源の評価。
pub fn evaluate_delta_directional_light<Id: SceneId>(
    scene: &scene::Scene<Id>,
    shading_point: &SurfaceInteraction<Id, Render>,
    intensity: &DeltaDirectionalLightIntensity<Render>,
    bsdf: &dyn BsdfSurfaceMaterial<Id>,
    lambda: &spectrum::SampledWavelengths,
    wo: &math::Vector3<Render>,
    render_to_tangent: &Transform<Render, ShadingTangent>,
    light_probability: f32,
) -> SampledSpectrum {
    // シャドウレイを飛ばして可視性を確認
    let shadow_ray = Ray::new(shading_point.position, intensity.direction);
    let visible = !scene.intersect_p(&shadow_ray, f32::MAX);

    if visible {
        let wo = render_to_tangent * wo;
        let wi = render_to_tangent * intensity.direction.normalize();
        let shading_point = render_to_tangent * shading_point;
        let material_result = bsdf.evaluate(lambda, &wo, &wi, &shading_point);

        // Normal mappingされた表面法線に対するcos項
        let cos_theta = material_result.normal.dot(&wi).abs();

        material_result.f * &intensity.intensity * cos_theta / light_probability
    } else {
        SampledSpectrum::zero()
    }
}

/// 面積光源の評価（MISなし）。
pub fn evaluate_area_light<Id: SceneId>(
    scene: &scene::Scene<Id>,
    shading_point: &SurfaceInteraction<Id, Render>,
    radiance: &AreaLightSampleRadiance<Id, Render>,
    bsdf: &dyn BsdfSurfaceMaterial<Id>,
    lambda: &spectrum::SampledWavelengths,
    wo: &math::Vector3<Render>,
    render_to_tangent: &Transform<Render, ShadingTangent>,
    light_probability: f32,
) -> SampledSpectrum {
    // シャドウレイを飛ばして可視性を確認
    let distance_vector = shading_point
        .position
        .vector_to(radiance.interaction.position);
    let shadow_ray = Ray::new(shading_point.position, distance_vector.normalize());
    let shadow_ray = shadow_ray.move_forward(RAY_FORWARD_EPSILON);
    let t = distance_vector.length() - 2.0 * RAY_FORWARD_EPSILON;
    let visible = !scene.intersect_p(&shadow_ray, t);

    if visible {
        let wi = distance_vector.normalize();
        let wo_tangent = render_to_tangent * wo;
        let wi_tangent = render_to_tangent * wi;
        let shading_point_tangent = render_to_tangent * shading_point;
        let pdf = radiance.pdf;
        let material_result =
            bsdf.evaluate(lambda, &wo_tangent, &wi_tangent, &shading_point_tangent);

        // 幾何項の計算
        let distance2 = distance_vector.length_squared();
        let material_normal = render_to_tangent.inverse() * material_result.normal;
        let light_normal = radiance.light_normal;
        let cos_material = material_normal.dot(&wi).abs();
        let cos_light = light_normal.dot(&(-wi)).abs();
        let g = cos_material * cos_light / distance2;

        material_result.f * &radiance.radiance * g / (pdf * light_probability)
    } else {
        SampledSpectrum::zero()
    }
}

/// 面積光源の評価（MIS付き）。
pub fn evaluate_area_light_with_mis<Id: SceneId>(
    scene: &scene::Scene<Id>,
    shading_point: &SurfaceInteraction<Id, Render>,
    radiance: &AreaLightSampleRadiance<Id, Render>,
    bsdf: &dyn BsdfSurfaceMaterial<Id>,
    lambda: &spectrum::SampledWavelengths,
    wo: &math::Vector3<Render>,
    render_to_tangent: &Transform<Render, ShadingTangent>,
    light_probability: f32,
) -> NeeResult {
    // シャドウレイを飛ばして可視性を確認
    let distance_vector = shading_point
        .position
        .vector_to(radiance.interaction.position);
    let shadow_ray = Ray::new(shading_point.position, distance_vector.normalize());
    let shadow_ray = shadow_ray.move_forward(RAY_FORWARD_EPSILON);
    let t = distance_vector.length() - 2.0 * RAY_FORWARD_EPSILON;
    let visible = !scene.intersect_p(&shadow_ray, t);

    if visible {
        let wo = render_to_tangent * wo;
        let wi = render_to_tangent * distance_vector.normalize();
        let shading_point = render_to_tangent * shading_point;
        let pdf = radiance.pdf;
        let material_result = bsdf.evaluate(lambda, &wo, &wi, &shading_point);

        // 幾何項の計算
        let distance2 = distance_vector.length_squared();
        let light_normal = render_to_tangent * radiance.light_normal;
        let cos_material = material_result.normal.dot(&wi).abs();
        let cos_light = light_normal.dot(&(-wi)).abs();
        let g = cos_material * cos_light / distance2;

        // MISのウエイトを計算
        let pdf_light_dir = radiance.pdf_dir;
        let pdf_bsdf_dir = bsdf.pdf(lambda, &wo, &wi, &shading_point);
        let mis_weight = balance_heuristic(pdf_light_dir, pdf_bsdf_dir);

        let contribution = material_result.f * &radiance.radiance * g / (pdf * light_probability);
        NeeResult {
            contribution,
            mis_weight,
        }
    } else {
        NeeResult {
            contribution: SampledSpectrum::zero(),
            mis_weight: 1.0,
        }
    }
}
