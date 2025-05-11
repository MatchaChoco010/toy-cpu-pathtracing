//! マテリアルを定義するモジュール。

use crate::{
    SceneId,
    material::{Bsdf, Edf},
};

/// 表面マテリアルの構造体。
pub struct SurfaceMaterial<Id: SceneId> {
    /// Surfaceマテリアルの界面のBSDF。
    pub bsdf: Option<Box<dyn Bsdf<Id>>>,
    /// Surfaceマテリアルの発光。
    pub edf: Option<Box<dyn Edf<Id>>>,
}
