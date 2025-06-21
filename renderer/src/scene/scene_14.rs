//! シーン14: Cornell box with 4 colored plastic bunnies showing different colors and roughness (based on scene 12)

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, LambertMaterial, NormalParameter,
    PlasticMaterial, Scene, SceneId, SpectrumParameter,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_14<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // scene 12をベースに、4つの色つき透明プラスチックバニーを異なる色とroughnessで並べたCornel boxシーン

    let bunny_geom = scene.load_obj("./renderer/assets/bunny.obj");
    let scale = 0.6; // バニーを小さくするスケール

    // 左から右に配置、roughnessも小さいものから大きいものへ
    let positions = [
        glam::vec3(-1.3, 0.0, -0.5), // 左端
        glam::vec3(-0.5, 0.0, -0.5), // 左中
        glam::vec3(0.3, 0.0, -0.5),  // 右中
        glam::vec3(1.1, 0.0, -0.5),  // 右端
    ];

    let roughness_values = [0.05, 0.1, 0.3, 0.5];

    // 異なる色のスペクトラム
    let colors = [
        ColorSrgb::new(1.0, 0.5, 0.5), // 赤
        ColorSrgb::new(0.5, 1.0, 0.5), // 緑
        ColorSrgb::new(0.5, 0.5, 1.0), // 青
        ColorSrgb::new(1.0, 0.8, 0.4), // 黄色
    ];

    for (i, (&position, &roughness)) in positions.iter().zip(roughness_values.iter()).enumerate() {
        let color_spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(colors[i].clone());
        scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
            geometry_index: bunny_geom,
            surface_material: PlasticMaterial::new(
                1.5,
                SpectrumParameter::constant(color_spectrum),
                NormalParameter::none(),
                false, // Thin Film効果を無効
                FloatParameter::constant(roughness),
            ),
            transform: Transform::from_scale(glam::vec3(scale, scale, scale)).translate(position),
        });
    }

    // Cornell boxの壁 (白)
    let geom = scene.load_obj("./renderer/assets/box.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // 左壁 (赤)
    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.9, 0.0, 0.0));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // 右壁 (緑)
    let geom = scene.load_obj("./renderer/assets/migi.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.0, 0.9, 0.0));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // 床 (白)
    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // 奥壁 (白)
    let geom = scene.load_obj("./renderer/assets/oku.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // 天井 (白)
    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // ライト
    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: EmissiveMaterial::new(
            SpectrumParameter::constant(presets::cie_illum_d6500()),
            FloatParameter::constant(10.0),
        ),
        transform: Transform::identity(),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
