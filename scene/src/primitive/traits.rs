//! プリミティブが実装すべきトレイトを定義するモジュール。

use math::{Bounds, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    AreaLightSampleRadiance, DeltaDirectionalLightIntensity, DeltaPointLightIntensity,
    Intersection, Material, PrimitiveIndex, SceneId, SurfaceInteraction,
    geometry::GeometryRepository,
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

    /// プリミティブをデルタ点光源プリミティブに変換する。
    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>>;

    /// プリミティブをデルタ指向性光源プリミティブに変換する。
    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>>;

    /// プリミティブを面積光源プリミティブに変換する。
    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>>;

    /// プリミティブを無限光源プリミティブに変換する。
    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>>;
}

/// ジオメトリタイプのPrimitiveを表すトレイト。
pub trait PrimitiveGeometry<Id: SceneId>: Primitive<Id> {
    /// バウンディングボックスを取得する。
    fn bounds(&self, geometry_repository: &GeometryRepository<Id>) -> Bounds<Render>;

    /// 表面マテリアルを取得する。
    fn surface_material(&self) -> Material;

    /// ジオメトリのBVHを構築する。
    fn build_geometry_bvh(&mut self, _geometry_repository: &mut GeometryRepository<Id>) {
        // デフォルト実装は何もしない
    }

    /// ジオメトリとレイの交差を行い情報を返す。
    fn intersect(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>>;

    /// ジオメトリとレイの交差判定を行う。
    fn intersect_p(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        geometry_repository: &GeometryRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> bool;
}

/// ライトタイプのPrimitiveを表すトレイト。
pub trait PrimitiveLight<Id: SceneId>: Primitive<Id> {
    /// サンプルした波長の中で最大となるライトのスペクトル放射束。
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum;

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
        geometry_repository: &GeometryRepository<Id>,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
        s: f32,
        uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render>;
}

/// Deltaな点光源のPrimitiveを表すトレイト。
pub trait PrimitiveDeltaPointLight<Id: SceneId>: PrimitiveLight<Id> {
    /// 与えた波長でのスペクトル放射強度の計算を行う。
    fn calculate_intensity(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
    ) -> DeltaPointLightIntensity<Render>;
}

/// Deltaな指向性光源のPrimitiveを表すトレイト。
pub trait PrimitiveDeltaDirectionalLight<Id: SceneId>: PrimitiveLight<Id> {
    /// 与えた波長でのスペクトル放射強度の計算を行う。
    fn calculate_intensity(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
    ) -> DeltaDirectionalLightIntensity<Render>;
}

/// 面積光源のPrimitiveを表すトレイト。
pub trait PrimitiveAreaLight<Id: SceneId>: PrimitiveNonDeltaLight<Id> {
    /// 与えた波長における交差点でのスペクトル放射輝度を計算する。
    fn intersect_radiance(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        interaction: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum;

    /// 交差点をライトのサンプルでサンプルしたときのpdfを計算する。
    fn pdf_light_sample(&self, intersection: &Intersection<Id, Render>) -> f32;
}

/// 無限光源のPrimitiveを表すトレイト。
pub trait PrimitiveInfiniteLight<Id: SceneId>: PrimitiveNonDeltaLight<Id> {
    /// 与えた波長における特定方向でのスペクトル放射輝度を計算する。
    fn direction_radiance(&self, ray: &Ray<Render>, lambda: &SampledWavelengths)
    -> SampledSpectrum;

    /// 与えられた方向に対するサンプリング確率密度を計算する。
    fn pdf_direction_sample(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        wi: math::Vector3<Render>,
    ) -> f32;
}
