//! テクスチャサンプリング機能。

use glam::Vec2;

/// バイリニア補間によるテクスチャサンプリング。
pub trait TextureSample<T> {
    /// UV座標（0.0-1.0）でテクスチャをサンプリングする。
    fn sample(&self, uv: Vec2) -> T;
}

/// UV座標を画像座標に変換し、バイリニア補間を行う。
pub fn bilinear_sample_rgb(data: &[u8], width: u32, height: u32, uv: Vec2) -> [f32; 3] {
    // UV座標をラップ
    let u = uv.x.fract().abs();
    let v = 1.0 - uv.y.fract().abs(); // Y軸反転を削除してテスト

    // 連続座標を計算
    let x = u * (width as f32 - 1.0);
    let y = v * (height as f32 - 1.0);

    // 整数部分と小数部分を取得
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    // 4つの隣接ピクセルを取得（8bit値をそのまま0.0-1.0にマッピング、ガンマ補正は後で処理）
    let get_pixel = |x: u32, y: u32| -> [f32; 3] {
        let idx = ((y * width + x) * 3) as usize;
        [
            data[idx] as f32 / 255.0,
            data[idx + 1] as f32 / 255.0,
            data[idx + 2] as f32 / 255.0,
        ]
    };

    let p00 = get_pixel(x0, y0);
    let p10 = get_pixel(x1, y0);
    let p01 = get_pixel(x0, y1);
    let p11 = get_pixel(x1, y1);

    // バイリニア補間
    [
        lerp2d(p00[0], p10[0], p01[0], p11[0], fx, fy),
        lerp2d(p00[1], p10[1], p01[1], p11[1], fx, fy),
        lerp2d(p00[2], p10[2], p01[2], p11[2], fx, fy),
    ]
}

/// UV座標を画像座標に変換し、バイリニア補間を行う（Float版）。
pub fn bilinear_sample_rgb_f32(data: &[f32], width: u32, height: u32, uv: Vec2) -> [f32; 3] {
    let u = uv.x.fract().abs();
    let v = 1.0 - uv.y.fract().abs();

    let x = u * (width as f32 - 1.0);
    let y = v * (height as f32 - 1.0);

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let get_pixel = |x: u32, y: u32| -> [f32; 3] {
        let idx = ((y * width + x) * 3) as usize;
        [data[idx], data[idx + 1], data[idx + 2]]
    };

    let p00 = get_pixel(x0, y0);
    let p10 = get_pixel(x1, y0);
    let p01 = get_pixel(x0, y1);
    let p11 = get_pixel(x1, y1);

    [
        lerp2d(p00[0], p10[0], p01[0], p11[0], fx, fy),
        lerp2d(p00[1], p10[1], p01[1], p11[1], fx, fy),
        lerp2d(p00[2], p10[2], p01[2], p11[2], fx, fy),
    ]
}

/// グレースケール画像のバイリニア補間。
pub fn bilinear_sample_gray(data: &[u8], width: u32, height: u32, uv: Vec2) -> f32 {
    let u = uv.x.fract().abs();
    let v = 1.0 - uv.y.fract().abs();

    let x = u * (width as f32 - 1.0);
    let y = v * (height as f32 - 1.0);

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let get_pixel = |x: u32, y: u32| -> f32 {
        let idx = (y * width + x) as usize;
        data[idx] as f32 / 255.0
    };

    let p00 = get_pixel(x0, y0);
    let p10 = get_pixel(x1, y0);
    let p01 = get_pixel(x0, y1);
    let p11 = get_pixel(x1, y1);

    lerp2d(p00, p10, p01, p11, fx, fy)
}

/// グレースケール画像のバイリニア補間（Float版）。
pub fn bilinear_sample_gray_f32(data: &[f32], width: u32, height: u32, uv: Vec2) -> f32 {
    let u = uv.x.fract().abs();
    let v = 1.0 - uv.y.fract().abs();

    let x = u * (width as f32 - 1.0);
    let y = v * (height as f32 - 1.0);

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let get_pixel = |x: u32, y: u32| -> f32 {
        let idx = (y * width + x) as usize;
        data[idx]
    };

    let p00 = get_pixel(x0, y0);
    let p10 = get_pixel(x1, y0);
    let p01 = get_pixel(x0, y1);
    let p11 = get_pixel(x1, y1);

    lerp2d(p00, p10, p01, p11, fx, fy)
}

/// 2D線形補間。
fn lerp2d(p00: f32, p10: f32, p01: f32, p11: f32, fx: f32, fy: f32) -> f32 {
    let top = p00 * (1.0 - fx) + p10 * fx;
    let bottom = p01 * (1.0 - fx) + p11 * fx;
    top * (1.0 - fy) + bottom * fy
}
