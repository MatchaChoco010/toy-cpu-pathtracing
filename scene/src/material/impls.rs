//! 具体的なマテリアル実装を定義するモジュール。

mod emissive_material;
mod glass_material;
mod lambert_material;
mod metal_material;
mod plastic_material;

pub use emissive_material::EmissiveMaterial;
pub use glass_material::{GlassMaterial, GlassType};
pub use lambert_material::LambertMaterial;
pub use metal_material::{MetalMaterial, MetalType};
pub use plastic_material::PlasticMaterial;
