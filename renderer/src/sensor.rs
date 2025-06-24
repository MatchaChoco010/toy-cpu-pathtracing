//! センサーによる寄与の蓄積を扱うモジュール。

use std::marker::PhantomData;

use color::{ColorImpl, eotf::Eotf, gamut::ColorGamut, tone_map::ToneMap};
use spectrum::{LAMBDA_MIN, N_SPECTRUM_SAMPLES, SampledSpectrum, SampledWavelengths, presets};

// spectrum crateの内部定数を定義
const N_SPECTRUM_DENSELY_SAMPLES: usize = (830.0 - 360.0) as usize;

/// 寄与を蓄積するセンサー構造体。
pub struct Sensor<G: ColorGamut, T: ToneMap, E: Eotf> {
    /// 密にサンプリングされたスペクトル配列（オリジナルと同じ）
    densely_sampled_spectrum: [f32; N_SPECTRUM_DENSELY_SAMPLES],
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
            densely_sampled_spectrum: [0.0; N_SPECTRUM_DENSELY_SAMPLES],
            exposure,
            spp,
            tone_map,
            _gamut: PhantomData,
            _eotf: PhantomData,
        }
    }

    /// スペクトラルサンプルを追加する。
    pub fn add_sample(&mut self, lambda: &SampledWavelengths, s: &SampledSpectrum) {
        let pdf = lambda.pdf();
        let count = if lambda.is_secondary_terminated() {
            1
        } else {
            N_SPECTRUM_SAMPLES
        };

        for index in 0..count {
            let l = lambda.lambda(index);
            let mut i = (l - LAMBDA_MIN).floor() as usize;
            i = if i == N_SPECTRUM_DENSELY_SAMPLES {
                0
            } else {
                i
            };

            self.densely_sampled_spectrum[i] +=
                s.value(index) / pdf.value(index) / N_SPECTRUM_SAMPLES as f32;
        }
    }

    /// 最終的なRGB値を取得する。
    pub fn to_rgb(&self) -> ColorImpl<G, T, E> {
        let mut averaged_spectrum = self.densely_sampled_spectrum;
        for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
            averaged_spectrum[i] /= self.spp as f32;
        }

        let mut xyz = glam::Vec3::ZERO;
        for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
            let lambda = LAMBDA_MIN + i as f32;
            let spectrum_value = averaged_spectrum[i];

            xyz.x += spectrum_value * presets::x().value(lambda);
            xyz.y += spectrum_value * presets::y().value(lambda);
            xyz.z += spectrum_value * presets::z().value(lambda);
        }

        let xyz_color = color::Xyz::from(xyz);
        let rgb_color = xyz_color.xyz_to_rgb::<G>();
        let exposed_color = rgb_color.apply_exposure(self.exposure);
        let tone_mapped = exposed_color.apply_tone_map(self.tone_map.clone());

        tone_mapped.apply_eotf::<E>()
    }
}
