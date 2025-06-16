//! RGB テクスチャ実装。

use std::path::Path;
use std::sync::Arc;

use glam::Vec2;

use color::{ColorImpl, eotf::Eotf, gamut::ColorGamut, tone_map::NoneToneMap};
use spectrum::Spectrum;

use super::{
    loader::{ImageData, load_rgb_image},
    sampler::{bilinear_sample_rgb, bilinear_sample_rgb_f32},
};

/// 型付きRGB テクスチャ。
#[derive(Clone)]
pub struct TypedRgbTexture<G: ColorGamut, E: Eotf> {
    data: ImageData,
    _color_type: std::marker::PhantomData<ColorImpl<G, NoneToneMap, E>>,
}

impl<G: ColorGamut, E: Eotf> TypedRgbTexture<G, E> {
    /// テクスチャ設定から RGB テクスチャを読み込む。
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let data = load_rgb_image(path.as_ref())?;
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutSrgb, E> {
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutDciP3D65, E> {
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutAdobeRgb, E> {
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutRec2020, E> {
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutAcesCg, E> {
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
impl<E: Eotf> TypedRgbTexture<color::gamut::GamutAces2065_1, E> {
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

/// 型消去されたRGB テクスチャ。
/// 色域情報を実行時に保持し、SpectrumParameterで使用される。
#[derive(Clone)]
pub enum RgbTexture {
    Srgb(Arc<TypedRgbTexture<color::gamut::GamutSrgb, color::eotf::GammaSrgb>>),
    DisplayP3(Arc<TypedRgbTexture<color::gamut::GamutDciP3D65, color::eotf::GammaSrgb>>),
    AdobeRgb(Arc<TypedRgbTexture<color::gamut::GamutAdobeRgb, color::eotf::GammaSrgb>>),
    Rec2020(Arc<TypedRgbTexture<color::gamut::GamutRec2020, color::eotf::GammaSrgb>>),
    AcesCg(Arc<TypedRgbTexture<color::gamut::GamutAcesCg, color::eotf::GammaSrgb>>),
    Aces2065_1(Arc<TypedRgbTexture<color::gamut::GamutAces2065_1, color::eotf::Linear>>),
}

impl RgbTexture {
    /// sRGB色域でテクスチャを読み込む。
    pub fn load_srgb(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutSrgb, color::eotf::GammaSrgb>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::Srgb(texture)))
    }

    /// Display P3色域でテクスチャを読み込む。
    pub fn load_display_p3(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutDciP3D65, color::eotf::GammaSrgb>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::DisplayP3(texture)))
    }

    /// Adobe RGB色域でテクスチャを読み込む。
    pub fn load_adobe_rgb(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutAdobeRgb, color::eotf::GammaSrgb>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::AdobeRgb(texture)))
    }

    /// Rec. 2020色域でテクスチャを読み込む。
    pub fn load_rec2020(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutRec2020, color::eotf::GammaSrgb>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::Rec2020(texture)))
    }

    /// ACES CG色域でテクスチャを読み込む。
    pub fn load_aces_cg(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutAcesCg, color::eotf::GammaSrgb>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::AcesCg(texture)))
    }

    /// ACES 2065-1色域でテクスチャを読み込む。
    pub fn load_aces_2065_1(path: impl AsRef<Path>) -> Result<Arc<Self>, image::ImageError> {
        let texture = TypedRgbTexture::<color::gamut::GamutAces2065_1, color::eotf::Linear>::load(
            path.as_ref(),
        )?;
        Ok(Arc::new(RgbTexture::Aces2065_1(texture)))
    }

    /// UV座標でRGB値をサンプリングする。
    pub fn sample(&self, uv: Vec2) -> [f32; 3] {
        match self {
            RgbTexture::Srgb(texture) => texture.sample(uv),
            RgbTexture::DisplayP3(texture) => texture.sample(uv),
            RgbTexture::AdobeRgb(texture) => texture.sample(uv),
            RgbTexture::Rec2020(texture) => texture.sample(uv),
            RgbTexture::AcesCg(texture) => texture.sample(uv),
            RgbTexture::Aces2065_1(texture) => texture.sample(uv),
        }
    }

    /// RGB値をスペクトラムに変換する。
    pub fn sample_spectrum(
        &self,
        uv: Vec2,
        spectrum_type: super::config::SpectrumType,
    ) -> Spectrum {
        match self {
            RgbTexture::Srgb(texture) => texture.sample_spectrum(uv, spectrum_type),
            RgbTexture::DisplayP3(texture) => texture.sample_spectrum(uv, spectrum_type),
            RgbTexture::AdobeRgb(texture) => texture.sample_spectrum(uv, spectrum_type),
            RgbTexture::Rec2020(texture) => texture.sample_spectrum(uv, spectrum_type),
            RgbTexture::AcesCg(texture) => texture.sample_spectrum(uv, spectrum_type),
            RgbTexture::Aces2065_1(texture) => texture.sample_spectrum(uv, spectrum_type),
        }
    }
}
