//! シーン19: Environment Light test

use std::path::PathBuf;

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{CreatePrimitiveDesc, LambertMaterial, NormalParameter, SpectrumParameter};
use spectrum::RgbAlbedoSpectrum;

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_19<Id: scene::SceneId, F: Filter>(
    scene: &mut scene::Scene<Id>,
    camera: &mut Camera<F>,
) {
    // Environment Light test scene

    // 床オブジェクト (yuka.obj)
    let floor_geom = scene.load_obj("./renderer/assets/yuka.obj");

    // 床の材質（ライトグレー）
    let floor_color =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));

    // 床をシーンに追加
    let _floor_primitive = scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: floor_geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(floor_color),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // ドラゴンモデル
    let dragon_geom = scene.load_obj("./renderer/assets/dragon.min.obj");

    // ドラゴンの材質（ミディアムグレー）
    let dragon_color =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.6, 0.6, 0.6));

    // ドラゴンをシーンに追加
    let _dragon_primitive = scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: dragon_geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(dragon_color),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // Environment Light を追加
    let _environment_light =
        scene.create_primitive(CreatePrimitiveDesc::EnvironmentLightPrimitive {
            intensity: 1.0,
            texture_path: PathBuf::from("./renderer/assets/sky/scythian_tombs_2_1k.exr"),
            transform: Transform::identity(),
        });

    // カメラ設定
    camera.set_look_to(
        Point3::new(0.0, 1.5, 4.0),
        Vector3::new(0.0, -0.3, -4.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
