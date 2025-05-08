//! 密にサンプルされたスペクトルを定義するモジュール。

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

use super::Spectrum;

/// 密にサンプリングされたスペクトルを表す構造体。
/// 1nmごとにサンプリングされたスペクトル強度を格納する。
#[derive(Clone)]
pub struct DenselySampledSpectrum {
    /// スペクトルの値を格納するベクタ。
    values: Vec<f32>,
    lambda_min: f32,
    lambda_max: f32,
    max_value: f32,
}
impl DenselySampledSpectrum {
    /// 新しい密にサンプリングされたスペクトルを作成する。
    pub fn new(values: Vec<f32>, lambda_min: f32, lambda_max: f32) -> Self {
        assert_eq!(values.len(), (lambda_max - lambda_min) as usize);
        let max_value = values.iter().cloned().fold(0.0, f32::max);
        Self {
            values,
            lambda_min,
            lambda_max,
            max_value,
        }
    }

    /// 与えられたスペクトルをサンプリングして、密にサンプリングされたスペクトルを作成する。
    pub fn from(spectrum: &Spectrum) -> Self {
        Self::from_range(spectrum, LAMBDA_MIN, LAMBDA_MAX)
    }

    /// 指定された波長範囲で与えられたスペクトルをサンプリングして、
    /// 密にサンプリングされたスペクトルを作成する。
    pub fn from_range(spectrum: &Spectrum, lambda_min: f32, lambda_max: f32) -> Self {
        let range = (lambda_max - lambda_min) as usize;
        let mut values = vec![0.0; range];
        let mut max_value = 0.0;
        for i in 0..range {
            let lambda = lambda_min + i as f32;
            values[i] = spectrum.value(lambda);
            if values[i] > max_value {
                max_value = values[i];
            }
        }
        Self {
            values,
            lambda_min,
            lambda_max,
            max_value,
        }
    }
}
impl SpectrumTrait for DenselySampledSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        if lambda < self.lambda_min || lambda > self.lambda_max {
            return 0.0;
        }
        let index = (lambda - self.lambda_min).round() as usize;
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
