//! RGB テクスチャ実装。

use std::sync::Arc;

use glam::Vec2;

use color::{ColorImpl, eotf::Eotf, gamut::ColorGamut, tone_map::NoneToneMap};
use spectrum::Spectrum;

use super::{
    config::TextureConfig,
    loader::{ImageData, load_rgb_image},
    sampler::{bilinear_sample_rgb, bilinear_sample_rgb_f32},
};

/// RGB テクスチャ。
#[derive(Clone)]
pub struct RgbTexture<G: ColorGamut, E: Eotf> {
    data: ImageData,
    _color_type: std::marker::PhantomData<ColorImpl<G, NoneToneMap, E>>,
}

impl<G: ColorGamut, E: Eotf> RgbTexture<G, E> {
    /// テクスチャ設定から RGB テクスチャを読み込む。
    pub fn load(config: TextureConfig) -> Result<Arc<Self>, image::ImageError> {
        let data = load_rgb_image(&config.path)?;
        Ok(Arc::new(Self {
            data,
            _color_type: std::marker::PhantomData,
        }))
    }

    /// UV座標でテクスチャをサンプリングしてRGB値を返す。
    pub fn sample(&self, uv: Vec2) -> [f32; 3] {
        match &self.data {
            ImageData::Rgb8(data, width, height) => bilinear_sample_rgb(data, *width, *height, uv),
            ImageData::RgbF32(data, width, height) => {
                bilinear_sample_rgb_f32(data, *width, *height, uv)
            }
            _ => [0.0, 0.0, 0.0], // 不正なデータタイプの場合は黒を返す
        }
    }
}

// sRGB色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutSrgb, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color = color::ColorSrgb::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorSrgb<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorSrgb<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorSrgb<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}

// Display P3色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutDciP3D65, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color =
            color::ColorDisplayP3::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorDisplayP3<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorDisplayP3<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorDisplayP3<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}

// Adobe RGB色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutAdobeRgb, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color =
            color::ColorAdobeRGB::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorAdobeRGB<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorAdobeRGB<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorAdobeRGB<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}

// Rec. 2020色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutRec2020, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color =
            color::ColorRec2020::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorRec2020<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorRec2020<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorRec2020<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}

// ACES CG色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutAcesCg, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color = color::ColorAcesCg::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorAcesCg<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorAcesCg<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorAcesCg<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}

// ACES 2065-1色域用の実装
impl<E: Eotf> RgbTexture<color::gamut::GamutAces2065_1, E> {
    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        let rgb = self.sample(uv);
        let color =
            color::ColorAces2065_1::<color::tone_map::NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        match spectrum_type {
            super::config::SpectrumType::Albedo => spectrum::RgbAlbedoSpectrum::<
                color::ColorAces2065_1<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Illuminant => spectrum::RgbIlluminantSpectrum::<
                color::ColorAces2065_1<color::tone_map::NoneToneMap>,
            >::new(color),
            super::config::SpectrumType::Unbounded => spectrum::RgbUnboundedSpectrum::<
                color::ColorAces2065_1<color::tone_map::NoneToneMap>,
            >::new(color),
        }
    }
}
