//! プリミティブが実装すべきトレイトを定義するモジュール。

use math::{Bounds, LightSampleContext, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::scene::{
    GeometryRepository, MaterialId, SceneId,
    primitive::{Interaction, Intersection, LightIrradiance, LightSampleRadiance, PrimitiveIndex},
};

/// プリミティブのトレイト。
pub trait Primitive<Id: SceneId>: Send + Sync {
    /// カメラから計算されるワールド座標系からレンダリング用の座標系への変換をPrimitiveに設定する。
    fn update_world_to_render(&mut self, transform: &Transform<World, Render>);

    /// プリミティブをジオメトリプリミティブに変換する。
    fn as_geometry(&self) -> Option<&dyn PrimitiveGeometry<Id>>;

    /// プリミティブを可変参照でジオメトリプリミティブに変換する。
    fn as_geometry_mut(&mut self) -> Option<&mut dyn PrimitiveGeometry<Id>>;

    /// プリミティブをライトプリミティブに変換する。
    fn as_light(&self) -> Option<&dyn PrimitiveLight<Id>>;

    /// プリミティブを可変参照でライトプリミティブに変換する。
    fn as_light_mut(&mut self) -> Option<&mut dyn PrimitiveLight<Id>>;

    /// プリミティブを非デルタライトプリミティブに変換する。
    fn as_non_delta_light(&self) -> Option<&dyn PrimitiveNonDeltaLight<Id>>;

    /// プリミティブをデルタライトプリミティブに変換する。
    fn as_delta_light(&self) -> Option<&dyn PrimitiveDeltaLight<Id>>;

    /// プリミティブを面積光源プリミティブに変換する。
    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>>;

    /// プリミティブを無限光源プリミティブに変換する。
    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>>;
}

/// ジオメトリタイプのPrimitiveを表すトレイト。
pub trait PrimitiveGeometry<Id: SceneId>: Primitive<Id> {
    /// バウンディングボックスを取得する。
    fn bounds(&self, _geometry_repository: &GeometryRepository<Id>) -> Bounds<Render>;

    /// マテリアルIDを取得する。
    fn material_id(&self) -> MaterialId<Id>;

    /// ジオメトリのBVHを構築する。
    fn build_geometry_bvh(&mut self, _geometry_repository: &mut GeometryRepository<Id>) {
        // デフォルト実装は何もしない
    }

    /// ジオメトリとレイの交差を計算する。
    fn intersect(
        &self,
        _primitive_index: PrimitiveIndex<Id>,
        _geometry_repository: &GeometryRepository<Id>,
        _ray: &Ray<Render>,
        _t_max: f32,
    ) -> Option<Intersection<Id, Render>>;
}

/// ライトタイプのPrimitiveを表すトレイト。
pub trait PrimitiveLight<Id: SceneId>: Primitive<Id> {
    /// サンプルした波長の中で最大となるライトのスペクトル放射束。
    fn phi(&self, lambda: &SampledWavelengths) -> f32;

    /// シーン全体のバウンディングボックスを与えてプリプロセスを行う。
    fn preprocess(&mut self, _scene_bounds: &Bounds<Render>) {
        // デフォルト実装は何もしない
    }
}

/// DeltaではないライトのPrimitiveを表すトレイト。
pub trait PrimitiveNonDeltaLight<Id: SceneId>: PrimitiveLight<Id> {
    /// ライト上の点とそのスペクトル放射輝度のサンプリングを行う。
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        // _material_repository: &GeometryRepository<Id>,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
        _s: f32,
        _uv: glam::Vec2,
    ) -> LightSampleRadiance<Id, Render>;

    /// 交差点をライトのサンプルでサンプルしたときのPDFを計算する。
    fn pdf_light_sample(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _interaction: &Interaction<Id, Render>,
    ) -> f32;
}

/// DeltaなライトのPrimitiveを表すトレイト。
pub trait PrimitiveDeltaLight<Id: SceneId>: PrimitiveLight<Id> {
    /// 与えた波長でのスペクトル放射照度の計算を行う。
    fn calculate_irradiance(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
    ) -> LightIrradiance;
}

/// 面積光源のPrimitiveを表すトレイト。
pub trait PrimitiveAreaLight<Id: SceneId>: PrimitiveNonDeltaLight<Id> {
    /// 与えた波長における交差点でのスペクトル放射輝度を計算する。
    fn intersect_radiance(
        &self,
        // _material_repository: &GeometryRepository<Id>,
        _interaction: &Interaction<Id, Render>,
        _lambda: &SampledWavelengths,
    ) -> SampledSpectrum;
}

/// 無限光源のPrimitiveを表すトレイト。
pub trait PrimitiveInfiniteLight<Id: SceneId>: PrimitiveNonDeltaLight<Id> {
    /// 与えた波長における特定方向でのスペクトル放射輝度を計算する。
    fn direction_radiance(
        &self,
        _ray: &Ray<Render>,
        _lambda: &SampledWavelengths,
    ) -> SampledSpectrum;
}
