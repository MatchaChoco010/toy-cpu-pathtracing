//! 全レンダラーの基底となるベースレンダラーの実装。

use color::ColorSrgb;
use color::eotf;
use color::tone_map::ToneMap;
use math::{Ray, Render, Tangent, Transform};
use scene::{BsdfSample, Intersection, SceneId, SurfaceInteraction};
use spectrum::{DenselySampledSpectrum, SampledSpectrum, SampledWavelengths, SpectrumTrait};

use crate::filter::Filter;
use crate::renderer::{Renderer, RendererArgs, RenderingStrategy};
use crate::sampler::Sampler;

/// BSDFサンプリングの結果を管理する構造体。
pub struct BsdfSamplingResult<Id: SceneId> {
    pub next_hit_info: Option<Intersection<Id, Render>>,
    pub next_emissive_contribution: SampledSpectrum,
    pub throughput_modifier: SampledSpectrum,
}

/// レンダラーの基底となるベースレンダラー実装。
#[derive(Clone)]
pub struct BaseSrgbRenderer<'a, Id: SceneId, F: Filter, T: ToneMap, Strategy: RenderingStrategy> {
    args: RendererArgs<'a, Id, F>,
    tone_map: T,
    exposure: f32,
    max_depth: usize,
    strategy: Strategy,
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap, Strategy: RenderingStrategy>
    BaseSrgbRenderer<'a, Id, F, T, Strategy>
{
    const RAY_FORWARD_EPSILON: f32 = 1e-4;

    /// 新しいベースレンダラーを作成する。
    pub fn new(
        args: RendererArgs<'a, Id, F>,
        tone_map: T,
        exposure: f32,
        max_depth: usize,
        strategy: Strategy,
    ) -> Self {
        Self {
            args,
            tone_map,
            exposure,
            max_depth,
            strategy,
        }
    }

    /// エミッシブ面からのradiance計算を行う。
    fn evaluate_emissive_surface(
        interaction: &SurfaceInteraction<Id, Render>,
        incoming_ray: &Ray<Render>,
        lambda: &SampledWavelengths,
    ) -> Option<SampledSpectrum> {
        let emissive_material = interaction.material.as_emissive_material::<Id>()?;

        // Render座標系からヒットした光源上の点のTangent座標系に変換
        let render_to_tangent = Transform::from_shading_normal_tangent(
            &interaction.shading_normal,
            &interaction.tangent,
        );

        // ヒットした光源面からの出射方向を計算
        let ray_tangent = &render_to_tangent * incoming_ray;
        let wo = -ray_tangent.dir;

        Some(emissive_material.radiance(lambda, wo, &(render_to_tangent * interaction)))
    }

    /// ロシアンルーレットによる経路終了判定を行う。
    fn apply_russian_roulette<S: Sampler>(
        throughout: &mut SampledSpectrum,
        sampler: &mut S,
    ) -> bool {
        let p_russian_roulette = throughout.max_value();
        let u = sampler.get_1d();
        if u < p_russian_roulette {
            *throughout /= p_russian_roulette;
            true // 継続
        } else {
            false // 終了
        }
    }

    /// スペクトルからsRGB色空間への最終変換を行う。
    fn finalize_spectrum_to_color(
        mut output: DenselySampledSpectrum,
        spp: u32,
        tone_map: T,
        exposure: f32,
    ) -> ColorSrgb<T> {
        // sppでoutputを除算
        output /= spp as f32;

        // outputのスペクトルをXYZに変換する。
        let xyz = output.to_xyz();
        // XYZをRGBに変換する。
        let rgb = xyz.xyz_to_rgb();
        // exposureを適用する。
        let rgb = rgb.apply_exposure(exposure);
        // ToneMapを適用する。
        let rgb = rgb.apply_tone_map(tone_map);
        // ガンマ補正のEOTFを適用する。

        rgb.apply_eotf::<eotf::GammaSrgb>()
    }

    /// BSDFサンプリングと次のレイのトレースを行う。
    fn process_bsdf_sampling(
        scene: &scene::Scene<Id>,
        lambda: &SampledWavelengths,
        bsdf_sample: &BsdfSample,
        render_to_tangent: &Transform<Render, Tangent>,
        shading_point: &SurfaceInteraction<Id, Render>,
    ) -> BsdfSamplingResult<Id> {
        match bsdf_sample {
            BsdfSample::Specular { f, wi, normal: _ } => {
                // wiの方向にレイを飛ばす
                let wi_render = &render_to_tangent.inverse() * wi;
                let next_ray = Ray::new(shading_point.position, wi_render)
                    .move_forward(Self::RAY_FORWARD_EPSILON);
                let intersect = scene.intersect(&next_ray, f32::MAX);

                let next_emissive_contribution = if let Some(ref next_hit_info) = intersect {
                    Self::evaluate_emissive_surface(&next_hit_info.interaction, &next_ray, lambda)
                        .map(|radiance| f * radiance)
                        .unwrap_or_else(SampledSpectrum::zero)
                } else {
                    SampledSpectrum::zero()
                };

                BsdfSamplingResult {
                    next_hit_info: intersect,
                    next_emissive_contribution,
                    throughput_modifier: f.clone(),
                }
            }
            BsdfSample::Bsdf { f, pdf, wi, normal } => {
                // wiの方向にレイを飛ばす
                let wi_render = &render_to_tangent.inverse() * wi;
                let next_ray = Ray::new(shading_point.position, wi_render)
                    .move_forward(Self::RAY_FORWARD_EPSILON);
                let intersect = scene.intersect(&next_ray, f32::MAX);

                // cos_thetaを計算（法線マップを考慮）
                let normal_vec = normal.to_vec3().normalize();
                let cos_theta = wi.to_vec3().dot(normal_vec).abs();

                let next_emissive_contribution = if let Some(ref next_hit_info) = intersect {
                    Self::evaluate_emissive_surface(&next_hit_info.interaction, &next_ray, lambda)
                        .map(|radiance| f * &radiance * cos_theta / pdf)
                        .unwrap_or_else(SampledSpectrum::zero)
                } else {
                    SampledSpectrum::zero()
                };

                BsdfSamplingResult {
                    next_hit_info: intersect,
                    next_emissive_contribution,
                    throughput_modifier: f * cos_theta / pdf,
                }
            }
        }
    }
}
impl<'a, Id: SceneId, F: Filter, T: ToneMap, Strategy: RenderingStrategy> Renderer
    for BaseSrgbRenderer<'a, Id, F, T, Strategy>
{
    type Color = ColorSrgb<T>;

    fn render<S: Sampler>(&mut self, p: glam::UVec2) -> Self::Color {
        let RendererArgs {
            resolution,
            spp,
            scene,
            camera,
            seed,
        } = self.args.clone();
        let mut sampler = S::new(spp, resolution, seed);

        let mut output = DenselySampledSpectrum::zero();

        // spp数だけループする
        'sample_loop: for sample_index in 0..spp {
            sampler.start_pixel_sample(p, sample_index);

            let mut hit_info;
            let mut throughout = SampledSpectrum::one();

            // このsample_indexでサンプルする波長をサンプリングする
            let u = sampler.get_1d();
            let lambda = SampledWavelengths::new_uniform(u);

            // カメラレイをサンプル
            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(p, uv);
            throughout *= rs.weight;

            // カメラレイをシーンに飛ばして交差を取得
            let ray = rs.ray.move_forward(Self::RAY_FORWARD_EPSILON);
            hit_info = match scene.intersect(&ray, f32::MAX) {
                Some(intersect) => intersect,
                None => continue 'sample_loop, // ヒットしなかった場合は次のサンプルへ
            };

            // 光源面にヒットした場合、radianceを取得してoutputに足し合わせる
            if let Some(radiance) =
                Self::evaluate_emissive_surface(&hit_info.interaction, &ray, &lambda)
            {
                output.add_sample(&lambda, &throughout * radiance);
            }

            // パストレーシングのメインループ
            'depth_loop: for _ in 1..=self.max_depth {
                // マテリアルのBSDFを取得
                let bsdf = match hit_info.interaction.material.as_bsdf_material::<Id>() {
                    Some(bsdf) => bsdf,
                    None => break 'depth_loop,
                };

                // Render座標系からヒットしたシェーディングポイントのTangent座標系に変換
                let render_to_tangent = Transform::from_shading_normal_tangent(
                    &hit_info.interaction.shading_normal,
                    &hit_info.interaction.tangent,
                );
                let wo = &render_to_tangent * hit_info.wo;

                // Tangent座標系でのシェーディング点の情報を計算
                let shading_point = &render_to_tangent * &hit_info.interaction;

                // BSDFのサンプリングを行う
                let uv = sampler.get_2d();
                let bsdf_sample = match bsdf.sample(uv, &lambda, &wo, &shading_point) {
                    Some(sample) => sample,
                    None => break 'depth_loop,
                };

                // 戦略に基づいてNEE寄与を評価
                if let Some(nee_result) = self.strategy.evaluate_nee(
                    scene,
                    &lambda,
                    &mut sampler,
                    &render_to_tangent,
                    &hit_info,
                    &bsdf_sample,
                ) {
                    // NEE寄与をoutputに追加（throughout、MISウエイト適用）
                    output.add_sample(
                        &lambda,
                        &throughout * &nee_result.contribution * nee_result.mis_weight,
                    );
                }

                // BSDFサンプルの処理と次のレイのトレース
                let bsdf_result = Self::process_bsdf_sampling(
                    scene,
                    &lambda,
                    &bsdf_sample,
                    &render_to_tangent,
                    &hit_info.interaction,
                );

                // 次のヒット情報を取得
                let next_hit_info = match bsdf_result.next_hit_info {
                    Some(next_hit) => next_hit,
                    None => continue 'sample_loop,
                };

                // BSDFサンプルのMISウエイトを計算
                let mis_weight = self.strategy.calculate_bsdf_mis_weight(
                    scene,
                    &lambda,
                    &hit_info,
                    &next_hit_info,
                    &bsdf_sample,
                );

                // 戦略に応じてBSDFサンプリング結果のエミッシブ寄与を追加
                if self.strategy.should_add_bsdf_emissive(&bsdf_sample) {
                    output.add_sample(
                        &lambda,
                        &throughout * bsdf_result.next_emissive_contribution * mis_weight,
                    );
                }

                // throughoutを更新（MISウエイト適用）
                throughout *= bsdf_result.throughput_modifier * mis_weight;

                // 次のヒット情報に進める
                hit_info = next_hit_info;

                // ロシアンルーレットで打ち切る
                if !Self::apply_russian_roulette(&mut throughout, &mut sampler) {
                    break 'depth_loop;
                }
            }
        }

        Self::finalize_spectrum_to_color(output, spp, self.tone_map.clone(), self.exposure)
    }
}
