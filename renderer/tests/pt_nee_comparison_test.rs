//! PTとNEEレンダラーの結果比較テスト

use image::ImageReader;
use std::process::Command;

const TOLERANCE: f64 = 0.01; // 1.0%の許容誤差
const SPP: &str = "2048"; // さらに高いsppでモンテカルロ誤差を減らす
const WIDTH: &str = "200"; // 画像サイズを半分に
const HEIGHT: &str = "150";

/// 画像全体のRGB平均値を計算する
fn calculate_full_image_average(image_path: &str) -> Option<(f64, f64, f64)> {
    let img = ImageReader::open(image_path).ok()?.decode().ok()?.to_rgb8();
    let (width, height) = img.dimensions();

    let mut sum_r = 0u64;
    let mut sum_g = 0u64;
    let mut sum_b = 0u64;
    let total_pixels = (width * height) as u64;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            sum_r += pixel[0] as u64;
            sum_g += pixel[1] as u64;
            sum_b += pixel[2] as u64;
        }
    }

    Some((
        sum_r as f64 / total_pixels as f64 / 255.0,
        sum_g as f64 / total_pixels as f64 / 255.0,
        sum_b as f64 / total_pixels as f64 / 255.0,
    ))
}

/// 相対誤差を計算する
fn calculate_relative_error(pt_rgb: (f64, f64, f64), nee_rgb: (f64, f64, f64)) -> f64 {
    let pt_luminance = 0.299 * pt_rgb.0 + 0.587 * pt_rgb.1 + 0.114 * pt_rgb.2;
    let nee_luminance = 0.299 * nee_rgb.0 + 0.587 * nee_rgb.1 + 0.114 * nee_rgb.2;

    if pt_luminance == 0.0 && nee_luminance == 0.0 {
        0.0
    } else if pt_luminance == 0.0 || nee_luminance == 0.0 {
        1.0 // 一方がゼロの場合は100%の差とする
    } else {
        (pt_luminance - nee_luminance).abs() / pt_luminance.max(nee_luminance)
    }
}

#[test]
fn test_pt_nee_normal_map_consistency() {
    println!("Testing PT vs NEE consistency with current normal map settings...");

    // PTレンダリング実行
    println!(
        "Running PT renderer with {} spp, {}x{} resolution...",
        SPP, WIDTH, HEIGHT
    );
    let pt_output = Command::new("cargo.exe")
        .args(&[
            "run",
            "--release",
            "--bin",
            "renderer",
            "--",
            "--scene",
            "5",
            "--renderer",
            "pt",
            "--spp",
            SPP,
            "--sampler",
            "sobol",
            "--width",
            WIDTH,
            "--height",
            HEIGHT,
            "--output",
            "output.test_pt_full.png",
        ])
        .current_dir("..") // プロジェクトルートに移動
        .output()
        .expect("Failed to run PT renderer");

    assert!(
        pt_output.status.success(),
        "PT renderer failed: {}",
        String::from_utf8_lossy(&pt_output.stderr)
    );

    // NEEレンダリング実行
    println!(
        "Running NEE renderer with {} spp, {}x{} resolution...",
        SPP, WIDTH, HEIGHT
    );
    let nee_output = Command::new("cargo.exe")
        .args(&[
            "run",
            "--release",
            "--bin",
            "renderer",
            "--",
            "--scene",
            "5",
            "--renderer",
            "nee",
            "--spp",
            SPP,
            "--sampler",
            "sobol",
            "--width",
            WIDTH,
            "--height",
            HEIGHT,
            "--output",
            "output.test_nee_full.png",
        ])
        .current_dir("..") // プロジェクトルートに移動
        .output()
        .expect("Failed to run NEE renderer");

    assert!(
        nee_output.status.success(),
        "NEE renderer failed: {}",
        String::from_utf8_lossy(&nee_output.stderr)
    );

    // 画像全体から平均値を計算
    let pt_avg = calculate_full_image_average("../output.test_pt_full.png")
        .expect("Failed to calculate PT average");

    let nee_avg = calculate_full_image_average("../output.test_nee_full.png")
        .expect("Failed to calculate NEE average");

    println!(
        "PT full image average RGB: ({:.6}, {:.6}, {:.6})",
        pt_avg.0, pt_avg.1, pt_avg.2
    );
    println!(
        "NEE full image average RGB: ({:.6}, {:.6}, {:.6})",
        nee_avg.0, nee_avg.1, nee_avg.2
    );

    // 相対誤差を計算
    let relative_error = calculate_relative_error(pt_avg, nee_avg);
    println!(
        "Relative error: {:.4} ({:.2}%)",
        relative_error,
        relative_error * 100.0
    );

    // RGB差分の詳細
    let rgb_diff = (
        (pt_avg.0 - nee_avg.0).abs(),
        (pt_avg.1 - nee_avg.1).abs(),
        (pt_avg.2 - nee_avg.2).abs(),
    );
    println!(
        "RGB absolute differences: ({:.6}, {:.6}, {:.6})",
        rgb_diff.0, rgb_diff.1, rgb_diff.2
    );

    // テスト判定
    if relative_error <= TOLERANCE {
        println!(
            "✓ Test PASSED: Relative error {:.2}% is within tolerance {:.2}%",
            relative_error * 100.0,
            TOLERANCE * 100.0
        );
    } else {
        panic!(
            "✗ Test FAILED: Relative error {:.2}% exceeds tolerance {:.2}%\n\
                PT and NEE should produce theoretically equivalent results.",
            relative_error * 100.0,
            TOLERANCE * 100.0
        );
    }
}
