//! 法線を出力するレンダラーを実装するモジュール。

use color::{ColorSrgb, tone_map};
use scene::{Intersection, MaterialSample, SceneId};
use spectrum::SampledWavelengths;

use crate::{
    filter::Filter,
    renderer::{Renderer, RendererArgs},
    sampler::Sampler,
};

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

            // dummyのlambda
            let u = sampler.get_1d();
            let lambda = SampledWavelengths::new_uniform(u);

            let intersect = scene.intersect(&rs.ray, f32::MAX);

            let color = match intersect {
                Some(intersect) => {
                    let Intersection {
                        interaction, wo, ..
                    } = intersect;
                    let render_to_tangent = interaction.shading_transform();
                    let wo = &render_to_tangent * wo;
                    let shading_point = &render_to_tangent * &interaction;

                    if let Some(bsdf) = interaction.material.as_bsdf_material() {
                        let uv = sampler.get_2d();
                        let material_sample = bsdf.sample(uv, &lambda, &wo, &shading_point);
                        let normal = match material_sample {
                            MaterialSample::NonSpecular { normal, .. } => normal,
                            MaterialSample::Specular { normal, .. } => normal,
                        };
                        let normal = render_to_tangent.inverse() * normal;
                        glam::Vec3::new(
                            normal.x() * 0.5 + 0.5,
                            normal.y() * 0.5 + 0.5,
                            normal.z() * 0.5 + 0.5,
                        )
                    } else {
                        let shading_normal = interaction.shading_normal;
                        glam::Vec3::new(
                            shading_normal.x() * 0.5 + 0.5,
                            shading_normal.y() * 0.5 + 0.5,
                            shading_normal.z() * 0.5 + 0.5,
                        )
                    }
                }
                None => glam::Vec3::ZERO,
            };

            acc_color += color * rs.weight;
        }
        let color = acc_color / spp as f32;

        Self::Color::from_rgb(color)
    }
}
