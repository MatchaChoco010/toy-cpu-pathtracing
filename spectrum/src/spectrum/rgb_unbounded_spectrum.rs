//! RGBから一般の非有界なスペクトルを生成するモジュール。

use color::{
    Aces2065_1Color, AcesCgColor, AdobeRGBColor, ColorTrait, DisplayP3Color, P3D65Color,
    Rec709Color, Rec2020Color, SrgbColor,
};

use crate::rgb_sigmoid_polynomial::RgbSigmoidPolynomial;
use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct RgbUnboundedSpectrum<C: ColorTrait> {
    scale: f32,
    table: RgbSigmoidPolynomial<C>,
}

macro_rules! impl_rgb_unbounded_spectrum {
    ($color:ty) => {
        impl RgbUnboundedSpectrum<$color> {
            /// 新しいRGB反射率スペクトルを作成する。
            pub fn new(color: $color) -> Self {
                let rgb = color.rgb();
                let max = rgb.max_element();
                let scale = 2.0 * max;
                let scaled_color = <$color>::new(rgb / scale);
                let table = RgbSigmoidPolynomial::from(scaled_color);
                Self { scale, table }
            }
        }
        impl Spectrum for RgbUnboundedSpectrum<$color> {
            fn value(&self, lambda: f32) -> f32 {
                self.scale * self.table.value(lambda)
            }

            fn max_value(&self) -> f32 {
                self.scale * self.table.max_value()
            }
        }
    };
}
impl_rgb_unbounded_spectrum!(SrgbColor);
impl_rgb_unbounded_spectrum!(DisplayP3Color);
impl_rgb_unbounded_spectrum!(P3D65Color);
impl_rgb_unbounded_spectrum!(AdobeRGBColor);
impl_rgb_unbounded_spectrum!(Rec709Color);
impl_rgb_unbounded_spectrum!(Rec2020Color);
impl_rgb_unbounded_spectrum!(AcesCgColor);
impl_rgb_unbounded_spectrum!(Aces2065_1Color);
