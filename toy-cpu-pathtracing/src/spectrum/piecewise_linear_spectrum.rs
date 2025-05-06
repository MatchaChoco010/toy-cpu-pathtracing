use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

/// 線形補間されたスペクトルを表す構造体。
pub struct PiecewiseLinearSpectrum {
    lambdas: Vec<f32>,
    values: Vec<f32>,
}
impl PiecewiseLinearSpectrum {
    /// 新しい線形補間されたスペクトルを作成する。
    pub fn new(lambdas: Vec<f32>, values: Vec<f32>) -> Self {
        assert_eq!(lambdas.len(), values.len());
        Self { lambdas, values }
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
