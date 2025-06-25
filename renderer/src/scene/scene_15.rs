//! シーン15: Cornell box with PBR dragon

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, FloatTexture, LambertMaterial,
    NormalParameter, NormalTexture, RgbTexture, Scene, SceneId, SimplePbrMaterial,
    SpectrumParameter, SpectrumType,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_15<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // PBRドラゴンを含むコーネルボックス

    // PBRドラゴン
    let geom = scene.load_obj("./renderer/assets/dragon.min.obj");

    // BaseColorテクスチャ
    let base_color_texture =
        RgbTexture::load_srgb("./renderer/assets/dragon-material/BaseColor.png")
            .expect("Failed to load base color texture");
    let base_color_param = SpectrumParameter::texture(base_color_texture, SpectrumType::Albedo);

    // Metallicテクスチャ
    let metallic_texture =
        FloatTexture::load("./renderer/assets/dragon-material/Metallic.png", false)
            .expect("Failed to load metallic texture");
    let metallic_param = FloatParameter::texture(metallic_texture);

    // Roughnessテクスチャ
    let roughness_texture =
        FloatTexture::load("./renderer/assets/dragon-material/Roughness.png", false)
            .expect("Failed to load roughness texture");
    let roughness_param = FloatParameter::texture(roughness_texture);

    // ノーマルマップテクスチャ
    let normal_texture = NormalTexture::load("./renderer/assets/dragon-material/Normal.png", false)
        .expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    // 屈折率設定
    let ior_param = FloatParameter::constant(1.5);

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: SimplePbrMaterial::new(
            base_color_param,
            metallic_param,
            roughness_param,
            normal_param,
            ior_param,
        ),
        transform: Transform::identity()
            .rotate(glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                0.0,
                120_f32.to_radians(),
                0.0,
            ))
            .scale(glam::vec3(2.5, 2.5, 2.5))
            .translate(glam::vec3(0.0, 0.0, 0.5)),
    });

    // コーネルボックスの背景壁
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

    // カメラ位置設定
    camera.set_look_to(
        Point3::new(0.0, 3.15221, 6.0),
        Vector3::new(0.0, -0.9, -3.2).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
