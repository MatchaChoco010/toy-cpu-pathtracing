//! NEEを組み込んだレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map::ToneMap};
use math::{Ray, Render, Transform, VertexNormalTangent};
use scene::{Intersection, LightIntensity, MaterialSample, SceneId, SurfaceInteraction};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    filter::Filter,
    renderer::{Renderer, RendererArgs, RenderingStrategy, base_renderer::BsdfSamplingResult},
    sampler::Sampler,
};

use super::base_renderer::BaseSrgbRenderer;
use super::common;

/// Next Event Estimationを評価する（MISなし）。
fn evaluate_next_event_estimation<Id: SceneId, S: Sampler>(
    scene: &scene::Scene<Id>,
    lambda: &spectrum::SampledWavelengths,
    sampler: &mut S,
    render_to_tangent: &Transform<Render, VertexNormalTangent>,
    current_hit_info: &Intersection<Id, Render>,
) -> SampledSpectrum {
    let shading_point = &current_hit_info.interaction;
    let bsdf = match shading_point.material.as_bsdf_material() {
        Some(bsdf) => bsdf,
        None => return SampledSpectrum::zero(),
    };

    // ライトサンプラーを取得
    let light_sampler = scene.light_sampler(lambda);

    // ライトをサンプリング
    let u = sampler.get_1d();
    let light_sample = match light_sampler.sample_light(u) {
        Some(sample) => sample,
        None => return SampledSpectrum::zero(),
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
            common::evaluate_delta_point_light(
                scene,
                shading_point,
                &intensity,
                bsdf,
                lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            )
        }
        LightIntensity::IntensityDeltaDirectionalLight(intensity) => {
            // デルタライトの場合はMISを適用しない
            common::evaluate_delta_directional_light(
                scene,
                shading_point,
                &intensity,
                bsdf,
                lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            )
        }
        LightIntensity::RadianceAreaLight(radiance) => common::evaluate_area_light(
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
            common::evaluate_infinite_light(
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

/// Next Event Estimation戦略。
#[derive(Clone)]
pub struct NeeStrategy;
impl RenderingStrategy for NeeStrategy {
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
        let contribution = evaluate_next_event_estimation(
            scene,
            lambda,
            sampler,
            render_to_tangent,
            current_hit_info,
        );
        // NEE寄与を一時変数に蓄積（throughout、MISウエイト適用）
        *sample_contribution += throughout * &contribution;
    }

    fn calculate_bsdf_contribution<Id: SceneId>(
        &self,
        material_sample: &MaterialSample,
        bsdf_result: &BsdfSamplingResult<Id>,
        _scene: &scene::Scene<Id>,
        _lambda: &SampledWavelengths,
        _current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &mut SampledSpectrum,
    ) {
        if material_sample.is_specular() {
            // Specularの場合はエミッシブ寄与を蓄積（NEEはSpecularに適用されない）
            *sample_contribution += &*throughout * &bsdf_result.next_emissive_contribution;
        } else {
            // NonSpecularの場合はNEEで寄与を蓄積済みなので、エミッシブ寄与は追加しない
        }

        // throughputを更新（MISウエイトなし）
        *throughout *= &bsdf_result.throughput_modifier;
    }

    fn calculate_infinite_light_contribution<Id: SceneId, S: Sampler>(
        &self,
        _scene: &scene::Scene<Id>,
        _lambda: &SampledWavelengths,
        _throughput: &SampledSpectrum,
        _ray: &Ray<Render>,
        _shading_point: &SurfaceInteraction<Render>,
        _sampler: &mut S,
    ) -> SampledSpectrum {
        // NEEでは明示的ライトサンプリングで既に処理されているため、
        // ここでは何も追加しない
        SampledSpectrum::zero()
    }
}

/// NEE付きのパストレーサーでsRGBレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct SrgbRendererNee<'a, Id: SceneId, F: Filter, T: ToneMap> {
    base_renderer: BaseSrgbRenderer<'a, Id, F, T, NeeStrategy>,
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> SrgbRendererNee<'a, Id, F, T> {
    /// 新しいsRGBレンダラーを作成する。
    pub fn new(
        args: RendererArgs<'a, Id, F>,
        tone_map: T,
        exposure: f32,
        max_depth: usize,
    ) -> Self {
        Self {
            base_renderer: BaseSrgbRenderer::new(args, tone_map, exposure, max_depth, NeeStrategy),
        }
    }
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> Renderer for SrgbRendererNee<'a, Id, F, T> {
    type Color = ColorSrgb<T>;

    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color {
        self.base_renderer.render::<S>(p)
    }
}
