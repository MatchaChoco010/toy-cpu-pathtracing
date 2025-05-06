use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

pub struct RgbAlbedoSpectrum {}
impl SpectrumTrait for RgbAlbedoSpectrum {
    fn value(&self, lambda: f32) -> f32 {
        todo!()
    }

    fn max_value(&self) -> f32 {
        todo!()
    }
}
