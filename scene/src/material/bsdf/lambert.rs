//! 正規化ランバート反射BSDF実装。

use std::f32::consts::PI;

use math::{ShadingNormalTangent, Vector3};
use spectrum::SampledSpectrum;

use super::{BsdfSample, BsdfSampleType};

/// Z軸方向を法線方向として、半球状のコサイン重点サンプリングを行う。
fn sample_cosine_hemisphere(uv: glam::Vec2) -> Vector3<ShadingNormalTangent> {
    use std::f32::consts::PI;
    let r = uv.x.sqrt();
    let theta = 2.0 * PI * uv.y;
    Vector3::new(r * theta.cos(), r * theta.sin(), (1.0 - uv.x).sqrt())
}

/// 正規化ランバート反射の純粋なBSDF計算を行う構造体。
/// パラメータは外部から与えられ、SurfaceInteractionに依存しない。
pub struct NormalizedLambertBsdf {
    /// 反射率スペクトル
    albedo: SampledSpectrum,
}
impl NormalizedLambertBsdf {
    /// 新しいNormalizedLambertBsdfを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率スペクトル
    pub fn new(albedo: SampledSpectrum) -> Self {
        Self { albedo }
    }

    /// BSDF方向サンプリングを行う。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `uv` - ランダムサンプル
    pub fn sample(&self, wo: &Vector3<ShadingNormalTangent>, uv: glam::Vec2) -> Option<BsdfSample> {
        let wo_cos_n = wo.z();
        if wo_cos_n == 0.0 {
            return None;
        }

        // ノーマルマップ接空間でのコサイン半球サンプリング
        let wi = sample_cosine_hemisphere(uv);
        let wi = if wo_cos_n < 0.0 {
            Vector3::new(wi.x(), wi.y(), -wi.z())
        } else {
            wi
        };

        // ノーマルマップ接空間でのコサイン項チェック
        let wi_cos_n = wi.z();
        if wi_cos_n == 0.0 {
            return None;
        }

        if wo_cos_n.signum() != wi_cos_n.signum() {
            return None; // 同じ半球内でない場合は無効
        }

        // BSDFの値を計算（cosine項を含む）
        let f = self.albedo.clone() * wi_cos_n.abs() / PI;

        // PDFを計算
        let pdf = wi_cos_n.abs() / PI;

        Some(BsdfSample::new(f, wi, pdf, BsdfSampleType::Diffuse))
    }

    /// BSDF値を評価する。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `wi` - 入射方向（ノーマルマップ接空間）
    pub fn evaluate(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> SampledSpectrum {
        let wo_cos_n = wo.z();
        let wi_cos_n = wi.z();

        if wo_cos_n == 0.0 || wi_cos_n == 0.0 {
            return SampledSpectrum::zero();
        }

        // 同じ半球内でない場合は反射しない
        if wo_cos_n.signum() != wi_cos_n.signum() {
            return SampledSpectrum::zero();
        }

        // BSDFの値を計算（cosine項を含む）
        self.albedo.clone() * wi_cos_n.abs() / PI
    }

    /// BSDF PDFを計算する。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（ノーマルマップ接空間）
    /// - `wi` - 入射方向（ノーマルマップ接空間）
    pub fn pdf(
        &self,
        wo: &Vector3<ShadingNormalTangent>,
        wi: &Vector3<ShadingNormalTangent>,
    ) -> f32 {
        let wo_cos_n = wo.z();
        let wi_cos_n = wi.z();

        if wo_cos_n == 0.0 || wi_cos_n == 0.0 {
            return 0.0;
        }

        // 同じ半球内でない場合はPDF = 0
        if wo_cos_n.signum() != wi_cos_n.signum() {
            return 0.0;
        }

        wi_cos_n.abs() / PI
    }
}
