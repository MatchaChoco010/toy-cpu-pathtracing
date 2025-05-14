//! スペクトルに関するモジュール。

use std::sync::Arc;

use color::Xyz;

mod black_body_spectrum;
mod constant_spectrum;
mod densely_sampled_spectrum;
mod piecewise_linear_spectrum;
mod rgb_albedo_spectrum;
mod rgb_illuminant_spectrum;
mod rgb_unbounded_spectrum;

pub use black_body_spectrum::*;
pub use constant_spectrum::*;
pub use densely_sampled_spectrum::*;
pub use piecewise_linear_spectrum::*;
pub use rgb_albedo_spectrum::*;
pub use rgb_illuminant_spectrum::*;
pub use rgb_unbounded_spectrum::*;

use crate::presets;
use crate::sampled_spectrum::{N_SPECTRUM_SAMPLES, SampledSpectrum, SampledWavelengths};

/// 可視光の波長の範囲の最小値 (nm)。
pub const LAMBDA_MIN: f32 = 360.0;
/// 可視光の波長の範囲の最大値 (nm)。
pub const LAMBDA_MAX: f32 = 830.0;

/// スペクトルのトレイト。
pub trait SpectrumTrait: Send + Sync {
    /// 波長lambda (nm)に対するスペクトル強度の値を取得する。
    fn value(&self, lambda: f32) -> f32;

    /// スペクトル強度の最大値を取得する。
    fn max_value(&self) -> f32;

    /// スペクトルをサンプルする。
    fn sample(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        let mut values = [0.0; N_SPECTRUM_SAMPLES];

        // 最初の波長以外が終了している場合は、最初の波長の値を返す。
        if lambda.is_secondary_terminated() {
            values[0] = self.value(lambda.lambda(0));
            return SampledSpectrum::from(values);
        }

        for i in 0..N_SPECTRUM_SAMPLES {
            values[i] = self.value(lambda.lambda(i));
        }
        SampledSpectrum::from(values)
    }

    /// スペクトルをXYZ色空間に変換する。
    fn to_xyz(&self) -> Xyz {
        let xyz = glam::vec3(
            inner_product(self, &presets::x()),
            inner_product(self, &presets::y()),
            inner_product(self, &presets::z()),
        ) / presets::y_integral();
        Xyz::from(xyz)
    }
}

/// スペクトル同士の内積を計算する関数。
fn inner_product<S>(s1: &S, s2: &Spectrum) -> f32
where
    S: SpectrumTrait + ?Sized,
{
    let mut sum = 0.0;
    let range = 0..(LAMBDA_MAX - LAMBDA_MIN) as usize;
    for i in range {
        let lambda = LAMBDA_MIN + i as f32;
        sum += s1.value(lambda) * s2.value(lambda);
    }
    sum
}

/// スペクトルの型エイリアス。
pub type Spectrum = Arc<dyn SpectrumTrait>;
