use std::{path::Path, process::Command};

use image::{ImageBuffer, Rgb};

/// sRGB逆ガンマ補正を適用して線形値に変換
fn srgb_to_linear(srgb: f64) -> f64 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// RMSE (Root Mean Square Error) を線形色空間で計算する
fn calculate_rmse(
    img1: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    img2: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> f64 {
    assert_eq!(img1.dimensions(), img2.dimensions());

    let mut sum_squared_diff = 0.0;
    let pixel_count = (img1.width() * img1.height() * 3) as f64; // RGB 3チャンネル

    for (p1, p2) in img1.pixels().zip(img2.pixels()) {
        for (c1, c2) in p1.0.iter().zip(p2.0.iter()) {
            // u8値を0-1の範囲に正規化
            let srgb_c1 = (*c1 as f64) / 255.0;
            let srgb_c2 = (*c2 as f64) / 255.0;

            // sRGB逆ガンマ補正を適用して線形値に変換
            let linear_c1 = srgb_to_linear(srgb_c1);
            let linear_c2 = srgb_to_linear(srgb_c2);

            let diff = linear_c1 - linear_c2;
            sum_squared_diff += diff * diff;
        }
    }

    (sum_squared_diff / pixel_count).sqrt()
}

/// レンダリングを実行してRMSE比較を行う
fn run_render_and_compare(
    scene: u32,
    renderer: &str,
    sampler: &str,
    spp: u32,
    output_file: &str,
    reference_file: &str,
    max_rmse: f64,
) {
    // レンダリング実行（プロジェクトルートディレクトリから実行）
    let output = Command::new("cargo.exe")
        .current_dir("..") // プロジェクトルートに移動
        .args(["run", "--release", "--bin", "renderer", "--"])
        .args(["--scene", &scene.to_string()])
        .args(["--renderer", renderer])
        .args(["--sampler", sampler])
        .args(["--spp", &spp.to_string()])
        .args(["--width", "200"])
        .args(["--height", "150"])
        .args(["--output", output_file])
        .output()
        .expect("Failed to execute renderer");

    if !output.status.success() {
        panic!(
            "Renderer failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // 画像を読み込み（プロジェクトルートから相対パス）
    let rendered_path = format!("../{}", output_file);
    let reference_path = format!("../{}", reference_file);

    let rendered_img = image::open(&rendered_path)
        .expect("Failed to open rendered image")
        .to_rgb8();

    let reference_img = image::open(&reference_path)
        .expect("Failed to open reference image")
        .to_rgb8();

    // RMSE計算
    let rmse = calculate_rmse(&rendered_img, &reference_img);

    println!(
        "RMSE for {}: {:.6} (max: {:.6})",
        output_file, rmse, max_rmse
    );

    // しきい値チェック
    assert!(
        rmse <= max_rmse,
        "RMSE {:.6} exceeds threshold {:.6} for {} vs {}",
        rmse,
        max_rmse,
        output_file,
        reference_file
    );

    // テスト用出力ファイルを削除
    if Path::new(&rendered_path).exists() {
        std::fs::remove_file(&rendered_path).ok();
    }
}

#[test]
fn regression_test_pt_random() {
    run_render_and_compare(
        0,
        "pt",
        "random",
        512,
        "output.test_pt_random.png",
        "test_references/reference_pt_random.png",
        0.05,
    );
}

#[test]
fn regression_test_pt_sobol() {
    run_render_and_compare(
        0,
        "pt",
        "sobol",
        512,
        "output.test_pt_sobol.png",
        "test_references/reference_pt_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_nee_random() {
    run_render_and_compare(
        0,
        "nee",
        "random",
        512,
        "output.test_nee_random.png",
        "test_references/reference_nee_random.png",
        0.05,
    );
}

#[test]
fn regression_test_nee_sobol() {
    run_render_and_compare(
        0,
        "nee",
        "sobol",
        512,
        "output.test_nee_sobol.png",
        "test_references/reference_nee_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_mis_random() {
    run_render_and_compare(
        0,
        "mis",
        "random",
        512,
        "output.test_mis_random.png",
        "test_references/reference_mis_random.png",
        0.05,
    );
}

#[test]
fn regression_test_mis_sobol() {
    run_render_and_compare(
        0,
        "mis",
        "sobol",
        512,
        "output.test_mis_sobol.png",
        "test_references/reference_mis_sobol.png",
        0.05,
    );
}

// Scene 3 (Normal mapped textured bunny) regression tests
#[test]
fn regression_test_scene3_pt_random() {
    run_render_and_compare(
        3,
        "pt",
        "random",
        512,
        "output.test_scene3_pt_random.png",
        "test_references/reference_scene3_pt_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene3_pt_sobol() {
    run_render_and_compare(
        3,
        "pt",
        "sobol",
        512,
        "output.test_scene3_pt_sobol.png",
        "test_references/reference_scene3_pt_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_scene3_nee_random() {
    run_render_and_compare(
        3,
        "nee",
        "random",
        512,
        "output.test_scene3_nee_random.png",
        "test_references/reference_scene3_nee_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene3_nee_sobol() {
    run_render_and_compare(
        3,
        "nee",
        "sobol",
        512,
        "output.test_scene3_nee_sobol.png",
        "test_references/reference_scene3_nee_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_scene3_mis_random() {
    run_render_and_compare(
        3,
        "mis",
        "random",
        512,
        "output.test_scene3_mis_random.png",
        "test_references/reference_scene3_mis_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene3_mis_sobol() {
    run_render_and_compare(
        3,
        "mis",
        "sobol",
        512,
        "output.test_scene3_mis_sobol.png",
        "test_references/reference_scene3_mis_sobol.png",
        0.05,
    );
}

// Scene 8 (SF11 glass bunny) regression tests
#[test]
fn regression_test_scene8_pt_random() {
    run_render_and_compare(
        8,
        "pt",
        "random",
        512,
        "output.test_scene8_pt_random.png",
        "test_references/reference_scene8_pt_random.png",
        0.07,
    );
}

#[test]
fn regression_test_scene8_pt_sobol() {
    run_render_and_compare(
        8,
        "pt",
        "sobol",
        512,
        "output.test_scene8_pt_sobol.png",
        "test_references/reference_scene8_pt_sobol.png",
        0.08,
    );
}

#[test]
fn regression_test_scene8_nee_random() {
    run_render_and_compare(
        8,
        "nee",
        "random",
        512,
        "output.test_scene8_nee_random.png",
        "test_references/reference_scene8_nee_random.png",
        0.07,
    );
}

#[test]
fn regression_test_scene8_nee_sobol() {
    run_render_and_compare(
        8,
        "nee",
        "sobol",
        512,
        "output.test_scene8_nee_sobol.png",
        "test_references/reference_scene8_nee_sobol.png",
        0.07,
    );
}

#[test]
fn regression_test_scene8_mis_random() {
    run_render_and_compare(
        8,
        "mis",
        "random",
        512,
        "output.test_scene8_mis_random.png",
        "test_references/reference_scene8_mis_random.png",
        0.07,
    );
}

#[test]
fn regression_test_scene8_mis_sobol() {
    run_render_and_compare(
        8,
        "mis",
        "sobol",
        512,
        "output.test_scene8_mis_sobol.png",
        "test_references/reference_scene8_mis_sobol.png",
        0.07,
    );
}

// Scene 9 (thin film plastic bunny) regression tests
#[test]
fn regression_test_scene9_pt_random() {
    run_render_and_compare(
        9,
        "pt",
        "random",
        512,
        "output.test_scene9_pt_random.png",
        "test_references/reference_scene9_pt_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene9_pt_sobol() {
    run_render_and_compare(
        9,
        "pt",
        "sobol",
        512,
        "output.test_scene9_pt_sobol.png",
        "test_references/reference_scene9_pt_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_scene9_nee_random() {
    run_render_and_compare(
        9,
        "nee",
        "random",
        512,
        "output.test_scene9_nee_random.png",
        "test_references/reference_scene9_nee_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene9_nee_sobol() {
    run_render_and_compare(
        9,
        "nee",
        "sobol",
        512,
        "output.test_scene9_nee_sobol.png",
        "test_references/reference_scene9_nee_sobol.png",
        0.05,
    );
}

#[test]
fn regression_test_scene9_mis_random() {
    run_render_and_compare(
        9,
        "mis",
        "random",
        512,
        "output.test_scene9_mis_random.png",
        "test_references/reference_scene9_mis_random.png",
        0.05,
    );
}

#[test]
fn regression_test_scene9_mis_sobol() {
    run_render_and_compare(
        9,
        "mis",
        "sobol",
        512,
        "output.test_scene9_mis_sobol.png",
        "test_references/reference_scene9_mis_sobol.png",
        0.05,
    );
}
