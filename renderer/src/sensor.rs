//! センサーによる寄与の蓄積を扱うモジュール。

use std::marker::PhantomData;

use color::{ColorImpl, eotf::Eotf, gamut::ColorGamut, tone_map::ToneMap};
use spectrum::{LAMBDA_MIN, N_SPECTRUM_SAMPLES, SampledSpectrum, SampledWavelengths, presets};

// spectrum crateの内部定数を定義
const N_SPECTRUM_DENSELY_SAMPLES: usize = (830.0 - 360.0) as usize;

/// 寄与を蓄積するセンサー構造体。
pub struct Sensor<G: ColorGamut, T: ToneMap, E: Eotf> {
    /// 露光量適用済みのRGB値を蓄積
    accumulated_rgb: glam::Vec3,
    /// 露光量。
    exposure: f32,
    /// サンプル数。
    spp: u32,
    /// トーンマップの設定を保持するためのマーカー。
    tone_map: T,
    /// ColorGamutの設定を保持するためのマーカー。
    _gamut: PhantomData<G>,
    /// EOTFの設定を保持するためのマーカー。
    _eotf: PhantomData<E>,
}

impl<G: ColorGamut, T: ToneMap, E: Eotf> Sensor<G, T, E> {
    /// 新しいセンサーを作成する。
    pub fn new(spp: u32, exposure: f32, tone_map: T) -> Self {
        Self {
            accumulated_rgb: glam::Vec3::ZERO,
            exposure,
            spp,
            tone_map,
            _gamut: PhantomData,
            _eotf: PhantomData,
        }
    }

    /// スペクトラルサンプルを追加する。
    pub fn add_sample(&mut self, lambda: &SampledWavelengths, s: &SampledSpectrum) {
        s.eprint_nan_inf("Sensor::add_sample");

        let pdf = lambda.pdf();
        let count = if lambda.is_secondary_terminated() {
            1
        } else {
            N_SPECTRUM_SAMPLES
        };

        let mut xyz = glam::Vec3::ZERO;

        for index in 0..count {
            let l = lambda.lambda(index);
            let mut i = (l - LAMBDA_MIN).floor() as usize;
            i = if i == N_SPECTRUM_DENSELY_SAMPLES {
                0
            } else {
                i
            };

            let normalized_contribution =
                s.value(index) / pdf.value(index) / N_SPECTRUM_SAMPLES as f32;

            let lambda_for_cmf = LAMBDA_MIN + i as f32;
            xyz.x += normalized_contribution * presets::x().value(lambda_for_cmf);
            xyz.y += normalized_contribution * presets::y().value(lambda_for_cmf);
            xyz.z += normalized_contribution * presets::z().value(lambda_for_cmf);
        }

        // 直接ガンマ行列でXYZからRGBに変換（クリッピングなし）
        let gamut = G::new();
        let rgb = gamut.xyz_to_rgb() * xyz;

        // exposureを適用してからRGB値を蓄積
        let exposed_rgb = rgb * self.exposure;
        self.accumulated_rgb += exposed_rgb;
    }

    /// 最終的なRGB値を取得する。
    pub fn to_rgb(&self) -> ColorImpl<G, T, E> {
        let averaged_rgb = self.accumulated_rgb / self.spp as f32;
        let clipped_rgb = averaged_rgb.max(glam::Vec3::ZERO);
        let rgb_color = ColorImpl::from_rgb(clipped_rgb);
        let tone_mapped = rgb_color.apply_tone_map(self.tone_map.clone());

        tone_mapped.apply_eotf::<E>()
    }
}
