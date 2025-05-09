//! RGBから光源のスペクトルを生成するモジュール。

use crate::color::{
    Aces2065_1Color, AcesCgColor, AdobeRGBColor, ColorTrait, DisplayP3Color, P3D65Color,
    Rec709Color, Rec2020Color, SrgbColor,
};
use crate::spectrum::{
    DenselySampledSpectrum, Spectrum, presets, rgb_sigmoid_polynomial::RgbSigmoidPolynomial,
};

#[derive(Clone)]
pub struct RgbIlluminantSpectrum<C: ColorTrait + Clone> {
    illuminant: DenselySampledSpectrum,
    scale: f32,
    table: RgbSigmoidPolynomial<C>,
}

macro_rules! impl_rgb_illuminant_spectrum {
    ($color:ty) => {
        impl RgbIlluminantSpectrum<$color> {
            /// 新しいRGB反射率スペクトルを作成する。
            pub fn new(color: $color) -> Self {
                let illuminant = presets::cie_illum_d6500();
                let rgb = color.rgb();
                let max = rgb.max_element();
                let scale = 2.0 * max;
                let scaled_color = <$color>::new(rgb / scale);
                let table = RgbSigmoidPolynomial::from(scaled_color);
                Self {
                    illuminant,
                    scale,
                    table,
                }
            }
        }
        impl Spectrum for RgbIlluminantSpectrum<$color> {
            fn value(&self, lambda: f32) -> f32 {
                self.scale * self.table.value(lambda) * self.illuminant.value(lambda)
            }

            fn max_value(&self) -> f32 {
                self.scale * self.table.max_value() * self.illuminant.max_value()
            }
        }
    };
}

impl_rgb_illuminant_spectrum!(SrgbColor);
impl_rgb_illuminant_spectrum!(DisplayP3Color);
impl_rgb_illuminant_spectrum!(P3D65Color);
impl_rgb_illuminant_spectrum!(AdobeRGBColor);
impl_rgb_illuminant_spectrum!(Rec709Color);
impl_rgb_illuminant_spectrum!(Rec2020Color);
impl_rgb_illuminant_spectrum!(AcesCgColor);
impl_rgb_illuminant_spectrum!(Aces2065_1Color);
