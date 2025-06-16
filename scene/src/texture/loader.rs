//! 画像ファイル読み込み機能。

use std::path::Path;

use image::{DynamicImage, ImageResult};

/// 画像データの種類。
#[derive(Clone)]
pub enum ImageData {
    /// 8bit RGB画像（0-255の値）。
    Rgb8(Vec<u8>, u32, u32),
    /// 32bit float RGB画像（0.0-1.0の値）。
    RgbF32(Vec<f32>, u32, u32),
    /// 8bit グレースケール画像（0-255の値）。
    Gray8(Vec<u8>, u32, u32),
    /// 32bit float グレースケール画像（0.0-1.0の値）。
    GrayF32(Vec<f32>, u32, u32),
}

impl ImageData {
    /// 画像の幅を取得する。
    pub fn width(&self) -> u32 {
        match self {
            ImageData::Rgb8(_, w, _) => *w,
            ImageData::RgbF32(_, w, _) => *w,
            ImageData::Gray8(_, w, _) => *w,
            ImageData::GrayF32(_, w, _) => *w,
        }
    }

    /// 画像の高さを取得する。
    pub fn height(&self) -> u32 {
        match self {
            ImageData::Rgb8(_, _, h) => *h,
            ImageData::RgbF32(_, _, h) => *h,
            ImageData::Gray8(_, _, h) => *h,
            ImageData::GrayF32(_, _, h) => *h,
        }
    }
}

/// 画像ファイルをRGB形式で読み込む。
pub fn load_rgb_image(path: impl AsRef<Path>) -> ImageResult<ImageData> {
    let img = image::open(path)?;

    match img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            Ok(ImageData::Rgb8(rgb_img.into_raw(), width, height))
        }
        DynamicImage::ImageRgb32F(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            Ok(ImageData::RgbF32(rgb_img.into_raw(), width, height))
        }
        _ => {
            // 他の形式はRGB8に変換
            let rgb_img = img.to_rgb8();
            let (width, height) = rgb_img.dimensions();
            Ok(ImageData::Rgb8(rgb_img.into_raw(), width, height))
        }
    }
}

/// 画像ファイルをグレースケール形式で読み込む。
pub fn load_grayscale_image(path: impl AsRef<Path>) -> ImageResult<ImageData> {
    let img = image::open(path)?;

    match img {
        DynamicImage::ImageLuma8(gray_img) => {
            let (width, height) = gray_img.dimensions();
            Ok(ImageData::Gray8(gray_img.into_raw(), width, height))
        }
        DynamicImage::ImageLumaA8(gray_img) => {
            // アルファチャンネルを無視してグレースケールのみ取得
            let (width, height) = gray_img.dimensions();
            let raw_data = gray_img.into_raw();
            let gray_only: Vec<u8> = raw_data.chunks(2).map(|chunk| chunk[0]).collect();
            Ok(ImageData::Gray8(gray_only, width, height))
        }
        _ => {
            // 他の形式はグレースケールに変換
            let gray_img = img.to_luma8();
            let (width, height) = gray_img.dimensions();
            Ok(ImageData::Gray8(gray_img.into_raw(), width, height))
        }
    }
}
