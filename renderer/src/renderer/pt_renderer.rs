//! 純粋なパストレーサーによるレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map::ToneMap};
use math::{Ray, Render, Transform, VertexNormalTangent};
use scene::{Intersection, MaterialSample, SceneId};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    filter::Filter,
    renderer::{Renderer, RendererArgs, RenderingStrategy, base_renderer::BsdfSamplingResult},
    sampler::Sampler,
};

use super::base_renderer::BaseSrgbRenderer;

/// 純粋なパストレーシング戦略（BSDFサンプリングのみ）。
#[derive(Clone)]
pub struct PtStrategy;
impl RenderingStrategy for PtStrategy {
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        _scene: &scene::Scene<Id>,
        _lambda: &SampledWavelengths,
        _sampler: &mut S,
        _render_to_tangent: &Transform<Render, VertexNormalTangent>,
        _current_hit_info: &Intersection<Id, Render>,
        _sample_contribution: &mut SampledSpectrum,
        _throughout: &SampledSpectrum,
    ) {
        // PTはNEEを実行しない
    }

    fn calculate_bsdf_contribution<Id: SceneId>(
        &self,
        _material_sample: &MaterialSample,
        bsdf_result: &BsdfSamplingResult<Id>,
        _scene: &scene::Scene<Id>,
        _lambda: &SampledWavelengths,
        _current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &mut SampledSpectrum,
    ) {
        // エミッシブ寄与を一時変数に蓄積（Specular/NonSpecular問わず同じ処理）
        *sample_contribution += &*throughout * &bsdf_result.next_emissive_contribution;

        // throughoutを更新（MISウエイト無し）
        *throughout *= &bsdf_result.throughput_modifier;
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

        // 無限光源の放射輝度をBSDFで重み付けして計算
        let radiance = scene.evaluate_infinite_light_radiance(&background_ray, lambda);
        *sample_contribution += throughput * &material_sample.f * radiance / material_sample.pdf;
    }
}

/// 純粋なパストレーサーでsRGBレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct SrgbRendererPt<'a, Id: SceneId, F: Filter, T: ToneMap> {
    base_renderer: BaseSrgbRenderer<'a, Id, F, T, PtStrategy>,
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> SrgbRendererPt<'a, Id, F, T> {
    /// 新しいsRGBレンダラーを作成する。
    pub fn new(
        args: RendererArgs<'a, Id, F>,
        tone_map: T,
        exposure: f32,
        max_depth: usize,
    ) -> Self {
        Self {
            base_renderer: BaseSrgbRenderer::new(args, tone_map, exposure, max_depth, PtStrategy),
        }
    }
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap> Renderer for SrgbRendererPt<'a, Id, F, T> {
    type Color = ColorSrgb<T>;

    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color {
        self.base_renderer.render::<S>(p)
    }
}
