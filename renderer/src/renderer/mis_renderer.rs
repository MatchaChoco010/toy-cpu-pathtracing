//! MISでBSDFサンプルとNEEを組み合わせたレンダラーを実装するモジュール。

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

/// Next Event EstimationをMIS付きで評価する。
fn evaluate_next_event_estimation_with_mis<Id: SceneId, S: Sampler>(
    scene: &scene::Scene<Id>,
    lambda: &spectrum::SampledWavelengths,
    sampler: &mut S,
    render_to_tangent: &Transform<Render, Tangent>,
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
    let light_sampler = scene.light_sampler(&lambda);

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
        &lambda,
        s,
        uv,
    ) {
        LightIntensity::IrradianceDeltaPointLight(irradiance) => {
            // デルタライトの場合はMISを適用しない
            let contribution = common::evaluate_delta_point_light(
                scene,
                &shading_point,
                &irradiance,
                bsdf,
                &lambda,
                &current_hit_info.wo,
                render_to_tangent,
                light_sample.probability,
            );
            NeeResult {
                contribution,
                mis_weight: 1.0,
            }
        }
        LightIntensity::IrradianceDeltaDirectionalLight(irradiance) => {
            // デルタライトの場合はMISを適用しない
            let contribution = common::evaluate_delta_directional_light(
                scene,
                &shading_point,
                &irradiance,
                bsdf,
                &lambda,
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
            &shading_point,
            &radiance,
            bsdf,
            &lambda,
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
        render_to_tangent: &Transform<Render, Tangent>,
        current_hit_info: &Intersection<Id, Render>,
        bsdf_sample: &BsdfSample,
    ) -> Option<NeeResult> {
        // 完全鏡面の場合はMISを行わない
        if matches!(bsdf_sample, BsdfSample::Specular { .. }) {
            return None;
        }

        // 非鏡面の場合のみMIS付きNEEを実行
        if let BsdfSample::Bsdf { .. } = bsdf_sample {
            let nee_result = evaluate_next_event_estimation_with_mis(
                scene,
                lambda,
                sampler,
                render_to_tangent,
                current_hit_info,
            );
            Some(nee_result)
        } else {
            None
        }
    }

    fn should_add_bsdf_emissive(&self, _bsdf_sample: &BsdfSample) -> bool {
        // MISは鏡面・非鏡面に関わらずBSDFサンプル結果のエミッシブ寄与を追加
        // （非鏡面の場合はMISウエイト適用）
        true
    }

    fn calculate_bsdf_mis_weight<Id: SceneId>(
        &self,
        scene: &scene::Scene<Id>,
        lambda: &spectrum::SampledWavelengths,
        current_hit_info: &Intersection<Id, Render>,
        next_hit_info: &Intersection<Id, Render>,
        bsdf_sample: &BsdfSample,
    ) -> f32 {
        match bsdf_sample {
            BsdfSample::Specular { .. } => {
                // 完全鏡面の場合はMISウエイトなし
                1.0
            }
            BsdfSample::Bsdf { pdf, .. } => {
                // 非鏡面の場合はMISウエイトを計算
                let light_sampler = scene.light_sampler(lambda);
                let pdf_bsdf_dir = *pdf;
                let pdf_light_dir = scene.pdf_light_sample(
                    &light_sampler,
                    &current_hit_info.interaction,
                    &next_hit_info.interaction,
                );
                common::balance_heuristic(pdf_bsdf_dir, pdf_light_dir)
            }
        }
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
