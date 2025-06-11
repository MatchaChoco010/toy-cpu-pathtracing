//! 空間上のバウンディングボックスを表す構造体を定義するモジュール。

use crate::{CoordinateSystem, Point3, Ray};

/// BoundsとRayの交差したtの値を表す構造体。
pub struct BoundsIntersection {
    pub t0: f32,
    pub t1: f32,
}

/// Bounds構造体。
#[derive(Debug, Clone)]
pub struct Bounds<C: CoordinateSystem> {
    pub min: Point3<C>,
    pub max: Point3<C>,
}
impl<C: CoordinateSystem> Bounds<C> {
    /// Boundsを作成する。
    #[inline(always)]
    pub fn new(min: impl AsRef<Point3<C>>, max: impl AsRef<Point3<C>>) -> Self {
        let min = *min.as_ref();
        let max = *max.as_ref();
        Self { min, max }
    }

    /// 交差を判定する。
    pub fn intersect(
        &self,
        ray: impl AsRef<Ray<C>>,
        t_max: f32,
        inv_dir: glam::Vec3,
    ) -> Option<BoundsIntersection> {
        let ray = ray.as_ref();

        let mut t0 = 0.0;
        let mut t1 = t_max;

        for i in 0..3 {
            let mut t_near = (self.min.to_vec3()[i] - ray.origin.to_vec3()[i]) * inv_dir[i];
            let mut t_far = (self.max.to_vec3()[i] - ray.origin.to_vec3()[i]) * inv_dir[i];

            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if t_far < t1 { t_far } else { t1 };

            if t0 > t1 {
                return None;
            }
        }

        Some(BoundsIntersection { t0, t1 })
    }

    /// Boundsの中心を取得する。
    #[inline(always)]
    pub fn center(&self) -> Point3<C> {
        let center = (self.min.to_vec3() + self.max.to_vec3()) * 0.5;
        Point3::from(center)
    }

    /// Boundsの表面積を取得する。
    #[inline(always)]
    pub fn area(&self) -> f32 {
        let d = self.max.to_vec3() - self.min.to_vec3();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    /// Boundsを含むバウンディングスフィアを計算する。
    #[inline(always)]
    pub fn bounding_sphere(&self) -> (Point3<C>, f32) {
        let center = self.center();
        let radius = center.distance(self.max);
        (center, radius)
    }

    /// Boundsをマージする。
    #[inline(always)]
    pub fn merge(&self, other: impl AsRef<Bounds<C>>) -> Self {
        let other = other.as_ref();
        let min = Point3::from(glam::Vec3::min(self.min.to_vec3(), other.min.to_vec3()));
        let max = Point3::from(glam::Vec3::max(self.max.to_vec3(), other.max.to_vec3()));
        Bounds::new(min, max)
    }

    /// 頂点を取得する。
    #[inline(always)]
    pub fn vertices(&self) -> [Point3<C>; 8] {
        let min = self.min.to_vec3();
        let max = self.max.to_vec3();
        [
            Point3::from(glam::Vec3::new(min.x, min.y, min.z)),
            Point3::from(glam::Vec3::new(max.x, min.y, min.z)),
            Point3::from(glam::Vec3::new(min.x, max.y, min.z)),
            Point3::from(glam::Vec3::new(max.x, max.y, min.z)),
            Point3::from(glam::Vec3::new(min.x, min.y, max.z)),
            Point3::from(glam::Vec3::new(max.x, min.y, max.z)),
            Point3::from(glam::Vec3::new(min.x, max.y, max.z)),
            Point3::from(glam::Vec3::new(max.x, max.y, max.z)),
        ]
    }
}
impl<C: CoordinateSystem> AsRef<Bounds<C>> for Bounds<C> {
    #[inline(always)]
    fn as_ref(&self) -> &Bounds<C> {
        self
    }
}
