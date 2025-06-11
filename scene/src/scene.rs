//! シーンを表す構造体とその関連のトレイトや構造体を定義するモジュール。

use std::fmt::Debug;

use math::{Ray, Render, World};
use spectrum::SampledWavelengths;

use crate::{
    CreatePrimitiveDesc, GeometryIndex, Intersection, LightIntensity, LightSampler, PrimitiveIndex,
    SurfaceInteraction,
    geometry::GeometryRepository,
    light_sampler::LightSamplerFactory,
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
    light_sampler_factory: Option<LightSamplerFactory<Id>>,
}
impl<Id: SceneId> Scene<Id> {
    fn __new() -> Self {
        Scene {
            geometry_repository: GeometryRepository::new(),
            primitive_repository: PrimitiveRepository::new(),
            bvh: None,
            light_sampler_factory: None,
        }
    }

    /// objファイルを読み込んでジオメトリを作成し、シーンに追加する。
    pub fn load_obj(&mut self, path: &str) -> GeometryIndex<Id> {
        self.geometry_repository.load_obj(path)
    }

    /// プリミティブを作成し、シーンに追加する。
    pub fn create_primitive(&mut self, desc: CreatePrimitiveDesc<Id>) -> PrimitiveIndex<Id> {
        self.primitive_repository
            .create_primitive(&self.geometry_repository, desc)
    }

    /// シーンを交差判定やライトサンプル用にビルドする。
    pub fn build(&mut self, camera: &impl WorldToRender) {
        self.primitive_repository
            .update_world_to_render(&camera.world_to_render());
        self.bvh = Some(PrimitiveBvh::build(
            &mut self.geometry_repository,
            &mut self.primitive_repository,
        ));
        let scene_bounds = self.bvh.as_ref().unwrap().scene_bounds();
        self.light_sampler_factory = Some(LightSamplerFactory::build(
            &mut self.primitive_repository,
            &scene_bounds,
        ));
    }

    /// 交差判定を計算する。
    /// build()を呼び出す前に呼び出すとpanicする。
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

    /// 交差判定を計算する。
    pub fn intersect_p(&self, ray: &Ray<Render>, t_max: f32) -> bool {
        if self.bvh.is_none() {
            panic!("BVH is not built");
        }
        self.bvh.as_ref().unwrap().intersect_p(
            &self.geometry_repository,
            &self.primitive_repository,
            ray,
            t_max,
        )
    }

    /// ライトのサンプラーを取得する。
    /// build()を呼び出す前に呼び出すとpanicする。
    pub fn light_sampler(&self, lambda: &SampledWavelengths) -> LightSampler<Id> {
        if self.light_sampler_factory.is_none() {
            panic!("Light sampler is not built");
        }
        self.light_sampler_factory
            .as_ref()
            .unwrap()
            .create(&self.primitive_repository, lambda)
    }

    /// 光源の強さを計算する。
    /// 光源がデルタライトの場合はLightIntensity::Intensityを返す。
    /// 面積光源の場合はLightIntensity::Radianceを返す。
    ///
    /// # Arguments
    /// * `primitive_index` - ライトのプリミティブのインデックス
    /// * `shading_point` - シェーディングポイントの情報
    /// * `lambda` - サンプルする波長
    /// * `s` - サンプリングのための1次元のランダム値
    /// * `uv` - サンプリングのための2次元のランダム値
    pub fn calculate_light(
        &self,
        primitive_index: PrimitiveIndex<Id>,
        shading_point: &SurfaceInteraction<Id, Render>,
        lambda: &SampledWavelengths,
        s: f32,
        uv: glam::Vec2,
    ) -> LightIntensity<Id, Render> {
        let primitive = self.primitive_repository.get(primitive_index);
        if let Some(delta_light) = primitive.as_delta_point_light() {
            let intensity = delta_light.calculate_intensity(shading_point, lambda);
            LightIntensity::IntensityDeltaPointLight(intensity)
        } else if let Some(delta_light) = primitive.as_delta_directional_light() {
            let intensity = delta_light.calculate_intensity(shading_point, lambda);
            LightIntensity::IntensityDeltaDirectionalLight(intensity)
        } else if let Some(_inf_light) = primitive.as_infinite_light() {
            todo!()
        } else if let Some(area_light) = primitive.as_area_light() {
            let radiance = area_light.sample_radiance(
                primitive_index,
                &self.geometry_repository,
                shading_point,
                lambda,
                s,
                uv,
            );
            LightIntensity::RadianceAreaLight(radiance)
        } else {
            panic!("Primitive is not a light");
        }
    }

    /// シーン上の点を光源としてサンプルする場合のpdfを計算する。
    pub fn pdf_light_sample(
        &self,
        light_sampler: &LightSampler<Id>,
        shading_point: &SurfaceInteraction<Id, Render>,
        interaction: &SurfaceInteraction<Id, Render>,
    ) -> f32 {
        let primitive_index = interaction.primitive_index;
        let primitive = self.primitive_repository.get(primitive_index);
        if let Some(light) = primitive.as_non_delta_light() {
            // ライトの選択確率を計算する。
            let probability = light_sampler.probability(&primitive_index);

            // ライトのpdfを計算する。
            let light_pdf_area = light.pdf_light_sample(interaction);

            // pdfを面積要素から方向要素に変換する。
            let distance_vector = interaction.position.vector_to(shading_point.position);
            let distance = distance_vector.length();
            let wo = -distance_vector.normalize();
            let light_pdf_dir =
                light_pdf_area * (distance * distance) / interaction.normal.dot(wo).abs();

            probability * light_pdf_dir
        } else {
            0.0
        }
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
