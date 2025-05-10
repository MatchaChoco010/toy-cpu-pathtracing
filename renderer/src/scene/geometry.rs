//! シーンのジオメトリデータに関連する実装を行うモジュール。

mod impls;
mod intersection;
mod repository;
mod traits;

pub use impls::TriangleMesh;
pub use intersection::Intersection;
pub use repository::{GeometryIndex, GeometryRepository};
pub use traits::Geometry;
