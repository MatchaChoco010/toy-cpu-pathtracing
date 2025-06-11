//! 純粋なBSDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

use std::f32::consts::PI;

use math::{Normal, Tangent, Transform, Vector3};
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
        let normal_vec = normal_map.to_vec3().normalize();

        let wo_cos_n = wo.to_vec3().dot(normal_vec);
        if wo_cos_n == 0.0 {
            return None;
        }

        // 法線に対応した座標系でサンプリング
        let transform = Transform::from_normal_map(normal_map);
        let transform_inv = transform.inverse();

        // 法線座標系でのコサイン半球サンプリング
        let wi_local = sample_cosine_hemisphere(uv);
        let wi_local = if wo_cos_n < 0.0 {
            Vector3::new(wi_local.x(), wi_local.y(), -wi_local.z())
        } else {
            wi_local
        };

        // 元の接空間に変換
        let wi = transform_inv * wi_local;

        let wi_cos_n = wi.to_vec3().dot(normal_vec);
        if wi_cos_n == 0.0 {
            return None;
        }

        // BSDFの値を計算
        let f = albedo.clone() / PI;

        // PDFを計算（法線に対するコサイン項）
        let pdf = wi_cos_n.abs() / PI;

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
        let normal_vec = normal_map.to_vec3().normalize();

        // 法線に対するバリデーション
        let wo_cos_n = wo.to_vec3().dot(normal_vec);
        let wi_cos_n = wi.to_vec3().dot(normal_vec);

        if wo_cos_n == 0.0 || wi_cos_n == 0.0 {
            return SampledSpectrum::zero();
        }

        // 同じ半球内でない場合は反射しない
        if wo_cos_n.signum() != wi_cos_n.signum() {
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
        let normal_vec = normal_map.to_vec3().normalize();

        // 法線に対するバリデーション
        let wo_cos_n = wo.to_vec3().dot(normal_vec);
        let wi_cos_n = wi.to_vec3().dot(normal_vec);

        if wo_cos_n == 0.0 || wi_cos_n == 0.0 {
            return 0.0;
        }

        // 同じ半球内でない場合はPDF = 0
        if wo_cos_n.signum() != wi_cos_n.signum() {
            return 0.0;
        }

        // 法線に対するコサイン項でPDFを計算
        wi_cos_n.abs() / PI
    }
}
