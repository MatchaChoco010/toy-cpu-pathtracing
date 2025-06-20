//! 密にサンプルされたスペクトルを定義するモジュール。

use std::sync::Arc;

use util_macros::impl_assign_ops;

use crate::{
    N_SPECTRUM_SAMPLES, SampledSpectrum, SampledWavelengths,
    spectrum::{LAMBDA_MAX, LAMBDA_MIN, Spectrum, SpectrumTrait},
};

pub const N_SPECTRUM_DENSELY_SAMPLES: usize = (LAMBDA_MAX - LAMBDA_MIN) as usize;

/// 密にサンプリングされたスペクトルを表す構造体。
/// 1nmごとにサンプリングされたスペクトル強度を格納する。
#[derive(Debug, Clone)]
pub struct DenselySampledSpectrum {
    values: [f32; N_SPECTRUM_DENSELY_SAMPLES],
    max_value: f32,
}
impl DenselySampledSpectrum {
    /// 新しい密にサンプリングされたスペクトルを作成する。
    pub fn new(values: [f32; N_SPECTRUM_DENSELY_SAMPLES]) -> Spectrum {
        let mut max_value = 0.0;
        let mut i = 0;
        while i < N_SPECTRUM_DENSELY_SAMPLES {
            if values[i] > max_value {
                max_value = values[i];
            }
            i += 1;
        }
        Arc::new(Self { values, max_value })
    }

    /// ゼロで初期化された密にサンプリングされたスペクトルを作成する。
    pub fn zero() -> Self {
        let values = [0.0; N_SPECTRUM_DENSELY_SAMPLES];
        Self {
            values,
            max_value: 0.0,
        }
    }

    /// SampledSpectrumを足し合わせる。
    pub fn add_sample(&mut self, lambda: &SampledWavelengths, s: SampledSpectrum) {
        s.eprint_nan_inf("DenselySampledSpectrum::add_sample");

        let count = if lambda.is_secondary_terminated() {
            1
        } else {
            N_SPECTRUM_SAMPLES
        };
        for index in 0..count {
            let l = lambda.lambda(index);
            let i = (l - LAMBDA_MIN).floor() as usize;
            let i = if i == N_SPECTRUM_DENSELY_SAMPLES {
                0
            } else {
                i
            };
            self.values[i] +=
                s.value(index) / lambda.pdf().value(index) / N_SPECTRUM_SAMPLES as f32;
            self.max_value = self.max_value.max(self.values[i]);
        }
    }

    /// 与えられたスペクトルをサンプリングして、密にサンプリングされたスペクトルを作成する。
    pub fn from(spectrum: &Spectrum) -> Spectrum {
        let mut values = [0.0; N_SPECTRUM_DENSELY_SAMPLES];
        let mut max_value = 0.0;
        for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
            let lambda = LAMBDA_MIN + i as f32;
            values[i] = spectrum.value(lambda);
            assert!(values[i] >= 0.0);
            if values[i] > max_value {
                max_value = values[i];
            }
        }
        Arc::new(Self { values, max_value })
    }
}
impl SpectrumTrait for DenselySampledSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        if !(LAMBDA_MIN..=LAMBDA_MAX).contains(&lambda) {
            return 0.0;
        }
        let index = (lambda - LAMBDA_MIN).floor() as usize;
        if index < self.values.len() {
            self.values[index]
        } else {
            0.0
        }
    }

    fn max_value(&self) -> f32 {
        self.max_value
    }
}
#[impl_assign_ops(MulAssign)]
fn mul_assign(lhs: &mut DenselySampledSpectrum, rhs: &Spectrum) {
    for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
        let lambda = LAMBDA_MIN + i as f32;
        lhs.values[i] *= rhs.value(lambda);
        if lhs.values[i] > lhs.max_value {
            lhs.max_value = lhs.values[i];
        }
    }
}
#[impl_assign_ops(DivAssign)]
fn div_assign(lhs: &mut DenselySampledSpectrum, rhs: &f32) {
    for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
        lhs.values[i] /= *rhs;
    }
}
