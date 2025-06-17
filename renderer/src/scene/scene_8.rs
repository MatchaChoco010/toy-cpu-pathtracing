//! シーン8: Cornell box with glass bunny

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, GlassMaterial, GlassType,
    LambertMaterial, NormalParameter, Scene, SceneId, SpectrumParameter,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_8<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // scene 3と同じコーネルボックスで、うさぎをガラスのマテリアルにするシーン

    // BK7ガラスのマテリアルでバニー
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: GlassMaterial::new(
            GlassType::Bk7,
            NormalParameter::none(),
            false, // 通常のガラス（thin filmではない）
        ),
        transform: Transform::identity(),
    });

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
