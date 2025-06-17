//! 導体（金属）BSDF実装。

use math::{NormalMapTangent, Vector3};
use spectrum::SampledSpectrum;

use super::BsdfSample;

/// 簡単な複素数実装
#[derive(Debug, Clone, Copy)]
struct Complex {
    real: f32,
    imag: f32,
}

impl Complex {
    fn new(real: f32, imag: f32) -> Self {
        Self { real, imag }
    }

    fn norm(self) -> f32 {
        self.real * self.real + self.imag * self.imag
    }

    fn sqrt(self) -> Self {
        let r = (self.real * self.real + self.imag * self.imag).sqrt();
        let theta = self.imag.atan2(self.real);
        let sqrt_r = r.sqrt();
        let half_theta = theta * 0.5;
        Self::new(sqrt_r * half_theta.cos(), sqrt_r * half_theta.sin())
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imag + other.imag)
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imag - other.imag)
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }
}

impl std::ops::Mul<f32> for Complex {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.real * scalar, self.imag * scalar)
    }
}

impl std::ops::Div for Complex {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let denom = other.real * other.real + other.imag * other.imag;
        if denom == 0.0 {
            Self::new(0.0, 0.0)
        } else {
            Self::new(
                (self.real * other.real + self.imag * other.imag) / denom,
                (self.imag * other.real - self.real * other.imag) / denom,
            )
        }
    }
}

/// 複素屈折率を用いたFresnel反射率の計算。
/// pbrt-v4のFrComplex関数に相当。
///
/// # Arguments
/// - `cos_theta_i` - 入射角のコサイン値
/// - `eta` - 屈折率の実部
/// - `k` - 屈折率の虚部（消散係数）
pub fn fresnel_complex(
    cos_theta_i: f32,
    eta: &SampledSpectrum,
    k: &SampledSpectrum,
) -> SampledSpectrum {
    use spectrum::N_SPECTRUM_SAMPLES;

    let cos_theta_i = cos_theta_i.clamp(0.0, 1.0);
    let mut values = [0.0; N_SPECTRUM_SAMPLES];

    // 各波長について個別にFresnel反射率を計算
    for i in 0..N_SPECTRUM_SAMPLES {
        let complex_eta = Complex::new(eta.value(i), k.value(i));

        // Snell's law で sin^2(theta_t) を計算
        let sin2_theta_i = 1.0 - cos_theta_i * cos_theta_i;
        let sin2_theta_t = Complex::new(sin2_theta_i, 0.0) / (complex_eta * complex_eta);
        let cos_theta_t = (Complex::new(1.0, 0.0) - sin2_theta_t).sqrt();

        // Fresnel方程式
        let r_parl =
            (complex_eta * cos_theta_i - cos_theta_t) / (complex_eta * cos_theta_i + cos_theta_t);
        let r_perp = (Complex::new(cos_theta_i, 0.0) - complex_eta * cos_theta_t)
            / (Complex::new(cos_theta_i, 0.0) + complex_eta * cos_theta_t);

        // 偏光の平均（norm は |z|^2 を計算）
        values[i] = (r_parl.norm() + r_perp.norm()) * 0.5;
    }

    SampledSpectrum::from(values)
}

/// 導体（金属）の純粋なBSDF計算を行う構造体。
/// パラメータは外部から与えられ、SurfaceInteractionに依存しない。
pub struct ConductorBsdf {
    /// 屈折率の実部（スペクトル依存）
    eta: SampledSpectrum,
    /// 屈折率の虚部（消散係数、スペクトル依存）
    k: SampledSpectrum,
}

impl ConductorBsdf {
    /// 新しいConductorBsdfを作成する。
    ///
    /// # Arguments
    /// - `eta` - 屈折率の実部（スペクトル依存）
    /// - `k` - 屈折率の虚部（消散係数、スペクトル依存）
    pub fn new(eta: SampledSpectrum, k: SampledSpectrum) -> Self {
        Self { eta, k }
    }

    /// BSDF方向サンプリングを行う。
    /// 完全鏡面の場合のみを実装（マイクロファセットは後で実装）。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `_uv` - ランダムサンプル（完全鏡面なので未使用）
    pub fn sample(&self, wo: &Vector3<NormalMapTangent>, _uv: glam::Vec2) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        // 完全鏡面反射: wi = (-wo.x, -wo.y, wo.z)
        let wi = Vector3::new(-wo.x(), -wo.y(), wo.z());
        let wi_cos_n = wi.z();

        if wi_cos_n == 0.0 {
            return None;
        }

        // Fresnel反射率を計算
        let fresnel = fresnel_complex(wi_cos_n.abs(), &self.eta, &self.k);

        // BSDF値: F / |cos(theta_i)|
        let f = fresnel / wi_cos_n.abs();

        Some(BsdfSample::Specular { f, wi })
    }

    /// BSDF値を評価する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    pub fn evaluate(
        &self,
        _wo: &Vector3<NormalMapTangent>,
        _wi: &Vector3<NormalMapTangent>,
    ) -> SampledSpectrum {
        // 完全鏡面反射の場合、evaluate()は常に0を返す
        SampledSpectrum::zero()
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は常に0を返す（デルタ関数のため）。
    pub fn pdf(&self, _wo: &Vector3<NormalMapTangent>, _wi: &Vector3<NormalMapTangent>) -> f32 {
        // 完全鏡面反射の場合、PDF()は常に0を返す
        0.0
    }

    /// Fresnel反射率を計算する。
    ///
    /// # Arguments
    /// - `cos_theta_i` - 入射角のコサイン値
    pub fn fresnel(&self, cos_theta_i: f32) -> SampledSpectrum {
        fresnel_complex(cos_theta_i, &self.eta, &self.k)
    }
}
