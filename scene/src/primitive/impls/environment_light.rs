//! 環境ライトのプリミティブの実装のモジュール。

use std::path::Path;

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Local, Ray, Render, Transform, World};
use spectrum::{RgbIlluminantSpectrum, SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    InfiniteLightSampleRadiance, SceneId, SurfaceInteraction,
    primitive::traits::{
        Primitive, PrimitiveAreaLight, PrimitiveDeltaDirectionalLight, PrimitiveDeltaPointLight,
        PrimitiveGeometry, PrimitiveInfiniteLight, PrimitiveLight,
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
    // 事前計算済みサンプリングデータ
    marginal_cdf: Vec<f32>,         // 各行の累積重み（行選択用）
    conditional_cdf: Vec<Vec<f32>>, // 各行内の累積重み（列選択用）
    total_weight: f32,              // 全体の重み合計
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

        // 2段階CDFテーブルを事前計算
        let (marginal_cdf, conditional_cdf, total_weight) =
            Self::build_2d_cdf(&texture_data, width, height);

        Self {
            intensity,
            integrated_spectrum,
            texture_data,
            texture_width: width,
            texture_height: height,
            local_to_world: transform,
            local_to_render: Transform::identity(), // buildで設定される
            marginal_cdf,
            conditional_cdf,
            total_weight,
        }
    }

    /// テクスチャ座標(u,v)から球面座標(theta, phi)に変換
    fn uv_to_spherical(u: f32, v: f32) -> (f32, f32) {
        let theta = v * std::f32::consts::PI;
        let phi = u * 2.0 * std::f32::consts::PI;
        (theta, phi)
    }

    /// 球面座標(theta, phi)から方向ベクトル(Local座標系、Y上)に変換
    fn spherical_to_direction(theta: f32, phi: f32) -> math::Vector3<Local> {
        let x = theta.sin() * phi.cos();
        let y = theta.cos();
        let z = theta.sin() * phi.sin();
        math::Vector3::new(x, y, z)
    }

    /// 方向ベクトル(Local座標系)から球面座標に変換
    fn direction_to_spherical(dir: math::Vector3<Local>) -> (f32, f32) {
        let theta = dir.y().acos().clamp(0.0, std::f32::consts::PI);
        let mut phi = dir.z().atan2(dir.x());
        if phi < 0.0 {
            phi += 2.0 * std::f32::consts::PI;
        }
        (theta, phi)
    }

    /// 球面座標からテクスチャ座標に変換
    fn spherical_to_uv(theta: f32, phi: f32) -> (f32, f32) {
        let u = phi / (2.0 * std::f32::consts::PI);
        let v = theta / std::f32::consts::PI;
        (u, v)
    }

    /// ヤコビアン重み（正距円筒図法の歪み補正）を計算
    fn jacobian_weight(theta: f32) -> f32 {
        theta.sin().max(1e-8)
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

    /// 2段階CDFテーブルを事前構築
    /// 重み: luminance * sin(theta) による重要度サンプリング
    /// 1段階目: 行選択用周辺CDF、2段階目: 各行内の列選択用条件付きCDF
    fn build_2d_cdf(
        texture_data: &[Vec<[f32; 3]>],
        width: u32,
        height: u32,
    ) -> (Vec<f32>, Vec<Vec<f32>>, f32) {
        let mut row_weights = vec![0.0f32; height as usize];
        let mut conditional_cdf = vec![vec![0.0f32; width as usize]; height as usize];

        for y in 0..height {
            let mut row_sum = 0.0f32;

            for x in 0..width {
                let u = (x as f32 + 0.5) / width as f32;
                let v = (y as f32 + 0.5) / height as f32;

                let (theta, _phi) = Self::uv_to_spherical(u, v);
                let pixel_rgb = texture_data[y as usize][x as usize];
                let luminance = 0.299 * pixel_rgb[0] + 0.587 * pixel_rgb[1] + 0.114 * pixel_rgb[2];
                let jacobian_weight = Self::jacobian_weight(theta);
                // 重要度サンプリング重み: 輝度 × sin(theta)
                let weight = luminance * jacobian_weight;
                row_sum += weight;

                conditional_cdf[y as usize][x as usize] = row_sum;
            }

            row_weights[y as usize] = row_sum;
            // 各行の条件付きCDFを正規化
            if row_sum > 0.0 {
                for x in 0..width {
                    conditional_cdf[y as usize][x as usize] /= row_sum;
                }
            }
        }

        // 周辺CDF（行選択用）を構築
        let total_weight: f32 = row_weights.iter().sum();
        let mut marginal_cdf = vec![0.0f32; height as usize];
        let mut cumulative = 0.0f32;

        for y in 0..height {
            cumulative += row_weights[y as usize];
            marginal_cdf[y as usize] = if total_weight > 0.0 {
                cumulative / total_weight
            } else {
                (y + 1) as f32 / height as f32
            };
        }

        (marginal_cdf, conditional_cdf, total_weight)
    }

    /// バイナリサーチでCDFから値をサンプリング
    fn sample_from_cdf(cdf: &[f32], u: f32) -> usize {
        match cdf.binary_search_by(|&x| x.partial_cmp(&u).unwrap()) {
            Ok(index) => index,
            Err(index) => index.min(cdf.len() - 1),
        }
    }

    /// 2段階CDFサンプリングでピクセル座標を取得
    /// u: 行選択、v: 列選択
    fn sample_pixel_2d(&self, u: f32, v: f32) -> (usize, usize) {
        let y = Self::sample_from_cdf(&self.marginal_cdf, u);
        let x = Self::sample_from_cdf(&self.conditional_cdf[y], v);

        (x, y)
    }

    fn calculate_direction_pdf(&self, direction: math::Vector3<Render>) -> f32 {
        if self.total_weight <= 0.0 {
            return 0.0;
        }

        let dir_local = &self.local_to_render.inverse() * &direction;
        let (theta, phi) = Self::direction_to_spherical(dir_local);
        let (u, v) = Self::spherical_to_uv(theta, phi);

        let x =
            ((u * self.texture_width as f32).floor() as usize).min(self.texture_width as usize - 1);
        let y = ((v * self.texture_height as f32).floor() as usize)
            .min(self.texture_height as usize - 1);

        let pixel_rgb = self.texture_data[y][x];
        let luminance = 0.299 * pixel_rgb[0] + 0.587 * pixel_rgb[1] + 0.114 * pixel_rgb[2];
        let sin_theta = theta.sin().max(1e-8);
        let pixel_weight = luminance * sin_theta;

        let pdf_texture = pixel_weight / self.total_weight;
        let texture_to_solid_angle_jacobian = (self.texture_width as f32)
            * (self.texture_height as f32)
            / (2.0 * std::f32::consts::PI * std::f32::consts::PI * sin_theta);

        pdf_texture * texture_to_solid_angle_jacobian
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
impl<Id: SceneId> PrimitiveInfiniteLight<Id> for EnvironmentLight {
    fn direction_radiance(
        &self,
        ray: &Ray<Render>,
        lambda: &SampledWavelengths,
    ) -> SampledSpectrum {
        let dir_local = &self.local_to_render.inverse() * &ray.dir;
        let (theta, phi) = Self::direction_to_spherical(dir_local);
        let (u, v) = Self::spherical_to_uv(theta, phi);
        let rgb = self.sample_texture(u, v);
        let color = ColorSrgb::<NoneToneMap>::new(rgb[0], rgb[1], rgb[2]);
        let spectrum = RgbIlluminantSpectrum::<ColorSrgb<NoneToneMap>>::new(color);
        spectrum.sample(lambda) * self.intensity
    }

    fn pdf_direction_sample(
        &self,
        _shading_point: &SurfaceInteraction<Render>,
        wi: math::Vector3<Render>,
    ) -> f32 {
        self.calculate_direction_pdf(wi)
    }

    fn sample_infinite_light(
        &self,
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
        uv: glam::Vec2,
    ) -> InfiniteLightSampleRadiance<Render> {
        let (x, y) = self.sample_pixel_2d(uv.x, uv.y);

        let u = (x as f32 + 0.5) / self.texture_width as f32;
        let v = (y as f32 + 0.5) / self.texture_height as f32;

        let (theta, phi) = Self::uv_to_spherical(u, v);
        let wi_local = Self::spherical_to_direction(theta, phi);
        let wi = &self.local_to_render * &wi_local;

        let pdf_dir = self.calculate_direction_pdf(wi);
        let ray = math::Ray::new(shading_point.position, wi);
        let radiance = <Self as PrimitiveInfiniteLight<Id>>::direction_radiance(self, &ray, lambda);

        InfiniteLightSampleRadiance {
            radiance,
            pdf_dir,
            wi,
        }
    }
}
