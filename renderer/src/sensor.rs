//! センサーによる寄与の蓄積を扱うモジュール。

use std::marker::PhantomData;

use color::{
    eotf::Eotf,
    gamut::ColorGamut,
    tone_map::ToneMap,
    ColorImpl,
};
use spectrum::{presets, SampledSpectrum, SampledWavelengths, LAMBDA_MIN, N_SPECTRUM_SAMPLES};

// spectrum crateの内部定数を定義
const N_SPECTRUM_DENSELY_SAMPLES: usize = (830.0 - 360.0) as usize;

/// 寄与を蓄積するセンサー構造体。
pub struct Sensor<G: ColorGamut, T: ToneMap, E: Eotf> {
    /// 密にサンプリングされたスペクトル配列（XYZ値として蓄積）
    densely_sampled_xyz: [glam::Vec3; N_SPECTRUM_DENSELY_SAMPLES],
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
            densely_sampled_xyz: [glam::Vec3::ZERO; N_SPECTRUM_DENSELY_SAMPLES],
            exposure,
            spp,
            tone_map,
            _gamut: PhantomData,
            _eotf: PhantomData,
        }
    }

    /// スペクトラルサンプルを追加する。
    pub fn add_sample(&mut self, lambda: &SampledWavelengths, s: &SampledSpectrum) {
        // 元のDenselySampledSpectrum::add_sampleと同じロジックに従う
        let pdf = lambda.pdf();
        let count = if lambda.is_secondary_terminated() {
            1
        } else {
            N_SPECTRUM_SAMPLES
        };
        
        for index in 0..count {
            let l = lambda.lambda(index);
            // 密にサンプリングされたインデックスにマッピング（元の実装と同じ）
            let mut i = (l - LAMBDA_MIN).floor() as usize;
            // 境界チェック（元の実装と同じ）
            i = if i == N_SPECTRUM_DENSELY_SAMPLES {
                0
            } else {
                i
            };
            
            // 正規化された寄与値を計算（元の実装と同じ）
            let normalized_contribution = s.value(index) / pdf.value(index) / N_SPECTRUM_SAMPLES as f32;
            
            // その波長でのcolor matching functionの値を取得
            let lambda_for_cmf = LAMBDA_MIN + i as f32;
            let x_val = presets::x().value(lambda_for_cmf);
            let y_val = presets::y().value(lambda_for_cmf);
            let z_val = presets::z().value(lambda_for_cmf);
            
            // XYZ値をその波長のインデックスに蓄積
            self.densely_sampled_xyz[i].x += normalized_contribution * x_val;
            self.densely_sampled_xyz[i].y += normalized_contribution * y_val;
            self.densely_sampled_xyz[i].z += normalized_contribution * z_val;
        }
    }

    /// 最終的なRGB値を取得する。
    pub fn to_rgb(&self) -> ColorImpl<G, T, E> {
        // オリジナルのinner_productと同じように、全波長のXYZ値を合計
        let mut total_xyz = glam::Vec3::ZERO;
        for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
            total_xyz += self.densely_sampled_xyz[i];
        }
        
        // sppで除算
        total_xyz /= self.spp as f32;
        
        // XYZからRGBに変換
        let xyz_color = color::Xyz::from(total_xyz);
        let rgb_color = xyz_color.xyz_to_rgb::<G>();
        
        // exposureを適用（オリジナルのfinalize_spectrum_to_colorと同じタイミング）
        let exposed_color = rgb_color.apply_exposure(self.exposure);
        
        // トーンマップを適用
        let tone_mapped = exposed_color.apply_tone_map(self.tone_map.clone());
        
        // EOTFを適用
        tone_mapped.apply_eotf::<E>()
    }
}