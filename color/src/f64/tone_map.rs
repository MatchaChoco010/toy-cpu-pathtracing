//! トーンマッピングのトレイトと実装のモジュール。

/// トーンマッピングのトレイト。
pub trait ToneMap: Sync + Send + Sized + Clone {
    fn transform(&self, color: glam::DVec3) -> glam::DVec3;
}
/// トーンマッピングが逆変換を持つ場合のトレイト。
pub trait InvertibleToneMap: ToneMap {
    fn inverse_transform(&self, color: glam::DVec3) -> glam::DVec3;
}

/// トーンマッピングを行わない場合の構造体。
#[derive(Clone)]
pub struct NoneToneMap;
impl ToneMap for NoneToneMap {
    fn transform(&self, color: glam::DVec3) -> glam::DVec3 {
        color
    }
}
impl InvertibleToneMap for NoneToneMap {
    fn inverse_transform(&self, color: glam::DVec3) -> glam::DVec3 {
        color
    }
}
