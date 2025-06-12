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

pub fn load_scene_0<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
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
            FloatParameter::constant(1.0),
        ),
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
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.5, 0.5, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.5, 0.8, 0.5));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
        transform: Transform::from_rotate(glam::Quat::from_rotation_y(30_f32.to_radians()))
            .translate(glam::vec3(-1.0, 1.0, 3.0)),
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

    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.5, 0.5));
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
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
        ),
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
    let spectrum = RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(spectrum),
            NormalParameter::none(),
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
    let config = TextureConfig::new("./renderer/assets/bunny-material-0/BaseColor.png");
    let texture = RgbTexture::load(config).expect("Failed to load texture");
    let _spectrum_param = SpectrumParameter::texture(texture, SpectrumType::Albedo);

    let normal_config = TextureConfig::new("./renderer/assets/bunny-material-0/Normal.png");
    let normal_texture =
        NormalTexture::load(normal_config, false).expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    // let normal_param = NormalParameter::none();

    let spectrum_param = SpectrumParameter::constant(
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8)),
    );

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(spectrum_param, normal_param),
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
            FloatParameter::constant(1.0),
        ),
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
    let config = TextureConfig::new("./renderer/assets/bunny-material-1/BaseColor.png");
    let texture = RgbTexture::load(config).expect("Failed to load texture");
    let spectrum_param = SpectrumParameter::texture(texture, SpectrumType::Albedo);

    let normal_config = TextureConfig::new("./renderer/assets/bunny-material-1/Normal.png");
    let normal_texture =
        NormalTexture::load(normal_config, false).expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(spectrum_param, normal_param),
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
            FloatParameter::constant(1.0),
        ),
        transform: Transform::identity(),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 6.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}

pub fn load_scene_5<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    // テクスチャ付きのbunny（法線マップも追加）
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    let config = TextureConfig::new("./renderer/assets/bunny-material-0/BaseColor.png");
    let texture = RgbTexture::load(config).expect("Failed to load texture");
    let _spectrum_param = SpectrumParameter::texture(texture, SpectrumType::Albedo);

    // Normal mapを有効にしてテスト
    let normal_config = TextureConfig::new("./renderer/assets/bunny-material-0/Normal.png");
    let normal_texture =
        NormalTexture::load(normal_config, false).expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    let spectrum_param = SpectrumParameter::constant(
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8)),
    );

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(spectrum_param, normal_param),
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
            FloatParameter::constant(1.0),
        ),
        transform: Transform::identity(),
    });

    // カメラをうさぎに近づける（より大きく映るように）
    camera.set_look_to(
        Point3::new(0.0, 2.0, 3.5),                // より近い位置
        Vector3::new(0.0, -0.5, -2.0).normalize(), // 少し上から見下ろす
        Vector3::new(0.0, 1.0, 0.0),
    );
}
