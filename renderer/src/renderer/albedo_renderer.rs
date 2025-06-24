//! 法線を出力するレンダラーを実装するモジュール。

use color::tone_map::NoneToneMap;
use color::{ColorSrgb, eotf, gamut::GamutSrgb, tone_map};
use scene::{SceneId, SurfaceInteraction};
use spectrum::{SampledWavelengths, presets};

use crate::sensor::Sensor;
use crate::{
    filter::Filter,
    renderer::{Renderer, RendererArgs},
    sampler::Sampler,
};

///  アルベドをレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct AlbedoRenderer<'a, Id: SceneId, F: Filter> {
    args: RendererArgs<'a, Id, F>,
}
impl<'a, Id: SceneId, F: Filter> AlbedoRenderer<'a, Id, F> {
    /// 新しいアルベドレンダラーを作成する。
    pub fn new(args: RendererArgs<'a, Id, F>) -> Self {
        Self { args }
    }
}
impl<'a, Id: SceneId, F: Filter> Renderer for AlbedoRenderer<'a, Id, F> {
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

        let mut sensor =
            Sensor::<GamutSrgb, NoneToneMap, eotf::GammaSrgb>::new(spp, 1.0, NoneToneMap);

        for sample_index in 0..spp {
            sampler.start_pixel_sample(p, sample_index);

            let u = sampler.get_1d();
            let lambda = SampledWavelengths::new_uniform(u);

            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(p, uv);

            let intersect = scene.intersect(&rs.ray, f32::MAX);

            match intersect {
                Some(intersect) => {
                    let SurfaceInteraction { uv, material, .. } = intersect.interaction;
                    material.as_bsdf_material().map(|s| {
                        let sample = s.sample_albedo_spectrum(uv, &lambda);
                        let sample = (sample * rs.weight)
                            .multiply_spectrum(&lambda, &presets::cie_illum_d6500());
                        sensor.add_sample(&lambda, &sample);
                    });
                }
                // None => glam::Vec3::ZERO,
                None => (),
            };
        }

        sensor.to_rgb()
    }
}
