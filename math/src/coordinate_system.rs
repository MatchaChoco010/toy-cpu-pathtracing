//! 座標系を表すマーカー構造体を定義するモジュール。

/// 座標系のマーカー用トレイト。
pub trait CoordinateSystem: std::fmt::Debug + Clone + Copy {}

/// ワールド座標系を表すマーカー構造体。
#[derive(Debug, Clone, Copy)]
pub struct World;
impl CoordinateSystem for World {}

/// モデルローカル座標系を表すマーカー構造体。
#[derive(Debug, Clone, Copy)]
pub struct Local;
impl CoordinateSystem for Local {}

/// レンダリング座標系を表すマーカー構造体。
///
/// レンダリング座標系はカメラを原点にして座標軸はワールド座標系と平行な座標系。
/// 多くの場合、シーンにはワールド座標の軸と平行な直線が含まれることがあり、特に地面などは軸とズレていないことも多い。
/// そのため、カメラが斜めになったときでもレンダリングに使う座標系では
/// ワールド座標系と軸が平行がそのままの方がバウンディングボックスがタイトになりやすく、多少良いBVHが構築できうる。
#[derive(Debug, Clone, Copy)]
pub struct Render;
impl CoordinateSystem for Render {}
