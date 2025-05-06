use image::{Rgb, RgbImage};
use rayon::prelude::*;

pub mod math;
pub mod scene;
pub mod spectrum;

pub struct Camera {
    position: math::Point3<math::World>,
    direction: math::Vector3<math::World>,
    up: math::Vector3<math::World>,
    fov: f32,
}
impl Camera {
    pub fn new(
        position: math::Point3<math::World>,
        direction: math::Vector3<math::World>,
        up: math::Vector3<math::World>,
        fov: f32,
    ) -> Self {
        Self {
            position,
            direction,
            up,
            fov,
        }
    }

    pub fn generate_ray(&self, x: u32, y: u32, res_x: u32, res_y: u32) -> math::Ray<math::Render> {
        let aspect_ratio = res_x as f32 / res_y as f32;

        let fov_rad = self.fov.to_radians();
        let scale = (fov_rad / 2.0).tan();
        let dir_x = (2.0 * (x as f32 + 0.5) / res_x as f32 - 1.0) * aspect_ratio * scale;
        let dir_y = (1.0 - 2.0 * (y as f32 + 0.5) / res_y as f32) * scale;

        let front = -self.direction.to_vec3().normalize();
        let right = self.up.to_vec3().cross(front).normalize();
        let up = front.cross(right).normalize();
        let mat = glam::Mat3::from_cols(right, up, front);
        let ray_direction = mat * glam::Vec3::new(dir_x, dir_y, -1.0).normalize();
        let ray_direction = math::Vector3::from(ray_direction).normalize();

        math::Ray::new(math::Point3::from(self.position.to_vec3()), ray_direction)
    }
}

fn add_scene_items_0<Id: scene::SceneId>(scene: &mut scene::Scene<Id>) {
    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/bunny.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/box.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/hidari.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/migi.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/yuka.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/oku.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/tenjou.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/light.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });
}

fn add_scene_items_1<Id: scene::SceneId>(scene: &mut scene::Scene<Id>) {
    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/bunny.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::rotation(glam::Quat::from_rotation_y(30_f32.to_radians()))
            .translate(glam::vec3(-1.0, 1.0, 3.0)),
    });

    let geom = scene.load_obj("./toy-cpu-pathtracing/assets/yuka.obj");
    scene.create_primitive(scene::CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: geom,
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::identity(),
    });

    scene.create_primitive(scene::CreatePrimitiveDesc::SingleTrianglePrimitive {
        positions: [
            math::Point3::new(-2.0, 0.0, 0.0),
            math::Point3::new(2.0, 0.0, 0.0),
            math::Point3::new(-2.0, 4.0, 0.0),
        ],
        normals: [
            math::Normal::new(0.0, 0.0, 1.0),
            math::Normal::new(0.0, 0.0, 1.0),
            math::Normal::new(0.0, 0.0, 1.0),
        ],
        uvs: [
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(1.0, 0.0),
            glam::Vec2::new(0.0, 1.0),
        ],
        material_id: scene::MaterialId::new(0),
        transform: math::Transform::rotation(glam::Quat::from_rotation_y(60_f32.to_radians())),
    });
}

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();

    let mut scene = scene::create_scene!(scene);

    let scene_number = argv
        .get(1)
        .map(|arg| arg.parse::<u32>().ok())
        .flatten()
        .unwrap_or(0);
    match scene_number {
        0 => add_scene_items_0(&mut scene),
        1 => add_scene_items_1(&mut scene),
        _ => panic!("Invalid scene number"),
    }

    println!("Start build scene...");
    let start = std::time::Instant::now();

    scene.build();

    let end = start.elapsed();
    println!("Finish build scene: {} seconds.", end.as_secs_f32());

    let camera = Camera::new(
        math::Point3::new(0.0, 3.5, 6.0),
        math::Vector3::new(0.0, -1.0, -3.0).normalize(),
        math::Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    let width = 800;
    let height = 600;

    println!("Start rendering...");
    let start = std::time::Instant::now();

    let mut img = RgbImage::new(width, height);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let ray = camera.generate_ray(*x, *y, width, height);
            let intersect = scene.intersect(&ray, f32::MAX);
            let color = match intersect {
                Some(intersect) => match intersect.interaction {
                    scene::Interaction::Surface { shading_normal, .. } => {
                        shading_normal.to_vec3() * 0.5 + glam::Vec3::splat(0.5)
                    }
                },
                None => glam::Vec3::ZERO,
            };

            // pixel[0] = (255.0 * (*x as f64 / width as f64)) as u8;
            // pixel[1] = (255.0 * (*y as f64 / height as f64)) as u8;
            // pixel[2] = 128;
            pixel[0] = (color.x * 255.0) as u8;
            pixel[1] = (color.y * 255.0) as u8;
            pixel[2] = (color.z * 255.0) as u8;
        });
    img.save("output.png").unwrap();

    let end = start.elapsed();
    println!("Finish rendering: {} seconds.", end.as_secs_f32());
}
