//! 密にサンプルされたスペクトルを定義するモジュール。

use std::sync::Arc;

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, Spectrum, SpectrumTrait};

const N_SPECTRUM_SAMPLES: usize = (LAMBDA_MAX - LAMBDA_MIN) as usize;

/// 密にサンプリングされたスペクトルを表す構造体。
/// 1nmごとにサンプリングされたスペクトル強度を格納する。
#[derive(Clone)]
pub struct DenselySampledSpectrum {
    values: [f32; N_SPECTRUM_SAMPLES],
    max_value: f32,
}
impl DenselySampledSpectrum {
    /// 新しい密にサンプリングされたスペクトルを作成する。
    pub fn new(values: [f32; N_SPECTRUM_SAMPLES]) -> Spectrum {
        let mut max_value = 0.0;
        let mut i = 0;
        while i < N_SPECTRUM_SAMPLES {
            if values[i] > max_value {
                max_value = values[i];
            }
            i += 1;
        }
        Arc::new(Self { values, max_value })
    }

    /// 与えられたスペクトルをサンプリングして、密にサンプリングされたスペクトルを作成する。
    pub fn from(spectrum: &Spectrum) -> Spectrum {
        let mut values = [0.0; N_SPECTRUM_SAMPLES];
        let mut max_value = 0.0;
        for i in 0..N_SPECTRUM_SAMPLES {
            let lambda = LAMBDA_MIN + i as f32;
            values[i] = spectrum.value(lambda);
            if values[i] > max_value {
                max_value = values[i];
            }
        }
        Arc::new(Self { values, max_value })
    }
}
impl SpectrumTrait for DenselySampledSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        if lambda < LAMBDA_MIN || lambda > LAMBDA_MAX {
            return 0.0;
        }
        let index = (lambda - LAMBDA_MIN).round() as usize;
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
