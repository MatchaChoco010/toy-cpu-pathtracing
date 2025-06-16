//! Float テクスチャ実装。

use super::{
    config::TextureConfig,
    loader::{ImageData, load_grayscale_image},
    sampler::{bilinear_sample_gray, bilinear_sample_gray_f32},
};
use color::eotf::{Eotf, GammaSrgb};
use glam::Vec2;
use std::sync::Arc;

/// Float テクスチャ。
#[derive(Clone)]
pub struct FloatTexture {
    data: ImageData,
    gamma_corrected: bool,
}

impl FloatTexture {
    /// テクスチャ設定から Float テクスチャを読み込む。
    pub fn load(config: TextureConfig) -> Result<Arc<Self>, image::ImageError> {
        let data = load_grayscale_image(&config.path)?;
        Ok(Arc::new(Self {
            data,
            gamma_corrected: config.gamma_corrected,
        }))
    }

    /// UV座標でテクスチャをサンプリングする。
    pub fn sample(&self, uv: Vec2) -> f32 {
        let value = match &self.data {
            ImageData::Gray8(data, width, height) => {
                bilinear_sample_gray(data, *width, *height, uv)
            }
            ImageData::GrayF32(data, width, height) => {
                bilinear_sample_gray_f32(data, *width, *height, uv)
            }
            _ => 0.0, // 不正なデータタイプの場合は0を返す
        };

        // ガンマ補正を除去
        if self.gamma_corrected {
            let vec = glam::Vec3::new(value, value, value);
            let linear_vec = GammaSrgb::inverse_transform(vec);
            linear_vec.x
        } else {
            value
        }
    }
}
