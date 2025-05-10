//! シーンを構成する要素のPrimitiveの関連構造体を定義するモジュール。

mod bvh;
mod create_desc;
mod impls;
mod repository;
mod samples;
mod traits;

pub use bvh::{Intersection, PrimitiveBvh};
pub use create_desc::CreatePrimitiveDesc;
pub use repository::{PrimitiveIndex, PrimitiveRepository};
pub use samples::{GeometryInfo, Interaction, LightIrradiance, LightSampleRadiance};
pub use traits::*;
