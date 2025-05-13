//! トーンマップを定義するモジュール。

use color::{Color, ColorImpl, eotf::Linear, gamut::ColorGamut, tone_map::NoneToneMap};

/// Reinhardトーンマッピングの構造体。
#[derive(Clone)]
pub struct ReinhardToneMap;
impl ReinhardToneMap {
    /// Reinhardトーンマッピングを生成する。
    pub fn new() -> Self {
        Self
    }
}
impl color::tone_map::ToneMap for ReinhardToneMap {
    fn transform<G: ColorGamut>(
        &self,
        color: &ColorImpl<G, NoneToneMap, Linear>,
    ) -> ColorImpl<G, Self, Linear> {
        let rgb = color.rgb();
        let scaled_rgb = rgb / (glam::Vec3::ONE + rgb);
        ColorImpl::from_rgb_tone_map(scaled_rgb, Self)
    }
}
