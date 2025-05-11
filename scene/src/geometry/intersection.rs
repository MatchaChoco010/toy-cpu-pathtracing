//! ジオメトリとレイの交差の情報を保持する構造体を定義するモジュール。

use math::{Local, Normal, Point3, Vector3};

/// ジオメトリとレイの交差の情報。
pub struct Intersection {
    /// サンプルした位置。
    pub position: Point3<Local>,
    /// サンプルした幾何法線。
    pub normal: Normal<Local>,
    /// サンプルしたシェーディング法線。
    pub shading_normal: Normal<Local>,
    /// サンプルしたタンジェントベクトル。
    pub tangent: Vector3<Local>,
    /// サンプルしたUV座標。
    pub uv: glam::Vec2,
    /// サンプルしたジオメトリのインデックス。
    pub index: u32,
    /// ヒットした距離。
    pub t_hit: f32,
}
