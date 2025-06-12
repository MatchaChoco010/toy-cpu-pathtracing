//! Normal マップテクスチャ実装。

use super::{
    config::TextureConfig,
    loader::{ImageData, load_rgb_image},
    sampler::{TextureSample, bilinear_sample_rgb, bilinear_sample_rgb_f32},
};
use glam::Vec2;
use math::{Normal, Tangent};
use std::sync::Arc;

/// Normal マップテクスチャ。
#[derive(Clone)]
pub struct NormalTexture {
    data: ImageData,
    flip_y: bool,
}

impl NormalTexture {
    /// テクスチャ設定から Normal テクスチャを読み込む。
    pub fn load(config: TextureConfig, flip_y: bool) -> Result<Arc<Self>, image::ImageError> {
        let data = load_rgb_image(&config.path)?;
        Ok(Arc::new(Self { data, flip_y }))
    }

    /// UV座標でノーマルマップをサンプリングし、接空間ノーマルを取得する。
    pub fn sample_normal(&self, uv: Vec2) -> Normal<Tangent> {
        let rgb = self.sample(uv);

        // RGB [0,1] を [-1,1] の範囲に変換
        let mut x = rgb[0] * 2.0 - 1.0;
        let mut y = rgb[1] * 2.0 - 1.0;
        let mut z = rgb[2] * 2.0 - 1.0;

        // Y軸反転（DirectX vs OpenGL）
        if self.flip_y {
            y = -y;
        }

        // ノーマライズ
        let length = (x * x + y * y + z * z).sqrt();
        if length > 0.0 {
            x /= length;
            y /= length;
            z /= length;

            // 接空間ノーマルとして返す
            Normal::new(x, y, z)
        } else {
            // 無効な場合はZ軸方向（デフォルトノーマル）
            Normal::new(0.0, 0.0, 1.0)
        }
    }
}

impl TextureSample<[f32; 3]> for NormalTexture {
    fn sample(&self, uv: Vec2) -> [f32; 3] {
        match &self.data {
            ImageData::Rgb8(data, width, height) => bilinear_sample_rgb(data, *width, *height, uv),
            ImageData::RgbF32(data, width, height) => {
                bilinear_sample_rgb_f32(data, *width, *height, uv)
            }
            _ => [0.5, 0.5, 1.0], // 不正なデータタイプの場合はデフォルトノーマル
        }
    }
}
