//! RGB テクスチャ実装。

use super::{
    config::TextureConfig,
    loader::{ImageData, load_rgb_image},
    sampler::{TextureSample, bilinear_sample_rgb, bilinear_sample_rgb_f32},
};
use glam::Vec2;
use spectrum::Spectrum;
use std::sync::Arc;

/// RGB テクスチャ。
#[derive(Clone)]
pub struct RgbTexture {
    data: ImageData,
    gamut: super::config::SupportedGamut,
}

impl RgbTexture {
    /// テクスチャ設定から RGB テクスチャを読み込む。
    pub fn load(config: TextureConfig) -> Result<Arc<Self>, image::ImageError> {
        let data = load_rgb_image(&config.path)?;
        Ok(Arc::new(Self {
            data,
            gamut: config.gamut,
        }))
    }

    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);

        // samplerからは0.0-1.0の値が返される（8bitの場合は/255.0された値）
        // RGB-to-spectrum内部でinvert_eotf()が行われるため、ここではガンマ補正除去を行わない
        let final_rgb = rgb;

        // 色域変換（現在はsRGBのみサポート、将来的に他の色域も実装可能）
        let texture_rgb = match self.gamut {
            super::config::SupportedGamut::SRgb => final_rgb,
            // 他の色域は将来的に実装
            _ => final_rgb,
        };

        // スペクトラムタイプに応じて変換
        // sRGB値をそのまま渡す（RGB-to-spectrum内部でinvert_eotf()が行われる）
        match spectrum_type {
            super::config::SpectrumType::Albedo => {
                let color = color::ColorSrgb::<color::tone_map::NoneToneMap>::new(
                    texture_rgb[0],
                    texture_rgb[1],
                    texture_rgb[2],
                );
                spectrum::RgbAlbedoSpectrum::<color::ColorSrgb<color::tone_map::NoneToneMap>>::new(
                    color,
                )
            }
            super::config::SpectrumType::Illuminant => {
                let color = color::ColorSrgb::<color::tone_map::NoneToneMap>::new(
                    texture_rgb[0],
                    texture_rgb[1],
                    texture_rgb[2],
                );
                spectrum::RgbIlluminantSpectrum::<color::ColorSrgb<color::tone_map::NoneToneMap>>::new(color)
            }
            super::config::SpectrumType::Unbounded => {
                let color = color::ColorSrgb::<color::tone_map::NoneToneMap>::new(
                    texture_rgb[0],
                    texture_rgb[1],
                    texture_rgb[2],
                );
                spectrum::RgbUnboundedSpectrum::<color::ColorSrgb<color::tone_map::NoneToneMap>>::new(color)
            }
        }
    }
}

impl TextureSample<[f32; 3]> for RgbTexture {
    fn sample(&self, uv: Vec2) -> [f32; 3] {
        match &self.data {
            ImageData::Rgb8(data, width, height) => bilinear_sample_rgb(data, *width, *height, uv),
            ImageData::RgbF32(data, width, height) => {
                bilinear_sample_rgb_f32(data, *width, *height, uv)
            }
            _ => [0.0, 0.0, 0.0], // 不正なデータタイプの場合は黒を返す
        }
    }
}
