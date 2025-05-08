use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

#[derive(Clone)]
pub struct RgbIlluminantSpectrum {}
impl SpectrumTrait for RgbIlluminantSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        todo!()
    }

    fn max_value(&self) -> f32 {
        todo!()
    }
}
