//! シーン20

use std::path::PathBuf;

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, LambertMaterial, NormalMappingMaterial,
    NormalParameter, NormalTexture, Scene, SceneId, SimplePbrMaterial, SpectrumParameter,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_20<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/sphere.obj");
    let spectrum_param = SpectrumParameter::constant(
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8)),
    );

    let normal_texture = NormalTexture::load("./renderer/assets/sphere/normals.png", false)
        .expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    let base_material = SimplePbrMaterial::new(
        // base_color
        spectrum_param,
        // metallic = 1.0 (完全金属)
        FloatParameter::constant(1.0),
        // roughness
        // FloatParameter::constant(0.35),
        // FloatParameter::constant(0.1_f32.sqrt()),
        FloatParameter::constant(0.1),
        // normal mapping
        NormalParameter::none(),
        // ior
        FloatParameter::constant(1.5),
    );

    // let base_material = LambertMaterial::new(spectrum_param, NormalParameter::none());

    let normal_mapped_material = NormalMappingMaterial::new(base_material, normal_param, 1.0, true);

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: normal_mapped_material,
        // surface_material: base_material,
        transform: Transform::identity()
            .scale(glam::vec3(3.0, 3.0, 3.0))
            .rotate(glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                30.0_f32.to_radians(),
                30.0_f32.to_radians(),
                -30.0_f32.to_radians(),
            ))
            .translate(glam::vec3(0.0, 1.5, 0.0)),
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

    scene.create_primitive(CreatePrimitiveDesc::EnvironmentLightPrimitive {
        intensity: 1.0,
        texture_path: PathBuf::from("./renderer/assets/sky/scythian_tombs_2_1k.exr"),
        transform: Transform::identity(),
    });

    // camera.set_look_to(
    //     Point3::new(0.0, 0.0, 1.5),
    //     Vector3::new(0.0, 0.0, -1.0).normalize(),
    //     Vector3::new(0.0, 1.0, 0.0),
    // );
    camera.set_look_to(
        Point3::new(0.0, 3.15221, 6.0),
        Vector3::new(0.0, -0.9, -3.2).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
