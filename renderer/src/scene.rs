//! シーンにオブジェクトを配置する関数を集めたモジュール。

use std::sync::Arc;

use math::{Normal, Point3, Transform, Vector3};
use scene::{CreatePrimitiveDesc, Scene, SceneId, SurfaceMaterial};

use crate::camera::Camera;
use crate::filter::Filter;

pub fn load_scene_0<Id: SceneId, F: Filter>(scene: &mut Scene<Id>, camera: &mut Camera<F>) {
    let geom = scene.load_obj("./renderer/assets/bunny.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/box.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/hidari.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/migi.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/oku.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/tenjou.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });

    let geom = scene.load_obj("./renderer/assets/light.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
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
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::identity(),
    });
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::rotation(glam::Quat::from_rotation_y(30_f32.to_radians()))
            .translate(glam::vec3(-1.0, 1.0, 3.0)),
    });

    let geom = scene.load_obj("./renderer/assets/yuka.obj");
    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
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
        surface_material: Arc::new(SurfaceMaterial {
            bsdf: None,
            edf: None,
        }),
        transform: Transform::rotation(glam::Quat::from_rotation_y(60_f32.to_radians())),
    });

    camera.set_look_to(
        Point3::new(0.0, 3.5, 7.0),
        Vector3::new(0.0, -1.0, -3.0).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
