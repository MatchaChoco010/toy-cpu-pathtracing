//! レンダラーの実装を行うモジュール。

use std::path::Path;

use image::{ImageFormat, Rgb, RgbImage};
use rayon::prelude::*;

use color::{Color, ColorSrgb};
use scene::{Scene, SceneId, SurfaceInteraction};

use crate::camera::Camera;
use crate::filter::Filter;
use crate::sampler::{Sampler, SamplerFactory};

/// レンダラーの作成のための引数。
#[derive(Clone)]
pub struct RendererArgs<'a, Id: SceneId, F: Filter, SF: SamplerFactory> {
    pub width: u32,
    pub height: u32,
    pub spp: u32,
    pub scene: &'a Scene<Id>,
    pub camera: &'a Camera<F>,
    pub sampler_factory: &'a SF,
}

/// レンダラーのトレイト。
pub trait Renderer: Send + Sync + Clone {
    type Color: Color;

    /// レンダリングを行い、RGBの色を返す。
    fn render(&mut self, x: u32, y: u32) -> Self::Color;
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
    pub fn render(&mut self) {
        self.pixels
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index as u32 % self.width;
                let y = index as u32 / self.width;
                let mut renderer = self.renderer.clone();
                let rgb = renderer.render(x, y).rgb();
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

/// 法線をレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct NormalRenderer<'a, Id: SceneId, F: Filter, SF: SamplerFactory> {
    args: RendererArgs<'a, Id, F, SF>,
}
impl<'a, Id: SceneId, F: Filter, SF: SamplerFactory> NormalRenderer<'a, Id, F, SF> {
    /// 新しい法線レンダラーを作成する。
    pub fn new(args: RendererArgs<'a, Id, F, SF>) -> Self {
        Self { args }
    }
}
impl<'a, Id: SceneId, F: Filter, SF: SamplerFactory> Renderer for NormalRenderer<'a, Id, F, SF> {
    type Color = ColorSrgb;

    fn render(&mut self, x: u32, y: u32) -> Self::Color {
        let RendererArgs {
            spp,
            scene,
            camera,
            sampler_factory,
            ..
        } = self.args.clone();

        let mut sampler = sampler_factory.create_sampler(x, y);

        let mut acc_color = glam::Vec3::ZERO;
        for dimension in 0..spp {
            sampler.start_pixel_sample(dimension);

            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(x, y, uv);

            let intersect = scene.intersect(&rs.ray, f32::MAX);

            let color = match intersect {
                Some(intersect) => match intersect.interaction {
                    SurfaceInteraction { shading_normal, .. } => {
                        shading_normal.to_vec3() * 0.5 + glam::Vec3::splat(0.5)
                    }
                },
                None => glam::Vec3::ZERO,
            };

            acc_color += color * rs.weight;
        }
        let color = acc_color / spp as f32;

        Self::Color::new(color)
    }
}
