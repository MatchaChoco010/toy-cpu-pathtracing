use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

/// 密にサンプリングされたスペクトルを表す構造体。
/// 1nmごとにサンプリングされたスペクトル強度を格納する。
pub struct DenselySampledSpectrum {
    /// スペクトルの値を格納するベクタ。
    values: Vec<f32>,
    lambda_min: f32,
    lambda_max: f32,
    max_value: f32,
}
impl DenselySampledSpectrum {
    /// 新しい密にサンプリングされたスペクトルを作成する。
    pub fn new(spectrum: &impl SpectrumTrait) -> Self {
        Self::new_range(spectrum, LAMBDA_MIN, LAMBDA_MAX)
    }

    /// 指定された波長範囲で与えられたスペクトルをサンプリングして、
    /// 新しい密にサンプリングされたスペクトルを作成する。
    pub fn new_range(spectrum: &impl SpectrumTrait, lambda_min: f32, lambda_max: f32) -> Self {
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

    pub fn new_x() -> Self {
        todo!()
    }

    pub fn new_y() -> Self {
        todo!()
    }

    pub fn new_z() -> Self {
        todo!()
    }

    pub fn new_d(temperature: f32) -> Self {
        todo!()
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
