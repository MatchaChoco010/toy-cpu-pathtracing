//! 数学関連のモジュール。
//! NaNを発生させない数学関数や、
//! 座標系を区別するベクトルや点、法線、レイ、変換行列などを定義する。

mod bounds;
mod coordinate_system;
mod light_sample_context;
mod matrix;
mod normal;
mod point;
mod ray;
mod safe_math;
mod transform;
mod vector;

pub use bounds::*;
pub use coordinate_system::*;
pub use light_sample_context::*;
pub use matrix::*;
pub use normal::*;
pub use point::*;
pub use ray::*;
pub use safe_math::*;
pub use transform::*;
pub use vector::*;

/// コンパイル時の3乗根関数。
pub const fn cube_root(x: f32) -> f32 {
    // 0.0の場合は0.0を返す。
    if x == 0.0 {
        return 0.0;
    }

    // ビットを使って3乗根の初期近似値を生成
    let x_bits = x.abs().to_bits();
    let approx_bits = x_bits / 3 + 709921077; // ≒ (127 << 23) / 3
    let mut guess = f32::from_bits(approx_bits);

    // ニュートン法で3乗根を計算
    let mut i = 0;
    while i < 20 {
        guess = (2.0 * guess + x / (guess * guess)) / 3.0;
        i += 1;
    }

    // 符号を元に戻す
    guess.copysign(x.signum())
}

/// コンパイル時の平方根関数。
pub const fn square_root(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }

    // ビット演算による初期近似（Quick and Dirty法）
    let x_bits = x.to_bits();
    let approx_bits = (x_bits >> 1) + 0x1fc00000;
    let mut guess = f32::from_bits(approx_bits);

    // ニュートン–ラフソン反復（精度重視）
    let mut i = 0;
    while i < 10 {
        guess = 0.5 * (guess + x / guess);
        i += 1;
    }

    guess
}
