//! 新しいマテリアルシステムを定義するモジュール。

mod bsdf;
mod edf;
mod impls;
mod parameter;
mod traits;

// BSDF/EDF実装
pub use bsdf::{ConductorBsdf, NormalizedLambertBsdf};
pub use edf::UniformEdf;

// マテリアルパラメータ
pub use parameter::{FloatParameter, NormalParameter, SpectrumParameter};

// マテリアルトレイト
pub use traits::{BsdfSurfaceMaterial, EmissiveSurfaceMaterial, Material, SurfaceMaterial};

// 具体的なマテリアル実装
pub use impls::{EmissiveMaterial, LambertMaterial, MetalMaterial, MetalType};
