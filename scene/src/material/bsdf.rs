//! 純粋なBSDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

use std::f32::consts::PI;

use math::{Normal, Tangent, Vector3};
use spectrum::SampledSpectrum;

use crate::BsdfSample;

/// Z軸方向を法線方向として、半球状のコサイン充填サンプリングを行う。
fn sample_cosine_hemisphere(uv: glam::Vec2) -> Vector3<Tangent> {
    let r = uv.x.sqrt();
    let theta = 2.0 * PI * uv.y;
    Vector3::new(r * theta.cos(), r * theta.sin(), (1.0 - uv.x).sqrt())
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
    /// - `normal_map` - 表面法線（接空間）
    pub fn sample(
        &self,
        albedo: &SampledSpectrum,
        wo: &Vector3<Tangent>,
        uv: glam::Vec2,
        normal_map: &Normal<Tangent>,
    ) -> Option<BsdfSample> {
        // 標準的なBSDFサンプリングを実行
        if wo.z() == 0.0 {
            return None;
        }

        let wi = sample_cosine_hemisphere(uv);
        let wi = if wo.z() < 0.0 {
            Vector3::new(wi.x(), wi.y(), -wi.z())
        } else {
            wi
        };

        if wi.z() == 0.0 {
            return None;
        }

        // BSDFの値を計算
        let f = albedo.clone() / PI;

        // 法線マップが適用されている場合、PDFを調整
        let normal_vec = normal_map.to_vec3().normalize();
        let pdf = if (normal_vec - glam::Vec3::Z).length() < 1e-6 {
            // デフォルト法線の場合は標準PDF
            wi.z().abs() / PI
        } else {
            // 法線マップ適用時は新しい法線でPDFを計算
            wi.to_vec3().dot(normal_vec).abs() / PI
        };

        Some(BsdfSample::Bsdf {
            f,
            pdf,
            wi,
            normal: *normal_map,
        })
    }

    /// BSDF値を評価する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率スペクトル
    /// - `wo` - 出射方向（接空間）
    /// - `wi` - 入射方向（接空間）
    /// - `normal_map` - 表面法線（接空間）
    pub fn evaluate(
        &self,
        albedo: &SampledSpectrum,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        normal_map: &Normal<Tangent>,
    ) -> SampledSpectrum {
        // 標準的なバリデーション
        if wo.z() == 0.0 || wi.z() == 0.0 {
            return SampledSpectrum::zero();
        }

        if wo.z().signum() != wi.z().signum() {
            return SampledSpectrum::zero();
        }

        // BSDFの値を計算（標準ランバート）
        albedo.clone() / PI
    }

    /// BSDF PDFを計算する。
    ///
    /// # Arguments
    /// - `wo` - 出射方向（接空間）
    /// - `wi` - 入射方向（接空間）
    /// - `normal_map` - 表面法線（接空間）
    pub fn pdf(
        &self,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        normal_map: &Normal<Tangent>,
    ) -> f32 {
        // 標準的なバリデーション
        if wo.z() == 0.0 || wi.z() == 0.0 {
            return 0.0;
        }

        if wo.z().signum() != wi.z().signum() {
            return 0.0;
        }

        // 法線マップが適用されている場合、PDFを調整
        let normal_vec = normal_map.to_vec3().normalize();
        if (normal_vec - glam::Vec3::Z).length() < 1e-6 {
            // デフォルト法線の場合は標準PDF
            wi.z().abs() / PI
        } else {
            // 法線マップ適用時は新しい法線でPDFを計算
            wi.to_vec3().dot(normal_vec).abs() / PI
        }
    }
}
