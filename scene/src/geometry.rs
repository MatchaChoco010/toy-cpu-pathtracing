//! シーンのジオメトリデータに関連する実装を行うモジュール。

mod intersection;
mod repository;
mod traits;

pub mod impls;

pub use intersection::Intersection;
pub use repository::{GeometryIndex, GeometryRepository};
pub use traits::Geometry;
