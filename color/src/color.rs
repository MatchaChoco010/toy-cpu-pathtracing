//! レンダラで扱う色や色空間などを扱うモジュール。

use std::marker::PhantomData;

use crate::eotf::{Eotf, Gamma2_2, Gamma2_6, GammaRec709, GammaSrgb, Linear};
use crate::gamut::{
    ColorGamut, GamutAces2065_1, GamutAcesCg, GamutAdobeRgb, GamutDciP3D65, GamutRec2020,
    GamutSrgb, xy_to_xyz,
};
use crate::tone_map::{InvertibleToneMap, NoneToneMap, ToneMap};

/// XYZ色空間の色を表す構造体。
#[derive(Debug, Clone)]
pub struct Xyz {
    xyz: glam::Vec3,
}
impl Xyz {
    /// RGBに変換する関数。
    pub fn xyz_to_rgb<G: ColorGamut>(&self) -> ColorImpl<G, NoneToneMap, Linear> {
        let xyz = self.xyz;
        let gamut = G::new();
        let rgb = (gamut.xyz_to_rgb() * xyz).max(glam::Vec3::splat(0.0));
        ColorImpl::create(rgb, gamut, NoneToneMap)
    }
}
impl From<glam::Vec3> for Xyz {
    /// XYZ値を持つXyzを生成する。
    fn from(xyz: glam::Vec3) -> Self {
        Self { xyz }
    }
}

/// 各種色空間の色が実装するトレイト。
pub trait Color: Sync + Send + Clone {
    /// RGB値を取得する。
    fn rgb(&self) -> glam::Vec3;

    /// XYZ値を取得する。
    fn xyz(&self) -> glam::Vec3;
}

/// RGB色空間の色を表す構造体。
/// ジェネリクスで指定された色域で、
/// ジェネリクスで指定されたトーンマップとEOTFがかかった後のrgb値を持つ。
#[derive(Clone)]
pub struct ColorImpl<G: ColorGamut, T: ToneMap, E: Eotf> {
    rgb: glam::Vec3,
    gamut: G,
    tone_map: T,
    _eotf: PhantomData<E>,
}
impl<G: ColorGamut, T: ToneMap, E: Eotf> ColorImpl<G, T, E> {
    /// Colorを生成する。
    fn create(rgb: glam::Vec3, gamut: G, tone_map: T) -> Self {
        Self {
            rgb,
            gamut,
            tone_map,
            _eotf: PhantomData,
        }
    }

    /// 別の色域の色から新しい色域に色域を変更する。
    pub fn from<FromGamut: ColorGamut>(color: &ColorImpl<FromGamut, T, E>) -> Self {
        let gamut = G::new();
        let xyz = color.xyz();
        let rgb = gamut.xyz_to_rgb() * xyz;
        ColorImpl::create(rgb, gamut, color.tone_map.clone())
    }

    /// RGBとToneMapを持つColorを生成する。
    pub fn from_rgb_tone_map(rgb: glam::Vec3, tone_map: T) -> Self {
        Self::create(rgb, G::new(), tone_map)
    }

    /// EOTFの逆変換を適用する。
    pub fn invert_eotf(&self) -> ColorImpl<G, T, Linear> {
        let rgb = E::inverse_transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}
impl<G: ColorGamut, T: ToneMap, E: Eotf> Color for ColorImpl<G, T, E> {
    fn rgb(&self) -> glam::Vec3 {
        self.rgb
    }

    fn xyz(&self) -> glam::Vec3 {
        self.gamut.rgb_to_xyz() * self.rgb
    }
}

// トーンマップが指定されていない場合、RGBの値から直接色を指定できる。
// 必要があればその後トーンマップを適用する。
impl<G: ColorGamut, E: Eotf> ColorImpl<G, NoneToneMap, E> {
    /// RGB値を持つColorを生成する。
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self::create(glam::vec3(r, g, b), G::new(), NoneToneMap)
    }

    /// RGB値を持つColorを生成する。
    pub fn from_rgb(rgb: glam::Vec3) -> Self {
        Self::create(rgb, G::new(), NoneToneMap)
    }
}

// EOTFを掛ける前の色の場合、EOTFを適用した色に変換できる。
// いわゆるガンマ処理。
impl<G: ColorGamut, T: ToneMap> ColorImpl<G, T, Linear> {
    /// EOTFを適用する。
    pub fn apply_eotf<E: Eotf>(&self) -> ColorImpl<G, T, E> {
        let rgb = E::transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}

// EOTF適用前でトーンマップが指定されておりトーンマップが逆変換可能な場合、
// トーンマップの逆変換を掛けてトーンマップ未指定の色を取得できる。
impl<G: ColorGamut, T: InvertibleToneMap> ColorImpl<G, T, Linear> {
    /// トーンマップの逆変換を行う。
    pub fn invert_tone_map(&self) -> ColorImpl<G, NoneToneMap, Linear> {
        self.tone_map.inverse_transform(self)
    }
}

// EOTFが適用前でさらにトーンマップが指定されていない場合、
// トーンマップ適用処理を行ってトーンマップ後の色を取得できる。
impl<G: ColorGamut> ColorImpl<G, NoneToneMap, Linear> {
    /// exposureを適用する。
    pub fn apply_exposure(&self, exposure: f32) -> ColorImpl<G, NoneToneMap, Linear> {
        let rgb = self.rgb * exposure;
        ColorImpl::create(rgb, self.gamut.clone(), NoneToneMap)
    }

    /// トーンマップを適用する。
    pub fn apply_tone_map<T: ToneMap>(&self, tone_map: T) -> ColorImpl<G, T, Linear> {
        tone_map.transform(self)
    }
}

/// sRGB色空間の色を表す構造体。
/// 色域がsRGBでEOTFがsRGBのガンマ関数。
pub type ColorSrgb<T> = ColorImpl<GamutSrgb, T, GammaSrgb>;

/// Display P3色空間の色を表す構造体。
/// 色域がDisplay P3でEOTFはsRGBのガンマ関数。
pub type ColorDisplayP3<T> = ColorImpl<GamutDciP3D65, T, GammaSrgb>;

/// P3-D65色空間の色を表す構造体。
/// 色域がP3-D65でEOTFはガンマ2.6のガンマ関数。
pub type ColorP3D65<T> = ColorImpl<GamutDciP3D65, T, Gamma2_6>;

/// Adobe RGB色空間の色を表す構造体。
/// 色域とEOTFがAdobe RGBのもの。
pub type ColorAdobeRGB<T> = ColorImpl<GamutAdobeRgb, T, Gamma2_2>;

/// Rec. 709色空間の色を表す構造体。
/// 色域がsRGBでEOTFはRec. 709のガンマ関数
pub type ColorRec709<T> = ColorImpl<GamutSrgb, T, GammaRec709>;

/// Rec. 2020色空間の色を表す構造体。
/// 色域がRec. 2020でEOTFはRec.709のガンマ関数
pub type ColorRec2020<T> = ColorImpl<GamutRec2020, T, GammaRec709>;

/// ACEScg色空間の色を表す構造体。
/// 色域がACEScgでEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type ColorAcesCg<T> = ColorImpl<GamutAcesCg, T, Linear>;

/// ACES2065-1色空間の色を表す構造体。
/// 色域がACES2065-1でEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type ColorAces2065_1<T> = ColorImpl<GamutAces2065_1, T, Linear>;
