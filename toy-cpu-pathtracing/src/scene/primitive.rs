//! シーンを構成する要素のPrimitiveとその間利用構造体を定義するモジュール。

use std::marker::PhantomData;
use std::path::PathBuf;

use glam::Vec2;

mod directional_light;
mod emissive_single_triangle;
mod emissive_triangle_mesh;
mod environment_light;
mod point_light;
mod single_triangle;
mod spot_light;
mod triangle_mesh;

use directional_light::DirectionalLight;
use emissive_single_triangle::EmissiveSingleTriangle;
use emissive_triangle_mesh::EmissiveTriangleMesh;
use environment_light::EnvironmentLight;
use macros::enum_methods;
use point_light::PointLight;
use single_triangle::SingleTriangle;
use spot_light::SpotLight;
use triangle_mesh::TriangleMesh;

use crate::math::{
    Bounds, CoordinateSystem, LightSampleContext, Local, Normal, Point3, Ray, Render, Transform,
    World,
};
use crate::scene::{Geometry, GeometryIndex, GeometryRepository, MaterialId, SceneId};
use crate::spectrum::{SampledSpectrum, SampledWavelengths};

use super::bvh::{Bvh, BvhItem, BvhItemData};

impl<Id: SceneId> BvhItemData<PrimitiveIndex<Id>>
    for (&GeometryRepository<Id>, &PrimitiveRepository<Id>)
{
    fn item_list(&self) -> impl Iterator<Item = PrimitiveIndex<Id>> {
        self.1.get_all_primitive_indices()
    }
}

/// PrimitiveRepositoryに登録したPrimitiveのID。
/// PrimitiveRepositoryからこのIDを使ってPrimitiveを取得できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveIndex<Id: SceneId>(pub usize, PhantomData<Id>);
impl<Id: SceneId> PrimitiveIndex<Id> {
    /// 新しいPrimitiveIndexを作成する。
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}
impl<Id: SceneId> BvhItem<Render> for PrimitiveIndex<Id> {
    type Data<'a>
        = (&'a GeometryRepository<Id>, &'a PrimitiveRepository<Id>)
    where
        Id: 'a;
    type Intersection = Intersection<Id, Render>;

    fn bounds<'a>(&self, data: &Self::Data<'a>) -> Bounds<Render>
    where
        Id: 'a,
    {
        let (geometry_repository, primitive_repository) = data;
        let primitive = primitive_repository.get(*self);
        let geometry = match primitive.as_geometry() {
            Some(geometry) => geometry,
            None => unreachable!(),
        };
        geometry.bounds(geometry_repository)
    }

    fn intersect<'a>(
        &self,
        data: &Self::Data<'a>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<super::bvh::HitInfo<Self::Intersection>>
    where
        Id: 'a,
    {
        let (geometry_repository, primitive_repository) = data;
        let primitive = primitive_repository.get(*self);
        let geometry = match primitive.as_geometry() {
            Some(geometry) => geometry,
            None => unreachable!(),
        };
        let intersection = geometry.intersect(*self, geometry_repository, ray, t_max)?;
        Some(super::bvh::HitInfo {
            t_hit: intersection.t_hit,
            intersection,
        })
    }
}

/// サンプルしたジオメトリを特定するための情報を持つ列挙型。
#[derive(Debug, Clone, Copy)]
pub enum GeometryInfo {
    /// サンプルした三角形メッシュの三角形を特定するための情報。
    TriangleMesh {
        /// サンプルした三角形メッシュのインデックス。
        triangle_index: u32,
    },
}

/// シーンをサンプルした結果の情報を持つ列挙型。
pub enum Interaction<Id: SceneId, C: CoordinateSystem> {
    Surface {
        /// サンプルした位置。
        position: Point3<C>,
        /// サンプルした幾何法線。
        normal: Normal<C>,
        /// サンプルしたシェーディング座標。
        shading_normal: Normal<C>,
        /// サンプルしたUV座標。
        uv: Vec2,
        /// サンプルしたプリミティブのインデックス。
        primitive_index: PrimitiveIndex<Id>,
        /// サンプルしたジオメトリの追加情報。
        geometry_info: GeometryInfo,
    },
    // Medium {
    //     ...
    // },
}

/// ライト上のサンプルされた放射輝度情報とPDFを持つ構造体。
pub struct LightSampleRadiance<Id: SceneId, C: CoordinateSystem> {
    /// サンプルした放射輝度。
    pub radiance: SampledSpectrum,
    /// サンプルのPDF。
    pub pdf: f32,
    /// シーンをサンプルした結果の情報。
    pub interaction: Interaction<Id, C>,
}

/// ライト上のサンプルされた放射照度情報を持つ構造体。
pub struct LightIrradiance {
    /// 計算した放射照度。
    pub irradiance: SampledSpectrum,
}

/// ジオメトリの交差判定の結果を持つ構造体。
pub struct Intersection<Id: SceneId, C: CoordinateSystem> {
    /// 交差した位置。
    pub t_hit: f32,
    /// 交差した情報。
    pub interaction: Interaction<Id, C>,
}

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
    fn update_world_to_render(&mut self, transform: &Transform<World, Render>),
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
        uvs: [Vec2; 3],
        /// マテリアルのID。
        material_id: MaterialId<Id>,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 点光源プリミティブを作成するための情報。
    PointLightPrimitive {
        /// 点光源の強度。
        intensity: f32,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// スポットライトプリミティブを作成するための情報。
    /// スポットライトの方向はモデルのZ+軸方向を向いている。
    SpotLightPrimitive {
        /// スポットライトの角度。
        angle: f32,
        /// スポットライトの強度。
        intensity: f32,
        /// モデルのワールド座標系への座標変換。
        transform: Transform<Local, World>,
    },
    /// 指向性光源プリミティブを作成するための情報。
    /// 指向性光源の方向はモデルのZ+軸方向を向いている。
    DirectionalLightPrimitive {
        /// 指向性光源の強度。
        intensity: f32,
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

/// プリミティブを保持するリポジトリの構造体。
pub struct PrimitiveRepository<Id: SceneId> {
    primitives: Vec<Primitive<Id>>,
}
impl<Id: SceneId> PrimitiveRepository<Id> {
    /// 新しいプリミティブリポジトリを作成する。
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    /// プリミティブをCreatePrimitiveDescから作成し登録する。
    pub fn create_primitive(
        &mut self,
        geometry_repository: &GeometryRepository<Id>,
        desc: CreatePrimitiveDesc<Id>,
    ) -> PrimitiveIndex<Id> {
        let primitive_index = PrimitiveIndex(self.primitives.len(), PhantomData);
        let primitive = match desc {
            CreatePrimitiveDesc::GeometryPrimitive {
                geometry_index,
                material_id,
                transform,
            } => {
                let geometry = geometry_repository.get(geometry_index);
                match geometry {
                    Geometry::TriangleMesh(_) => {
                        // TODO: materialがemissiveか判断する
                        Primitive::TriangleMesh(TriangleMesh::new(
                            geometry_index,
                            material_id,
                            transform,
                        ))
                    }
                }
            }
            CreatePrimitiveDesc::SingleTrianglePrimitive {
                positions,
                normals,
                uvs,
                material_id,
                transform,
            } => {
                // TODO: materialがemissiveか判断する
                Primitive::SingleTriangle(SingleTriangle::new(
                    positions,
                    normals,
                    uvs,
                    material_id,
                    transform,
                ))
            }
            CreatePrimitiveDesc::PointLightPrimitive {
                intensity,
                transform,
            } => Primitive::PointLight(PointLight::new(intensity, transform)),
            CreatePrimitiveDesc::SpotLightPrimitive {
                angle,
                intensity,
                transform,
            } => Primitive::SpotLight(SpotLight::new(angle, intensity, transform)),
            CreatePrimitiveDesc::DirectionalLightPrimitive {
                intensity,
                transform,
            } => Primitive::DirectionalLight(DirectionalLight::new(intensity, transform)),
            CreatePrimitiveDesc::EnvironmentLightPrimitive {
                intensity,
                texture_path,
                transform,
            } => Primitive::EnvironmentLight(EnvironmentLight::new(
                intensity,
                texture_path,
                transform,
            )),
        };
        self.primitives.push(primitive);
        primitive_index
    }

    /// プリミティブを取得する。
    pub fn get(&self, index: PrimitiveIndex<Id>) -> &Primitive<Id> {
        &self.primitives[index.0]
    }

    /// プリミティブを可変参照で取得する。
    pub fn get_mut(&mut self, index: PrimitiveIndex<Id>) -> &mut Primitive<Id> {
        &mut self.primitives[index.0]
    }

    /// プリミティブのインデックスのイテレーターを取得する。
    pub fn get_all_primitive_indices(&self) -> impl Iterator<Item = PrimitiveIndex<Id>> {
        (0..self.primitives.len()).map(move |i| PrimitiveIndex::new(i))
    }

    /// world_to_renderの座標変換を更新する。
    pub fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        for primitive in &mut self.primitives {
            primitive.update_world_to_render(world_to_render);
        }
    }
}

/// アクセラレーションストラクチャーの構造体。
pub struct PrimitiveBvh<Id: SceneId> {
    bvh: Bvh<Render, PrimitiveIndex<Id>>,
}
impl<Id: SceneId> PrimitiveBvh<Id> {
    /// AccelerationStructureを構築する。
    pub fn build(
        geometry_repository: &mut GeometryRepository<Id>,
        primitive_repository: &mut PrimitiveRepository<Id>,
    ) -> Self {
        let primitive_index_list = primitive_repository
            .get_all_primitive_indices()
            .collect::<Vec<_>>();
        for index in primitive_index_list {
            let primitive = primitive_repository.get_mut(index);
            let geometry = match primitive.as_geometry_mut() {
                Some(geometry) => geometry,
                None => continue,
            };
            geometry.build_geometry_bvh(geometry_repository);
        }

        let data = (&*geometry_repository, &*primitive_repository);
        let bvh = Bvh::build(&data);

        Self { bvh }
    }

    /// シーン全体のジオメトリのバウンディングボックスを返す。
    pub fn scene_bounds(&self) -> Bounds<Render> {
        self.bvh.bounds()
    }

    /// シーン内のプリミティブとの交差判定を行う。
    pub fn intersect(
        &self,
        geometry_repository: &GeometryRepository<Id>,
        primitive_repository: &PrimitiveRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> Option<Intersection<Id, Render>> {
        let data = (geometry_repository, primitive_repository);
        self.bvh.intersect(&data, ray, t_max)
    }
}
