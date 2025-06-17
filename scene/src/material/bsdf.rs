//! 純粋なBSDF実装（SurfaceInteractionに依存しない）を定義するモジュール。

mod conductor;
mod lambert;

pub use conductor::ConductorBsdf;
pub use lambert::NormalizedLambertBsdf;

use math::NormalMapTangent;

// Bsdfのサンプリング結果を表す列挙型。
#[derive(Debug, Clone)]
pub enum BsdfSample {
    Bsdf {
        f: spectrum::SampledSpectrum,
        pdf: f32,
        wi: math::Vector3<NormalMapTangent>,
    },
    Specular {
        f: spectrum::SampledSpectrum,
        wi: math::Vector3<NormalMapTangent>,
    },
}
