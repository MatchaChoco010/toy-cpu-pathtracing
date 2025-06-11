//! 法線を出力するレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map};
use scene::{SceneId, SurfaceInteraction};

use crate::filter::Filter;
use crate::renderer::{Renderer, RendererArgs};
use crate::sampler::Sampler;

/// 法線をレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct NormalRenderer<'a, Id: SceneId, F: Filter> {
    args: RendererArgs<'a, Id, F>,
}
impl<'a, Id: SceneId, F: Filter> NormalRenderer<'a, Id, F> {
    /// 新しい法線レンダラーを作成する。
    pub fn new(args: RendererArgs<'a, Id, F>) -> Self {
        Self { args }
    }
}
impl<'a, Id: SceneId, F: Filter> Renderer for NormalRenderer<'a, Id, F> {
    type Color = ColorSrgb<tone_map::NoneToneMap>;

    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color {
        let RendererArgs {
            resolution,
            spp,
            scene,
            camera,
            seed,
        } = self.args.clone();
        let mut sampler = S::new(spp, resolution, seed);

        let mut acc_color = glam::Vec3::ZERO;
        for sample_index in 0..spp {
            sampler.start_pixel_sample(p, sample_index);

            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(p, uv);

            let intersect = scene.intersect(&rs.ray, f32::MAX);

            let color = match intersect {
                Some(intersect) => {
                    let SurfaceInteraction { shading_normal, .. } = intersect.interaction;
                    shading_normal.to_vec3() * 0.5 + glam::Vec3::splat(0.5)
                }
                None => glam::Vec3::ZERO,
            };

            acc_color += color * rs.weight;
        }
        let color = acc_color / spp as f32;

        Self::Color::from_rgb(color)
    }
}
