//! シーンにオブジェクトを配置する関数を集めたモジュール。

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Normal, Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, EmissiveMaterial, FloatParameter, LambertMaterial, NormalParameter,
    NormalTexture, RgbTexture, Scene, SceneId, SpectrumParameter, SpectrumType, TextureConfig,
};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::camera::Camera;
use crate::filter::Filter;

/// RGB値からLambertマテリアルを作成するヘルパー関数。
/// RGB値はsRGB値として扱い、RGB-to-spectrum内部でガンマ補正除去が行われる。
fn create_lambert_rgb(r: f32, g: f32, b: f32) -> scene::Material {
    // sRGB値をそのまま渡す（RGB-to-spectrum内部でinvert_eotf()が行われる）
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(r, g, b));
    LambertMaterial::new_simple(SpectrumParameter::constant(spectrum))
}

/// スペクトラムとintensityからEmissiveマテリアルを作成するヘルパー関数。
fn create_emissive(spectrum: spectrum::Spectrum, intensity: f32) -> scene::Material {
    EmissiveMaterial::new(
        SpectrumParameter::constant(spectrum),
        FloatParameter::constant(intensity),
    )
}

/// テクスチャからLambertマテリアルを作成するヘルパー関数。
fn create_lambert_texture(texture_path: &str) -> scene::Material {
    let config = TextureConfig::new(texture_path);
    let texture = RgbTexture::load(config).expect("Failed to load texture");
    let spectrum_param = SpectrumParameter::texture(texture, SpectrumType::Albedo);
    LambertMaterial::new_simple(spectrum_param)
}

/// テクスチャとノーマルマップからLambertマテリアルを作成するヘルパー関数。
fn create_lambert_texture_with_normal(texture_path: &str, normal_path: &str) -> scene::Material {
    let config = TextureConfig::new(texture_path);
    let texture = RgbTexture::load(config).expect("Failed to load texture");
    let spectrum_param = SpectrumParameter::texture(texture, SpectrumType::Albedo);

    let normal_config = TextureConfig::new(normal_path);
    let normal_texture =
        NormalTexture::load(normal_config, false).expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    LambertMaterial::new(spectrum_param, normal_param)
}

pub fn load_scene_0<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.9, 0.0, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.0, 0.9, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    // ライト
    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_emissive(presets::cie_illum_d6500(), 1.0),
        transform: Transform::identity(),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}

pub fn load_scene_1<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.5, 0.5, 0.8),
        transform: Transform::identity(),
    });
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.5, 0.8, 0.5),
        transform: Transform::from_rotate(glam::Quat::from_rotation_y(30_f32.to_radians()))
            .translate(glam::vec3(-1.0, 1.0, 3.0)),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    scene.create_primitive(CreatePrimitiveDesc::SingleTrianglePrimitive {
        positions: [
            Point3::new(-2.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(-2.0, 4.0, 0.0),
        ],
        normals: [
            Normal::new(0.0, 0.0, 1.0),
            Normal::new(0.0, 0.0, 1.0),
            Normal::new(0.0, 0.0, 1.0),
        ],
        uvs: [
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(1.0, 0.0),
            glam::Vec2::new(0.0, 1.0),
        ],
        surface_material: create_lambert_rgb(0.8, 0.5, 0.5),
        transform: Transform::from_rotate(glam::Quat::from_rotation_y(60_f32.to_radians())),
    });

    scene.create_primitive(CreatePrimitiveDesc::PointLightPrimitive {
        intensity: 1.0,
        spectrum: presets::cie_illum_d6500(),
        transform: Transform::from_translate(glam::vec3(0.0, 3.0, 0.0)),
    });

    scene.create_primitive(CreatePrimitiveDesc::PointLightPrimitive {
        intensity: 2.0,
        spectrum: presets::cie_illum_d6500(),
        transform: Transform::from_translate(glam::vec3(3.0, 5.0, 0.0)),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 7.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}

pub fn load_scene_2<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.9, 0.0, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.0, 0.9, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    scene.create_primitive(CreatePrimitiveDesc::PointLightPrimitive {
        intensity: 1.0,
        spectrum: presets::cie_illum_d6500(),
        transform: Transform::from_translate(glam::vec3(0.0, 3.0, 0.0)),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}

pub fn load_scene_3<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // scene 0をベースに、bunnyにテクスチャを適用したシーン

    // テクスチャ付きのbunny（法線マップも追加）
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    let bunny_material = create_lambert_texture_with_normal(
        "./renderer/assets/bunny-material-0/BaseColor.png",
        "./renderer/assets/bunny-material-0/Normal.png",
    );
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: bunny_material,
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.9, 0.0, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.0, 0.9, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    // ライト
    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_emissive(presets::cie_illum_d6500(), 1.0),
        transform: Transform::identity(),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}

pub fn load_scene_4<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // scene 3と同様だが、bunny-material-1のテクスチャを使用したシーン

    // bunny-material-1のテクスチャ付きのbunny
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    let bunny_material = create_lambert_texture_with_normal(
        "./renderer/assets/bunny-material-1/BaseColor.png",
        "./renderer/assets/bunny-material-1/Normal.png",
    );
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: bunny_material,
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.9, 0.0, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.0, 0.9, 0.0),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_lambert_rgb(0.8, 0.8, 0.8),
        transform: Transform::identity(),
    });

    // ライト
    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: create_emissive(presets::cie_illum_d6500(), 1.0),
        transform: Transform::identity(),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
