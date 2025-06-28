//! 環境ライトのプリミティブの実装のモジュール。

use std::path::Path;

use image;
use math::{Local, Ray, Render, Transform, World};
use spectrum::{RgbIlluminantSpectrum, SampledSpectrum, SampledWavelengths, Spectrum};
use color::{ColorSrgb, tone_map::NoneToneMap};

use crate::{
    AreaLightSampleRadiance, SceneId, SurfaceInteraction,
    geometry::GeometryRepository,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight, PrimitiveNonDeltaLight,
    },
};

/// 環境ライトのプリミティブの構造体。
pub struct EnvironmentLight {
    intensity: f32,
    integrated_spectrum: Spectrum,
    // HDRIテクスチャデータ（高さ×幅×RGB）
    texture_data: Vec<Vec<[f32; 3]>>,
    texture_width: u32,
    texture_height: u32,
    local_to_world: Transform<Local, World>,
    local_to_render: Transform<Local, Render>,
}
impl EnvironmentLight {
    /// 新しい環境ライトのプリミティブを作成する。
    pub fn new(intensity: f32, path: impl AsRef<Path>, transform: Transform<Local, World>) -> Self {
        // EXRファイルを読み込む
        let img = image::open(path).expect("Failed to load EXR file");
        let rgb_img = img.to_rgb32f();
        let (width, height) = rgb_img.dimensions();

        // テクスチャデータを3次元配列として格納
        let mut texture_data = vec![vec![[0.0f32; 3]; width as usize]; height as usize];
        for y in 0..height {
            for x in 0..width {
                let pixel = rgb_img.get_pixel(x, y);
                texture_data[y as usize][x as usize] = [pixel[0], pixel[1], pixel[2]];
            }
        }

        // テクスチャ全体の積分値を計算してintegrated_spectrumを作成
        let mut total_rgb = [0.0f32; 3];
        for row in &texture_data {
            for pixel in row {
                total_rgb[0] += pixel[0];
                total_rgb[1] += pixel[1];
                total_rgb[2] += pixel[2];
            }
        }

        // 平均値を計算（面積で正規化）
        let pixel_count = (width * height) as f32;
        total_rgb[0] /= pixel_count;
        total_rgb[1] /= pixel_count;
        total_rgb[2] /= pixel_count;

        let color = ColorSrgb::<NoneToneMap>::new(total_rgb[0], total_rgb[1], total_rgb[2]);
        let integrated_spectrum = RgbIlluminantSpectrum::<ColorSrgb<NoneToneMap>>::new(color);

        Self {
            intensity,
            integrated_spectrum,
            texture_data,
            texture_width: width,
            texture_height: height,
            local_to_world: transform,
            local_to_render: Transform::identity(), // buildで設定される
        }
    }

    /// テクスチャ座標(u,v)から球面座標(theta, phi)に変換
    fn uv_to_spherical(u: f32, v: f32) -> (f32, f32) {
        let theta = v * std::f32::consts::PI; // 0 ≤ theta ≤ π
        let phi = u * 2.0 * std::f32::consts::PI; // 0 ≤ phi ≤ 2π
        (theta, phi)
    }

    /// 球面座標(theta, phi)から方向ベクトル(Render座標系、Y上)に変換
    fn spherical_to_direction(theta: f32, phi: f32) -> math::Vector3<Render> {
        let x = theta.sin() * phi.cos();
        let y = theta.cos();
        let z = theta.sin() * phi.sin();
        math::Vector3::new(x, y, z)
    }

    /// 方向ベクトルから球面座標に変換
    fn direction_to_spherical(dir: math::Vector3<Render>) -> (f32, f32) {
        let theta = dir.y().acos().clamp(0.0, std::f32::consts::PI);
        let phi = dir.z().atan2(dir.x());
        let phi = if phi < 0.0 {
            phi + 2.0 * std::f32::consts::PI
        } else {
            phi
        };
        (theta, phi)
    }

    /// 球面座標からテクスチャ座標に変換
    fn spherical_to_uv(theta: f32, phi: f32) -> (f32, f32) {
        let u = phi / (2.0 * std::f32::consts::PI);
        let v = theta / std::f32::consts::PI;
        (u, v)
    }

    /// テクスチャをサンプリング（バイリニア補間）
    fn sample_texture(&self, u: f32, v: f32) -> [f32; 3] {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let x = u * (self.texture_width - 1) as f32;
        let y = v * (self.texture_height - 1) as f32;

        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.texture_width as usize - 1);
        let y1 = (y0 + 1).min(self.texture_height as usize - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let p00 = self.texture_data[y0][x0];
        let p01 = self.texture_data[y0][x1];
        let p10 = self.texture_data[y1][x0];
        let p11 = self.texture_data[y1][x1];

        let p0 = [
            p00[0] * (1.0 - fx) + p01[0] * fx,
            p00[1] * (1.0 - fx) + p01[1] * fx,
            p00[2] * (1.0 - fx) + p01[2] * fx,
        ];
        let p1 = [
            p10[0] * (1.0 - fx) + p11[0] * fx,
            p10[1] * (1.0 - fx) + p11[1] * fx,
            p10[2] * (1.0 - fx) + p11[2] * fx,
        ];

        [
            p0[0] * (1.0 - fy) + p1[0] * fy,
            p0[1] * (1.0 - fy) + p1[1] * fy,
            p0[2] * (1.0 - fy) + p1[2] * fy,
        ]
    }
}
impl<Id: SceneId> Primitive<Id> for EnvironmentLight {
    fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        self.local_to_render = world_to_render * &self.local_to_world;
    }

    fn as_geometry(&self) -> Option<&dyn PrimitiveGeometry<Id>> {
        None
    }

    fn as_geometry_mut(&mut self) -> Option<&mut dyn PrimitiveGeometry<Id>> {
        None
    }

    fn as_light(&self) -> Option<&dyn PrimitiveLight<Id>> {
        Some(self)
    }

    fn as_light_mut(&mut self) -> Option<&mut dyn PrimitiveLight<Id>> {
        Some(self)
    }

    fn as_non_delta_light(&self) -> Option<&dyn PrimitiveNonDeltaLight<Id>> {
        Some(self)
    }

    fn as_delta_point_light(&self) -> Option<&dyn PrimitiveDeltaPointLight<Id>> {
        None
    }

    fn as_delta_directional_light(&self) -> Option<&dyn PrimitiveDeltaDirectionalLight<Id>> {
        None
    }

    fn as_area_light(&self) -> Option<&dyn PrimitiveAreaLight<Id>> {
        None
    }

    fn as_infinite_light(&self) -> Option<&dyn PrimitiveInfiniteLight<Id>> {
        Some(self)
    }
}
impl<Id: SceneId> PrimitiveLight<Id> for EnvironmentLight {
    fn phi(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        self.intensity * self.integrated_spectrum.sample(lambda)
    }
}
impl<Id: SceneId> PrimitiveNonDeltaLight<Id> for EnvironmentLight {
    fn sample_radiance(
        &self,
        _geometry_repository: &GeometryRepository<Id>,
        _shading_point: &SurfaceInteraction<Render>,
        _lambda: &SampledWavelengths,
        _s: f32,
        _uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render> {
        todo!()
    }
}
impl<Id: SceneId> PrimitiveInfiniteLight<Id> for EnvironmentLight {
    fn direction_radiance(
        &self,
        ray: &Ray<Render>,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        // 方向を球面座標に変換
        let (theta, phi) = Self::direction_to_spherical(ray.dir);

        // 球面座標をテクスチャ座標に変換
        let (u, v) = Self::spherical_to_uv(theta, phi);

        // テクスチャをサンプリング
        let rgb = self.sample_texture(u, v);

        // RGBからスペクトルに変換してintensityを適用
        let color = ColorSrgb::<NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        let spectrum = RgbIlluminantSpectrum::<ColorSrgb<NoneToneMap>>::new(color);
        spectrum.sample(lambda) * self.intensity
    }

    fn pdf_direction_sample(
        &self,
        _shading_point: &SurfaceInteraction<Render>,
        _wi: math::Vector3<Render>,
    ) -> f32 {
        todo!()
    }
}
