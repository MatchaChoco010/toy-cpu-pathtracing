//! プリミティブのBVHを構築するモジュール。

use math::{Bounds, CoordinateSystem, Ray, Render, Transform, Vector3};
use util_macros::impl_binary_ops;

use crate::{
    InteractGeometryInfo, PrimitiveIndex, SceneId, SurfaceInteraction,
    bvh::{Bvh, BvhItem, BvhItemData, HitInfo},
    geometry::GeometryRepository,
    primitive::PrimitiveRepository,
};

/// BVHのアイテム用のトレイトをPrimitiveIndexに実装する。
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
    ) -> Option<HitInfo<Self::Intersection>>
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
        Some(HitInfo {
            t_hit: intersection.t_hit,
            intersection,
        })
    }

    fn intersect_p<'a>(&self, data: &Self::Data<'a>, ray: &Ray<Render>, t_max: f32) -> bool
    where
        Id: 'a,
    {
        let (geometry_repository, primitive_repository) = data;
        let primitive = primitive_repository.get(*self);
        let geometry = match primitive.as_geometry() {
            Some(geometry) => geometry,
            None => unreachable!(),
        };
        geometry.intersect_p(*self, geometry_repository, ray, t_max)
    }
}

/// BVHのアイテム用のデータトレイトをリポジトリのタプルに実装する。
impl<Id: SceneId> BvhItemData<PrimitiveIndex<Id>>
    for (&GeometryRepository<Id>, &PrimitiveRepository<Id>)
{
    fn item_list(&self) -> impl Iterator<Item = PrimitiveIndex<Id>> {
        // BVHに含めるのはジオメトリプリミティブのみ。
        self.1.get_all_primitive_indices().filter(|index| {
            let primitive = self.1.get(*index);
            primitive.as_geometry().is_some()
        })
    }
}

/// ジオメトリの交差判定の結果を持つ構造体。
pub struct Intersection<Id: SceneId, C: CoordinateSystem> {
    /// 交差した位置。
    pub t_hit: f32,
    /// 出射方向。
    pub wo: Vector3<C>,
    /// 交差したプリミティブのインデックス。
    pub primitive_index: PrimitiveIndex<Id>,
    /// 交差したジオメトリの追加情報。
    pub geometry_info: InteractGeometryInfo,
    /// 交差した情報。
    pub interaction: SurfaceInteraction<C>,
}
#[impl_binary_ops(Mul)]
fn mul<Id: SceneId, From: CoordinateSystem, To: CoordinateSystem>(
    lhs: &Transform<From, To>,
    rhs: &Intersection<Id, From>,
) -> Intersection<Id, To> {
    Intersection {
        t_hit: rhs.t_hit,
        wo: lhs * rhs.wo,
        primitive_index: rhs.primitive_index,
        geometry_info: rhs.geometry_info,
        interaction: lhs * &rhs.interaction,
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

    /// シーン内のプリミティブとの交差判定を行い情報を返す。
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

    /// シーン内のプリミティブとの交差判定を行う。
    pub fn intersect_p(
        &self,
        geometry_repository: &GeometryRepository<Id>,
        primitive_repository: &PrimitiveRepository<Id>,
        ray: &Ray<Render>,
        t_max: f32,
    ) -> bool {
        let data = (geometry_repository, primitive_repository);
        self.bvh.intersect_p(&data, ray, t_max)
    }
}
