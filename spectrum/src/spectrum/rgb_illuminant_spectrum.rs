//! RGBから光源のスペクトルを生成するモジュール。

use std::sync::Arc;

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorP3D65, ColorRec709,
    ColorRec2020, ColorSrgb, ColorSrgbLinear, tone_map::NoneToneMap,
};

use crate::{
    presets,
    rgb_sigmoid_polynomial::RgbSigmoidPolynomial,
    spectrum::{Spectrum, SpectrumTrait},
};

#[derive(Clone)]
pub struct RgbIlluminantSpectrum<C: Color + Clone> {
    illuminant: Spectrum,
    scale: f32,
    table: RgbSigmoidPolynomial<C>,
}

macro_rules! impl_rgb_illuminant_spectrum {
    ($color:ty) => {
        impl RgbIlluminantSpectrum<$color> {
            /// 新しいRGB反射率スペクトルを作成する。
            pub fn new(color: $color) -> Spectrum {
                let illuminant = presets::cie_illum_d6500();
                let rgb = color.rgb();
                let max = rgb.max_element();
                let scale = 2.0 * max;
                let scaled_color = <$color>::from_rgb(rgb / scale);
                let table = RgbSigmoidPolynomial::from(scaled_color);
                Arc::new(Self {
                    illuminant,
                    scale,
                    table,
                })
            }
        }
        impl SpectrumTrait for RgbIlluminantSpectrum<$color> {
            fn value(&self, lambda: f32) -> f32 {
                self.scale * self.table.value(lambda) * self.illuminant.value(lambda)
            }

            fn max_value(&self) -> f32 {
                self.scale * self.table.max_value() * self.illuminant.max_value()
            }
        }
    };
}

impl_rgb_illuminant_spectrum!(ColorSrgb<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorSrgbLinear<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorDisplayP3<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorP3D65<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorAdobeRGB<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorRec709<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorRec2020<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorAcesCg<NoneToneMap>);
impl_rgb_illuminant_spectrum!(ColorAces2065_1<NoneToneMap>);
