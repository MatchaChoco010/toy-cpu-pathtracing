//! シーンに含まれるジオメトリデータを管理するモジュール。
//!
//! 現時点では三角形メッシュのみを扱う。
//! 将来的にはヘアなどのジオメトリもここで扱いうる。
//!
//! 単純なSingleTriangleなどの小さいジオメトリはPrimitiveに直接埋め込まれるためここには含まない。

use std::marker::PhantomData;

mod triangle_mesh;

use crate::scene::SceneId;

pub use triangle_mesh::TriangleMesh;

/// シーンに含まれるジオメトリのIndex。
/// GeometryRepositoryからジオメトリを取得するために使える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometryIndex<Id: SceneId>(pub usize, PhantomData<Id>);
impl<Id: SceneId> GeometryIndex<Id> {
    /// GeometryIndexを新しく作成する。
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}

/// シーンに含まれるジオメトリの列挙型。
pub enum Geometry<Id: SceneId> {
    /// 三角形メッシュ。
    TriangleMesh(TriangleMesh<Id>),
}

/// シーンに含まれるジオメトリを管理する構造体。
pub struct GeometryRepository<Id: SceneId> {
    geometries: Vec<Geometry<Id>>,
}
impl<Id: SceneId> GeometryRepository<Id> {
    /// 新しいGeometryRepositoryを作成する。
    pub fn new() -> Self {
        Self {
            geometries: Vec::new(),
        }
    }

    /// OBJファイルを読み込み、TriangleMeshのジオメトリを作成し登録する。
    pub fn load_obj(&mut self, path: &str) -> GeometryIndex<Id> {
        let mesh_id = GeometryIndex::new(self.geometries.len());
        let triangle_mesh = TriangleMesh::load_obj(path);
        let geometry = Geometry::TriangleMesh(triangle_mesh);
        self.geometries.push(geometry);
        mesh_id
    }

    /// ジオメトリの参照を取得する。
    pub fn get(&self, index: GeometryIndex<Id>) -> &Geometry<Id> {
        &self.geometries[index.0]
    }

    /// ジオメトリの可変参照を取得する。
    pub fn get_mut(&mut self, index: GeometryIndex<Id>) -> &mut Geometry<Id> {
        &mut self.geometries[index.0]
    }
}
