//! センサーによる寄与の蓄積を扱うモジュール。

use std::marker::PhantomData;

use color::{
    eotf::{Eotf, Linear},
    gamut::ColorGamut,
    tone_map::{NoneToneMap, ToneMap},
    Color, ColorImpl,
};
use spectrum::{presets, SampledSpectrum, SampledWavelengths, N_SPECTRUM_SAMPLES};

/// 寄与を蓄積するセンサー構造体。
pub struct Sensor<G: ColorGamut, T: ToneMap, E: Eotf> {
    /// 蓄積値の色。
    color: ColorImpl<G, NoneToneMap, Linear>,
    /// 露光量。
    exposure: f32,
    /// サンプル数。
    spp: u32,
    /// トーンマップの設定を保持するためのマーカー。
    tone_map: T,
    /// EOTFの設定を保持するためのマーカー。
    _eotf: PhantomData<E>,
}

impl<G: ColorGamut, T: ToneMap, E: Eotf> Sensor<G, T, E> {
    /// 新しいセンサーを作成する。
    pub fn new(spp: u32, exposure: f32, tone_map: T) -> Self {
        Self {
            color: ColorImpl::from_rgb(glam::Vec3::ZERO),
            exposure,
            spp,
            tone_map,
            _eotf: PhantomData,
        }
    }

    /// スペクトラルサンプルを追加する。
    pub fn add_sample(&mut self, lambda: &SampledWavelengths, s: &SampledSpectrum) {
        // XYZを計算する
        let mut xyz = glam::Vec3::ZERO;
        
        let pdf = lambda.pdf();
        
        // 最初の波長以外が終了している場合は、最初の波長の値のみを使用
        if lambda.is_secondary_terminated() {
            let lambda_val = lambda.lambda(0);
            let s_val = s.value(0) / pdf.value(0) / N_SPECTRUM_SAMPLES as f32;
            
            let x_val = presets::x().value(lambda_val);
            let y_val = presets::y().value(lambda_val);
            let z_val = presets::z().value(lambda_val);
            
            xyz.x += s_val * x_val * self.exposure;
            xyz.y += s_val * y_val * self.exposure;
            xyz.z += s_val * z_val * self.exposure;
        } else {
            // 全波長サンプルを使用
            for i in 0..N_SPECTRUM_SAMPLES {
                let lambda_val = lambda.lambda(i);
                let s_val = s.value(i) / pdf.value(i) / N_SPECTRUM_SAMPLES as f32;
                
                let x_val = presets::x().value(lambda_val);
                let y_val = presets::y().value(lambda_val);
                let z_val = presets::z().value(lambda_val);
                
                xyz.x += s_val * x_val * self.exposure;
                xyz.y += s_val * y_val * self.exposure;
                xyz.z += s_val * z_val * self.exposure;
            }
        }
        
        // XYZをRGBに変換してクリップ
        let xyz_color = color::Xyz::from(xyz);
        let rgb_color = xyz_color.xyz_to_rgb::<G>();
        let clipped_rgb_vec = rgb_color.rgb().max(glam::Vec3::ZERO);
        let clipped_rgb = ColorImpl::from_rgb(clipped_rgb_vec);
        
        // 蓄積
        self.color.add_sample(&clipped_rgb);
    }

    /// 最終的なRGB値を取得する。
    pub fn to_rgb(&self) -> ColorImpl<G, T, E> {
        // sppで除算
        let averaged_color = ColorImpl::from_rgb(self.color.rgb() / self.spp as f32);
        
        // トーンマップを適用
        let tone_mapped = averaged_color.apply_tone_map(self.tone_map.clone());
        
        // EOTFを適用
        tone_mapped.apply_eotf::<E>()
    }
}