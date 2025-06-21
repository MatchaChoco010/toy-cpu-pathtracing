//! 全レンダラーの基底となるベースレンダラーの実装。

use color::{ColorSrgb, eotf, tone_map::ToneMap};
use math::{Ray, Render, Transform, VertexNormalTangent};
use scene::{Intersection, MaterialSample, SceneId, SurfaceInteraction};
use spectrum::{DenselySampledSpectrum, SampledSpectrum, SampledWavelengths, SpectrumTrait};

use crate::{
    filter::Filter,
    renderer::{Renderer, RendererArgs, RenderingStrategy},
    sampler::Sampler,
};

/// BSDFサンプリングの結果を管理する構造体。
pub struct BsdfSamplingResult<Id: SceneId> {
    pub next_hit_info: Intersection<Id, Render>,
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
    const RAY_FORWARD_EPSILON: f32 = 1e-5;

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
        interaction: &SurfaceInteraction<Render>,
        incoming_ray: &Ray<Render>,
        lambda: &SampledWavelengths,
    ) -> Option<SampledSpectrum> {
        let emissive_material = interaction.material.as_emissive_material()?;

        // Render座標系からヒットした光源上の点のVertexNormalTangent座標系に変換
        let render_to_tangent = interaction.shading_transform();

        // ヒットした光源面からの出射方向を計算
        let ray_tangent = &render_to_tangent * incoming_ray;
        let wo = -ray_tangent.dir;

        let emission_radiance =
            emissive_material.radiance(lambda, wo, &(render_to_tangent * interaction));

        // PT（方向積分）では面積光源の放射輝度をそのまま使用
        Some(emission_radiance)
    }

    /// ロシアンルーレットによる経路終了判定を行う。
    fn apply_russian_roulette<S: Sampler>(
        throughout: &mut SampledSpectrum,
        sampler: &mut S,
    ) -> bool {
        let p_russian_roulette = throughout.max_value();
        if p_russian_roulette >= 1.0 {
            // ロシアンルーレットの確率が1.0以上の場合は常に継続
            return true;
        }
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
        material_sample: &MaterialSample,
        render_to_tangent: &Transform<Render, VertexNormalTangent>,
        shading_point: &SurfaceInteraction<Render>,
    ) -> Option<BsdfSamplingResult<Id>> {
        if !material_sample.is_sampled() {
            return None;
        }

        let cos_theta = material_sample.normal.dot(material_sample.wi).abs();
        let sample_wi = material_sample.wi;
        let f_value = &material_sample.f;
        let throughput_factor = cos_theta / material_sample.pdf;

        // wiの方向にレイを飛ばす
        let wi_render = &render_to_tangent.inverse() * &sample_wi;
        let offset_dir: &math::Vector3<_> = shading_point.normal.as_ref();
        let sign = if shading_point.normal.dot(wi_render) < 0.0 {
            -1.0
        } else {
            1.0
        };
        let origin = shading_point
            .position
            .translate(sign * offset_dir * Self::RAY_FORWARD_EPSILON);
        let next_ray = Ray::new(origin, wi_render).move_forward(Self::RAY_FORWARD_EPSILON);
        let intersect = scene.intersect(&next_ray, f32::MAX);

        intersect.map(|next_hit_info| {
            let next_emissive_contribution = if let Some(radiance) =
                Self::evaluate_emissive_surface(&next_hit_info.interaction, &next_ray, lambda)
            {
                f_value * &radiance * &throughput_factor
            } else {
                SampledSpectrum::zero()
            };

            BsdfSamplingResult {
                next_hit_info,
                next_emissive_contribution,
                throughput_modifier: f_value * &throughput_factor,
            }
        })
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
            let mut sample_contribution = SampledSpectrum::zero();

            // このsample_indexでサンプルする波長をサンプリングする
            let u = sampler.get_1d();
            let mut lambda = SampledWavelengths::new_uniform(u);

            // カメラレイをサンプル
            let uv = sampler.get_2d_pixel();
            let rs = camera.sample_ray(p, uv);
            throughout *= rs.weight;

            // カメラレイをシーンに飛ばして交差を取得
            let ray = rs.ray.move_forward(Self::RAY_FORWARD_EPSILON);
            hit_info = match scene.intersect(&ray, f32::MAX) {
                Some(intersect) => intersect,
                None => {
                    // ヒットしなかった場合はsample_contribution = 0のまま終了
                    output.add_sample(&lambda, sample_contribution);
                    continue 'sample_loop;
                }
            };

            // 光源面にヒットした場合、radianceを一時変数に蓄積
            if let Some(radiance) =
                Self::evaluate_emissive_surface(&hit_info.interaction, &ray, &lambda)
            {
                sample_contribution += &throughout * radiance;
            }

            // パストレーシングのメインループ
            'depth_loop: for _ in 1..=self.max_depth {
                // マテリアルのBSDFを取得
                let bsdf_material = match hit_info.interaction.material.as_bsdf_material() {
                    Some(bsdf_material) => bsdf_material,
                    None => break 'depth_loop,
                };

                // Render座標系からヒットしたシェーディングポイントのVertexNormalTangent座標系に変換
                let render_to_tangent = hit_info.interaction.shading_transform();
                let wo = &render_to_tangent * hit_info.wo;

                // VertexNormalTangent座標系でのシェーディング点の情報を計算
                let shading_point = &render_to_tangent * &hit_info.interaction;

                // マテリアルのサンプリングを行う
                let uv = sampler.get_2d();
                let material_sample = bsdf_material.sample(uv, &mut lambda, &wo, &shading_point);

                // サンプルしたマテリアルが非Specularな場合、NEEを評価する
                if material_sample.is_non_specular() {
                    self.strategy.evaluate_nee(
                        scene,
                        &lambda,
                        &mut sampler,
                        &render_to_tangent,
                        &hit_info,
                        &mut sample_contribution,
                        &mut throughout,
                    );
                }

                // BSDFサンプルの処理と次のレイのトレース
                let bsdf_result = Self::process_bsdf_sampling(
                    scene,
                    &lambda,
                    &material_sample,
                    &render_to_tangent,
                    &hit_info.interaction,
                );

                // BSDFサンプリング失敗の場合は深度ループ終了
                let Some(bsdf_result) = bsdf_result else {
                    break 'depth_loop;
                };

                // strategyに応じたBSDFサンプルの光源ヒット時の寄与計算
                self.strategy.calculate_bsdf_contribution(
                    &material_sample,
                    &bsdf_result,
                    scene,
                    &lambda,
                    &hit_info,
                    &mut sample_contribution,
                    &mut throughout,
                );

                // 次のヒット情報に進める
                hit_info = bsdf_result.next_hit_info;

                // ロシアンルーレットで打ち切る
                if !Self::apply_russian_roulette(&mut throughout, &mut sampler) {
                    break 'depth_loop;
                }
            }

            // 蓄積した寄与をoutputに追加
            output.add_sample(&lambda, sample_contribution);
        }

        Self::finalize_spectrum_to_color(output, spp, self.tone_map.clone(), self.exposure)
    }
}
