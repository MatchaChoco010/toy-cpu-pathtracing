//! 純粋なBSDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

mod conductor;
mod dielectric;
mod generalized_schlick;
mod lambert;

pub use conductor::{ConductorBsdf, fresnel_complex};
pub use dielectric::DielectricBsdf;
pub use lambert::NormalizedLambertBsdf;

use math::ShadingNormalTangent;

/// BSDFサンプルのタイプを表す列挙型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BsdfSampleType {
    /// 拡散反射（例：Lambert BSDF）
    Diffuse,
    /// 完全鏡面反射・透過（例：デルタ関数BSDF）
    Specular,
    /// 光沢反射（例：マイクロファセットBSDF）
    Glossy,
}

/// 散乱モードを表す列挙型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScatterMode {
    /// 反射のみ
    R,
    /// 反射と透過
    RT,
}

// Bsdfのサンプリング結果を表す構造体。
#[derive(Debug, Clone)]
pub struct BsdfSample {
    /// BSDF値
    pub f: spectrum::SampledSpectrum,
    /// サンプルされた入射方向（ノーマルマップ接空間）
    pub wi: math::Vector3<ShadingNormalTangent>,
    /// 確率密度関数値（スペキュラの場合は1.0）
    pub pdf: f32,
    /// BSDFサンプルのタイプ
    pub sample_type: BsdfSampleType,
}

impl BsdfSample {
    /// 新しいBsdfSampleを作成する。
    pub fn new(
        f: spectrum::SampledSpectrum,
        wi: math::Vector3<ShadingNormalTangent>,
        pdf: f32,
        sample_type: BsdfSampleType,
    ) -> Self {
        Self {
            f,
            wi,
            pdf,
            sample_type,
        }
    }

    /// Specularのサンプリングかどうか。
    pub fn is_specular(&self) -> bool {
        self.sample_type == BsdfSampleType::Specular
    }

    /// 非Specularのサンプリングかどうか。
    pub fn is_non_specular(&self) -> bool {
        self.sample_type == BsdfSampleType::Diffuse || self.sample_type == BsdfSampleType::Glossy
    }
}
