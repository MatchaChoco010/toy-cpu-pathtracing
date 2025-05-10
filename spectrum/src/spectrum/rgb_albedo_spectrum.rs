//! RGB色から反射率スペクトルを生成するモジュール。

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorP3D65, ColorRec709,
    ColorRec2020, ColorSrgb,
};

use crate::rgb_sigmoid_polynomial::RgbSigmoidPolynomial;
use crate::spectrum::Spectrum;

#[derive(Clone)]
pub struct RgbAlbedoSpectrum<C: Color + Clone> {
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

impl_rgb_albedo_spectrum!(ColorSrgb);
impl_rgb_albedo_spectrum!(ColorDisplayP3);
impl_rgb_albedo_spectrum!(ColorP3D65);
impl_rgb_albedo_spectrum!(ColorAdobeRGB);
impl_rgb_albedo_spectrum!(ColorRec709);
impl_rgb_albedo_spectrum!(ColorRec2020);
impl_rgb_albedo_spectrum!(ColorAcesCg);
impl_rgb_albedo_spectrum!(ColorAces2065_1);
