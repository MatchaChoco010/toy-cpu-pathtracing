//! シーンを表す構造体とその関連のトレイトや構造体を定義するモジュール。

use std::fmt::Debug;

use math::{Ray, Render, World};

use crate::{
    CreatePrimitiveDesc, GeometryIndex, Intersection, PrimitiveIndex,
    geometry::GeometryRepository,
    light_sampler::LightSampler,
    primitive::{PrimitiveBvh, PrimitiveRepository},
};

/// ワールド座標系からレンダリング座標系への変換を取得するトレイト。
/// カメラなどのオブジェクトに実装することを想定している。
pub trait WorldToRender {
    /// ワールド座標系からレンダリング座標系への変換を取得する。
    fn world_to_render(&self) -> math::Transform<World, Render>;
}

/// シーンのマーカー用のトレイト。
/// シーンには別のシーンから作ったインデックスを他のシーンで使えなくするためにマーカー型を使う。
/// これはそのマーカー型が満たすべきトレイト。
pub trait SceneId: Send + Sync + Debug + Clone + Copy + PartialEq + Eq + 'static {}

/// シーンのデータを表す構造体。
///
/// シーンは、ジオメトリ、マテリアル、プリミティブ、
/// アクセラレーションストラクチャー、ライトサンプラーなどを持つ。
///
/// 3Dモデルなどのプリミティブを読み込んでシーンに追加でき、
/// シーンのアクセラレーションストラクチャをbuildした後は、
/// シーンにレイを飛ばして交差を取得したり、ライトをサンプリングしたりできる。
pub struct Scene<Id: SceneId> {
    geometry_repository: GeometryRepository<Id>,
    primitive_repository: PrimitiveRepository<Id>,
    bvh: Option<PrimitiveBvh<Id>>,
    light_sampler: Option<LightSampler<Id>>,
}
impl<Id: SceneId> Scene<Id> {
    fn __new() -> Self {
        Scene {
            geometry_repository: GeometryRepository::new(),
            primitive_repository: PrimitiveRepository::new(),
            bvh: None,
            light_sampler: None,
        }
    }

    pub fn load_obj(&mut self, path: &str) -> GeometryIndex<Id> {
        self.geometry_repository.load_obj(path)
    }

    pub fn create_primitive(&mut self, desc: CreatePrimitiveDesc<Id>) -> PrimitiveIndex<Id> {
        self.primitive_repository
            .create_primitive(&self.geometry_repository, desc)
    }

    pub fn build(&mut self, camera: &impl WorldToRender) {
        self.primitive_repository
            .update_world_to_render(&camera.world_to_render());
        self.bvh = Some(PrimitiveBvh::build(
            &mut self.geometry_repository,
            &mut self.primitive_repository,
        ));
        let scene_bounds = self.bvh.as_ref().unwrap().scene_bounds();
        self.light_sampler = Some(LightSampler::build(
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
#[macro_export]
macro_rules! create_scene {
    () => {{
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct SceneId;
        impl $scene::SceneId for SceneId {}
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
        impl $crate::SceneId for SceneId<$label> {}
        #[allow(deprecated)]
        $crate::internal::__create_scene::<SceneId<$label>>()
    }};
}
