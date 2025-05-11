//! レンダラで扱う色や色空間などを扱うモジュール。

use std::marker::PhantomData;

use crate::eotf::{Eotf, Gamma2_2, Gamma2_6, GammaRec709, GammaSrgb, Linear, NonLinearEotf};
use crate::gamut::{
    ColorGamut, GamutAces2065_1, GamutAcesCg, GamutAdobeRgb, GamutDciP3D65, GamutRec2020,
    GamutSrgb, xy_to_xyz,
};
use crate::tone_map::{InvertibleToneMap, NoneToneMap, ToneMap};

/// XYZ色空間の色を表す構造体。
#[derive(Clone)]
pub struct Xyz {
    #[allow(dead_code)]
    xyz: glam::Vec3,
}
impl Xyz {
    /// XYZからLabに変換する関数。
    pub fn xyz_to_lab(&self) -> glam::Vec3 {
        let xyz = self.xyz;
        fn f(t: f32) -> f32 {
            if t > 0.008856 {
                t.powf(1.0 / 3.0)
            } else {
                (t * 903.3 + 16.0) / 116.0
            }
        }
        let w = glam::vec2(0.34567, 0.35850);
        let w_xyz = xy_to_xyz(w);

        let xr = xyz.x / w_xyz.x;
        let yr = xyz.y / w_xyz.y;
        let zr = xyz.z / w_xyz.z;

        let l = 116.0 * f(yr) - 16.0;
        let a = 500.0 * (f(xr) - f(yr));
        let b = 200.0 * (f(yr) - f(zr));

        glam::vec3(l, a, b)
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
        let rgb = gamut.xyz_to_rgb(xyz);
        ColorImpl::create(rgb, gamut, color.tone_map.clone())
    }
}
impl<G: ColorGamut, T: ToneMap, E: Eotf> Color for ColorImpl<G, T, E> {
    fn rgb(&self) -> glam::Vec3 {
        self.rgb
    }

    fn xyz(&self) -> glam::Vec3 {
        self.gamut.rgb_to_xyz(self.rgb)
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
    pub fn from_vec3(rgb: glam::Vec3) -> Self {
        Self::create(rgb, G::new(), NoneToneMap)
    }
}

// トーンマップが指定されておりトーンマップが逆変換可能な場合、
// トーンマップの逆変換を掛けてトーンマップ未指定の色を取得できる。
impl<G: ColorGamut, T: InvertibleToneMap, E: Eotf> ColorImpl<G, T, E> {
    /// トーンマップの逆変換を行う。
    pub fn invert_tone_map(&self) -> ColorImpl<G, NoneToneMap, E> {
        let rgb = self.tone_map.inverse_transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), NoneToneMap)
    }
}

// EOTFを掛けた後の色の場合、EOTFの逆関数を掛けてリニアな色に変換できる。
// いわゆるデガンマ処理。
impl<G: ColorGamut, T: ToneMap, E: NonLinearEotf> ColorImpl<G, T, E> {
    /// EOTFの逆変換を適用する。
    pub fn invert_eotf(&self) -> ColorImpl<G, T, Linear> {
        let rgb = E::inverse_transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}

// EOTFを掛ける前の色の場合、EOTFを適用した色に変換できる。
// いわゆるガンマ処理。
impl<G: ColorGamut, T: ToneMap> ColorImpl<G, T, Linear> {
    /// EOTFを適用する。
    pub fn apply_eotf<E: NonLinearEotf>(&self) -> ColorImpl<G, T, E> {
        let rgb = E::transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), self.tone_map.clone())
    }
}

// EOTFが適用前でさらにトーンマップが指定されていない場合、
// トーンマップ適用処理を行ってトーンマップ後の色を取得できる。
impl<G: ColorGamut> ColorImpl<G, NoneToneMap, Linear> {
    /// トーンマップを適用する。
    pub fn apply_tone_map<T: ToneMap>(&self, tone_map: T) -> ColorImpl<G, T, Linear> {
        let rgb = tone_map.transform(self.rgb);
        ColorImpl::create(rgb, self.gamut.clone(), tone_map)
    }
}

/// sRGB色空間の色を表す構造体。
/// 色域がsRGBでEOTFがsRGBのガンマ関数。
pub type ColorSrgb = ColorImpl<GamutSrgb, NoneToneMap, GammaSrgb>;

/// Display P3色空間の色を表す構造体。
/// 色域がDisplay P3でEOTFはsRGBのガンマ関数。
pub type ColorDisplayP3 = ColorImpl<GamutDciP3D65, NoneToneMap, GammaSrgb>;

/// P3-D65色空間の色を表す構造体。
/// 色域がP3-D65でEOTFはガンマ2.6のガンマ関数。
pub type ColorP3D65 = ColorImpl<GamutDciP3D65, NoneToneMap, Gamma2_6>;

/// Adobe RGB色空間の色を表す構造体。
/// 色域とEOTFがAdobe RGBのもの。
pub type ColorAdobeRGB = ColorImpl<GamutAdobeRgb, NoneToneMap, Gamma2_2>;

/// Rec. 709色空間の色を表す構造体。
/// 色域がsRGBでEOTFはRec. 709のガンマ関数
pub type ColorRec709 = ColorImpl<GamutSrgb, NoneToneMap, GammaRec709>;

/// Rec. 2020色空間の色を表す構造体。
/// 色域がRec. 2020でEOTFはRec.709のガンマ関数
pub type ColorRec2020 = ColorImpl<GamutRec2020, NoneToneMap, GammaRec709>;

/// ACEScg色空間の色を表す構造体。
/// 色域がACEScgでEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type ColorAcesCg = ColorImpl<GamutAcesCg, NoneToneMap, Linear>;

/// ACES2065-1色空間の色を表す構造体。
/// 色域がACES2065-1でEOTFはシーンリニアを想定してリニアとする。
/// ACESのワークフローでディスプレイに表示するにはRRTとODTにあたるトーンマップを適用する必要がある。
pub type ColorAces2065_1 = ColorImpl<GamutAces2065_1, NoneToneMap, Linear>;
