//! 環境ライトのプリミティブの実装のモジュール。

use std::path::Path;

use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Local, Ray, Render, Transform, World};
use spectrum::{RgbIlluminantSpectrum, SampledSpectrum, SampledWavelengths, Spectrum};

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

    /// ヤコビアン重み（正距円筒図法の歪み補正）を計算
    fn jacobian_weight(theta: f32) -> f32 {
        1.0 / theta.sin().max(1e-8)
    }

    /// 各ピクセルの重みを計算してテーブルを構築
    fn build_weight_table(
        &self,
        shading_normal: math::Normal<Render>,
        lambda: &SampledWavelengths,
    ) -> (Vec<Vec<f32>>, f32) {
        let mut weight_table =
            vec![vec![0.0f32; self.texture_width as usize]; self.texture_height as usize];
        let mut total_weight = 0.0f32;

        for y in 0..self.texture_height {
            for x in 0..self.texture_width {
                let u = (x as f32 + 0.5) / self.texture_width as f32;
                let v = (y as f32 + 0.5) / self.texture_height as f32;

                // テクスチャ座標から方向を計算
                let (theta, phi) = Self::uv_to_spherical(u, v);
                let wi = Self::spherical_to_direction(theta, phi);

                // 3つの重みを計算
                let jacobian_weight = Self::jacobian_weight(theta);
                let cosine_weight = shading_normal.dot(wi).max(0.0);

                let pixel_rgb = self.texture_data[y as usize][x as usize];
                let color = ColorSrgb::<NoneToneMap>::new(pixel_rgb[0], pixel_rgb[1], pixel_rgb[2]);
                let pixel_spectrum = RgbIlluminantSpectrum::<ColorSrgb<NoneToneMap>>::new(color);
                let luminance_weight = pixel_spectrum.sample(lambda).average();

                // 全ての重みを掛け合わせる
                let weight = jacobian_weight * cosine_weight * luminance_weight;
                weight_table[y as usize][x as usize] = weight;
                total_weight += weight;
            }
        }

        (weight_table, total_weight)
    }

    /// 累積分布関数テーブルを構築
    fn build_cdf_table(weight_table: &[Vec<f32>], total_weight: f32) -> Vec<Vec<f32>> {
        let height = weight_table.len();
        let width = weight_table[0].len();
        let mut cdf_table = vec![vec![0.0f32; width]; height];

        let mut cumulative = 0.0f32;
        for y in 0..height {
            for x in 0..width {
                cumulative += weight_table[y][x];
                cdf_table[y][x] = cumulative / total_weight;
            }
        }

        cdf_table
    }

    /// 2次元乱数でCDFテーブルからピクセルをサンプリング
    fn sample_pixel_from_cdf(cdf_table: &[Vec<f32>], u: f32, _v: f32) -> (usize, usize) {
        let height = cdf_table.len();
        let width = cdf_table[0].len();

        // 線形検索でピクセルを見つける
        for y in 0..height {
            for x in 0..width {
                if u <= cdf_table[y][x] {
                    return (x, y);
                }
            }
        }

        // フォールバック（数値誤差対策）
        (width - 1, height - 1)
    }

    /// 特定の方向に対するPDFを計算
    fn calculate_direction_pdf(
        &self,
        weight_table: &[Vec<f32>],
        total_weight: f32,
        direction: math::Vector3<Render>,
    ) -> f32 {
        // 方向からテクスチャ座標を計算
        let (theta, phi) = Self::direction_to_spherical(direction);
        let (u, v) = Self::spherical_to_uv(theta, phi);

        // 最も近いピクセルの重みを取得
        let x =
            ((u * self.texture_width as f32).floor() as usize).min(self.texture_width as usize - 1);
        let y = ((v * self.texture_height as f32).floor() as usize)
            .min(self.texture_height as usize - 1);

        if total_weight > 0.0 {
            weight_table[y][x] / total_weight
        } else {
            0.0
        }
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
        shading_point: &SurfaceInteraction<Render>,
        lambda: &SampledWavelengths,
        _s: f32,
        uv: glam::Vec2,
    ) -> AreaLightSampleRadiance<Render> {
        // 重みテーブルを構築
        let (weight_table, total_weight) =
            self.build_weight_table(shading_point.shading_normal, lambda);

        if total_weight <= 0.0 {
            // 重みが0の場合は失敗
            return AreaLightSampleRadiance {
                radiance: SampledSpectrum::zero(),
                pdf: 0.0,
                light_normal: shading_point.normal, // ダミー値
                pdf_dir: 0.0,
                interaction: SurfaceInteraction {
                    position: shading_point.position,
                    normal: shading_point.normal,
                    shading_normal: shading_point.shading_normal,
                    tangent: shading_point.tangent,
                    uv: shading_point.uv,
                    material: shading_point.material.clone(),
                },
            };
        }

        // 累積分布関数テーブルを構築
        let cdf_table = Self::build_cdf_table(&weight_table, total_weight);

        // 2次元乱数でピクセルをサンプリング
        let (pixel_x, pixel_y) = Self::sample_pixel_from_cdf(&cdf_table, uv.x, uv.y);

        // ピクセル座標からテクスチャ座標を計算
        let u = (pixel_x as f32 + 0.5) / self.texture_width as f32;
        let v = (pixel_y as f32 + 0.5) / self.texture_height as f32;

        // テクスチャ座標から方向を計算
        let (theta, phi) = Self::uv_to_spherical(u, v);
        let wi = Self::spherical_to_direction(theta, phi);

        // テクスチャから放射輝度を取得
        let pixel_rgb = self.texture_data[pixel_y][pixel_x];
        let color = ColorSrgb::<NoneToneMap>::new(pixel_rgb[0], pixel_rgb[1], pixel_rgb[2]);
        let spectrum = RgbIlluminantSpectrum::<ColorSrgb<NoneToneMap>>::new(color);
        let radiance = spectrum.sample(lambda) * self.intensity;

        // PDFを計算
        let pdf_dir = self.calculate_direction_pdf(&weight_table, total_weight, wi);

        // ダミーのSurfaceInteractionを作成（Environment Lightは表面を持たない）
        let dummy_interaction = SurfaceInteraction {
            position: shading_point.position + wi * 1000.0, // 遠方の点
            normal: (-wi).into(),
            shading_normal: (-wi).into(),
            tangent: math::Vector3::new(1.0, 0.0, 0.0), // ダミー
            uv: glam::Vec2::new(u, v),
            material: shading_point.material.clone(), // ダミー
        };

        AreaLightSampleRadiance {
            radiance,
            pdf: 1.0, // 面積PDFはダミー
            light_normal: (-wi).into(),
            pdf_dir,
            interaction: dummy_interaction,
        }
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
        shading_point: &SurfaceInteraction<Render>,
        wi: math::Vector3<Render>,
    ) -> f32 {
        // 重みテーブルを構築（サンプリング時と同じ条件で）
        let lambda = &SampledWavelengths::new_uniform(0.5); // RGB用の固定波長
        let (weight_table, total_weight) =
            self.build_weight_table(shading_point.shading_normal, lambda);

        self.calculate_direction_pdf(&weight_table, total_weight, wi)
    }
}
