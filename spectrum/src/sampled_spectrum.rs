//! 波長でサンプルした後のスペクトルに関するモジュール。

use util_macros::{impl_assign_ops, impl_binary_ops};

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

/// レンダリング時にスペクトルをサンプルする波長の数。
pub const N_SPECTRUM_SAMPLES: usize = 4;

/// 特定の波長の列でサンプルしたスペクトル強度のリスト。
#[derive(Debug, Clone)]
pub struct SampledSpectrum {
    values: [f32; N_SPECTRUM_SAMPLES],
}
#[impl_binary_ops(Add)]
fn add(lhs: &SampledSpectrum, rhs: &SampledSpectrum) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        result.values[i] = lhs.values[i] + rhs.values[i];
    }
    result.clone().clone()
}
#[impl_binary_ops(Sub)]
fn sub(lhs: &SampledSpectrum, rhs: &SampledSpectrum) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        result.values[i] = lhs.values[i] - rhs.values[i];
    }
    result.clone().clone()
}
#[impl_binary_ops(Mul)]
fn mul(lhs: &SampledSpectrum, rhs: &f32) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        result.values[i] = lhs.values[i] * rhs;
    }
    result.clone().clone()
}
#[impl_binary_ops(Mul)]
fn mul(lhs: &f32, rhs: &SampledSpectrum) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        result.values[i] = lhs * rhs.values[i];
    }
    result.clone().clone()
}
#[impl_binary_ops(Mul)]
fn mul(lhs: &SampledSpectrum, rhs: &SampledSpectrum) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        result.values[i] = lhs.values[i] * rhs.values[i];
    }
    result.clone().clone()
}
#[impl_binary_ops(Div)]
fn div(lhs: &SampledSpectrum, rhs: &f32) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        if rhs == &0.0 {
            result.values[i] = 0.0;
        } else {
            result.values[i] = lhs.values[i] / rhs;
        }
    }
    result.clone().clone()
}
#[impl_binary_ops(Div)]
fn div(lhs: &SampledSpectrum, rhs: &SampledSpectrum) -> SampledSpectrum {
    let mut result = SampledSpectrum::new();
    for i in 0..N_SPECTRUM_SAMPLES {
        if rhs.values[i] == 0.0 {
            result.values[i] = 0.0;
        } else {
            result.values[i] = lhs.values[i] / rhs.values[i];
        }
    }
    result.clone().clone()
}
#[impl_assign_ops(AddAssign)]
fn add_assign(lhs: &mut SampledSpectrum, rhs: &SampledSpectrum) {
    for i in 0..N_SPECTRUM_SAMPLES {
        lhs.values[i] += rhs.values[i];
    }
}
#[impl_assign_ops(SubAssign)]
fn sub_assign(lhs: &mut SampledSpectrum, rhs: &SampledSpectrum) {
    for i in 0..N_SPECTRUM_SAMPLES {
        lhs.values[i] -= rhs.values[i];
    }
}
#[impl_assign_ops(MulAssign)]
fn mul_assign(lhs: &mut SampledSpectrum, rhs: &f32) {
    for i in 0..N_SPECTRUM_SAMPLES {
        lhs.values[i] *= *rhs;
    }
}
#[impl_assign_ops(MulAssign)]
fn mul_assign(lhs: &mut SampledSpectrum, rhs: &SampledSpectrum) {
    for i in 0..N_SPECTRUM_SAMPLES {
        lhs.values[i] *= rhs.values[i];
    }
}
#[impl_assign_ops(DivAssign)]
fn div_assign(lhs: &mut SampledSpectrum, rhs: &f32) {
    if *rhs == 0.0 {
        SampledSpectrum::new();
    } else {
        for i in 0..N_SPECTRUM_SAMPLES {
            lhs.values[i] /= *rhs;
        }
    }
}
#[impl_assign_ops(DivAssign)]
fn div_assign(lhs: &mut SampledSpectrum, rhs: &SampledSpectrum) {
    for i in 0..N_SPECTRUM_SAMPLES {
        if rhs.values[i] == 0.0 {
            lhs.values[i] = 0.0;
        } else {
            lhs.values[i] /= rhs.values[i];
        }
    }
}
impl Default for SampledSpectrum {
    fn default() -> Self {
        Self::new()
    }
}

impl SampledSpectrum {
    /// ゼロのスペクトルを作成する。
    pub fn zero() -> Self {
        Self {
            values: [0.0; N_SPECTRUM_SAMPLES],
        }
    }

    /// 1のスペクトルを作成する。
    pub fn one() -> Self {
        Self {
            values: [1.0; N_SPECTRUM_SAMPLES],
        }
    }

    /// 定数のスペクトルを作成する。
    pub fn constant(value: f32) -> Self {
        Self {
            values: [value; N_SPECTRUM_SAMPLES],
        }
    }

    /// 新しいサンプルスペクトルを作成する。
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            values: [0.0; N_SPECTRUM_SAMPLES],
        }
    }

    /// 新しいサンプルスペクトルを作成する。
    #[inline(always)]
    pub fn from(values: [f32; N_SPECTRUM_SAMPLES]) -> Self {
        Self { values }
    }

    /// 値を取得する。
    #[inline(always)]
    pub fn value(&self, index: usize) -> f32 {
        self.values[index]
    }

    /// すべての値が0.0であるかどうかをチェックする。
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.values.iter().all(|&v| v == 0.0)
    }

    /// 線形補間する。
    #[inline(always)]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        &(self * t) + &(other * (1.0 - t))
    }

    /// 平方根を計算する。
    pub fn sqrt(&self) -> Self {
        let mut result = self.clone();
        for i in 0..N_SPECTRUM_SAMPLES {
            result.values[i] = self.values[i].sqrt();
        }
        result
    }

    /// 値をクランプする。
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        let mut result = self.clone();
        for i in 0..N_SPECTRUM_SAMPLES {
            result.values[i] = result.values[i].clamp(min, max);
        }
        result
    }

    /// べき乗を計算する。
    pub fn pow(&self, exponent: f32) -> Self {
        let mut result = self.clone();
        for i in 0..N_SPECTRUM_SAMPLES {
            result.values[i] = self.values[i].powf(exponent);
        }
        result
    }

    /// 指数を計算する。
    pub fn exp(&self) -> Self {
        let mut result = self.clone();
        for i in 0..N_SPECTRUM_SAMPLES {
            result.values[i] = self.values[i].exp();
        }
        result
    }

    /// 最初の波長以外を0にする。
    pub fn terminate_secondary(&mut self) {
        if self.is_zero() {
            return;
        }

        for i in 1..N_SPECTRUM_SAMPLES {
            self.values[i] = 0.0;
        }
    }

    /// 最小値を取得する。
    #[inline(always)]
    pub fn min_value(&self) -> f32 {
        self.values.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// 最大値を取得する。
    #[inline(always)]
    pub fn max_value(&self) -> f32 {
        self.values
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// 平均値を取得する。
    pub fn average(&self) -> f32 {
        let sum: f32 = self.values.iter().sum();
        sum / N_SPECTRUM_SAMPLES as f32
    }

    /// 定数かどうかを判定する
    pub fn is_constant(&self) -> bool {
        let first_value = self.values[0];
        self.values.iter().all(|&v| v == first_value)
    }

    /// NaNやInfinityの値をチェックして警告を出力する。
    pub fn eprint_nan_inf(&self, label: &str) {
        for (i, &value) in self.values.iter().enumerate() {
            if value.is_nan() {
                eprintln!(
                    "[{label}] Warning: SampledSpectrum value at index {} is NaN.",
                    i
                );
            }
            if value.is_infinite() {
                eprintln!(
                    "[{label}] Warning: SampledSpectrum value at index {} is infinite.",
                    i
                );
            }
        }
    }
}

/// スペクトルをサンプルした波長の列を保持する構造体。
#[derive(Debug, Clone)]
pub struct SampledWavelengths {
    /// サンプルした波長のリスト。
    lambda: [f32; N_SPECTRUM_SAMPLES],
    /// サンプルごとの確率密のリスト。
    pdf: [f32; N_SPECTRUM_SAMPLES],
}
impl SampledWavelengths {
    /// 可視光の範囲で均等にサンプリングされた波長を生成する。
    #[inline(always)]
    pub fn new_uniform(u: f32) -> Self {
        Self::new_uniform_range(u, LAMBDA_MIN, LAMBDA_MAX)
    }

    /// 指定された範囲で均等にサンプリングされた波長を生成する。
    pub fn new_uniform_range(u: f32, lambda_min: f32, lambda_max: f32) -> Self {
        let mut result = Self {
            lambda: [0.0; N_SPECTRUM_SAMPLES],
            pdf: [1.0 / (lambda_max - lambda_min); N_SPECTRUM_SAMPLES],
        };

        // 最初のサンプルを乱数uに基づいて配置する
        result.lambda[0] = lambda_min + u * (lambda_max - lambda_min);

        // 残りは等間隔で回り込んで配置する
        let delta = (lambda_max - lambda_min) / (N_SPECTRUM_SAMPLES as f32);
        for i in 1..N_SPECTRUM_SAMPLES {
            result.lambda[i] = result.lambda[i - 1] + delta;
            if result.lambda[i] >= lambda_max {
                result.lambda[i] = lambda_min + (result.lambda[i] - lambda_max);
            }
        }
        result
    }

    /// lambdaの値を取得する。
    #[inline(always)]
    pub fn lambda(&self, index: usize) -> f32 {
        self.lambda[index]
    }

    /// サンプルした波長ごとにpdfの値を格納したSampledSpectrumを生成する。
    #[inline(always)]
    pub fn pdf(&self) -> SampledSpectrum {
        SampledSpectrum::from(self.pdf)
    }

    /// 最初の波長以外を終了する。
    pub fn terminate_secondary(&mut self) {
        if self.is_secondary_terminated() {
            return;
        }

        for i in 1..N_SPECTRUM_SAMPLES {
            self.pdf[i] = 0.0;
        }
        self.pdf[0] /= N_SPECTRUM_SAMPLES as f32;
    }

    /// 最初の波長以外が終了しているかどうかをチェックする。
    pub fn is_secondary_terminated(&self) -> bool {
        self.pdf[1..].iter().all(|&v| v == 0.0)
    }
}
