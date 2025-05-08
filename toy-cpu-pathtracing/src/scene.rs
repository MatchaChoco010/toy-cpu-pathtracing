//! シーンを表す構造体とその関連のトレイトや構造体を定義するモジュール。

pub mod bvh;
mod geometry;
mod light_sampler;
mod material;
pub mod primitive;

use crate::camera::Camera;
use crate::filter::Filter;
use crate::math::{Ray, Render};

pub use geometry::{Geometry, GeometryIndex, GeometryRepository};
pub use material::MaterialId;
use primitive::Intersection;
pub use primitive::{
    CreatePrimitiveDesc, GeometryInfo, Interaction, LightIrradiance, PrimitiveBvh, PrimitiveIndex,
    PrimitiveRepository,
};

/// シーンのマーカー用のトレイト。
/// シーンには別のシーンから作ったインデックスを他のシーンで使えなくするためにマーカー型を使う。
/// これはそのマーカー型が満たすべきトレイト。
pub trait SceneId: Send + Sync + std::fmt::Debug + Clone + Copy + PartialEq + Eq {}

/// シーンのデータを表す構造体。
///
/// シーンは、ジオメトリ、マテリアル、プリミティブ、
/// アクセラレーションストラクチャー、ライトサンプラーなどを持つ。
///
/// 3Dモデルなどのプリミティブを読み込んでシーンに追加でき、
/// シーンのアクセラレーションストラクチャをbuildした後は、
/// シーンにレイを飛ばして交差を取得したり、ライトをサンプリングしたりできる。
pub struct Scene<Id: SceneId> {
    geometry_repository: geometry::GeometryRepository<Id>,
    primitive_repository: primitive::PrimitiveRepository<Id>,
    bvh: Option<PrimitiveBvh<Id>>,
    light_sampler: Option<light_sampler::LightSampler<Id>>,
}
impl<Id: SceneId> Scene<Id> {
    fn __new() -> Self {
        Scene {
            geometry_repository: geometry::GeometryRepository::new(),
            primitive_repository: primitive::PrimitiveRepository::new(),
            bvh: None,
            light_sampler: None,
        }
    }

    pub fn load_obj(&mut self, path: &str) -> GeometryIndex<Id> {
        self.geometry_repository.load_obj(path)
    }

    pub fn create_primitive(
        &mut self,
        desc: primitive::CreatePrimitiveDesc<Id>,
    ) -> PrimitiveIndex<Id> {
        self.primitive_repository
            .create_primitive(&self.geometry_repository, desc)
    }

    pub fn build<F: Filter>(&mut self, camera: &Camera<F>) {
        self.primitive_repository
            .update_world_to_render(&camera.world_to_render());
        self.bvh = Some(PrimitiveBvh::build(
            &mut self.geometry_repository,
            &mut self.primitive_repository,
        ));
        let scene_bounds = self.bvh.as_ref().unwrap().scene_bounds();
        self.light_sampler = Some(light_sampler::LightSampler::build(
            &mut self.primitive_repository,
            &scene_bounds,
        ));
    }

    pub fn intersect(&self, ray: &Ray<Render>, t_max: f32) -> Option<Intersection<Id, Render>> {
        if self.bvh.is_none() {
            panic!("BVH is not built");
        }
        self.bvh.as_ref().unwrap().intersect(
            &self.geometry_repository,
            &self.primitive_repository,
            ray,
            t_max,
        )
    }
}

// マクロからしか使わない想定の関数をinternalに隔離する。
#[doc(hidden)]
pub mod internal {
    use super::*;

    // マクロ以外から使うとdeprecatedの警告が出るようにする。
    // Idのマーカーを重複させると、別のシーンで作ったインデックスを弾く機能が正しく動作しなくなるので、
    // マーカー型の定義と同時にSceneを作る`create_scene!`マクロを使うことを推奨している。
    #[deprecated(note = "Use `create_scene!` macro instead")]
    pub fn __create_scene<Id: SceneId>() -> Scene<Id> {
        Scene::__new()
    }
}

/// シーンを作成するマクロ。
/// シーンの型を識別するためのラベルを指定することができる。
/// ```
/// # use crate::scene::create_scene;
/// let scene = create_scene!(SceneLabel);
/// ```
///
/// マクロの内部ではシーンを識別するマーカー型を定義し利用している。
/// このマーカー型のおかげで他のシーンで作ったプリミティブのインデックスを取り違えるとコンパイルエラーになる。
macro_rules! create_scene {
    () => {{
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct SceneId;
        impl $crate::scene::SceneId for SceneId {}
        #[allow(deprecated)]
        $crate::scene::internal::__create_scene::<SceneId>()
    }};
    ($label:ident) => {{
        use std::marker::PhantomData;
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct $label;
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct SceneId<$label>(PhantomData<$label>);
        impl $crate::scene::SceneId for SceneId<$label> {}
        #[allow(deprecated)]
        $crate::scene::internal::__create_scene::<SceneId<$label>>()
    }};
}
pub(crate) use create_scene;
