//! 導体（金属）BSDF実装。

use math::{ShadingNormalTangent, Vector3};
use spectrum::SampledSpectrum;

use crate::material::{
    bsdf::{BsdfSample, BsdfSampleType},
    common::{
        cos_phi, cos2_theta, half_vector, reflect, same_hemisphere, sample_uniform_disk_polar,
        sin_phi, tan2_theta,
    },
};

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
    /// X方向のroughness parameter (α_x)
    alpha_x: f32,
    /// Y方向のroughness parameter (α_y)
    alpha_y: f32,
}
impl ConductorBsdf {
    /// ConductorBsdfを作成する。
    /// alpha_x, alpha_yが0.0に近い場合は完全鏡面反射となる。
    ///
    /// # Arguments
    /// - `eta` - 屈折率の実部（スペクトル依存）
    /// - `k` - 屈折率の虚部（消散係数、スペクトル依存）
    /// - `alpha_x` - X方向のroughness parameter
    /// - `alpha_y` - Y方向のroughness parameter
    pub fn new(eta: SampledSpectrum, k: SampledSpectrum, alpha_x: f32, alpha_y: f32) -> Self {
        Self {
            eta,
            k,
            alpha_x,
            alpha_y,
        }
    }

    /// 表面が事実上滑らかかどうかを判定する。
    fn effectively_smooth(&self) -> bool {
        self.alpha_x.max(self.alpha_y) < 1e-3
    }

    /// Trowbridge-Reitz分布関数 D(ωm)を計算する。
    fn microfacet_distribution(&self, wm: &Vector3<ShadingNormalTangent>) -> f32 {
        let tan2_theta = tan2_theta(wm);
        if !tan2_theta.is_finite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wm).powi(2);
        let e = tan2_theta
            * (cos_phi(wm).powi(2) / self.alpha_x.powi(2)
                + sin_phi(wm).powi(2) / self.alpha_y.powi(2));

        1.0 / (std::f32::consts::PI * self.alpha_x * self.alpha_y * cos4_theta * (1.0 + e).powi(2))
    }

    /// Lambda関数を計算する。
    fn lambda(&self, w: &Vector3<ShadingNormalTangent>) -> f32 {
        let tan2_theta = tan2_theta(w);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let alpha2 = (cos_phi(w) * self.alpha_x).powi(2) + (sin_phi(w) * self.alpha_y).powi(2);
        ((1.0 + alpha2 * tan2_theta).sqrt() - 1.0) / 2.0
    }

    /// 単方向マスキング関数 G1(ω)を計算する。
    fn masking_g1(&self, w: &Vector3<ShadingNormalTangent>) -> f32 {
        1.0 / (1.0 + self.lambda(w))
    }

    /// 双方向マスキング・シャドウイング関数 G(ωo, ωi)を計算する。
    fn masking_shadowing_g(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    /// 可視法線分布 D_ω(ωm)を計算する。
    fn visible_normal_distribution(
        &self,
        w: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        let cos_theta_w = w.z().abs();
        if cos_theta_w == 0.0 {
            return 0.0;
        }
        self.masking_g1(w) / cos_theta_w * self.microfacet_distribution(wm) * w.dot(wm).abs()
    }

    /// 可視法線をサンプリングする。
    fn sample_visible_normal(
        &self,
        w: &Vector3<ShadingNormalTangent>,
        u: glam::Vec2,
    ) -> Vector3<ShadingNormalTangent> {
        // wを半球構成に変換
        let mut wh: Vector3<ShadingNormalTangent> =
            Vector3::new(self.alpha_x * w.x(), self.alpha_y * w.y(), w.z()).normalize();
        if wh.z() < 0.0 {
            wh = -wh;
        }

        // 可視法線サンプリング用の直交基底を見つける
        let t1 = if wh.z() < 0.99999 {
            Vector3::new(0.0, 0.0, 1.0).cross(wh).normalize()
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let t2 = wh.cross(t1);

        // 単位円盤上に均等分布点を生成
        let p = sample_uniform_disk_polar(u);

        // 半球投影を可視法線サンプリング用にワープ
        let h = (1.0 - p.x * p.x).max(0.0).sqrt();
        let lerp_factor = (1.0 + wh.z()) / 2.0;
        let p_y = h * (1.0 - lerp_factor) + p.y * lerp_factor;

        // 半球に再投影し、法線を楕円体構成に変換
        let pz = (1.0 - p.x * p.x - p_y * p_y).max(0.0).sqrt();
        let nh = t1 * p.x + t2 * p_y + wh * pz;

        Vector3::new(
            self.alpha_x * nh.x(),
            self.alpha_y * nh.y(),
            (1e-6_f32).max(nh.z()),
        )
        .normalize()
    }

    /// BSDF方向サンプリングを行う。
    /// 表面の粗さに応じて完全鏡面またはマイクロファセットサンプリングを使用。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `uv` - ランダムサンプル
    pub fn sample(&self, wo: &Vector3<ShadingNormalTangent>, uv: glam::Vec2) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        if self.effectively_smooth() {
            // 完全鏡面反射
            self.sample_perfect_specular(wo)
        } else {
            // マイクロファセットサンプリング
            self.sample_microfacet(wo, uv)
        }
    }

    /// 完全鏡面反射サンプリング。
    fn sample_perfect_specular(&self, wo: &Vector3<ShadingNormalTangent>) -> Option<BsdfSample> {
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

        Some(BsdfSample::new(f, wi, 1.0, BsdfSampleType::Specular))
    }

    /// マイクロファセットサンプリング（Torrance-Sparrow model）。
    fn sample_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        uv: glam::Vec2,
    ) -> Option<BsdfSample> {
        // 可視法線をサンプリング
        let wm = self.sample_visible_normal(wo, uv);

        // 鏡面反射方向を計算
        let wi = reflect(wo, &wm);

        // 同じ半球にあるかチェック
        if !same_hemisphere(wo, &wi) {
            return None;
        }

        // Torrance-Sparrow BRDF値とPDFを計算
        let f = self.evaluate_torrance_sparrow(wo, &wi, &wm);
        let pdf = self.pdf_microfacet(wo, &wi);

        Some(BsdfSample::new(f, wi, pdf, BsdfSampleType::Glossy))
    }

    /// Torrance-Sparrow BRDF を評価する。
    fn evaluate_torrance_sparrow(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
        wm: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let cos_theta_o = wo.z().abs();
        let cos_theta_i = wi.z().abs();

        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return SampledSpectrum::zero();
        }

        // Fresnel項: F(ωo·ωm)
        let fresnel = fresnel_complex((wo.dot(wm)).abs(), &self.eta, &self.k);

        // 分布項: D(ωm)
        let distribution = self.microfacet_distribution(wm);

        // マスキング・シャドウイング項: G(ωo, ωi)
        let masking_shadowing = self.masking_shadowing_g(wo, wi);

        // Torrance-Sparrow BRDF: D(ωm) * F(ωo·ωm) * G(ωo, ωi) / (4 * cos θi * cos θo)
        fresnel * distribution * masking_shadowing / (4.0 * cos_theta_i * cos_theta_o)
    }

    /// BSDF値を評価する。
    /// 完全鏡面の場合は0、マイクロファセットの場合はTorrance-Sparrow BRDFを返す。
    pub fn evaluate(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        if self.effectively_smooth() {
            // 完全鏡面反射の場合、evaluate()は常に0を返す（デルタ関数のため）
            SampledSpectrum::zero()
        } else {
            // マイクロファセットの場合、Torrance-Sparrow BRDFを評価
            self.evaluate_microfacet(wo, wi)
        }
    }

    /// マイクロファセットBRDFを評価する。
    fn evaluate_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let cos_theta_o = wo.z().abs();
        let cos_theta_i = wi.z().abs();

        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return SampledSpectrum::zero();
        }

        // 同じ半球にあるかチェック
        if !same_hemisphere(wo, wi) {
            return SampledSpectrum::zero();
        }

        // ハーフベクトルを計算
        let wm = match half_vector(wo, wi) {
            Some(wm) => wm,
            None => return SampledSpectrum::zero(),
        };

        self.evaluate_torrance_sparrow(wo, wi, &wm)
    }

    /// BSDF PDFを計算する。
    /// 完全鏡面の場合は0、マイクロファセットの場合はTorrance-Sparrow PDFを返す。
    pub fn pdf(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        if self.effectively_smooth() {
            // 完全鏡面反射の場合、PDF()は常に0を返す（デルタ関数のため）
            0.0
        } else {
            // マイクロファセットの場合、Torrance-Sparrow PDFを計算
            self.pdf_microfacet(wo, wi)
        }
    }

    /// マイクロファセットPDFを計算する。
    fn pdf_microfacet(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        // 同じ半球にあるかチェック
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        // ハーフベクトルを計算
        let wm = match half_vector(wo, wi) {
            Some(wm) => wm,
            None => return 0.0,
        };

        // 可視法線分布のPDF
        let visible_normal_pdf = self.visible_normal_distribution(wo, &wm);

        // ヤコビアン変換: dωm/dωi = 1/(4|ωo·ωm|)
        let jacobian = 4.0 * (wo.dot(wm)).abs();
        if jacobian == 0.0 {
            return 0.0;
        }

        visible_normal_pdf / jacobian
    }
}
