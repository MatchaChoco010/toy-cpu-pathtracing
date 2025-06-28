//! レンダラーの実装を行うモジュール。

use std::path::Path;

use image::{ImageFormat, Rgb, RgbImage};
use rayon::prelude::*;

use color::Color;
use math::{Render, Transform, VertexNormalTangent};
use scene::{Intersection, MaterialSample, Scene, SceneId};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::camera::Camera;
use crate::filter::Filter;
use crate::renderer::base_renderer::BsdfSamplingResult;
use crate::sampler::Sampler;

mod base_renderer;
mod common;

mod albedo_renderer;
mod mis_renderer;
mod nee_renderer;
mod normal_renderer;
mod pt_renderer;

pub use albedo_renderer::AlbedoRenderer;
pub use mis_renderer::SrgbRendererMis;
pub use nee_renderer::SrgbRendererNee;
pub use normal_renderer::NormalRenderer;
pub use pt_renderer::SrgbRendererPt;

/// NEE評価結果を表す構造体。
#[derive(Clone, Debug)]
pub struct NeeResult {
    /// NEEによる寄与値
    pub contribution: SampledSpectrum,
    /// NEEに適用するMISウエイト
    pub mis_weight: f32,
}

/// レンダリング戦略を定義するトレイト。
pub trait RenderingStrategy: Clone + Send + Sync {
    /// Next Event Estimationを評価する。
    fn evaluate_nee<Id: SceneId, S: Sampler>(
        &self,
        scene: &Scene<Id>,
        lambda: &SampledWavelengths,
        sampler: &mut S,
        render_to_tangent: &Transform<Render, VertexNormalTangent>,
        current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &SampledSpectrum,
    );

    /// マテリアルサンプリング結果に応じた寄与計算を行う。
    fn calculate_bsdf_contribution<Id: SceneId>(
        &self,
        material_sample: &MaterialSample,
        bsdf_result: &BsdfSamplingResult<Id>,
        scene: &Scene<Id>,
        lambda: &SampledWavelengths,
        current_hit_info: &Intersection<Id, Render>,
        sample_contribution: &mut SampledSpectrum,
        throughout: &mut SampledSpectrum,
    );

    /// BSDFサンプリング後のレイが背景にヒットした場合の寄与を計算する。
    fn calculate_bsdf_infinite_light_contribution<Id: SceneId, S: Sampler>(
        &self,
        scene: &Scene<Id>,
        lambda: &SampledWavelengths,
        material_sample: &MaterialSample,
        throughput: &SampledSpectrum,
        render_to_tangent: &Transform<Render, VertexNormalTangent>,
        current_hit_info: &Intersection<Id, Render>,
        sampler: &mut S,
        sample_contribution: &mut SampledSpectrum,
    );
}

/// レンダラーの作成のための引数。
#[derive(Clone)]
pub struct RendererArgs<'a, Id: SceneId, F: Filter> {
    pub resolution: glam::UVec2,
    pub spp: u32,
    pub seed: u32,
    pub scene: &'a Scene<Id>,
    pub camera: &'a Camera<F>,
}

/// レンダラーのトレイト。
pub trait Renderer: Send + Sync + Clone {
    type Color: Color;

    /// レンダリングを行い、RGBの色を返す。
    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color;
}

/// レンダラーで書き出す画像の構造体。
pub struct RendererImage<R: Renderer> {
    pixels: Vec<[f32; 3]>,
    width: u32,
    height: u32,
    renderer: R,
}
impl<R: Renderer> RendererImage<R> {
    /// 新しいレンダラー画像を作成する。
    pub fn new(width: u32, height: u32, renderer: R) -> Self {
        let pixels = vec![[0.0; 3]; (width * height) as usize];
        Self {
            pixels,
            width,
            height,
            renderer,
        }
    }

    /// 画像に対してレンダリングを行う。
    pub fn render<S: Sampler>(&mut self) {
        self.pixels
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index as u32 % self.width;
                let y = index as u32 / self.width;
                let p = glam::uvec2(x, y);
                let mut renderer = self.renderer.clone();
                let rgb = renderer.render::<S>(p).rgb();
                pixel[0] = rgb.x;
                pixel[1] = rgb.y;
                pixel[2] = rgb.z;
            });
    }

    /// 画像を保存する。
    pub fn save(&self, path: impl AsRef<Path>) {
        RgbImage::from_fn(self.width, self.height, |x, y| {
            let pixel = self.pixels[(y * self.width + x) as usize];
            Rgb([
                (pixel[0] * 255.0) as u8,
                (pixel[1] * 255.0) as u8,
                (pixel[2] * 255.0) as u8,
            ])
        })
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
    }
}
