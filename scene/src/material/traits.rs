//! マテリアルトレイト階層を定義するモジュール。

use math::{ShadingTangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};
use std::sync::Arc;

use crate::{MaterialDirectionSample, MaterialEvaluationResult, SceneId, SurfaceInteraction};

/// 基底マテリアルtrait - 全マテリアルが実装する。
/// 複数のシーンで使い回せるよう、トレイト自体にはIdジェネリクスを持たない。
pub trait SurfaceMaterial: Send + Sync + std::any::Any {
    /// 型の安全なダウンキャストのためのas_any実装
    fn as_any(&self) -> &dyn std::any::Any;
}

/// マテリアルアクセスヘルパー関数
impl dyn SurfaceMaterial {
    /// BSDF実装への安全なダウンキャスト。
    pub fn as_bsdf_material<Id: SceneId>(&self) -> Option<&dyn BsdfSurfaceMaterial<Id>> {
        self.as_any()
            .downcast_ref::<crate::material::impls::LambertMaterial>()
            .map(|m| m as &dyn BsdfSurfaceMaterial<Id>)
    }

    /// Emissive実装への安全なダウンキャスト。
    pub fn as_emissive_material<Id: SceneId>(&self) -> Option<&dyn EmissiveSurfaceMaterial<Id>> {
        self.as_any()
            .downcast_ref::<crate::material::impls::EmissiveMaterial>()
            .map(|m| m as &dyn EmissiveSurfaceMaterial<Id>)
    }
}

/// BSDF表面反射計算を提供するマテリアルトレイト。
/// 散乱・反射・透過の計算を担当する。
pub trait BsdfSurfaceMaterial<Id: SceneId>: SurfaceMaterial {
    /// BSDF方向サンプリングを行う。
    ///
    /// # Arguments
    /// - `uv` - 2次元乱数サンプル
    /// - `lambda` - サンプルされた波長
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `shading_point` - シェーディング点情報
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
    ) -> Option<MaterialDirectionSample>;

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
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
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
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<Id, ShadingTangent>,
    ) -> f32;
}

/// EDF発光計算を提供するマテリアルトレイト。
/// 表面からの光の放射を担当する。
pub trait EmissiveSurfaceMaterial<Id: SceneId>: SurfaceMaterial {
    /// 指定方向の放射輝度を計算する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    /// - `wo` - 出射方向（シェーディング接空間）
    /// - `light_sample_point` - ライトサンプル点情報
    fn radiance(
        &self,
        lambda: &SampledWavelengths,
        wo: Vector3<ShadingTangent>,
        light_sample_point: &SurfaceInteraction<Id, ShadingTangent>,
    ) -> SampledSpectrum;

    /// 平均強度を計算する。
    ///
    /// # Arguments
    /// - `lambda` - サンプルされた波長
    fn average_intensity(&self, lambda: &SampledWavelengths) -> SampledSpectrum;
}

/// 簡潔なマテリアル型エイリアス
pub type Material = Arc<dyn SurfaceMaterial>;
