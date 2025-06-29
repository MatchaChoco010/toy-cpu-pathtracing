//! RGB色から反射率スペクトルを生成するモジュール。

use std::sync::Arc;

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorP3D65, ColorRec709,
    ColorRec2020, ColorSrgb, ColorSrgbLinear, tone_map::NoneToneMap,
};

use crate::{
    rgb_sigmoid_polynomial::RgbSigmoidPolynomial,
    spectrum::{Spectrum, SpectrumTrait},
};

#[derive(Clone)]
pub struct RgbAlbedoSpectrum<C: Color + Clone> {
    table: RgbSigmoidPolynomial<C>,
}

macro_rules! impl_rgb_albedo_spectrum {
    ($color:ty) => {
        impl RgbAlbedoSpectrum<$color> {
            /// 新しいRGB反射率スペクトルを作成する。
            pub fn new(color: $color) -> Spectrum {
                let table = RgbSigmoidPolynomial::from(color);
                Arc::new(Self { table })
            }
        }
        impl SpectrumTrait for RgbAlbedoSpectrum<$color> {
            fn value(&self, lambda: f32) -> f32 {
                self.table.value(lambda)
            }

            fn max_value(&self) -> f32 {
                self.table.max_value()
            }
        }
    };
}

impl_rgb_albedo_spectrum!(ColorSrgb<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorSrgbLinear<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorDisplayP3<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorP3D65<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorAdobeRGB<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorRec709<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorRec2020<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorAcesCg<NoneToneMap>);
impl_rgb_albedo_spectrum!(ColorAces2065_1<NoneToneMap>);
