//! トーンマッピングのトレイトと実装のモジュール。

use crate::{ColorImpl, eotf::Linear, gamut::ColorGamut};

/// トーンマッピングのトレイト。
pub trait ToneMap: Sync + Send + Sized + Clone {
    fn transform<G: ColorGamut>(
        &self,
        color: &ColorImpl<G, NoneToneMap, Linear>,
    ) -> ColorImpl<G, Self, Linear>;
}
/// トーンマッピングが逆変換を持つ場合のトレイト。
pub trait InvertibleToneMap: ToneMap {
    fn inverse_transform<G: ColorGamut>(
        &self,
        color: &ColorImpl<G, Self, Linear>,
    ) -> ColorImpl<G, NoneToneMap, Linear>;
}

/// トーンマッピングを行わない場合の構造体。
#[derive(Clone)]
pub struct NoneToneMap;
impl ToneMap for NoneToneMap {
    fn transform<G: ColorGamut>(
        &self,
        color: &ColorImpl<G, NoneToneMap, Linear>,
    ) -> ColorImpl<G, Self, Linear> {
        color.clone()
    }
}
impl InvertibleToneMap for NoneToneMap {
    fn inverse_transform<G: ColorGamut>(
        &self,
        color: &ColorImpl<G, Self, Linear>,
    ) -> ColorImpl<G, NoneToneMap, Linear> {
        color.clone()
    }
}
