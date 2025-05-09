//! 指向性ライトのプリミティブの実装のモジュール。

use math::{Bounds, LightSampleContext, Local, Render, Transform, World};
use spectrum::SampledWavelengths;

use crate::scene::SceneId;
use crate::scene::primitive::{
    LightIrradiance, PrimitiveDeltaLight, PrimitiveLight, PrimitiveTrait,
};

/// 指向性ライトのプリミティブの構造体。
/// 指向性ライトの方向はローカル座標系のZ+軸方向である。
pub struct DirectionalLight {
    intensity: f32,
    area: Option<f32>,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl DirectionalLight {
    /// 指向性ライトの新しいインスタンスを作成する。
    pub fn new(intensity: f32, local_to_world: Transform<Local, World>) -> Self {
        Self {
            intensity,
            area: None,
            local_to_world,
            local_to_render: Transform::identity(),
        }
    }
}
impl PrimitiveTrait for DirectionalLight {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }
}
impl PrimitiveLight for DirectionalLight {
    fn phi(&self, lambda: &SampledWavelengths) -> f32 {
        let area = self.area.expect("preprocess not called!");
        self.intensity * area
    }

    fn preprocess(&mut self, bounds: &Bounds<Render>) {
        // phiを計算する際に必要なareaを計算する。
        // 垂直放射照度intensityと照射した面積を掛け合わせることで放射束phiが求まる。
        // ここではシーン全体のバウンディングスフィアの断面を照射面積の近似とする。
        let (_center, radius) = bounds.bounding_sphere();
        let area = std::f32::consts::PI * radius * radius;
        self.area = Some(area);
    }
}
impl<Id: SceneId> PrimitiveDeltaLight<Id> for DirectionalLight {
    fn calculate_irradiance(
        &self,
        _light_sample_context: &LightSampleContext<Render>,
        _lambda: &SampledWavelengths,
    ) -> LightIrradiance {
        todo!()
    }
}
