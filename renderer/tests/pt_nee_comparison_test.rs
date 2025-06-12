//! PTとNEEレンダラーの結果比較テスト

use image::ImageReader;
use std::process::Command;

const RMSE_TOLERANCE: f64 = 0.0118; // 1.18% RMSE許容誤差（normal map無しでの基準値）
const SPP: &str = "2048";
const WIDTH: &str = "200";
const HEIGHT: &str = "150";

/// 2つの画像間のRMSEを計算する（リニア色空間で）
fn calculate_rmse(pt_image_path: &str, nee_image_path: &str) -> Option<f64> {
    let pt_img = ImageReader::open(pt_image_path)
        .ok()?
        .decode()
        .ok()?
        .to_rgb8();
    let nee_img = ImageReader::open(nee_image_path)
        .ok()?
        .decode()
        .ok()?
        .to_rgb8();

    let (pt_width, pt_height) = pt_img.dimensions();
    let (nee_width, nee_height) = nee_img.dimensions();

    if pt_width != nee_width || pt_height != nee_height {
        return None; // 画像サイズが異なる場合
    }

    let mut sum_squared_diff = 0.0;

    let total_pixels = pt_width * pt_height;

    for y in 0..pt_height {
        for x in 0..pt_width {
            let pt_pixel = pt_img.get_pixel(x, y);
            let nee_pixel = nee_img.get_pixel(x, y);

            // sRGBからリニア色空間に変換（簡易近似: gamma 2.2）
            let pt_r = (pt_pixel[0] as f64 / 255.0).powf(2.2);
            let pt_g = (pt_pixel[1] as f64 / 255.0).powf(2.2);
            let pt_b = (pt_pixel[2] as f64 / 255.0).powf(2.2);

            let nee_r = (nee_pixel[0] as f64 / 255.0).powf(2.2);
            let nee_g = (nee_pixel[1] as f64 / 255.0).powf(2.2);
            let nee_b = (nee_pixel[2] as f64 / 255.0).powf(2.2);

            // 各チャンネルの二乗誤差
            let diff_r = pt_r - nee_r;
            let diff_g = pt_g - nee_g;
            let diff_b = pt_b - nee_b;

            sum_squared_diff += diff_r * diff_r + diff_g * diff_g + diff_b * diff_b;
        }
    }

    // RMSE計算（3チャンネル分で正規化）
    let mse = sum_squared_diff / (total_pixels as f64 * 3.0);
    Some(mse.sqrt())
}

/// 画像全体のリニア色空間での平均値を計算する
fn calculate_linear_average(image_path: &str) -> Option<(f64, f64, f64)> {
    let img = ImageReader::open(image_path).ok()?.decode().ok()?.to_rgb8();
    let (width, height) = img.dimensions();

    let mut sum_r = 0.0;
    let mut sum_g = 0.0;
    let mut sum_b = 0.0;
    let total_pixels = (width * height) as f64;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // sRGBからリニア色空間に変換
            sum_r += (pixel[0] as f64 / 255.0).powf(2.2);
            sum_g += (pixel[1] as f64 / 255.0).powf(2.2);
            sum_b += (pixel[2] as f64 / 255.0).powf(2.2);
        }
    }

    Some((
        sum_r / total_pixels,
        sum_g / total_pixels,
        sum_b / total_pixels,
    ))
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
            "random",
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
            "random",
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

    // median filter適用
    println!("Applying median filter to PT output...");
    let pt_image = image::open("../output.test_pt_full.png")
        .expect("Failed to open PT output image")
        .to_rgb8();
    let pt_filtered = imageproc::filter::median_filter(&pt_image, 1, 1);
    pt_filtered
        .save("../output.test_pt_full_filtered.png")
        .expect("Failed to save filtered PT image");

    println!("Applying median filter to NEE output...");
    let nee_image = image::open("../output.test_nee_full.png")
        .expect("Failed to open NEE output image")
        .to_rgb8();
    let nee_filtered = imageproc::filter::median_filter(&nee_image, 1, 1);
    nee_filtered
        .save("../output.test_nee_full_filtered.png")
        .expect("Failed to save filtered NEE image");

    // RMSE計算
    let rmse = calculate_rmse(
        "../output.test_pt_full_filtered.png",
        "../output.test_nee_full_filtered.png",
    )
    .expect("Failed to calculate RMSE");

    println!(
        "RMSE (linear color space): {:.6} ({:.3}%)",
        rmse,
        rmse * 100.0
    );

    // 平均値計算（参考情報として）
    let pt_avg = calculate_linear_average("../output.test_pt_full.png")
        .expect("Failed to calculate PT average");

    let nee_avg = calculate_linear_average("../output.test_nee_full.png")
        .expect("Failed to calculate NEE average");

    println!(
        "PT linear average RGB: ({:.6}, {:.6}, {:.6})",
        pt_avg.0, pt_avg.1, pt_avg.2
    );
    println!(
        "NEE linear average RGB: ({:.6}, {:.6}, {:.6})",
        nee_avg.0, nee_avg.1, nee_avg.2
    );

    // RGB差分の詳細
    let rgb_diff = (
        (pt_avg.0 - nee_avg.0).abs(),
        (pt_avg.1 - nee_avg.1).abs(),
        (pt_avg.2 - nee_avg.2).abs(),
    );
    println!(
        "Linear RGB absolute differences: ({:.6}, {:.6}, {:.6})",
        rgb_diff.0, rgb_diff.1, rgb_diff.2
    );

    // テスト判定
    if rmse <= RMSE_TOLERANCE {
        println!(
            "✓ Test PASSED: RMSE {:.2}% is within tolerance {:.3}%",
            rmse * 100.0,
            RMSE_TOLERANCE * 100.0
        );
    } else {
        panic!(
            "✗ Test FAILED: RMSE {:.2}% exceeds tolerance {:.3}%\n\
                PT and NEE should produce theoretically equivalent results.\n\
                This indicates a bug in normal mapping consistency between PT and NEE.",
            rmse * 100.0,
            RMSE_TOLERANCE * 100.0
        );
    }
}
