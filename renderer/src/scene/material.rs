//! シーンのマテリアルとその管理に使う構造体などを定義するモジュール。

use std::marker::PhantomData;

use crate::scene::SceneId;

/// MaterialRepositoryに登録したMaterialのID。
/// MaterialRepositoryからこのIDを使ってMaterialを取得できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialId<Id: SceneId>(pub usize, PhantomData<Id>);
impl<Id: SceneId> MaterialId<Id> {
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}
