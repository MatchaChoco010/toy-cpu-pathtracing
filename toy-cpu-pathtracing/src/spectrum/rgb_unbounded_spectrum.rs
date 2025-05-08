//! RGBから一般の非有界なスペクトルを生成するモジュール。

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

#[derive(Clone)]
pub struct RgbUnboundedSpectrum {}
impl SpectrumTrait for RgbUnboundedSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        todo!()
    }

    fn max_value(&self) -> f32 {
        todo!()
    }
}
