//! MISでBSDFサンプルとNEEを組み合わせたレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map::ToneMap};
use math::{Render, ShadingTangent, Transform};
use scene::{Intersection, LightIntensity, NonSpecularDirectionSample, SceneId};
use spectrum::SampledSpectrum;

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
    render_to_tangent: &Transform<Render, ShadingTangent>,
    current_hit_info: &Intersection<Id, Render>,
) -> NeeResult {
    let shading_point = &current_hit_info.interaction;
    let bsdf = match shading_point.material.as_bsdf_material::<Id>() {
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
    }
}

/// Multiple Importance Sampling戦略。
#[derive(Clone)]
pub struct MisStrategy;
impl RenderingStrategy for MisStrategy {
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &spectrum::SampledWavelengths,
        sampler: &mut S,
        render_to_tangent: &Transform<Render, ShadingTangent>,
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

    fn calculate_bsdf<Id: SceneId>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &spectrum::SampledWavelengths,
        current_hit_info: &Intersection<Id, Render>,
        non_specular_sample: &NonSpecularDirectionSample,
        bsdf_result: &BsdfSamplingResult<Id>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &mut SampledSpectrum,
    ) {
        let next_hit_info = &bsdf_result.next_hit_info;
        let light_sampler = scene.light_sampler(lambda);
        let pdf_bsdf_dir = non_specular_sample.pdf;
        let pdf_light_dir = scene.pdf_light_sample(
            &light_sampler,
            &current_hit_info.interaction,
            &next_hit_info.interaction,
        );
        let mis_weight = balance_heuristic(pdf_bsdf_dir, pdf_light_dir);

        // MISウエイトが有効の場合（MIS）、
        // エミッシブ寄与をMISウエイト付きで一時変数に蓄積
        *sample_contribution += &*throughout * &bsdf_result.next_emissive_contribution * mis_weight;

        // throughoutを更新（MISウエイト適用）
        *throughout *= &bsdf_result.throughput_modifier * mis_weight;
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
