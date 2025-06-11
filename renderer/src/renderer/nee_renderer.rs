//! NEEを組み込んだレンダラーを実装するモジュール。

use color::ColorSrgb;
use color::tone_map::ToneMap;
use math::{Render, Tangent, Transform};
use scene::{BsdfSample, Intersection, LightIntensity, SceneId};
use spectrum::SampledSpectrum;

use crate::filter::Filter;
use crate::renderer::{NeeResult, Renderer, RendererArgs, RenderingStrategy};
use crate::sampler::Sampler;

use super::base_renderer::BaseSrgbRenderer;
use super::common;

/// Next Event Estimationを評価する（MISなし）。
fn evaluate_next_event_estimation<Id: SceneId, S: Sampler>(
    scene: &scene::Scene<Id>,
    lambda: &spectrum::SampledWavelengths,
    sampler: &mut S,
    render_to_tangent: &Transform<Render, Tangent>,
    current_hit_info: &Intersection<Id, Render>,
) -> SampledSpectrum {
    let shading_point = &current_hit_info.interaction;
    let bsdf = match shading_point.material.as_bsdf_material::<Id>() {
        Some(bsdf) => bsdf,
        None => return SampledSpectrum::zero(),
    };

    // ライトサンプラーを取得
    let light_sampler = scene.light_sampler(lambda);

    // ライトをサンプリング
    let u = sampler.get_1d();
    let light_sample = match light_sampler.sample_light(u) {
        Some(sample) => sample,
        None => return SampledSpectrum::zero(), // ライトがない場合は寄与なし
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
        LightIntensity::IntensityDeltaPointLight(intensity) => common::evaluate_delta_point_light(
            scene,
            shading_point,
            &intensity,
            bsdf,
            lambda,
            &current_hit_info.wo,
            render_to_tangent,
            light_sample.probability,
        ),
        LightIntensity::IntensityDeltaDirectionalLight(intensity) => {
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
    }
}

/// Next Event Estimation戦略。
#[derive(Clone)]
pub struct NeeStrategy;
impl RenderingStrategy for NeeStrategy {
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &spectrum::SampledWavelengths,
        sampler: &mut S,
        render_to_tangent: &Transform<Render, Tangent>,
        current_hit_info: &Intersection<Id, Render>,
        bsdf_sample: &BsdfSample,
    ) -> Option<NeeResult> {
        // 完全鏡面の場合はNEEを行わない
        if matches!(bsdf_sample, BsdfSample::Specular { .. }) {
            return None;
        }

        // 非鏡面の場合のみNEEを実行
        if let BsdfSample::Bsdf { .. } = bsdf_sample {
            let contribution = evaluate_next_event_estimation(
                scene,
                lambda,
                sampler,
                render_to_tangent,
                current_hit_info,
            );
            Some(NeeResult {
                contribution,
                mis_weight: 1.0, // NEE戦略ではMISを使わない
            })
        } else {
            None
        }
    }

    fn should_add_bsdf_emissive(&self, bsdf_sample: &BsdfSample) -> bool {
        // 完全鏡面の場合のみBSDFサンプル結果のエミッシブ寄与を追加
        // （非鏡面の場合はNEEで代替されるのでダブルカウント防止）
        matches!(bsdf_sample, BsdfSample::Specular { .. })
    }

    fn calculate_bsdf_mis_weight<Id: SceneId>(
        &self,
        _scene: &scene::Scene<Id>,
        _lambda: &spectrum::SampledWavelengths,
        _current_hit_info: &Intersection<Id, Render>,
        _next_hit_info: &Intersection<Id, Render>,
        _bsdf_sample: &BsdfSample,
    ) -> f32 {
        // NEEはMISウエイトなし
        1.0
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
