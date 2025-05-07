use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::camera::Camera;
use crate::filter::Filter;
use crate::sampler::{Sampler, SamplerFactory};
use crate::scene::{Interaction, Scene, SceneId};

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
    /// レンダリングを行い、RGBの色を返す。
    fn render(&mut self, x: u32, y: u32) -> [u8; 3];
}

/// レンダラーで書き出す画像の構造体。
pub struct RendererImage<R: Renderer> {
    img: RgbImage,
    renderer: R,
}
impl<R: Renderer> RendererImage<R> {
    /// 新しいレンダラー画像を作成する。
    pub fn new(width: u32, height: u32, renderer: R) -> Self {
        let img = RgbImage::new(width, height);
        Self { img, renderer }
    }

    /// 画像に対してレンダリングを行う。
    pub fn render(&mut self) {
        self.img
            .enumerate_pixels_mut()
            .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
            .par_iter_mut()
            .for_each(|(x, y, pixel)| {
                let mut renderer = self.renderer.clone();
                let color = renderer.render(*x, *y);
                pixel[0] = color[0];
                pixel[1] = color[1];
                pixel[2] = color[2];
            });
    }

    /// 画像を保存する。
    pub fn save(&self, filename: &str) {
        self.img.save(filename).unwrap();
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
    fn render(&mut self, x: u32, y: u32) -> [u8; 3] {
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
                    Interaction::Surface { shading_normal, .. } => {
                        shading_normal.to_vec3() * 0.5 + glam::Vec3::splat(0.5)
                    }
                },
                None => glam::Vec3::ZERO,
            };

            acc_color += color * rs.weight;
        }
        let color = acc_color / spp as f32;

        [
            (color.x * 255.0) as u8,
            (color.y * 255.0) as u8,
            (color.z * 255.0) as u8,
        ]
    }
}
