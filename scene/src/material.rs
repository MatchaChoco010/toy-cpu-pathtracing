//! 新しいマテリアルシステムを定義するモジュール。

mod bsdf;
mod edf;
mod impls;
mod traits;

// BSDF/EDF実装
pub use bsdf::NormalizedLambertBsdf;
pub use edf::UniformEdf;

// マテリアルトレイト
pub use traits::{BsdfSurfaceMaterial, EmissiveSurfaceMaterial, Material, SurfaceMaterial};

// 具体的なマテリアル実装
pub use impls::{EmissiveMaterial, LambertMaterial};
