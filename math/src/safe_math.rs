//! NaNを発生させない数学関数を定義するモジュール。

/// 定義域外の値を与えてもNaNを返さないacos関数の実装トレイト。
pub trait SafeAcos {
    fn safe_acos(self) -> f32;
}
impl SafeAcos for f32 {
    #[inline(always)]
    fn safe_acos(self) -> f32 {
        if self < -1.0 {
            -std::f32::consts::PI
        } else if self > 1.0 {
            std::f32::consts::PI
        } else {
            self.acos()
        }
    }
}
