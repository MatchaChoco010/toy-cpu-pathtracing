//! 法線を出力するレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map};
use scene::{SceneId, SurfaceInteraction};

use crate::filter::Filter;
use crate::renderer::{Renderer, RendererArgs};
use crate::sampler::{Sampler, SamplerFactory};

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
    type Color = ColorSrgb<tone_map::NoneToneMap>;

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

        Self::Color::from_rgb(color)
    }
}
