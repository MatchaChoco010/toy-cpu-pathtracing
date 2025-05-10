//! ジオメトリデータが実装するべきトレイトを定義するモジュール。

use math::{Bounds, Local, Ray};

use crate::scene::{SceneId, geometry::Intersection};

/// ジオメトリデータが実装するトレイト。
pub trait Geometry<Id: SceneId>: Send + Sync {
    /// ジオメトリのバウンディングボックスを取得する。
    fn bounds(&self) -> Bounds<Local>;

    /// ジオメトリのBVHを構築する。
    fn build_bvh(&mut self) {
        // デフォルトでは何もしない。
    }

    /// ジオメトリとレイの交差判定を計算する。
    fn intersect(&self, ray: &Ray<Local>, t_max: f32) -> Option<Intersection>;
}
