//! MISでBSDFサンプルとNEEを組み合わせたレンダラーを実装するモジュール。

use color::tone_map::ToneMap;

use color::{ColorSrgb, eotf};
use math::{Ray, Transform};
use scene::{BsdfSample, LightIntensity, SceneId};
use spectrum::{DenselySampledSpectrum, SampledSpectrum, SampledWavelengths, SpectrumTrait};

use crate::filter::Filter;
use crate::renderer::{Renderer, RendererArgs};
use crate::sampler::{Sampler, SamplerFactory};

/// pdfのバランスヒューリスティックを計算する関数。
fn balance_heuristic(pdf_a: f32, pdf_b: f32) -> f32 {
    if pdf_a == 0.0 && pdf_b == 0.0 {
        return 0.0;
    }
    pdf_a / (pdf_a + pdf_b)
}

/// MISでBSDFサンプルとNEEを組み合わせたパストレーサーでsRGBレンダリングするためのレンダラー。
#[derive(Clone)]
pub struct SrgbRendererMis<'a, Id: SceneId, F: Filter, SF: SamplerFactory, T: ToneMap> {
    args: RendererArgs<'a, Id, F, SF>,
    tone_map: T,
    exposure: f32,
    max_depth: usize,
}
impl<'a, Id: SceneId, F: Filter, SF: SamplerFactory, T: ToneMap> SrgbRendererMis<'a, Id, F, SF, T> {
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
    for SrgbRendererMis<'a, Id, F, SF, T>
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

            // ライトサンプラーを取得する。
            let light_sampler = scene.light_sampler(&lambda);

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
                // Render座標系からヒットしたシェーディングポイントのTangent座標系に
                // 変換するTransformを取得する。
                let render_to_tangent = Transform::from_shading_normal_tangent(
                    &hit_info.interaction.shading_normal,
                    &hit_info.interaction.tangent,
                );

                // 光源面のTangent座標系での情報を計算する。
                let emissive_point = &render_to_tangent * &hit_info.interaction;

                // ヒットした光源面からの出射方向を計算する。
                let ray_tangent = &render_to_tangent * &ray;
                let wo = -ray_tangent.dir;

                // edfからradianceを取得してoutputに足し合わせる。
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

                // Render座標系からヒットしたシェーディングポイントのTangent座標系に
                // 変換するTransformを取得する。
                let render_to_tangent = Transform::from_shading_normal_tangent(
                    &hit_info.interaction.shading_normal,
                    &hit_info.interaction.tangent,
                );
                let wo = &render_to_tangent * hit_info.wo;

                // Tangent座標系でのシェーディング点の情報を計算する。
                let shading_point = &render_to_tangent * &hit_info.interaction;

                // BSDFのサンプリングを行う。
                let uv = sampler.get_2d();
                let bsdf_sample = bsdf.sample(uv, &lambda, &wo, &shading_point);

                // BSDFのサンプリングの結果によって処理を分岐する。
                match bsdf_sample {
                    // 完全鏡面反射だった場合。
                    Some(BsdfSample::Specular { f, wi }) => {
                        // wiの方向にレイを飛ばす。
                        let wi_render = &render_to_tangent.inverse() * wi;
                        let next_ray = Ray::new(hit_info.interaction.position, wi_render)
                            .move_forward(RAY_FORWARD_EPSILON);
                        let intersect = scene.intersect(&next_ray, f32::MAX);
                        let next_hit_info = match intersect {
                            Some(next_hit_info) => next_hit_info,
                            None => {
                                // TODO: 環境ライトの寄与をoutputに追加する。
                                // ヒットしなかった場合はこのdimensionを終了する。
                                continue 'dimension_loop;
                            }
                        };

                        // 光源面にヒットした場合、radianceを取得してoutputに足し合わせる。
                        if let Some(edf) = &next_hit_info.interaction.material.edf {
                            // Render座標系からnext_hitの位置のTangent座標系に変換するTransformを取得する。
                            let render_to_tangent = Transform::from_shading_normal_tangent(
                                &next_hit_info.interaction.shading_normal,
                                &next_hit_info.interaction.tangent,
                            );

                            // next_hitからの出射方向を計算する。
                            let ray_tangent = &render_to_tangent * &ray;
                            let wo = -ray_tangent.dir;

                            // next_hitのTangent座標系での情報を計算する。
                            let emissive_point = &render_to_tangent * &next_hit_info.interaction;

                            // edfからradianceを取得してoutputに足し合わせる。
                            if let Some(radiance) = edf.radiance(&lambda, emissive_point, wo) {
                                output.add_sample(&lambda, &throughout * &f * radiance);
                            }
                        }

                        // throughoutにサンプルしたBSDFを掛けて更新する。
                        // 完全鏡面なのでcos_thetaやpdfは考慮しない。
                        throughout *= f;

                        // ヒット情報を次に進めてループを進める。
                        hit_info = next_hit_info;
                    }
                    // BSDFのサンプリング結果があった場合。
                    Some(BsdfSample::Bsdf { f, pdf, wi }) => {
                        // ライトをサンプリングする。
                        let u = sampler.get_1d();
                        let light_sample = light_sampler.sample_light(u);

                        // サンプリングしたライトの強さをoutputに足し合わせる。
                        let s = sampler.get_1d();
                        let uv = sampler.get_2d();
                        match scene.calculate_light(
                            light_sample.primitive_index,
                            &hit_info.interaction,
                            &lambda,
                            s,
                            uv,
                        ) {
                            LightIntensity::IrradianceDeltaPointLight(irradiance) => {
                                // シャドウレイを飛ばして可視性を確認する。
                                let distance_vector =
                                    hit_info.interaction.position.vector_to(irradiance.position);
                                let shadow_ray = Ray::new(
                                    hit_info.interaction.position,
                                    distance_vector.normalize(),
                                );
                                let shadow_ray = shadow_ray.move_forward(RAY_FORWARD_EPSILON);
                                let t = distance_vector.length() - 2.0 * RAY_FORWARD_EPSILON;
                                let visible = !scene.intersect_p(&shadow_ray, t);

                                if visible {
                                    // デルタ点光源の場合、irradianceが取得できるので、
                                    // bsdfの値と掛け合わせて点光源の選択確率で割ってoutputに足し合わせる。
                                    let wi = &render_to_tangent * distance_vector.normalize();
                                    let li = irradiance.irradiance;
                                    let f = bsdf.evaluate(&lambda, &wo, &wi, &shading_point);

                                    let sample_contribution =
                                        &throughout * f * li / light_sample.probability;
                                    output.add_sample(&lambda, sample_contribution);
                                }
                            }
                            LightIntensity::IrradianceDeltaDirectionalLight(irradiance) => {
                                // シャドウレイを飛ばして可視性を確認する。
                                let shadow_ray =
                                    Ray::new(hit_info.interaction.position, irradiance.direction);
                                let visible = !scene.intersect_p(&shadow_ray, f32::MAX);

                                if visible {
                                    // デルタ方向光源の場合、irradianceが取得できるので、
                                    // bsdfの値と掛け合わせて点光源の選択確率で割ってoutputに足し合わせる。
                                    let wi = &render_to_tangent * irradiance.direction.normalize();
                                    let li = irradiance.irradiance;
                                    let f = bsdf.evaluate(&lambda, &wo, &wi, &shading_point);

                                    let sample_contribution =
                                        &throughout * f * li / light_sample.probability;
                                    output.add_sample(&lambda, sample_contribution);
                                }
                            }
                            LightIntensity::RadianceAreaLight(radiance) => {
                                // シャドウレイを飛ばして可視性を確認する。
                                let distance_vector = hit_info
                                    .interaction
                                    .position
                                    .vector_to(radiance.interaction.position);
                                let shadow_ray = Ray::new(
                                    hit_info.interaction.position,
                                    distance_vector.normalize(),
                                );
                                let shadow_ray = shadow_ray.move_forward(RAY_FORWARD_EPSILON);
                                let t = distance_vector.length() - 2.0 * RAY_FORWARD_EPSILON;
                                let visible = !scene.intersect_p(&shadow_ray, t);

                                if visible {
                                    // 面積光源の場合、radianceが取得できるので、
                                    // bsdfの値と幾何項Gと掛け合わせて
                                    // pdfとライトの選択確率で割ってoutputに足し合わせる。
                                    let wi = &render_to_tangent * distance_vector.normalize();
                                    let li = &radiance.radiance;
                                    let pdf = radiance.pdf;
                                    let g = radiance.g;
                                    let f = bsdf.evaluate(&lambda, &wo, &wi, &shading_point);

                                    // MISのウエイトを計算する。
                                    let pdf_light_dir = radiance.pdf_dir;
                                    let pdf_bsdf_dir = bsdf.pdf(&lambda, &wo, &wi, &shading_point);
                                    let mis_weight = balance_heuristic(pdf_light_dir, pdf_bsdf_dir);

                                    let sample_contribution =
                                        &throughout * &f * li * g * mis_weight / pdf;
                                    output.add_sample(&lambda, sample_contribution);
                                }
                            }
                        }

                        // wiの方向にレイを飛ばす。
                        let wi_render = &render_to_tangent.inverse() * wi;
                        let next_ray = Ray::new(hit_info.interaction.position, wi_render)
                            .move_forward(RAY_FORWARD_EPSILON);
                        let intersect = scene.intersect(&next_ray, f32::MAX);
                        let next_hit_info = match intersect {
                            Some(next_hit_info) => next_hit_info,
                            None => {
                                // TODO: 環境ライトの寄与をoutputに追加する。
                                // ヒットしなかった場合はこのdimensionを終了する。
                                continue 'dimension_loop;
                            }
                        };

                        // MISのウエイトを計算する。
                        let pdf_bsdf_dir = pdf;
                        let pdf_light_dir = scene.pdf_light_sample(
                            &light_sampler,
                            &hit_info.interaction,
                            &next_hit_info.interaction,
                        );
                        let mis_weight = balance_heuristic(pdf_bsdf_dir, pdf_light_dir);

                        // 次のサンプル方向のcos_thetaを計算する。
                        let cos_theta = wi.y().abs();

                        // throughoutにサンプルしたBSDFとcos_thetaを掛けてpdfで割って更新する。
                        throughout *= f * cos_theta * mis_weight / pdf;

                        // ヒット情報を次に進めてループを進める。
                        hit_info = next_hit_info;
                    }
                    // woについてBSDFが反射を返さない場合。
                    None => {
                        // BSDFの値がサンプリングされない場合は終了する。
                        break 'depth_loop;
                    }
                }

                // ロシアンルーレットで打ち切る。
                let p_russian_roulette = throughout.max_value();
                let u = sampler.get_1d();
                if u < p_russian_roulette {
                    // throughoutをp_russian_rouletteで割る。
                    throughout /= p_russian_roulette;
                } else {
                    break 'depth_loop;
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
