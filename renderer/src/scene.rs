//! シーンにオブジェクトを配置する関数を集めたモジュール。

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Normal, Point3, Transform, Vector3};
use scene::{CreatePrimitiveDesc, EmissiveMaterial, LambertMaterial, Scene, SceneId};
use spectrum::{RgbAlbedoSpectrum, presets};

use crate::camera::Camera;
use crate::filter::Filter;

pub fn load_scene_0<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(
            ColorSrgb::new(0.9, 0.0, 0.0),
            // ColorSrgb::new(0.9, 0.0, 0.9),
            // ColorSrgb::new(0.8, 0.8, 0.8),
        )),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(
            ColorSrgb::new(0.0, 0.9, 0.0),
            // ColorSrgb::new(0.0, 0.9, 0.9),
            // ColorSrgb::new(0.8, 0.8, 0.8),
            // ColorSrgb::new(137.0 / 255.0, 91.0 / 255.0, 138.0 / 255.0), // 古代紫
            // ColorSrgb::new(107.0 / 255.0, 123.0 / 255.0, 110.0 / 255.0), // 青鈍
            // ColorSrgb::new(147.0 / 255.0, 182.0 / 255.0, 156.0 / 255.0), // 薄青
            // ColorSrgb::new(228.0 / 255.0, 220.0 / 255.0, 138.0 / 255.0), // 枯草色
            // ColorSrgb::new(211.0 / 255.0, 207.0 / 255.0, 217.0 / 255.0), // 暁鼠
            // ColorSrgb::new(170.0 / 255.0, 207.0 / 255.0, 83.0 / 255.0), // 萌黄
            // ColorSrgb::new(235.0 / 255.0, 110.0 / 255.0, 165.0 / 255.0), // 赤紫
            // ColorSrgb::new(204.0 / 255.0, 166.0 / 255.0, 191.0 / 255.0), // 紅藤色
            // ColorSrgb::new(110.0 / 255.0, 121.0 / 255.0, 85.0 / 255.0), // 麹塵
            // ColorSrgb::new(0.0 / 255.0, 164.0 / 255.0, 151.0 / 255.0), // 青緑
            // ColorSrgb::new(56.0 / 255.0, 180.0 / 255.0, 139.0 / 255.0), // 翡翠色
            // ColorSrgb::new(230.0 / 255.0, 180.0 / 255.0, 34.0 / 255.0), // 黄金
        )),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    // ライト
    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: EmissiveMaterial::new(presets::cie_illum_d6500(), 1.0),
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
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.5, 0.5, 0.8,
        ))),
        transform: Transform::identity(),
    });
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.5, 0.8, 0.5,
        ))),
        transform: Transform::from_rotate(glam::Quat::from_rotation_y(30_f32.to_radians()))
            .translate(glam::vec3(-1.0, 1.0, 3.0)),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
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
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.5, 0.5,
        ))),
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
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(
            ColorSrgb::new(0.9, 0.0, 0.0),
            // ColorSrgb::new(0.9, 0.0, 0.9),
            // ColorSrgb::new(0.8, 0.8, 0.8),
        )),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.0, 0.9, 0.0,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: LambertMaterial::new(RgbAlbedoSpectrum::<
            ColorSrgb<NoneToneMap>,
        >::new(ColorSrgb::new(
            0.8, 0.8, 0.8,
        ))),
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
