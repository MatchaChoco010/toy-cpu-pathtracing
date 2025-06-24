//! 新しいマテリアルシステムを定義するモジュール。

pub mod bsdf;
pub mod common;
mod edf;
mod impls;
mod parameter;
mod traits;

// BSDF/EDF実装
pub use bsdf::{BsdfSample, BsdfSampleType, ConductorBsdf, DielectricBsdf, NormalizedLambertBsdf};
pub use edf::UniformEdf;

// マテリアルパラメータ
pub use parameter::{FloatParameter, NormalParameter, SpectrumParameter};

// マテリアルトレイト
pub use traits::{BsdfSurfaceMaterial, EmissiveSurfaceMaterial, Material, SurfaceMaterial};

// 具体的なマテリアル実装
pub use impls::{
    EmissiveMaterial, GlassMaterial, GlassType, LambertMaterial, MetalMaterial, MetalType,
    PlasticMaterial,
};
