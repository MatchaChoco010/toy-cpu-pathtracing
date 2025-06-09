//! 具体的なマテリアル実装を定義するモジュール。

mod emissive_material;
mod lambert_material;

pub use emissive_material::EmissiveMaterial;
pub use lambert_material::LambertMaterial;
