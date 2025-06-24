//! マテリアルトレイト階層を定義するモジュール。

use math::{Vector3, VertexNormalTangent};
use spectrum::{SampledSpectrum, SampledWavelengths};
use std::sync::Arc;

use crate::{MaterialEvaluationResult, MaterialSample, SurfaceInteraction};

/// 基底マテリアルtrait - 全マテリアルが実装する。
pub trait SurfaceMaterial: Send + Sync + std::any::Any {
    /// 型の安全なダウンキャストのためのas_any実装
    fn as_any(&self) -> &dyn std::any::Any;

    /// BSDF実装への安全なダウンキャスト。
    /// デフォルト実装はNoneを返す。各マテリアルでオーバーライドする。
    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        None
    }

    /// Emissive実装への安全なダウンキャスト。
    /// デフォルト実装はNoneを返す。各マテリアルでオーバーライドする。
    fn as_emissive_material(&self) -> Option<&dyn EmissiveSurfaceMaterial> {
        None
    }
}

/// BSDF表面反射計算を提供するマテリアルトレイト。
/// 散乱・反射・透過の計算を担当する。
pub trait BsdfSurfaceMaterial {
    /// BSDF方向サンプリングを行う。
    ///
    /// # Arguments
    /// - `uc` - 1次元乱数サンプル
    /// - `uv` - 2次元乱数サンプル
    /// - `lambda` - サンプルされた波長（分散処理のため可変）
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `shading_point` - シェーディング点情報
    fn sample(
        &self,
        uc: f32,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialSample;

    /// BSDF値を評価する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `wi` - 入射方向（シェーディング接空間）
    /// - `shading_point` - シェーディング点情報
    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> MaterialEvaluationResult;

    /// BSDF PDFを計算する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `wi` - 入射方向（シェーディング接空間）
    /// - `shading_point` - シェーディング点情報
    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<VertexNormalTangent>,
        wi: &Vector3<VertexNormalTangent>,
        shading_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> f32;

    /// Albedoスペクトルをサンプリングする。
    fn sample_albedo_spectrum(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum;
}

/// EDF発光計算を提供するマテリアルトレイト。
/// 表面からの光の放射を担当する。
pub trait EmissiveSurfaceMaterial {
    /// 指定方向の放射輝度を計算する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `light_sample_point` - ライトサンプル点情報
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        wo: Vector3<VertexNormalTangent>,
        light_sample_point: &SurfaceInteraction<VertexNormalTangent>,
    ) -> SampledSpectrum;

    /// 平均強度を計算する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum;
}

/// 簡潔なマテリアル型エイリアス
pub type Material = Arc<dyn SurfaceMaterial>;
