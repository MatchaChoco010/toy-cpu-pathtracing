use crate::spectrum::SpectrumTrait;

/// 定数スペクトルを表す構造体。
pub struct ConstantSpectrum {
    c: f32,
}
impl ConstantSpectrum {
    /// 新しい定数スペクトルを作成する。
    pub fn new(c: f32) -> Self {
        Self { c }
    }
}
impl SpectrumTrait for ConstantSpectrum {
    fn value(&self, _lambda: f32) -> f32 {
        self.c
    }

    fn max_value(&self) -> f32 {
        self.c
    }
}
