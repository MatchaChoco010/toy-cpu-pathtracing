//! レンダラで扱う色や色空間などを扱うモジュール。

use std::marker::PhantomData;

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, Spectrum, presets};

pub mod eotf;
pub mod gamut;
pub mod tone_map;

use eotf::{Eotf, Gamma2_2, Gamma2_6, GammaRec709, GammaSrgb, Linear, NonLinearEotf};
use gamut::{
    Aces2065_1Gamut, AcesCgGamut, AdobeRgbGamut, ColorGamut, DciP3D65Gamut, Rec2020Gamut, SrgbGamut,
};
use tone_map::{InvertibleToneMap, NoneToneMap, ToneMap};

/// スペクトル同士の内積を計算する関数。
fn inner_product(s1: &Spectrum, s2: &Spectrum) -> f32 {
    let mut sum = 0.0;
    let range = 0..(LAMBDA_MAX - LAMBDA_MIN) as usize;
    for i in range {
        let lambda = LAMBDA_MIN + i as f32;
        sum += s1.value(lambda) * s2.value(lambda);
    }
    sum
}

/// XYZ色空間の色を表す構造体。
pub struct Xyz {
    xyz: glam::Vec3,
}
impl From<Spectrum> for Xyz {
    /// SpectrumからXyzに変換する。
    fn from(s: Spectrum) -> Self {
        let xyz = glam::vec3(
            inner_product(&presets::x(), &s),
            inner_product(&presets::y(), &s),
            inner_product(&presets::z(), &s),
        ) / presets::y_integral();
        Self { xyz }
    }
}

pub trait ColorTrait {
    fn rgb(&self) -> glam::Vec3;
}

/// RGB色空間の色を表す構造体。
/// ジェネリクスで指定された色域で、
/// ジェネリクスで指定されたトーンマップとEOTFがかかった後のrgb値を持つ。
pub struct Color<G: ColorGamut, T: ToneMap, E: Eotf> {
    rgb: glam::Vec3,
    gamut: G,
    tone_map: T,
    _eotf: PhantomData<E>,
}
impl<G: ColorGamut, T: ToneMap, E: Eotf> Color<G, T, E> {
    /// Colorを生成する。
    fn create(rgb: glam::Vec3, gamut: G, tone_map: T) -> Self {
        Self {
            rgb,
            gamut,
            tone_map,
            _eotf: PhantomData,
        }
    }

    /// XYZ値を取得する。
    fn xyz(&self) -> glam::Vec3 {
        self.gamut.rgb_to_xyz(self.rgb)
    }

    /// 別の色域の色から色域を変更する。
    pub fn from<G2: ColorGamut>(color: Color<G2, T, E>) -> Self {
        let xyz = color.xyz();
        let gamut = G::new();
        let rgb = gamut.xyz_to_rgb(xyz);
        Color::create(rgb, gamut, color.tone_map.clone())
    }
}
impl<G: ColorGamut, T: ToneMap, E: Eotf> ColorTrait for Color<G, T, E> {
    /// RGB値を取得する。
    fn rgb(&self) -> glam::Vec3 {
        self.rgb
    }
}
impl<G: ColorGamut> From<Xyz> for Color<G, NoneToneMap, Linear> {
    /// XyzからColorを生成する。
    fn from(xyz: Xyz) -> Self {
        let gamut = G::new();
        let rgb = gamut.xyz_to_rgb(xyz.xyz);
        Color::create(rgb, gamut, NoneToneMap)
    }
}
// トーンマップが逆変換可能な場合。
impl<G: ColorGamut, T: InvertibleToneMap, E: Eotf> Color<G, T, E> {
    /// トーンマップの逆変換を行う。
    pub fn invert_tone_map(&self) -> Color<G, NoneToneMap, E> {
        let rgb = self.tone_map.inverse_transform(self.rgb);
        Color::create(rgb, self.gamut.clone(), NoneToneMap)
    }
}
// EOTFを掛けた後の色の場合。
impl<G: ColorGamut, T: ToneMap, E: NonLinearEotf> Color<G, T, E> {
    /// EOTFの逆変換を適用する。
    pub fn invert_eotf(&self) -> Color<G, T, Linear> {
        let rgb = E::inverse_transform(self.rgb);
        Color::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}
// EOTFを掛ける前の色の場合。
impl<G: ColorGamut, T: ToneMap> Color<G, T, Linear> {
    /// EOTFを適用する。
    pub fn apply_eotf<E: NonLinearEotf>(&self) -> Color<G, T, E> {
        let rgb = E::transform(self.rgb);
        Color::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}
// EOTFが適用前でさらにトーンマップが指定されていない場合。
impl<G: ColorGamut> Color<G, NoneToneMap, Linear> {
    /// トーンマップを適用する。
    pub fn apply_tone_map<T: ToneMap>(&self, tone_map: T) -> Color<G, T, Linear> {
        let rgb = tone_map.transform(self.rgb);
        Color::create(rgb, self.gamut.clone(), tone_map)
    }
}

/// sRGB色空間の色を表す構造体。
/// 色域がsRGBでEOTFがsRGBのガンマ関数。
pub type SrgbColor = Color<SrgbGamut, NoneToneMap, GammaSrgb>;
impl SrgbColor {
    /// sRGB色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, SrgbGamut::new(), NoneToneMap)
    }
}

/// Display P3色空間の色を表す構造体。
/// 色域がDisplay P3でEOTFはsRGBのガンマ関数。
pub type DisplayP3Color = Color<DciP3D65Gamut, NoneToneMap, GammaSrgb>;
impl DisplayP3Color {
    /// Display P3色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, DciP3D65Gamut::new(), NoneToneMap)
    }
}

/// P3-D65色空間の色を表す構造体。
/// 色域がP3-D65でEOTFはガンマ2.6のガンマ関数。
pub type P3D65Color = Color<DciP3D65Gamut, NoneToneMap, Gamma2_6>;
impl P3D65Color {
    /// P3-D65色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, DciP3D65Gamut::new(), NoneToneMap)
    }
}

/// Adobe RGB色空間の色を表す構造体。
/// 色域とEOTFがAdobe RGBのもの。
pub type AdobeRGBColor = Color<AdobeRgbGamut, NoneToneMap, Gamma2_2>;
impl AdobeRGBColor {
    /// Adobe RGB色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, AdobeRgbGamut::new(), NoneToneMap)
    }
}

/// Rec. 709色空間の色を表す構造体。
/// 色域がsRGBでEOTFはRec. 709のガンマ関数
pub type Rec709Color = Color<SrgbGamut, NoneToneMap, GammaRec709>;
impl Rec709Color {
    /// Rec.709色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, SrgbGamut::new(), NoneToneMap)
    }
}

/// Rec. 2020色空間の色を表す構造体。
/// 色域がRec. 2020でEOTFはRec.709のガンマ関数
pub type Rec2020Color = Color<Rec2020Gamut, NoneToneMap, GammaRec709>;
impl Rec2020Color {
    /// Rec.2020色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, Rec2020Gamut::new(), NoneToneMap)
    }
}

/// ACEScg色空間の色を表す構造体。
/// 色域がACEScgでEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type AcesCgColor = Color<AcesCgGamut, NoneToneMap, Linear>;
impl AcesCgColor {
    /// ACEScg色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, AcesCgGamut::new(), NoneToneMap)
    }
}

/// ACES2065-1色空間の色を表す構造体。
/// 色域がACES2065-1でEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type Aces2065_1Color = Color<Aces2065_1Gamut, NoneToneMap, Linear>;
impl Aces2065_1Color {
    /// ACES2065-1色空間の色を生成する。
    pub fn new(rgb: glam::Vec3) -> Self {
        Self::create(rgb, Aces2065_1Gamut::new(), NoneToneMap)
    }
}
