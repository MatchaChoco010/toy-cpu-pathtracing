//! シーンに含まれるジオメトリデータを管理するリポジトリを定義するモジュール。

use std::marker::PhantomData;

use crate::{
    SceneId,
    geometry::{Geometry, impls::*},
};

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

/// シーンに含まれるジオメトリを管理する構造体。
pub struct GeometryRepository<Id: SceneId> {
    geometries: Vec<Box<dyn Geometry<Id>>>,
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
        let triangle_mesh: Box<dyn Geometry<Id>> = Box::new(TriangleMesh::load_obj(path));
        self.geometries.push(triangle_mesh);
        mesh_id
    }

    /// ジオメトリの参照を取得する。
    pub fn get(&self, index: GeometryIndex<Id>) -> &Box<dyn Geometry<Id>> {
        &self.geometries[index.0]
    }

    /// ジオメトリの可変参照を取得する。
    pub fn get_mut(&mut self, index: GeometryIndex<Id>) -> &mut Box<dyn Geometry<Id>> {
        &mut self.geometries[index.0]
    }
}
