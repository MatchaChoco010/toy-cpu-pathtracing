//! 純粋なパストレーサーによるレンダラーを実装するモジュール。

use color::ColorSrgb;
use color::tone_map::ToneMap;
use math::{Render, Tangent, Transform};
use scene::{BsdfSample, Intersection, SceneId};

use crate::filter::Filter;
use crate::renderer::{NeeResult, Renderer, RendererArgs, RenderingStrategy};
use crate::sampler::Sampler;

use super::base_renderer::BaseSrgbRenderer;

/// 純粋なパストレーシング戦略（BSDFサンプリングのみ）。
#[derive(Clone)]
pub struct PtStrategy;
impl RenderingStrategy for PtStrategy {
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        _scene: &scene::Scene<Id>,
        _lambda: &spectrum::SampledWavelengths,
        _sampler: &mut S,
        _render_to_tangent: &Transform<Render, Tangent>,
        _current_hit_info: &Intersection<Id, Render>,
        _bsdf_sample: &BsdfSample,
    ) -> Option<NeeResult> {
        // PTはNEEを実行しない
        None
    }

    fn should_add_bsdf_emissive(&self, _bsdf_sample: &BsdfSample) -> bool {
        // PTは全てのBSDFサンプル結果のエミッシブ寄与を追加
        true
    }

    fn calculate_bsdf_mis_weight<Id: SceneId>(
        &self,
        _scene: &scene::Scene<Id>,
        _lambda: &spectrum::SampledWavelengths,
        _current_hit_info: &Intersection<Id, Render>,
        _next_hit_info: &Intersection<Id, Render>,
        _bsdf_sample: &BsdfSample,
    ) -> f32 {
        // PTはMISウエイトなし
        1.0
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
