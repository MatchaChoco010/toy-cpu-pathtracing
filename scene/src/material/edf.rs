//! 純粋なEDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

use spectrum::SampledSpectrum;

/// 一様EDF（全方向に一定の放射輝度）の純粋な計算を行う構造体。
/// パラメータは外部から与えられ、SurfaceInteractionに依存しない。
#[derive(Default)]
pub struct UniformEdf;
impl UniformEdf {
    /// 新しいUniformEdfを作成する。
    pub fn new() -> Self {
        Self
    }

    /// 指定方向の放射輝度を計算する。
    ///
    /// # Arguments
    /// - `radiance` - 基本放射輝度スペクトル
    /// - `intensity` - 強度乗算係数
    pub fn radiance(&self, radiance: &SampledSpectrum, intensity: f32) -> SampledSpectrum {
        radiance.clone() * intensity
    }

    /// 平均強度を計算する。
    ///
    /// # Arguments
    /// - `radiance` - 基本放射輝度スペクトル
    /// - `intensity` - 強度乗算係数
    pub fn average_intensity(&self, radiance: &SampledSpectrum, intensity: f32) -> SampledSpectrum {
        radiance.clone() * intensity
    }
}
