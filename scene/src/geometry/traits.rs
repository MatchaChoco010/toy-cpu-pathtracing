//! ジオメトリデータが実装するべきトレイトを定義するモジュール。

use std::any::Any;
use std::fmt::Debug;

use math::{Bounds, Local, Ray};

use crate::{SceneId, geometry::Intersection};

/// ジオメトリデータが実装するトレイト。
pub trait Geometry<Id: SceneId>: Send + Sync + Any + Debug {
    /// ジオメトリのバウンディングボックスを取得する。
    fn bounds(&self) -> Bounds<Local>;

    /// ジオメトリのBVHを構築する。
    fn build_bvh(&mut self) {
        // デフォルトでは何もしない。
    }

    /// ジオメトリとレイの交差判定を行い情報を取得する。
    fn intersect(&self, ray: &Ray<Local>, t_max: f32) -> Option<Intersection>;

    /// ジオメトリとレイの交差判定を行う。
    fn intersect_p(&self, ray: &Ray<Local>, t_max: f32) -> bool;

    /// Anyトレイトにキャストする。
    fn as_any(&self) -> &dyn Any;
}
