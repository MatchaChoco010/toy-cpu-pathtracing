//! プリミティブを作成するための情報を定義するモジュール。

use std::path::PathBuf;

use math::{Local, Normal, Point3, Transform, World};
use spectrum::Spectrum;

use crate::{GeometryIndex, MaterialId, SceneId};

/// プリミティブを作成するための情報。
pub enum CreatePrimitiveDesc<Id: SceneId> {
    /// ジオメトリプリミティブを作成するための情報。
    GeometryPrimitive {
        /// ジオメトリのインデックス。
        geometry_index: GeometryIndex<Id>,
        /// マテリアルのID。
        material_id: MaterialId<Id>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 単一の三角形プリミティブを作成するための情報。
    SingleTrianglePrimitive {
        /// 三角形の頂点座標。
        positions: [Point3<Local>; 3],
        /// 三角形の法線ベクトル。
        normals: [Normal<Local>; 3],
        /// 三角形のUV座標。
        uvs: [glam::Vec2; 3],
        /// マテリアルのID。
        material_id: MaterialId<Id>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 点光源プリミティブを作成するための情報。
    PointLightPrimitive {
        /// 点光源の強度。
        intensity: f32,
        /// 点光源のスペクトル。
        spectrum: Box<dyn Spectrum>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// スポットライトプリミティブを作成するための情報。
    /// スポットライトの方向はモデルのZ+軸方向を向いている。
    SpotLightPrimitive {
        /// スポットライトの光の減衰を始める内側の角度。
        angle_inner: f32,
        /// スポットライトの光の減衰を終わる外側の角度。
        angle_outer: f32,
        /// スポットライトの強度。
        intensity: f32,
        /// スポットライトのスペクトル。
        spectrum: Box<dyn Spectrum>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 指向性光源プリミティブを作成するための情報。
    /// 指向性光源の方向はモデルのZ+軸方向を向いている。
    DirectionalLightPrimitive {
        /// 指向性光源の強度。
        intensity: f32,
        /// 指向性光源のスペクトル。
        spectrum: Box<dyn Spectrum>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 環境光源プリミティブを作成するための情報。
    EnvironmentLightPrimitive {
        /// 環境光源の強度。
        intensity: f32,
        /// 環境光源のテクスチャパス。
        texture_path: PathBuf,
        /// モデルのワールド座標系への座標変換。
        /// 平行移動は無示され回転のみが適用される。
        transform: Transform<Local, World>,
    },
}
