//! シーン16: Cornell box with clearcoat PBR bunny

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, LambertMaterial, NormalParameter,
    SimpleClearcoatPbrMaterial, SpectrumParameter,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_16<Id: scene::SceneId, F: Filter>(
    scene: &mut scene::Scene<Id>,
    camera: &mut Camera<F>,
) {
    // clearcoat PBR bunnyのシーン

    // clearcoat PBR bunny
    let geom = scene.load_obj("./renderer/assets/bunny.obj");

    // 銀色のベースカラー
    let base_color_spectrum =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    let base_color_param = SpectrumParameter::constant(base_color_spectrum);

    // metallic = 1.0 (完全金属)
    let metallic_param = FloatParameter::constant(1.0);

    // roughness
    let roughness_param = FloatParameter::constant(0.1);

    // ior
    let ior_param = FloatParameter::constant(1.5);

    // clearcoat設定
    let clearcoat_ior_param = FloatParameter::constant(1.5);
    let clearcoat_roughness_param = FloatParameter::constant(0.08);

    // 青いtint
    let clearcoat_tint_spectrum =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.7, 0.8, 1.0));
    let clearcoat_tint_param = SpectrumParameter::constant(clearcoat_tint_spectrum);

    // thickness（0.001m = 1mm程度）
    let clearcoat_thickness_param = FloatParameter::constant(0.001);

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: SimpleClearcoatPbrMaterial::new(
            base_color_param,
            metallic_param,
            roughness_param,
            NormalParameter::none(),
            ior_param,
            clearcoat_ior_param,
            clearcoat_roughness_param,
            clearcoat_tint_param,
            clearcoat_thickness_param,
        ),
        transform: Transform::identity(),
    });

    // Cornell boxの他の部分
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
        Point3::new(0.0, 3.15221, 6.0),
        Vector3::new(0.0, -0.9, -3.2).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
