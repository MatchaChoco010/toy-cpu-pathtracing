//! スペクトルに関するモジュール。

use macros::enum_methods;

mod black_body_spectrum;
mod constant_spectrum;
mod densely_sampled_spectrum;
mod piecewise_linear_spectrum;
mod rgb_albedo_spectrum;
mod rgb_illuminant_spectrum;
mod rgb_unbounded_spectrum;
mod sampled_spectrum;

pub use black_body_spectrum::*;
pub use constant_spectrum::*;
pub use densely_sampled_spectrum::*;
pub use piecewise_linear_spectrum::*;
pub use rgb_albedo_spectrum::*;
pub use rgb_illuminant_spectrum::*;
pub use rgb_unbounded_spectrum::*;
pub use sampled_spectrum::*;

/// 可視光の波長の範囲の最小値 (nm)。
pub const LAMBDA_MIN: f32 = 360.0;
/// 可視光の波長の範囲の最大値 (nm)。
pub const LAMBDA_MAX: f32 = 830.0;

/// スペクトルのトレイト。
pub trait SpectrumTrait {
    /// 波長lambda (nm)に対するスペクトル強度の値を取得する。
    fn value(&self, lambda: f32) -> f32;

    /// スペクトル強度の最大値を取得する。
    fn max_value(&self) -> f32;

    /// スペクトルをサンプルする。
    fn sample(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        let mut values = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES {
            values[i] = self.value(lambda.lambda(i));
        }
        SampledSpectrum::from(values)
    }
}

/// スペクトルを表現する列挙型。
#[enum_methods {
    fn value(&self, lambda: f32) -> f32,
    fn max_value(&self) -> f32,
    fn sample(&self, lambda: &SampledWavelengths) -> SampledSpectrum,
}]
pub enum Spectrum {
    Constant(ConstantSpectrum),
    DenselySampled(DenselySampledSpectrum),
    PiecewiseLinear(PiecewiseLinearSpectrum),
    RgbAlbedo(RgbAlbedoSpectrum),
    RgbUnbounded(RgbUnboundedSpectrum),
    RgbIlluminant(RgbIlluminantSpectrum),
    BlackBody(BlackBodySpectrum),
}
