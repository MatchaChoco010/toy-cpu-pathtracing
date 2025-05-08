use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, SpectrumTrait};

/// 与えられた波長lambda (nm) と温度temperature (K) に対して、プランクの法則に基づいて黒体放射を計算する。
fn black_body(lambda: f32, temperature: f32) -> f32 {
    if temperature <= 0.0 {
        return 0.0;
    }
    const C: f32 = 299892458.0; // 光速 (m/s)
    const H: f32 = 6.62606957e-34; // プランク定数 (J·s)
    const KB: f32 = 1.3806488e-23; // ボルツマン定数 (J/K)

    let l = lambda * 1e-9; // 波長をメートルに変換

    let numerator = 2.0 * H * C * C;
    let denominator = l.powi(5) * ((H * C / (l * KB * temperature)).exp() - 1.0);
    numerator / denominator
}

/// 黒体スペクトルを表す構造体。
#[derive(Clone)]
pub struct BlackBodySpectrum {
    temperature: f32,
}
impl BlackBodySpectrum {
    /// 新しい黒体スペクトルを作成する。
    /// 国体の温度 (K) を引数に取る。
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}
impl SpectrumTrait for BlackBodySpectrum {
    fn value(&self, lambda: f32) -> f32 {
        black_body(lambda, self.temperature)
    }

    fn max_value(&self) -> f32 {
        let mut max_value = 0.0;
        for lambda in (LAMBDA_MIN as u32..=LAMBDA_MAX as u32).map(|l| l as f32) {
            let value = self.value(lambda);
            if value > max_value {
                max_value = value;
            }
        }
        max_value
    }
}
