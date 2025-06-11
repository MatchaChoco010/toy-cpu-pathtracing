//! RgbSpectrumTableのテーブル配列を定義するためのスクリプト。

use rayon::prelude::*;

use color::gamut::ColorGamut;

use crate::data::{
    CIE_X, CIE_Y, CIE_Z, H, LAMBDA_MAX, LAMBDA_MIN, N_CIE_FINE_SAMPLES, N_CIE_SAMPLES,
};

/// 多項式のテーブルのサイズ。
pub const TABLE_SIZE: usize = 64;

/// CIEの波長列を線形保管してサンプルする関数。
fn cie_interpret(values: &[f32; N_CIE_SAMPLES], lambda: f32) -> f32 {
    let x = lambda - LAMBDA_MIN;
    let mut x = x * ((N_CIE_SAMPLES - 1) as f32 / (LAMBDA_MAX - LAMBDA_MIN));
    if x < 0.0 {
        x = 0.0;
    }
    let mut offset = x as usize;
    if offset > N_CIE_SAMPLES - 2 {
        offset = N_CIE_SAMPLES - 2;
    }
    let weight = x - offset as f32;
    (1.0 - weight) * values[offset] + weight * values[offset + 1]
}

/// シグモイド関数。
fn sigmoid(x: f32) -> f32 {
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }
    0.5 + x / (2.0 * (1.0 + x * x).sqrt())
}

/// 係数から二次式を計算する関数。
fn parabolic(t: f32, coefficients: &[f32]) -> f32 {
    // coefficients[0]だけxに平行移動してcoefficients[1]だけyに平行移動した、
    // 係数coefficients[2]に100を乗じた二次式。
    let x = t - coefficients[0];
    let xx = 100.0 * coefficients[2] * x * x;
    xx + coefficients[1]
}

fn xyy_to_xyz(xy: glam::Vec2, y: f32) -> glam::Vec3 {
    if xy.y == 0.0 {
        return glam::Vec3::ZERO;
    }
    let x = xy.x * y / xy.y;
    let z = (1.0 - xy.x - xy.y) * y / xy.y;
    glam::vec3(x, y, z)
}

fn xy_to_xyz(xy: glam::Vec2) -> glam::Vec3 {
    xyy_to_xyz(xy, 1.0)
}

/// RGBからCIE Labに変換する関数。
fn rgb_to_lab(rgb: glam::Vec3, rgb_to_xyz: glam::Mat3) -> [f32; 3] {
    fn f(t: f32) -> f32 {
        if t > 0.008856 {
            t.powf(1.0 / 3.0)
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }

    // RGBをXYZに変換する。
    let xyz = rgb_to_xyz * rgb;

    // D50白色点のXYZ値を取得する。
    let w = glam::vec2(0.34567, 0.35850);
    let w_xyz = xy_to_xyz(w);

    // Xr, Yr, Zrを計算する。
    let xr = xyz[0] / w_xyz.x;
    let yr = xyz[1] / w_xyz.y;
    let zr = xyz[2] / w_xyz.z;

    // Xr, Yr, Zrを使ってL*, a*, b*を計算する。
    let l = 116.0 * f(yr) - 16.0;
    let a = 500.0 * (f(xr) - f(yr));
    let b = 200.0 * (f(yr) - f(zr));

    [l, a, b]
}

/// 目標RGBとcoefficientsのスペクトラルから作られるRGBを
/// Lab空間で距離を計算してdelta Eを計算する関数。
fn eval_delta_e<G: ColorGamut>(
    coefficients: [f32; 3],
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f32; 3]; N_CIE_FINE_SAMPLES],
) -> f32 {
    let mut out_rgb = [0.0; 3];
    for i in 0..N_CIE_FINE_SAMPLES {
        // lambdaの値を0..1の範囲に正規化する。
        let lambda = (lambda_table[i] - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN);

        // 二次式の計算
        let value = parabolic(lambda, &coefficients);

        // シグモイドの計算。
        let sigmoid_value = sigmoid(value);

        // スペクトルをXYZに変換する。
        out_rgb[0] += rgb_table[i][0] * sigmoid_value;
        out_rgb[1] += rgb_table[i][1] * sigmoid_value;
        out_rgb[2] += rgb_table[i][2] * sigmoid_value;
    }

    // Labに変換する。
    let out_lab = rgb_to_lab(glam::Vec3::from(out_rgb), G::new().rgb_to_xyz());
    let rgb_lab = rgb_to_lab(rgb, G::new().rgb_to_xyz());

    // delta Eを計算する。
    let x_diff = rgb_lab[0] - out_lab[0];
    let y_diff = rgb_lab[1] - out_lab[1];
    let z_diff = rgb_lab[2] - out_lab[2];
    let squared_diff = x_diff * x_diff + y_diff * y_diff + z_diff * z_diff;
    squared_diff.sqrt()
}

/// 係数の勾配を計算する関数。
fn eval_gradient<G: ColorGamut>(
    coefficients: [f32; 3],
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f32; 3]; N_CIE_FINE_SAMPLES],
) -> [f32; 3] {
    const EPS: f32 = 1e-4;
    let mut grad = [0.0; 3];

    for i in 0..3 {
        let mut coefficients_neg = coefficients;
        coefficients_neg[i] -= EPS;
        let r_neg = eval_delta_e::<G>(coefficients_neg, rgb, lambda_table, rgb_table);

        let mut coefficients_pos = coefficients;
        coefficients_pos[i] += EPS;
        let r_pos = eval_delta_e::<G>(coefficients_pos, rgb, lambda_table, rgb_table);

        grad[i] = (r_pos - r_neg) / (2.0 * EPS);
    }

    grad
}

/// 渡されたRGBに対してフィットするようにAdamで二次式の係数を更新して計算する関数。
fn calculate_coefficients<G: ColorGamut>(
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f32; 3]; N_CIE_FINE_SAMPLES],
) -> [f32; 3] {
    const ITERATION: usize = 16;

    // 係数の初期化の初期値。
    // 初期値に割と鋭敏で初期値によっては収束しない。
    // この値は試行錯誤してだいたいのRGBで収束しそうなことを確認したもの。
    let mut coefficients = [0.5, 0.0, -0.1];

    // Adamのパラメータ。
    let mut m = [0.0; 5];
    let mut v = [0.0; 5];
    let beta1 = 0.9;
    let beta2 = 0.999;
    let epsilon = 1e-8;
    let lr = 0.1;

    for i in 0..ITERATION {
        // 現在の係数での誤差のdelta Eを計算する。
        let delta_e = eval_delta_e::<G>(coefficients, rgb, lambda_table, rgb_table);

        // 係数の勾配を計算する。
        let gradient = eval_gradient::<G>(coefficients, rgb, lambda_table, rgb_table);

        // Adamで係数を更新する。
        for j in 0..3 {
            m[j] = beta1 * m[j] + (1.0 - beta1) * gradient[j];
            v[j] = beta2 * v[j] + (1.0 - beta2) * gradient[j].powi(2);
            let m_hat = m[j] / (1.0 - beta1.powi(i as i32 + 1));
            let v_hat = v[j] / (1.0 - beta2.powi(i as i32 + 1));
            coefficients[j] -= lr * m_hat / (v_hat.sqrt() + epsilon);
        }

        // 誤差が十分小さくなったら終了する。
        if delta_e < 1e-3 {
            break;
        }
    }

    coefficients
}

/// delta Eを計算する関数を与えて、
/// RgbからSigmoidPolynomialの二次式の係数を引くための事前計算テーブルをコンパイル時に生成する。
pub fn init_table<G: ColorGamut>(z_nodes: &mut [f32], table: &mut Vec<Vec<Vec<Vec<[f32; 3]>>>>) {
    // 精度を0と1の付近に割り振るために、zの非線形なマッピングを計算する。
    const fn z_mapping(zi: usize) -> f32 {
        const fn smoothstep(x: f32) -> f32 {
            x * x * (3.0 - 2.0 * x)
        }
        smoothstep(smoothstep(zi as f32 / (TABLE_SIZE - 1) as f32))
    }
    for (i, z_node) in z_nodes.iter_mut().enumerate() {
        *z_node = z_mapping(i);
    }

    // CIE Yの360nmから830nmまで積分した値を求める。
    // 定数スペクトルで積分したときにCIE_Yが1になるようにこの値で積分をスケールする。
    let cie_y_integral: f32 = (0..N_CIE_FINE_SAMPLES)
        .into_par_iter()
        .map(|i| {
            // CIE Yの値を取得する。
            let lambda = LAMBDA_MIN + H * i as f32;
            let y = cie_interpret(&CIE_Y, lambda);

            // シンプソンの3/8公式で重みを計算する。
            let mut weight = 3.0 / 8.0 * H;
            if i == 0 || i == N_CIE_FINE_SAMPLES - 1 {
                // do nothing
            } else if (i - 1) % 3 == 2 {
                weight *= 2.0;
            } else {
                weight *= 3.0;
            }

            y * weight
        })
        .sum();

    // 事前にXYZからRGBに変換するための重みのテーブルを作っておく。
    let (lambda_table, rgb_table): (Vec<_>, Vec<_>) = (0..N_CIE_FINE_SAMPLES)
        .into_par_iter()
        .map(|i| {
            // lambda_tableとxyz_tableを計算する。
            let lambda = LAMBDA_MIN + H * i as f32;
            let x = cie_interpret(&CIE_X, lambda);
            let y = cie_interpret(&CIE_Y, lambda);
            let z = cie_interpret(&CIE_Z, lambda);

            // シンプソンの3/8公式で重みを計算する。
            let mut weight = 3.0 / 8.0 * H;
            if i == 0 || i == N_CIE_FINE_SAMPLES - 1 {
                // do nothing
            } else if (i - 1) % 3 == 2 {
                weight *= 2.0;
            } else {
                weight *= 3.0;
            }

            let xyz = glam::vec3(x, y, z) * weight / cie_y_integral;
            let rgb = G::new().xyz_to_rgb() * xyz;

            // (lambda, [x * weight, y * weight, z * weight])
            (lambda, [rgb.x, rgb.y, rgb.z])
        })
        .unzip();
    let lambda_table = lambda_table.try_into().unwrap();
    let rgb_table = rgb_table.try_into().unwrap();

    // 事前計算テーブルを計算する。
    (0..(3 * TABLE_SIZE * TABLE_SIZE * TABLE_SIZE))
        .into_par_iter()
        .map(|i| {
            // インデックスを計算する。
            let max_component = i / (TABLE_SIZE * TABLE_SIZE * TABLE_SIZE);
            let zi = (i / (TABLE_SIZE * TABLE_SIZE)) % TABLE_SIZE;
            let yi = (i / TABLE_SIZE) % TABLE_SIZE;
            let xi = i % TABLE_SIZE;

            // RGBの成分を計算する。
            let z = z_nodes[zi];
            let y = yi as f32 / (TABLE_SIZE - 1) as f32 * z;
            let x = xi as f32 / (TABLE_SIZE - 1) as f32 * z;
            let rgb = match max_component {
                0 => glam::vec3(z, x, y),
                1 => glam::vec3(y, z, x),
                2 => glam::vec3(x, y, z),
                _ => unreachable!(),
            };

            // 係数を計算する。
            calculate_coefficients::<G>(rgb, &lambda_table, &rgb_table)
        })
        .enumerate()
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|(i, item)| {
            // インデックスを計算する。
            let max_component = i / (TABLE_SIZE * TABLE_SIZE * TABLE_SIZE);
            let zi = (i / (TABLE_SIZE * TABLE_SIZE)) % TABLE_SIZE;
            let yi = (i / TABLE_SIZE) % TABLE_SIZE;
            let xi = i % TABLE_SIZE;

            // テーブルに格納する。
            table[max_component][zi][yi][xi] = [item[0], item[1], item[2]];
        });
}
