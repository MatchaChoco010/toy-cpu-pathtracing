//! マテリアルに関連する定義を行うモジュール。

mod material;

pub mod bsdf;
pub mod edf;

pub use bsdf::{Bsdf, BsdfSample};
pub use edf::Edf;
pub use material::SurfaceMaterial;
