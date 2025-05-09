//! RGB色から反射率スペクトルを生成するモジュール。

use color::{
    Aces2065_1Color, AcesCgColor, AdobeRGBColor, ColorTrait, DisplayP3Color, P3D65Color,
    Rec709Color, Rec2020Color, SrgbColor,
};

use crate::rgb_sigmoid_polynomial::RgbSigmoidPolynomial;
use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct RgbAlbedoSpectrum<C: ColorTrait + Clone> {
    table: RgbSigmoidPolynomial<C>,
}

macro_rules! impl_rgb_albedo_spectrum {
    ($color:ty) => {
        impl RgbAlbedoSpectrum<$color> {
            /// 新しいRGB反射率スペクトルを作成する。
            pub fn new(color: $color) -> Self {
                let table = RgbSigmoidPolynomial::from(color);
                Self { table }
            }
        }
        impl Spectrum for RgbAlbedoSpectrum<$color> {
            fn value(&self, lambda: f32) -> f32 {
                self.table.value(lambda)
            }

            fn max_value(&self) -> f32 {
                self.table.max_value()
            }
        }
    };
}

impl_rgb_albedo_spectrum!(SrgbColor);
impl_rgb_albedo_spectrum!(DisplayP3Color);
impl_rgb_albedo_spectrum!(P3D65Color);
impl_rgb_albedo_spectrum!(AdobeRGBColor);
impl_rgb_albedo_spectrum!(Rec709Color);
impl_rgb_albedo_spectrum!(Rec2020Color);
impl_rgb_albedo_spectrum!(AcesCgColor);
impl_rgb_albedo_spectrum!(Aces2065_1Color);
