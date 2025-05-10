//! RGBから一般の非有界なスペクトルを生成するモジュール。

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorP3D65, ColorRec709,
    ColorRec2020, ColorSrgb,
};

use crate::rgb_sigmoid_polynomial::RgbSigmoidPolynomial;
use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct RgbUnboundedSpectrum<C: Color> {
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
impl_rgb_unbounded_spectrum!(ColorSrgb);
impl_rgb_unbounded_spectrum!(ColorDisplayP3);
impl_rgb_unbounded_spectrum!(ColorP3D65);
impl_rgb_unbounded_spectrum!(ColorAdobeRGB);
impl_rgb_unbounded_spectrum!(ColorRec709);
impl_rgb_unbounded_spectrum!(ColorRec2020);
impl_rgb_unbounded_spectrum!(ColorAcesCg);
impl_rgb_unbounded_spectrum!(ColorAces2065_1);
