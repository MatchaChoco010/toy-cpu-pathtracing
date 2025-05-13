//! レンダラーの実装を行うモジュール。

use std::path::Path;

use color::tone_map::ToneMap;
use image::{ImageFormat, Rgb, RgbImage};
use rayon::prelude::*;

use color::{Color, ColorSrgb, eotf, tone_map};
use math::{Ray, Transform};
use scene::{BsdfSample, Scene, SceneId, SurfaceInteraction};
use spectrum::{DenselySampledSpectrum, SampledSpectrum, SampledWavelengths, SpectrumTrait};

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

/// sRGBレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct SrgbRenderer<'a, Id: SceneId, F: Filter, SF: SamplerFactory, T: ToneMap> {
    args: RendererArgs<'a, Id, F, SF>,
    tone_map: T,
    exposure: f32,
    max_depth: usize,
}
impl<'a, Id: SceneId, F: Filter, SF: SamplerFactory, T: ToneMap> SrgbRenderer<'a, Id, F, SF, T> {
    /// 新しいsRGBレンダラーを作成する。
    pub fn new(
        args: RendererArgs<'a, Id, F, SF>,
        tone_map: T,
        exposure: f32,
        max_depth: usize,
    ) -> Self {
        Self {
            args,
            tone_map,
            exposure,
            max_depth,
        }
    }
}
impl<'a, Id: SceneId, F: Filter, SF: SamplerFactory, T: ToneMap> Renderer
    for SrgbRenderer<'a, Id, F, SF, T>
{
    type Color = ColorSrgb<T>;

    fn render(&mut self, x: u32, y: u32) -> Self::Color {
        const RAY_FORWARD_EPSILON: f32 = 1e-4;

        let RendererArgs {
            spp,
            scene,
            camera,
            sampler_factory,
            ..
        } = self.args.clone();

        let mut sampler = sampler_factory.create_sampler(x, y);

        let mut output = DenselySampledSpectrum::zero();

        // spp数だけループする。
        'dimension_loop: for dimension in 0..spp {
            sampler.start_pixel_sample(dimension);

            // このdimensionでサンプルする波長をサンプリングする。
            let u = sampler.get_1d();
            let lambda = SampledWavelengths::new_uniform(u);

            // // ライトサンプラーを取得する。
            // let light_sampler = scene.light_sampler(&lambda);

            // パストレーシングのpayloadを初期化する。
            let mut throughout = SampledSpectrum::one();

            // カメラ絵レイをサンプルする。
            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(x, y, uv);
            throughout *= rs.weight;

            // カメラレイをシーンに飛ばして交差を取得する。
            let ray = rs.ray.move_forward(RAY_FORWARD_EPSILON);
            let intersect = scene.intersect(&ray, f32::MAX);
            let mut hit_info = match intersect {
                None => {
                    // ヒットしなかった場合は、このdimensionを終了する。
                    // TODO: 環境ライトの寄与をoutputに追加する。
                    continue 'dimension_loop;
                }
                Some(intersect) => intersect,
            };

            // 光源面にヒットした場合、radianceを取得してoutputに足し合わせる。
            if let Some(edf) = &hit_info.interaction.material.edf {
                let render_to_tangent = Transform::from_shading_normal_tangent(
                    &hit_info.interaction.shading_normal,
                    &hit_info.interaction.tangent,
                );
                let emissive_point = &render_to_tangent * &hit_info.interaction;
                let ray_tangent = &render_to_tangent * &ray;
                let wo = -ray_tangent.dir;
                if let Some(radiance) = edf.radiance(&lambda, emissive_point, -wo) {
                    output.add_sample(&lambda, &throughout * radiance);
                }
            }

            'depth_loop: for _ in 1..=self.max_depth {
                // マテリアルのBSDFを取得する。
                let bsdf = if let Some(bsdf) = &hit_info.interaction.material.bsdf {
                    bsdf
                } else {
                    // BSDFがない場所にヒットした場合は、レイのトレースを終了する。
                    break 'depth_loop;
                };

                // BSDFのサンプリングを行う。
                let render_to_tangent = Transform::from_shading_normal_tangent(
                    &hit_info.interaction.shading_normal,
                    &hit_info.interaction.tangent,
                );
                let ray_tangent = &render_to_tangent * &ray;
                let wo = -ray_tangent.dir;
                let shading_point = &render_to_tangent * &hit_info.interaction;

                let uv = sampler.get_2d();
                let bsdf_sample = bsdf.sample(uv, &lambda, &wo, &shading_point);

                // BSDFのサンプリングの結果によって処理を分岐する。
                match bsdf_sample {
                    // 完全鏡面反射だった場合。
                    Some(BsdfSample::Specular { f, wi }) => {
                        // wiの方向にレイを飛ばす。
                        let ray = Ray::new(shading_point.position, wi);
                        let ray = &render_to_tangent.inverse() * ray;
                        let ray = ray.move_forward(RAY_FORWARD_EPSILON);
                        let intersect = scene.intersect(&ray, f32::MAX);
                        let next_hit_info = match intersect {
                            Some(next_hit_info) => next_hit_info,
                            None => {
                                // TODO: 環境ライトの寄与をoutputに追加する。
                                // ヒットしなかった場合はこのdimensionを終了する。
                                continue 'dimension_loop;
                            }
                        };
                        let render_to_tangent = Transform::from_shading_normal_tangent(
                            &next_hit_info.interaction.shading_normal,
                            &next_hit_info.interaction.tangent,
                        );
                        let ray_tangent = &render_to_tangent * &ray;
                        let wo = -ray_tangent.dir;

                        // 光源面にヒットした場合、radianceを取得してoutputに足し合わせる。
                        if let Some(edf) = &next_hit_info.interaction.material.edf {
                            let emissive_point = &render_to_tangent * &next_hit_info.interaction;
                            if let Some(radiance) = edf.radiance(&lambda, emissive_point, wo) {
                                output.add_sample(&lambda, &throughout * &f * radiance);
                            }
                        }

                        hit_info = next_hit_info;
                        throughout *= f;
                    }
                    // BSDFのサンプリング結果があった場合。
                    Some(BsdfSample::Bsdf { f, pdf, wi }) => {
                        // wiの方向にレイを飛ばす。
                        let ray = Ray::new(shading_point.position, wi);
                        let ray = &render_to_tangent.inverse() * ray;
                        let ray = ray.move_forward(RAY_FORWARD_EPSILON);
                        let intersect = scene.intersect(&ray, f32::MAX);
                        let next_hit_info = match intersect {
                            Some(next_hit_info) => next_hit_info,
                            None => {
                                // TODO: 環境ライトの寄与をoutputに追加する。
                                // ヒットしなかった場合はこのdimensionを終了する。
                                continue 'dimension_loop;
                            }
                        };
                        let render_to_tangent = Transform::from_shading_normal_tangent(
                            &next_hit_info.interaction.shading_normal,
                            &next_hit_info.interaction.tangent,
                        );
                        let ray_tangent = &render_to_tangent * &ray;
                        let wo = -ray_tangent.dir;
                        let cos_theta = wo.y().abs();

                        // 光源面にヒットした場合、radianceを取得してoutputに足し合わせる。
                        if let Some(edf) = &next_hit_info.interaction.material.edf {
                            let emissive_point = &render_to_tangent * &next_hit_info.interaction;
                            if let Some(radiance) = edf.radiance(&lambda, emissive_point, wo) {
                                output.add_sample(
                                    &lambda,
                                    &throughout * &f * cos_theta * radiance / pdf,
                                );
                            }
                        }

                        hit_info = next_hit_info;
                        throughout *= f * cos_theta / pdf;
                    }
                    // woについてBSDFが反射を返さない場合。
                    None => {
                        // BSDFの値がサンプリングされない場合は終了する。
                        break 'depth_loop;
                    }
                }
            }
        }

        // sppでoutputを除算する。
        output /= spp as f32;

        // outputのスペクトルをXYZに変換する。
        let xyz = output.to_xyz();
        // XYZをRGBに変換する。
        let rgb = xyz.xyz_to_rgb();
        // exposureを適用する。
        let rgb = rgb.apply_exposure(self.exposure);
        // ToneMapを適用する。
        let rgb = rgb.apply_tone_map(self.tone_map.clone());
        // ガンマ補正のEOTFを適用する。
        let color = rgb.apply_eotf::<eotf::GammaSrgb>();

        color
    }
}
