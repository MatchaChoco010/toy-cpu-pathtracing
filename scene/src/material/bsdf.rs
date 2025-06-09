//! 純粋なBSDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

use std::f32::consts::PI;

use math::{Tangent, Vector3};
use spectrum::SampledSpectrum;

use crate::BsdfSample;

/// Y軸方向を法線方向として、半球状のコサイン充填サンプリングを行う。
fn sample_cosine_hemisphere(uv: glam::Vec2) -> Vector3<Tangent> {
    let r = uv.x.sqrt();
    let theta = 2.0 * PI * uv.y;
    Vector3::new(r * theta.cos(), (1.0 - uv.x).sqrt(), r * theta.sin())
}

/// 正規化ランバート反射の純粋なBSDF計算を行う構造体。
/// パラメータは外部から与えられ、SurfaceInteractionに依存しない。
#[derive(Default)]
pub struct NormalizedLambertBsdf;
impl NormalizedLambertBsdf {
    /// 新しいNormalizedLambertBsdfを作成する。
    pub fn new() -> Self {
        Self
    }

    /// BSDF方向サンプリングを行う。
    ///
    /// # Arguments
    /// - `albedo` - 反射率スペクトル
    /// - `wo` - 出射方向（接空間）
    /// - `uv` - ランダムサンプル
    pub fn sample(
        &self,
        albedo: &SampledSpectrum,
        wo: &Vector3<Tangent>,
        uv: glam::Vec2,
    ) -> Option<BsdfSample> {
        if wo.y() == 0.0 {
            // woが完全に接戦方向の場合はBsdfをサンプリングしない。
            return None;
        }

        // ランダムな方向をサンプリングする。
        let wi = sample_cosine_hemisphere(uv);
        let wi = if wo.y() < 0.0 {
            // woが法線と逆の方向の場合はwiを反転する。
            Vector3::new(wi.x(), -wi.y(), wi.z())
        } else {
            wi
        };

        if wi.y() == 0.0 {
            // wiが完全に接戦方向の場合はBsdfをサンプリングしない。
            return None;
        }

        // BSDFの値を計算する。
        let f = albedo.clone() / PI;

        // pdfを計算する。
        let cos_theta = wi.y().abs();
        let pdf = cos_theta / PI;

        Some(BsdfSample::Bsdf { f, pdf, wi })
    }

    /// BSDF値を評価する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率スペクトル
    /// - `wo` - 出射方向（接空間）
    /// - `wi` - 入射方向（接空間）
    pub fn evaluate(
        &self,
        albedo: &SampledSpectrum,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
    ) -> SampledSpectrum {
        if wo.y() == 0.0 || wi.y() == 0.0 {
            // woまたはwiが完全に接戦方向の場合はBSDFを評価しない。
            return SampledSpectrum::zero();
        }

        if wo.y().signum() != wi.y().signum() {
            // woとwiが逆方向の場合はBSDFを評価しない。
            return SampledSpectrum::zero();
        }

        // BSDFの値を計算する。
        albedo.clone() / PI
    }

    /// BSDF PDFを計算する。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（接空間）
    /// - `wi` - 入射方向（接空間）
    pub fn pdf(&self, wo: &Vector3<Tangent>, wi: &Vector3<Tangent>) -> f32 {
        if wo.y() == 0.0 || wi.y() == 0.0 {
            // woまたはwiが完全に接戦方向の場合はサンプリングされないのでpdfは0とする。
            return 0.0;
        }

        if wo.y().signum() != wi.y().signum() {
            // woとwiが逆方向の場合はpdfはBSDFの値が0なのでpdfも0。
            return 0.0;
        }

        // pdfを計算する。
        let cos_theta = wi.y().abs();
        cos_theta / PI
    }
}
