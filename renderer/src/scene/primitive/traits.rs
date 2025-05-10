//! プリミティブが実装すべきトレイトを定義するモジュール。

use glam::Vec2;

use math::{Bounds, LightSampleContext, Ray, Render, Transform, World};
use spectrum::{SampledSpectrum, SampledWavelengths};
use util_macros::enum_methods;

use crate::scene::{
    GeometryRepository, MaterialId, SceneId,
    primitive::{
        Interaction, Intersection, LightIrradiance, LightSampleRadiance, PrimitiveIndex, impls::*,
    },
};

/// プリミティブのトレイト。
pub trait PrimitiveTrait {
    /// カメラから計算されるワールド座標系からレンダリング用の座標系への変換をPrimitiveに設定する。
    fn update_world_to_render(&mut self, transform: &Transform<World, Render>);
}
/// ジオメトリタイプのPrimitiveを表すトレイト。
pub trait PrimitiveGeometry<Id: SceneId>: PrimitiveTrait {
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
pub trait PrimitiveLight: PrimitiveTrait {
    /// サンプルした波長の中で最大となるライトのスペクトル放射束。
    fn phi(&self, lambda: &SampledWavelengths) -> f32;

    /// シーン全体のバウンディングボックスを与えてプリプロセスを行う。
    fn preprocess(&mut self, _scene_bounds: &Bounds<Render>) {
        // デフォルト実装は何もしない
    }
}
/// DeltaではないライトのPrimitiveを表すトレイト。
pub trait PrimitiveNonDeltaLight<Id: SceneId>: PrimitiveLight {
    /// ライト上の点とそのスペクトル放射輝度のサンプリングを行う。
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        // _material_repository: &GeometryRepository<Id>,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
        _s: f32,
        _uv: Vec2,
    ) -> LightSampleRadiance<Id, Render>;

    /// 交差点をライトのサンプルでサンプルしたときのPDFを計算する。
    fn pdf_light_sample(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _interaction: &Interaction<Id, Render>,
    ) -> f32;
}
/// DeltaなライトのPrimitiveを表すトレイト。
pub trait PrimitiveDeltaLight<Id: SceneId>: PrimitiveLight {
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

/// プリミティブを表す列挙型。
/// プリミティブは、三角形メッシュ、単一の三角形、点光源、環境光源などを表す。
#[enum_methods {
    pub fn update_world_to_render(&mut self, transform: &Transform<World, Render>),
}]
pub enum Primitive<Id: SceneId> {
    TriangleMesh(TriangleMesh<Id>),
    SingleTriangle(SingleTriangle<Id>),
    EmissiveTriangleMesh(EmissiveTriangleMesh<Id>),
    EmissiveSingleTriangle(EmissiveSingleTriangle<Id>),
    PointLight(PointLight),
    DirectionalLight(DirectionalLight),
    SpotLight(SpotLight),
    EnvironmentLight(EnvironmentLight),
}

impl<Id: SceneId> Primitive<Id> {
    /// プリミティブをジオメトリプリミティブに変換する。
    pub fn as_geometry(&self) -> Option<&dyn PrimitiveGeometry<Id>> {
        match self {
            Primitive::TriangleMesh(mesh) => Some(mesh),
            Primitive::SingleTriangle(triangle) => Some(triangle),
            Primitive::EmissiveTriangleMesh(mesh) => Some(mesh),
            Primitive::EmissiveSingleTriangle(triangle) => Some(triangle),
            _ => None,
        }
    }

    /// プリミティブを可変参照でジオメトリプリミティブに変換する。
    pub fn as_geometry_mut(&mut self) -> Option<&mut dyn PrimitiveGeometry<Id>> {
        match self {
            Primitive::TriangleMesh(mesh) => Some(mesh),
            Primitive::SingleTriangle(triangle) => Some(triangle),
            Primitive::EmissiveTriangleMesh(mesh) => Some(mesh),
            Primitive::EmissiveSingleTriangle(triangle) => Some(triangle),
            _ => None,
        }
    }

    /// プリミティブをライトプリミティブに変換する。
    pub fn as_light(&self) -> Option<&dyn PrimitiveLight> {
        match self {
            Primitive::EmissiveTriangleMesh(light) => Some(light),
            Primitive::EmissiveSingleTriangle(light) => Some(light),
            Primitive::PointLight(light) => Some(light),
            Primitive::DirectionalLight(light) => Some(light),
            Primitive::SpotLight(light) => Some(light),
            Primitive::EnvironmentLight(light) => Some(light),
            _ => None,
        }
    }

    /// プリミティブを可変参照でライトプリミティブに変換する。
    pub fn as_light_mut(&mut self) -> Option<&mut dyn PrimitiveLight> {
        match self {
            Primitive::EmissiveTriangleMesh(light) => Some(light),
            Primitive::EmissiveSingleTriangle(light) => Some(light),
            Primitive::PointLight(light) => Some(light),
            Primitive::DirectionalLight(light) => Some(light),
            Primitive::SpotLight(light) => Some(light),
            Primitive::EnvironmentLight(light) => Some(light),
            _ => None,
        }
    }

    /// プリミティブを非デルタライトプリミティブに変換する。
    pub fn as_non_delta_light(&self) -> Option<&dyn PrimitiveNonDeltaLight<Id>> {
        match self {
            Primitive::EmissiveTriangleMesh(light) => Some(light),
            Primitive::EmissiveSingleTriangle(light) => Some(light),
            Primitive::EnvironmentLight(light) => Some(light),
            _ => None,
        }
    }

    /// プリミティブをデルタライトプリミティブに変換する。
    pub fn as_delta_light(&self) -> Option<&dyn PrimitiveDeltaLight<Id>> {
        match self {
            Primitive::PointLight(light) => Some(light),
            Primitive::DirectionalLight(light) => Some(light),
            Primitive::SpotLight(light) => Some(light),
            _ => None,
        }
    }

    /// プリミティブを面積光源プリミティブに変換する。
    pub fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        match self {
            Primitive::EmissiveTriangleMesh(light) => Some(light),
            Primitive::EmissiveSingleTriangle(light) => Some(light),
            _ => None,
        }
    }

    /// プリミティブを無限光源プリミティブに変換する。
    pub fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        match self {
            Primitive::EnvironmentLight(light) => Some(light),
            _ => None,
        }
    }
}
