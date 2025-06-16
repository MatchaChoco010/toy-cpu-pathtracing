//! PTレンダラーとNEE/MISレンダラーの一貫性テスト

use image::ImageReader;
use std::fs;
use std::process::Command;

const RMSE_TOLERANCE: f64 = 0.013; // 1.3% RMSE許容誤差（normal mapありでの基準値）
const SPP: &str = "2048";
const WIDTH: &str = "200";
const HEIGHT: &str = "150";

/// テスト用の一意なファイル名を生成
fn generate_unique_filename(base: &str, scene: u32, renderer1: &str, renderer2: &str, suffix: &str) -> String {
    let test_id = std::thread::current().id();
    format!("output.test_{}_{}_vs_{}_scene{}_{:?}{}", base, renderer1, renderer2, scene, test_id, suffix)
}

/// 2つの画像間のRMSEを計算する（リニア色空間で）
fn calculate_rmse(image1_path: &str, image2_path: &str) -> Option<f64> {
    let img1 = ImageReader::open(image1_path)
        .ok()?
        .decode()
        .ok()?
        .to_rgb8();
    let img2 = ImageReader::open(image2_path)
        .ok()?
        .decode()
        .ok()?
        .to_rgb8();

    let (width1, height1) = img1.dimensions();
    let (width2, height2) = img2.dimensions();

    if width1 != width2 || height1 != height2 {
        return None; // 画像サイズが異なる場合
    }

    let mut sum_squared_diff = 0.0;

    let total_pixels = width1 * height1;

    for y in 0..height1 {
        for x in 0..width1 {
            let pixel1 = img1.get_pixel(x, y);
            let pixel2 = img2.get_pixel(x, y);

            // sRGBからリニア色空間に変換（簡易近似: gamma 2.2）
            let r1 = (pixel1[0] as f64 / 255.0).powf(2.2);
            let g1 = (pixel1[1] as f64 / 255.0).powf(2.2);
            let b1 = (pixel1[2] as f64 / 255.0).powf(2.2);

            let r2 = (pixel2[0] as f64 / 255.0).powf(2.2);
            let g2 = (pixel2[1] as f64 / 255.0).powf(2.2);
            let b2 = (pixel2[2] as f64 / 255.0).powf(2.2);

            // 各チャンネルの二乗誤差
            let diff_r = r1 - r2;
            let diff_g = g1 - g2;
            let diff_b = b1 - b2;

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

/// 指定されたレンダラーでレンダリングを実行する
fn render_scene(scene: u32, renderer: &str, output_filename: &str) -> Result<(), String> {
    println!(
        "Running {} renderer for scene {} with {} spp, {}x{} resolution...",
        renderer.to_uppercase(),
        scene,
        SPP,
        WIDTH,
        HEIGHT
    );

    let output = Command::new("cargo.exe")
        .args([
            "run",
            "--release",
            "--bin",
            "renderer",
            "--",
            "--scene",
            &scene.to_string(),
            "--renderer",
            renderer,
            "--spp",
            SPP,
            "--sampler",
            "random",
            "--width",
            WIDTH,
            "--height",
            HEIGHT,
            "--output",
            output_filename,
        ])
        .current_dir("..") // プロジェクトルートに移動
        .output()
        .map_err(|e| format!("Failed to run {} renderer: {}", renderer, e))?;

    if !output.status.success() {
        return Err(format!(
            "{} renderer failed: {}",
            renderer.to_uppercase(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// 画像にmedian filterを適用する
fn apply_median_filter(input_path: &str, output_path: &str) -> Result<(), String> {
    println!("Applying median filter to {}...", input_path);
    let image = image::open(input_path)
        .map_err(|e| format!("Failed to open {}: {}", input_path, e))?
        .to_rgb8();
    let filtered = imageproc::filter::median_filter(&image, 1, 1);
    filtered
        .save(output_path)
        .map_err(|e| format!("Failed to save filtered image to {}: {}", output_path, e))?;
    Ok(())
}

/// テスト用ファイルをクリーンアップする
fn cleanup_test_files(files: &[String]) {
    for file in files {
        if let Err(e) = fs::remove_file(file) {
            // ファイルが存在しない場合はエラーを無視
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Warning: Failed to cleanup file {}: {}", file, e);
            }
        }
    }
}

/// レンダラー間の一貫性をテストする共通関数
fn test_renderer_consistency(
    scene: u32,
    renderer1: &str,
    renderer2: &str,
) -> Result<(), String> {
    // 一意なファイル名を生成
    let output1 = generate_unique_filename("raw", scene, renderer1, renderer2, &format!("_{}.png", renderer1));
    let output2 = generate_unique_filename("raw", scene, renderer1, renderer2, &format!("_{}.png", renderer2));
    let filtered1 = generate_unique_filename("filtered", scene, renderer1, renderer2, &format!("_{}.png", renderer1));
    let filtered2 = generate_unique_filename("filtered", scene, renderer1, renderer2, &format!("_{}.png", renderer2));

    // クリーンアップ用ファイルリスト
    let cleanup_files = vec![
        format!("../{}", output1),
        format!("../{}", output2),
        format!("../{}", filtered1),
        format!("../{}", filtered2),
    ];

    // レンダリング実行
    let result = (|| -> Result<(), String> {
        render_scene(scene, renderer1, &output1)?;
        render_scene(scene, renderer2, &output2)?;

        // median filter適用
        apply_median_filter(&format!("../{}", output1), &format!("../{}", filtered1))?;
        apply_median_filter(&format!("../{}", output2), &format!("../{}", filtered2))?;

        // RMSE計算
        let rmse = calculate_rmse(&format!("../{}", filtered1), &format!("../{}", filtered2))
            .ok_or_else(|| format!("Failed to calculate RMSE between {} and {}", renderer1, renderer2))?;

        println!(
            "{} vs {} (Scene {}) RMSE (linear color space): {:.6} ({:.3}%)",
            renderer1.to_uppercase(),
            renderer2.to_uppercase(),
            scene,
            rmse,
            rmse * 100.0
        );

        // 平均値計算（参考情報として）
        let avg1 = calculate_linear_average(&format!("../{}", output1))
            .ok_or_else(|| format!("Failed to calculate {} average", renderer1))?;

        let avg2 = calculate_linear_average(&format!("../{}", output2))
            .ok_or_else(|| format!("Failed to calculate {} average", renderer2))?;

        println!(
            "{} (Scene {}) linear average RGB: ({:.6}, {:.6}, {:.6})",
            renderer1.to_uppercase(),
            scene,
            avg1.0,
            avg1.1,
            avg1.2
        );
        println!(
            "{} (Scene {}) linear average RGB: ({:.6}, {:.6}, {:.6})",
            renderer2.to_uppercase(),
            scene,
            avg2.0,
            avg2.1,
            avg2.2
        );

        // RGB差分の詳細
        let rgb_diff = (
            (avg1.0 - avg2.0).abs(),
            (avg1.1 - avg2.1).abs(),
            (avg1.2 - avg2.2).abs(),
        );
        println!(
            "{} vs {} (Scene {}) linear RGB absolute differences: ({:.6}, {:.6}, {:.6})",
            renderer1.to_uppercase(),
            renderer2.to_uppercase(),
            scene,
            rgb_diff.0,
            rgb_diff.1,
            rgb_diff.2
        );

        // テスト判定
        if rmse <= RMSE_TOLERANCE {
            println!(
                "✓ {} vs {} (Scene {}) Test PASSED: RMSE {:.2}% is within tolerance {:.3}%",
                renderer1.to_uppercase(),
                renderer2.to_uppercase(),
                scene,
                rmse * 100.0,
                RMSE_TOLERANCE * 100.0
            );
            Ok(())
        } else {
            Err(format!(
                "✗ {} vs {} (Scene {}) Test FAILED: RMSE {:.2}% exceeds tolerance {:.3}%\n\
                 {} and {} should produce theoretically equivalent results.\n\
                 This indicates a bug in normal mapping consistency between renderers.",
                renderer1.to_uppercase(),
                renderer2.to_uppercase(),
                scene,
                rmse * 100.0,
                RMSE_TOLERANCE * 100.0,
                renderer1.to_uppercase(),
                renderer2.to_uppercase()
            ))
        }
    })();

    // テスト結果に関係なく常にクリーンアップを実行
    cleanup_test_files(&cleanup_files);

    result
}

#[test]
fn test_pt_vs_nee_consistency_scene3() {
    println!("Testing PT vs NEE consistency for Scene 3 (normal mapped textured bunny)...");

    if let Err(e) = test_renderer_consistency(3, "pt", "nee") {
        panic!("{}", e);
    }
}

#[test]
fn test_pt_vs_mis_consistency_scene3() {
    println!("Testing PT vs MIS consistency for Scene 3 (normal mapped textured bunny)...");

    if let Err(e) = test_renderer_consistency(3, "pt", "mis") {
        panic!("{}", e);
    }
}

#[test]
fn test_pt_vs_nee_consistency_scene5() {
    println!("Testing PT vs NEE consistency for Scene 5 (current normal map settings)...");

    if let Err(e) = test_renderer_consistency(5, "pt", "nee") {
        panic!("{}", e);
    }
}

#[test]
fn test_pt_vs_mis_consistency_scene5() {
    println!("Testing PT vs MIS consistency for Scene 5 (current normal map settings)...");

    if let Err(e) = test_renderer_consistency(5, "pt", "mis") {
        panic!("{}", e);
    }
}