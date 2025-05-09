//! EOTF (Electro-Optical Transfer Function) の実装を行うモジュール。

/// EOTFのTransfer Functionを表すトレイト。
pub trait Eotf: Sync + Send + Clone {}

/// ノンリニアなEOTFのトレイト。
pub trait NonLinearEotf: Eotf {
    fn transform(color: glam::Vec3) -> glam::Vec3;
    fn inverse_transform(color: glam::Vec3) -> glam::Vec3;
}

/// ガンマ2.2のガンマ補正のEOTF。
#[derive(Clone)]
pub struct Gamma2_2;
impl Eotf for Gamma2_2 {}
impl NonLinearEotf for Gamma2_2 {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(1.0 / 2.2)
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(2.2)
    }
}

/// ガンマ2.4のガンマ補正のEOTF。
#[derive(Clone)]
pub struct Gamma2_4;
impl Eotf for Gamma2_4 {}
impl NonLinearEotf for Gamma2_4 {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(1.0 / 2.4)
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(2.4)
    }
}

/// ガンマ2.6のガンマ補正のEOTF。
#[derive(Clone)]
pub struct Gamma2_6;
impl Eotf for Gamma2_6 {}
impl NonLinearEotf for Gamma2_6 {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(1.0 / 2.6)
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        color.powf(2.6)
    }
}

/// sRGBのガンマ補正のEOTF。
/// 0.0031308以下の値はリニアに変換され、0.0031308以上の値はガンマ2.4でガンマ補正される。
#[derive(Clone)]
pub struct GammaSrgb;
impl Eotf for GammaSrgb {}
impl NonLinearEotf for GammaSrgb {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        color.map(|c| {
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        })
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        color.map(|c| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    }
}

/// Adobe RGBのガンマ補正のEOTF。
/// AdobeRGBのガンマは563/256≒2.19921875である。
#[derive(Clone)]
pub struct GammaAdobeRgb;
impl Eotf for GammaAdobeRgb {}
impl NonLinearEotf for GammaAdobeRgb {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        let gamma = 563.0 / 256.0;
        color.map(|c| c.powf(1.0 / gamma))
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        let gamma = 563.0 / 256.0;
        color.map(|c| c.powf(gamma))
    }
}

/// Rec. 709のガンマ補正のEOTF。
/// 0.018以下の値はリニアに変換され、0.018以上の値はガンマ2.4でガンマ補正される。
#[derive(Clone)]
pub struct GammaRec709;
impl Eotf for GammaRec709 {}
impl NonLinearEotf for GammaRec709 {
    fn transform(color: glam::Vec3) -> glam::Vec3 {
        color.map(|c| {
            if c < 0.018 {
                4.5 * c
            } else {
                1.099 * c.powf(0.45) - 0.099
            }
        })
    }

    fn inverse_transform(color: glam::Vec3) -> glam::Vec3 {
        color.map(|c| {
            if c < 0.081 {
                c / 4.5
            } else {
                ((c + 0.099) / 1.099).powf(1.0 / 0.45)
            }
        })
    }
}

/// リニアのEOTF。
/// EOTF変換前の色をそのまま表す際に使う。
#[derive(Clone)]
pub struct Linear;
impl Eotf for Linear {}
