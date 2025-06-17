//! シーン7: Cornell box with 4 gold bunnies showing microfacet roughness values

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, LambertMaterial, MetalMaterial,
    MetalType, NormalParameter, Scene, SceneId, SpectrumParameter,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_7<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // 4つの金のバニーを異なるroughnessで並べたCornel boxシーン

    let bunny_geom = scene.load_obj("./renderer/assets/bunny.obj");
    let scale = 0.6; // バニーを小さくするスケール

    // 左から右に配置、roughnessも小さいものから大きいものへ
    let positions = [
        glam::vec3(-1.3, 0.0, -0.5), // 左端
        glam::vec3(-0.5, 0.0, -0.5), // 左中
        glam::vec3(0.3, 0.0, -0.5),  // 右中
        glam::vec3(1.1, 0.0, -0.5),  // 右端
    ];

    let roughness_values = [0.05, 0.25, 0.5, 0.75];

    for (&position, &roughness) in positions.iter().zip(roughness_values.iter()) {
        scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
            geometry_index: bunny_geom,
            surface_material: MetalMaterial::new(
                MetalType::Gold,
                NormalParameter::none(),
                FloatParameter::constant(roughness),
            ),
            transform: Transform::from_scale(glam::vec3(scale, scale, scale)).translate(position),
        });
    }

    // Cornell boxの壁
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

    // 左の壁（赤）
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

    // 右の壁（緑）
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

    // 床
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

    // 奥の壁
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

    // 天井
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

    // カメラ位置を調整（4つのバニーがよく見えるように）
    camera.set_look_to(
        Point3::new(0.0, 2.5, 5.0),
        Vector3::new(0.0, -0.7, -2.5).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
