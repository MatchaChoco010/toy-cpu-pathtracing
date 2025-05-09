//! カメラを定義するモジュール。

use math::{Point3, Ray, Render, Transform, Vector3, World};

use crate::filter::Filter;

/// カメラからサンプルしたレイとその重みの構造体。
pub struct RaySample {
    pub ray: Ray<Render>,
    pub weight: f32,
}

/// カメラの構造体。
pub struct Camera<F: Filter> {
    position: Point3<World>,
    direction: Vector3<World>,
    up: Vector3<World>,
    fov: f32,
    width: u32,
    height: u32,
    filter: F,
}
impl<F: Filter> Camera<F> {
    /// 新しいカメラを作成する。
    pub fn new(fov: f32, width: u32, height: u32, filter: F) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fov,
            width,
            height,
            filter,
        }
    }

    /// カメラの姿勢を更新する。
    pub fn set_look_to(
        &mut self,
        position: Point3<World>,
        direction: Vector3<World>,
        up: Vector3<World>,
    ) {
        self.position = position;
        self.direction = direction.normalize();
        self.up = up.normalize();
    }

    /// ピンホールカメラのレイを生成する。
    fn generate_ray(&self, x: f32, y: f32) -> Ray<Render> {
        // カメラ座標系でのレイを生成する。
        let aspect_ratio = self.width as f32 / self.height as f32;
        let fov_rad = self.fov.to_radians();
        let scale = (fov_rad / 2.0).tan();
        let dir_x = (2.0 * x / self.width as f32 - 1.0) * aspect_ratio * scale;
        let dir_y = (1.0 - 2.0 * y / self.height as f32) * scale;
        let ray_direction = glam::Vec3::new(dir_x, dir_y, -1.0).normalize();

        // カメラ座標系からレンダリング座標系に変換する。
        let mat = glam::Mat3::look_to_rh(self.direction.to_vec3(), self.up.to_vec3()).transpose();
        let ray_direction = Vector3::from(mat * ray_direction).normalize();

        Ray::new(Point3::from(glam::Vec3::ZERO), ray_direction)
    }

    /// 指定されたピクセル座標からレイをサンプリングする。
    pub fn sample_ray(&self, x: u32, y: u32, uv: glam::Vec2) -> RaySample {
        // フィルタからサンプリングする。
        let fs = self.filter.sample(uv);
        let x = x as f32 + fs.x + 0.5;
        let y = y as f32 + fs.y + 0.5;

        // カメラ座標系でのレイを生成する。
        let ray = self.generate_ray(x, y);

        RaySample {
            ray,
            weight: fs.weight,
        }
    }

    /// ワールド座標系からレンダリング座標系への変換を取得する。
    pub fn world_to_render(&self) -> Transform<World, Render> {
        Transform::translation(-self.position.to_vec3())
    }
}
