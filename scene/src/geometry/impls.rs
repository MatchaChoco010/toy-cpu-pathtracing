//! ジオメトリの実装をまとめたモジュール。
//!
//! 現時点では三角形メッシュのみを扱う。
//! 将来的にはヘアなどのジオメトリもここで扱いうる。
//!
//! 単純なSingleTriangleなどの小さいジオメトリはPrimitiveに直接埋め込まれるためここには含まない。

mod triangle_mesh;

pub use triangle_mesh::TriangleMesh;
