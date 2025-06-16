//! 法線を出力するレンダラーを実装するモジュール。

use color::{ColorSrgb, eotf, gamut::GamutSrgb, tone_map};
use scene::{SceneId, SurfaceInteraction};
use spectrum::{DenselySampledSpectrum, SampledWavelengths, SpectrumTrait, presets};

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

        let mut acc_sample = DenselySampledSpectrum::zero();

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
                        acc_sample.add_sample(&lambda, sample * rs.weight);
                    });
                }
                // None => glam::Vec3::ZERO,
                None => (),
            };
        }
        // sppでoutputを除算
        acc_sample /= spp as f32;

        // d65光源のスペクトルをかけ合わせる。
        let d65 = presets::cie_illum_d6500();
        acc_sample *= d65;

        // outputのスペクトルをXYZに変換する。
        let xyz = acc_sample.to_xyz();
        // XYZをRGBに変換する。
        let rgb = xyz.xyz_to_rgb::<GamutSrgb>();
        // ガンマ補正のEOTFを適用する。

        rgb.apply_eotf::<eotf::GammaSrgb>()
    }
}
