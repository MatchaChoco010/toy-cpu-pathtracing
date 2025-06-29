//! 新しいマテリアルシステムを定義するモジュール。

pub mod bsdf;
pub mod common;
mod edf;
mod impls;
mod normal_mapping;
mod parameter;
mod traits;

// BSDF/EDF実装
pub use bsdf::{BsdfSample, BsdfSampleType, ConductorBsdf, DielectricBsdf, NormalizedLambertBsdf};
pub use edf::UniformEdf;

// Normal mapping実装
pub use normal_mapping::{
    evaluate_microfacet_normal_mapping, pdf_microfacet_normal_mapping,
    sample_microfacet_normal_mapping,
};

// マテリアルパラメータ
pub use parameter::{FloatParameter, NormalParameter, SpectrumParameter};

// マテリアルトレイト
pub use traits::{BsdfSurfaceMaterial, EmissiveSurfaceMaterial, Material, SurfaceMaterial};

// 具体的なマテリアル実装
pub use impls::{
    EmissiveMaterial, GlassMaterial, GlassType, LambertMaterial, MetalMaterial, MetalType,
    PlasticMaterial, SimpleClearcoatPbrMaterial, SimplePbrMaterial,
};
