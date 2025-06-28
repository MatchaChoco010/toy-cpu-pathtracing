//! MISでBSDFサンプルとNEEを組み合わせたレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map::ToneMap};
use math::{Ray, Render, Transform, VertexNormalTangent};
use scene::{Intersection, LightIntensity, MaterialSample, SceneId, SurfaceInteraction};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    filter::Filter,
    renderer::{
        NeeResult, Renderer, RendererArgs, RenderingStrategy, base_renderer::BsdfSamplingResult,
        common::balance_heuristic,
    },
    sampler::Sampler,
};

use super::base_renderer::BaseSrgbRenderer;
use super::common;

/// Next Event EstimationをMIS付きで評価する。
fn evaluate_next_event_estimation_with_mis<Id: SceneId, S: Sampler>(
    scene: &scene::Scene<Id>,
    lambda: &spectrum::SampledWavelengths,
    sampler: &mut S,
    render_to_tangent: &Transform<Render, VertexNormalTangent>,
    current_hit_info: &Intersection<Id, Render>,
) -> NeeResult {
    let shading_point = &current_hit_info.interaction;
    let bsdf = match shading_point.material.as_bsdf_material() {
        Some(bsdf) => bsdf,
        None => {
            return NeeResult {
                contribution: SampledSpectrum::zero(),
                mis_weight: 1.0,
            };
        }
    };

    // ライトサンプラーを取得
    let light_sampler = scene.light_sampler(lambda);

    // ライトをサンプリング
    let u = sampler.get_1d();
    let light_sample = match light_sampler.sample_light(u) {
        Some(sample) => sample,
        None => {
            return NeeResult {
                contribution: SampledSpectrum::zero(),
                mis_weight: 1.0,
            };
        }
    };

    // サンプリングしたライトの強さを計算
    let s = sampler.get_1d();
    let uv = sampler.get_2d();

    match scene.calculate_light(
        light_sample.primitive_index,
        &current_hit_info.interaction,
        lambda,
        s,
        uv,
    ) {
        LightIntensity::IntensityDeltaPointLight(intensity) => {
            // デルタライトの場合はMISを適用しない
            let contribution = common::evaluate_delta_point_light(
                scene,
                shading_point,
                &intensity,
                bsdf,
                lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            );
            NeeResult {
                contribution,
                mis_weight: 1.0,
            }
        }
        LightIntensity::IntensityDeltaDirectionalLight(intensity) => {
            // デルタライトの場合はMISを適用しない
            let contribution = common::evaluate_delta_directional_light(
                scene,
                shading_point,
                &intensity,
                bsdf,
                lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            );
            NeeResult {
                contribution,
                mis_weight: 1.0,
            }
        }
        LightIntensity::RadianceAreaLight(radiance) => common::evaluate_area_light_with_mis(
            scene,
            shading_point,
            &radiance,
            bsdf,
            lambda,
            &current_hit_info.wo,
            render_to_tangent,
            light_sample.probability,
        ),
        LightIntensity::RadianceInfinityLight(radiance_sample) => {
            // 無限光源の明示的ライトサンプリング
            common::evaluate_infinite_light_with_mis(
                scene,
                shading_point,
                &radiance_sample,
                bsdf,
                lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            )
        }
    }
}

/// Multiple Importance Sampling戦略。
#[derive(Clone)]
pub struct MisStrategy;
impl RenderingStrategy for MisStrategy {
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &SampledWavelengths,
        sampler: &mut S,
        render_to_tangent: &Transform<Render, VertexNormalTangent>,
        current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &SampledSpectrum,
    ) {
        // MISはNEEを実行する（MISウエイト付き）
        let nee_result = evaluate_next_event_estimation_with_mis(
            scene,
            lambda,
            sampler,
            render_to_tangent,
            current_hit_info,
        );
        // NEE寄与を一時変数に蓄積（throughout、MISウエイト適用）
        *sample_contribution += throughout * &nee_result.contribution * nee_result.mis_weight;
    }

    fn calculate_bsdf_contribution<Id: SceneId>(
        &self,
        material_sample: &MaterialSample,
        bsdf_result: &BsdfSamplingResult<Id>,
        scene: &scene::Scene<Id>,
        lambda: &SampledWavelengths,
        current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &mut SampledSpectrum,
    ) {
        if material_sample.is_specular() {
            // Specularの場合はMISを適用せずエミッシブ寄与をそのまま蓄積
            *sample_contribution += &*throughout * &bsdf_result.next_emissive_contribution;
        } else if material_sample.is_sampled() {
            // NonSpecularの場合はMISウエイトを計算
            let next_hit_info = &bsdf_result.next_hit_info;
            let light_sampler = scene.light_sampler(lambda);
            let pdf_bsdf_dir = material_sample.pdf;
            let pdf_light_dir = scene.pdf_light_sample(
                &light_sampler,
                &current_hit_info.interaction,
                next_hit_info,
            );
            let mis_weight = balance_heuristic(pdf_bsdf_dir, pdf_light_dir);

            // エミッシブ寄与をMISウエイト付きで一時変数に蓄積
            *sample_contribution +=
                &*throughout * &bsdf_result.next_emissive_contribution * mis_weight;
        }
        *throughout *= &bsdf_result.throughput_modifier;
    }

    fn calculate_infinite_light_contribution<Id: SceneId, S: Sampler>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &SampledWavelengths,
        throughput: &SampledSpectrum,
        ray: &Ray<Render>,
        shading_point: &SurfaceInteraction<Render>,
        _sampler: &mut S,
        sample_contribution: &mut SampledSpectrum,
    ) {
        // MISでは無限光源の放射輝度にMIS重みを適用
        let radiance = scene.evaluate_infinite_light_radiance(ray, lambda);

        // MIS重みを計算
        let light_sampler = scene.light_sampler(lambda);
        let light_pdf = scene.pdf_infinite_light_sample(&light_sampler, shading_point, ray.dir);

        let render_to_tangent = shading_point.shading_transform();
        let ray_tangent = &render_to_tangent * ray;
        let shading_point_tangent = &render_to_tangent * shading_point;

        let bsdf_pdf = if let Some(bsdf_material) = shading_point.material.as_bsdf_material() {
            // woは表面から外向きの方向（outgoing）、wiは入射方向（incoming）
            let wo_tangent = -ray_tangent.dir; // カメラレイの反対方向
            let wi_tangent = ray_tangent.dir; // レイの方向
            bsdf_material.pdf(lambda, &wo_tangent, &wi_tangent, &shading_point_tangent)
        } else {
            0.0
        };

        let mis_weight = balance_heuristic(bsdf_pdf, light_pdf);

        *sample_contribution += throughput * radiance * mis_weight;
    }

    fn calculate_bsdf_infinite_light_contribution<Id: SceneId, S: Sampler>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &SampledWavelengths,
        material_sample: &MaterialSample,
        throughput: &SampledSpectrum,
        render_to_tangent: &Transform<Render, VertexNormalTangent>,
        current_hit_info: &Intersection<Id, Render>,
        _sampler: &mut S,
        sample_contribution: &mut SampledSpectrum,
    ) {
        if !material_sample.is_sampled {
            return;
        }

        // BSDFサンプリング後のレイを構築
        let wi_render = &render_to_tangent.inverse() * &material_sample.wi;
        let offset_dir: &math::Vector3<_> = current_hit_info.interaction.normal.as_ref();
        let sign = if current_hit_info.interaction.normal.dot(wi_render) < 0.0 {
            -1.0
        } else {
            1.0
        };
        let origin = current_hit_info
            .interaction
            .position
            .translate(sign * offset_dir * 1e-5);
        let background_ray = Ray::new(origin, wi_render).move_forward(1e-5);

        // MISでは無限光源の放射輝度にMIS重みを適用
        let radiance = scene.evaluate_infinite_light_radiance(&background_ray, lambda);

        // MIS重みを計算
        let light_sampler = scene.light_sampler(lambda);
        let light_pdf = scene.pdf_infinite_light_sample(
            &light_sampler,
            &current_hit_info.interaction,
            background_ray.dir,
        );
        let bsdf_pdf = material_sample.pdf;
        let mis_weight = balance_heuristic(bsdf_pdf, light_pdf);

        // BSDF項を計算
        let cos_theta = material_sample.normal.dot(material_sample.wi).abs();
        let throughput_factor = cos_theta / material_sample.pdf;

        *sample_contribution +=
            throughput * &material_sample.f * radiance * throughput_factor * mis_weight;
    }
}

/// MISでBSDFサンプルとNEEを組み合わせたパストレーサーでsRGBレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct SrgbRendererMis<'a, Id: SceneId, F: Filter, T: ToneMap> {
    base_renderer: BaseSrgbRenderer<'a, Id, F, T, MisStrategy>,
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> SrgbRendererMis<'a, Id, F, T> {
    /// 新しいsRGBレンダラーを作成する。
    pub fn new(
        args: RendererArgs<'a, Id, F>,
        tone_map: T,
        exposure: f32,
        max_depth: usize,
    ) -> Self {
        Self {
            base_renderer: BaseSrgbRenderer::new(args, tone_map, exposure, max_depth, MisStrategy),
        }
    }
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> Renderer for SrgbRendererMis<'a, Id, F, T> {
    type Color = ColorSrgb<T>;

    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color {
        self.base_renderer.render::<S>(p)
    }
}
