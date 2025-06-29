//! 線形補間されたスペクトルを定義するモジュール。

use std::sync::Arc;

use crate::{
    ConstantSpectrum, DenselySampledSpectrum, LAMBDA_MIN, N_SPECTRUM_DENSELY_SAMPLES,
    inner_product, presets,
    spectrum::{Spectrum, SpectrumTrait},
};

/// 線形補間されたスペクトルを表す構造体。
#[derive(Clone)]
pub struct PiecewiseLinearSpectrum {
    lambdas: Vec<f32>,
    values: Vec<f32>,
}
impl PiecewiseLinearSpectrum {
    /// 新しい線形補間されたスペクトルを作成する。
    pub fn new(lambdas: Vec<f32>, values: Vec<f32>) -> Spectrum {
        assert_eq!(lambdas.len(), values.len());
        Arc::new(Self { lambdas, values })
    }

    /// 波長の配列と値のペアの配列から新しい線形補間されたスペクトルを作成する。
    pub fn from_lambda_and_value(lambdas: &[f32], values: &[f32]) -> Spectrum {
        assert_eq!(lambdas.len(), values.len());
        Arc::new(Self {
            lambdas: lambdas.to_vec(),
            values: values.to_vec(),
        })
    }

    /// 波長と値が交互に並んだ配列から新しい線形補間されたスペクトルを作成する。
    pub fn from_interleaved(lambdas_and_values: &[f32], normalized: bool) -> Spectrum {
        assert_eq!(lambdas_and_values.len() % 2, 0);
        let len = lambdas_and_values.len() / 2;
        let (lambdas, values) = lambdas_and_values.chunks_exact(2).fold(
            (Vec::with_capacity(len), Vec::with_capacity(len)),
            |(mut lambdas, mut values), chunk| {
                lambdas.push(chunk[0]);
                values.push(chunk[1]);
                (lambdas, values)
            },
        );
        let spec = Self { lambdas, values };

        // 光源用の正規化されたスペクトルの場合、Yとの内積で割る
        if normalized {
            let y = presets::y();
            let y_self = inner_product(&spec, &y);
            if y_self == 0.0 {
                ConstantSpectrum::new(0.0)
            } else {
                let mut values = [0.0; N_SPECTRUM_DENSELY_SAMPLES];
                for i in 0..N_SPECTRUM_DENSELY_SAMPLES {
                    let lambda = LAMBDA_MIN + i as f32;
                    values[i] = spec.value(lambda) / y_self;
                }
                DenselySampledSpectrum::new(values)
            }
        } else {
            Arc::new(spec)
        }
    }
}
impl SpectrumTrait for PiecewiseLinearSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        if self.lambdas.is_empty() {
            return 0.0;
        }
        if lambda < self.lambdas[0] || lambda > self.lambdas[self.lambdas.len() - 1] {
            return 0.0;
        }
        let mut i = 0;
        while i < self.lambdas.len() - 1 && self.lambdas[i + 1] < lambda {
            i += 1;
        }
        let t = (lambda - self.lambdas[i]) / (self.lambdas[i + 1] - self.lambdas[i]);
        self.values[i] * (1.0 - t) + self.values[i + 1] * t
    }

    fn max_value(&self) -> f32 {
        self.values
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0)
    }
}
