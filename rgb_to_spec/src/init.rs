//! gbSpectrumTableのテーブル配列を定義するためのビルドスクリプト。

use glam::DMat3;
use rand::Rng;
use rayon::prelude::*;

use color::f64::{Xyz, gamut::ColorGamut};

use crate::data::{
    CIE_D65, CIE_X, CIE_Y, CIE_Z, H, LAMBDA_MAX, LAMBDA_MIN, N_CIE_FINE_SAMPLES, N_CIE_SAMPLES,
};
use crate::matrix::{lup_decompose, lup_solve};

// pub const TABLE_SIZE: usize = 64;
pub const TABLE_SIZE: usize = 16;

const ITERATION: usize = 16;
// const ITERATION: usize = 64;
// const ITERATION: usize = 128;
// const ITERATION: usize = 256;
// const ITERATION: usize = 4096;

/// CIEの波長列を線形保管してサンプルする関数。
fn cie_interpret(values: &[f64; N_CIE_SAMPLES], lambda: f64) -> f64 {
    let x = lambda - LAMBDA_MIN;
    let mut x = x * ((N_CIE_SAMPLES - 1) as f64 / (LAMBDA_MAX - LAMBDA_MIN));
    if x < 0.0 {
        x = 0.0;
    }
    let mut offset = x as usize;
    if offset > N_CIE_SAMPLES - 2 {
        offset = N_CIE_SAMPLES - 2;
    }
    let weight = x - offset as f64;
    (1.0 - weight) * values[offset] + weight * values[offset + 1]
}

/// シグモイド関数。
fn sigmoid(x: f64) -> f64 {
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }
    0.5 + x / (2.0 * (1.0 + x * x).sqrt())
}

fn exp(x: f64) -> f64 {
    if x.is_infinite() {
        return 0.0;
    }
    x.exp()
}

/// 多項式を計算する関数。
fn evaluate_polynomial(t: f64, coefficients: &[f64]) -> f64 {
    // if coefficients.len() == 1 {
    //     return coefficients[0];
    // }
    // let (&c, cs) = coefficients.split_first().unwrap();
    // t * evaluate_polynomial(t, cs) + c
    // let mut x = 0.0;
    // for c in coefficients.iter() {
    //     x = x * t + c;
    // }
    // x

    // coefficients[0]だけxに平行移動してcoefficients[1]だけyに平行移動した、
    // 係数coefficients[2]に100を乗じた二次式。
    let x = t - coefficients[0];
    let xx = 100.0 * coefficients[2] * x * x;
    xx + coefficients[1]
}

use autodiff::*;

/// 多項式を計算する関数。
fn evaluate_polynomial_autodiff(t: F1, coefficients: &[F1]) -> F1 {
    // if coefficients.len() == 1 {
    //     return coefficients[0];
    // }
    // let (&c, cs) = coefficients.split_first().unwrap();
    // t * evaluate_polynomial_autodiff(t, cs) + c

    // coefficients[0]だけxに平行移動してcoefficients[1]だけyに平行移動した、
    // 係数coefficients[2]に100を乗じた二次式。
    let x = t - coefficients[0];
    let xx = 100.0 * coefficients[2] * x * x;
    xx + coefficients[1]
}

/// シグモイド関数。
fn sigmoid_autodiff(x: F1) -> F1 {
    if x.is_infinite() {
        return if x > F1::cst(0.0) {
            F1::cst(1.0)
        } else {
            F1::cst(0.0)
        };
    }
    0.5 + x / (2.0 * (x * x + 1.0).sqrt())
}

fn exp_autodiff(x: F1) -> F1 {
    if x.is_infinite() {
        return F1::cst(0.0);
    }
    x.exp()
}

fn mul_autodiff(a: glam::DMat3, v: &[F1; 3]) -> [F1; 3] {
    let mut out = [F1::cst(0.0); 3];
    for i in 0..3 {
        out[i] = a.x_axis[i] * v[0] + a.y_axis[i] * v[1] + a.z_axis[i] * v[2];
    }
    out
}

fn xyy_to_xyz(xy: glam::DVec2, y: f64) -> glam::DVec3 {
    if xy.y == 0.0 {
        return glam::DVec3::ZERO;
    }
    let x = xy.x * y / xy.y;
    let z = (1.0 - xy.x - xy.y) * y / xy.y;
    glam::dvec3(x, y, z)
}

fn xy_to_xyz(xy: glam::DVec2) -> glam::DVec3 {
    xyy_to_xyz(xy, 1.0)
}

/// RGBからCIE Labに変換する関数。
fn rgb_to_lab_autodiff(rgb: &[F1; 3], rgb_to_xyz: glam::DMat3) -> [F1; 3] {
    fn f(t: F1) -> F1 {
        if t > F1::cst(0.008856) {
            t.powf(F1::cst(1.0 / 3.0))
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }

    // RGBをXYZに変換する。
    let xyz = mul_autodiff(rgb_to_xyz, rgb);

    // D50白色点のXYZ値を取得する。
    let w = glam::dvec2(0.34567, 0.35850);
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

/// RGBからCIE Labに変換する関数。
fn rgb_to_lab(rgb: glam::DVec3, rgb_to_xyz: glam::DMat3) -> [f64; 3] {
    fn f(t: f64) -> f64 {
        if t > 0.008856 {
            t.powf(1.0 / 3.0)
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }

    // RGBをXYZに変換する。
    let xyz = rgb_to_xyz * rgb;

    // D50白色点のXYZ値を取得する。
    let w = glam::dvec2(0.34567, 0.35850);
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

fn delta_e_autodiff<G: ColorGamut>(
    coefficients: &[F1; 3],
    rgb: glam::DVec3,
    lambda_table: &[f64; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
) -> F1 {
    let mut out_rgb = [F1::cst(0.0); 3];
    for i in 0..N_CIE_FINE_SAMPLES {
        // lambdaの値を0..1の範囲に正規化する。
        let lambda = (lambda_table[i] - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN);
        let lambda = F1::cst(lambda);

        // 多項式の計算
        let value = evaluate_polynomial_autodiff(lambda, coefficients);

        // シグモイドの計算。
        let sigmoid_value = sigmoid_autodiff(value);

        // スペクトルをXYZに変換する。
        out_rgb[0] += rgb_table[i][0] * sigmoid_value;
        out_rgb[1] += rgb_table[i][1] * sigmoid_value;
        out_rgb[2] += rgb_table[i][2] * sigmoid_value;

        // let exp_value = exp_autodiff(value);

        // out_rgb[0] += rgb_table[i][0] * exp_value;
        // out_rgb[1] += rgb_table[i][1] * exp_value;
        // out_rgb[2] += rgb_table[i][2] * exp_value;
    }

    // Labに変換する。
    let out_lab = rgb_to_lab_autodiff(&out_rgb, G::new().rgb_to_xyz());
    let rgb_lab = rgb_to_lab_autodiff(
        &[F1::cst(rgb.x), F1::cst(rgb.y), F1::cst(rgb.z)],
        G::new().rgb_to_xyz(),
    );

    // delta Eを計算する。
    let x_diff = rgb_lab[0] - out_lab[0];
    let y_diff = rgb_lab[1] - out_lab[1];
    let z_diff = rgb_lab[2] - out_lab[2];
    let squared_diff = x_diff * x_diff + y_diff * y_diff + z_diff * z_diff;
    squared_diff.sqrt()
}

fn eval_delta_e<G: ColorGamut>(
    coefficients: [f64; 3],
    rgb: glam::DVec3,
    lambda_table: &[f64; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
) -> f64 {
    let mut out_rgb = [0.0; 3];
    for i in 0..N_CIE_FINE_SAMPLES {
        // lambdaの値を0..1の範囲に正規化する。
        let lambda = (lambda_table[i] - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN);

        // 多項式の計算
        let value = evaluate_polynomial(lambda, &coefficients);

        // シグモイドの計算。
        let sigmoid_value = sigmoid(value);

        // スペクトルをXYZに変換する。
        out_rgb[0] += rgb_table[i][0] * sigmoid_value;
        out_rgb[1] += rgb_table[i][1] * sigmoid_value;
        out_rgb[2] += rgb_table[i][2] * sigmoid_value;
    }

    // Labに変換する。
    let out_lab = rgb_to_lab(glam::DVec3::from(out_rgb), G::new().rgb_to_xyz());
    let rgb_lab = rgb_to_lab(rgb, G::new().rgb_to_xyz());

    // delta Eを計算する。
    let x_diff = rgb_lab[0] - out_lab[0];
    let y_diff = rgb_lab[1] - out_lab[1];
    let z_diff = rgb_lab[2] - out_lab[2];
    let squared_diff = x_diff * x_diff + y_diff * y_diff + z_diff * z_diff;
    squared_diff.sqrt()
}

fn eval_gradient<G: ColorGamut>(
    coefficients: [f64; 3],
    rgb: glam::DVec3,
    lambda_table: &[f64; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
) -> [f64; 3] {
    const EPS: f64 = 1e-4;
    let mut grad = [0.0; 3];

    for i in 0..3 {
        let mut coeff_neg = coefficients;
        coeff_neg[i] -= EPS;
        let r_neg = eval_delta_e::<G>(coeff_neg, rgb, lambda_table, rgb_table);

        let mut coeff_pos = coefficients;
        coeff_pos[i] += EPS;
        let r_pos = eval_delta_e::<G>(coeff_pos, rgb, lambda_table, rgb_table);

        grad[i] = (r_pos - r_neg) / (2.0 * EPS);
    }

    grad
}

// /// 多項式の係数と目標のRGBを渡して、
// /// そのシグモイドを掛けた多項式で求まる色とのLab空間での差を計算する。
// fn eval_residual<G: ColorGamut>(
//     coefficients: [f64; 5],
//     rgb: glam::DVec3,
//     lambda_table: &[f64; N_CIE_FINE_SAMPLES],
//     rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
// ) -> glam::DVec3 {
//     let mut out_rgb = [0.0; 3];
//     for i in 0..N_CIE_FINE_SAMPLES {
//         // lambdaの値を0..1の範囲に正規化する。
//         let lambda = (lambda_table[i] - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN);

//         // 多項式の計算
//         let value = evaluate_polynomial(lambda, &coefficients);

//         // シグモイドの計算。
//         let sigmoid_value = sigmoid(value);

//         // スペクトルをXYZに変換する。
//         out_rgb[0] += rgb_table[i][0] * sigmoid_value;
//         out_rgb[1] += rgb_table[i][1] * sigmoid_value;
//         out_rgb[2] += rgb_table[i][2] * sigmoid_value;
//     }

//     // Labに変換する。
//     let out_rgb = glam::DVec3::from(out_rgb);
//     let out_lab = G::new().rgb_to_lab(out_rgb);
//     let rgb_lab = G::new().rgb_to_lab(rgb);

//     // Lab空間での差を計算する。
//     glam::dvec3(
//         rgb_lab.x - out_lab.x,
//         rgb_lab.y - out_lab.y,
//         rgb_lab.z - out_lab.z,
//     )
// }

// /// 多項式の係数と目標のRGBを渡して、係数を前後にずらしたときの残差から
// /// ヤコビアンを計算する。
// fn eval_jacobian<G: ColorGamut>(
//     coefficients: [f64; 5],
//     rgb: glam::DVec3,
//     lambda_table: &[f64; N_CIE_FINE_SAMPLES],
//     rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
// ) -> glam::DMat3 {
//     const RGB2SPEC_EPSILON: f64 = 1e-4;

//     let mut jacobian = [[0.0; 3]; 3];

//     for i in 0..3 {
//         // 係数を小さく少しずらして評価する。
//         let mut delta_neg = coefficients.clone();
//         delta_neg[i] -= RGB2SPEC_EPSILON;
//         let r0 = eval_residual::<G>(delta_neg, rgb, lambda_table, rgb_table);

//         // 係数を大きく少しずらして評価する。
//         let mut delta_pos = coefficients.clone();
//         delta_pos[i] += RGB2SPEC_EPSILON;
//         let r1 = eval_residual::<G>(delta_pos, rgb, lambda_table, rgb_table);

//         for j in 0..3 {
//             jacobian[j][i] = (r1[j] - r0[j]) / (2.0 * RGB2SPEC_EPSILON);
//         }
//     }

//     glam::DMat3::from_cols_array_2d(&jacobian)
// }

/// 渡されたRGBに対してTikhonov正則化/L-M法で係数を更新し計算する。
fn calculate_coefficients<G: ColorGamut>(
    rgb: glam::DVec3,
    lambda_table: &[f64; N_CIE_FINE_SAMPLES],
    rgb_table: &[[f64; 3]; N_CIE_FINE_SAMPLES],
) -> [f64; 3] {
    // let mut coefficients = [0.0; 5];

    // let mut rng = rand::rng();
    // let mut coefficients = [-11.0, 50.0, -50.0];
    // let mut coefficients = [-50.0, 50.0, -11.0];
    // let mut coefficients = [0.0, 0.0, 0.0];
    // let mut coefficients = [1.0, 0.0, -50.0];
    // let mut coefficients = [0.5, 1.0, -1.0];
    // let mut coefficients = [0.5, 0.0, -0.1];
    let mut coefficients = [0.5, 0.0, -0.1];
    // let mut coefficients = [0.0; 3];

    // let mut rs = vec![];

    // Adamのパラメータ。
    let mut m = [0.0; 5];
    let mut v = [0.0; 5];
    let beta1 = 0.9;
    let beta2 = 0.999;
    let epsilon = 1e-8;
    let lr = 0.1;
    // let lr = 0.01;
    // let lr = 0.05;

    // let mut lambda = 1e-3; // 正則化項
    for i in 0..ITERATION {
        // // 誤差とヤコビアンを計算する。
        // let residual_before = eval_residual::<G>(coefficients, rgb, lambda_table, rgb_table);
        // let jacobian = eval_jacobian::<G>(coefficients, rgb, lambda_table, rgb_table);

        // // Tikhonov正則化/L-M法で更新用のupdateを計算する。
        // let jtj = jacobian.transpose() * jacobian;
        // let jtr = jacobian.transpose() * residual_before;
        // let mut jtj_lambda = jtj;
        // jtj_lambda +=
        //     lambda * DMat3::from_diagonal(glam::dvec3(jtj.x_axis.x, jtj.y_axis.y, jtj.z_axis.z));
        // let mut update = jtj_lambda.inverse() * jtr;

        // // 最大ステップ幅に収める。
        // let delta_norm = update.length();
        // let max_step = 150.0;
        // if delta_norm > max_step {
        //     update *= max_step / delta_norm;
        // }

        // // updateで新しい係数を計算する。
        // let mut new_coefficients = coefficients;
        // for j in 0..3 {
        //     new_coefficients[j] -= update[j];
        // }

        // // 新田しい係数での誤差を計算する。
        // let residual_after = eval_residual::<G>(new_coefficients, rgb, lambda_table, rgb_table);
        // let r_before = residual_before.length_squared();
        // let r_after = residual_after.length_squared();

        // // 更新前後で誤差が改善していたら受け入れて正則化項を減らし、
        // // 誤差がほとんど変わらない場合は更新を受け入れるが正則化項を増やし、
        // // 誤差が悪化しているかほとんど変わらない場合は更新を破棄して正則化項を増やす。
        // if r_after < r_before * (1.0 - 1e-3) {
        //     coefficients = new_coefficients;
        //     lambda *= 0.1;
        // } else if r_after < r_before {
        //     coefficients = new_coefficients;
        //     lambda *= 10.0;
        // } else {
        //     lambda *= 100.0;
        // }

        // // 誤差が十分小さくなったら終了する。
        // let r = residual_before.length_squared();
        // if r < 1e-3 {
        //     break;
        // }

        // // 誤差とヤコビアンを計算する。
        // let residual = eval_residual::<G>(coefficients, rgb, lambda_table, rgb_table);
        // let jacobian = eval_jacobian::<G>(coefficients, rgb, lambda_table, rgb_table);
        // let (l, u, p) = lup_decompose(jacobian, 1e-15);
        // let x = lup_solve(l, u, p, residual);

        // let mut r = 0.0;
        // for j in 0..3 {
        //     coefficients[j] -= x[j];
        //     r += x[j] * x[j];
        // }
        // // let max = coefficients
        // //     .iter()
        // //     .cloned()
        // //     .fold(f64::NEG_INFINITY, f64::max);
        // let max = coefficients
        //     .iter()
        //     .cloned()
        //     .map(|x| x.abs())
        //     .fold(f64::NEG_INFINITY, f64::max);
        // if max > 200.0 {
        //     coefficients[0] *= 200.0 / max;
        //     coefficients[1] *= 200.0 / max;
        //     coefficients[2] *= 200.0 / max;
        // }

        // // 誤差を計算する。
        // let residual = eval_residual::<G>(coefficients, rgb, lambda_table, rgb_table);
        // let mut r = 0.0;
        // for j in 0..3 {
        //     coefficients[j] += residual[j] * 0.1;
        //     r += residual[j] * residual[j];
        // }

        // rs.push(r);

        // if i == ITERATION - 1 {
        //     if r > 15.0 {
        //         println!("=== {r} ===\n    {coefficients:?}");
        //         println!("residual history: {rs:?}\n");
        //     }
        // }

        // let lr = if i < 50 {
        //     0.5
        // } else if i < 100 {
        //     0.1
        // } else {
        //     0.01
        // };

        let delta_e = eval_delta_e::<G>(coefficients, rgb, lambda_table, rgb_table);

        let gradient = eval_gradient::<G>(coefficients, rgb, lambda_table, rgb_table);
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

        // let current_coefficients = [
        //     F1::var(coefficients[0]),
        //     F1::var(coefficients[1]),
        //     F1::var(coefficients[2]),
        // ];
        // let delta_e = delta_e_autodiff::<G>(&current_coefficients, rgb, lambda_table, rgb_table);

        // let f = |cs: &[F1]| {
        //     let cs = [cs[0], cs[1], cs[2]];
        //     delta_e_autodiff::<G>(&cs, rgb, lambda_table, rgb_table)
        // };
        // let gradient = grad(f, &coefficients);

        // for j in 0..3 {
        //     m[j] = beta1 * m[j] + (1.0 - beta1) * gradient[j];
        //     v[j] = beta2 * v[j] + (1.0 - beta2) * gradient[j].powi(2);
        //     let m_hat = m[j] / (1.0 - beta1.powi(i as i32 + 1));
        //     let v_hat = v[j] / (1.0 - beta2.powi(i as i32 + 1));
        //     coefficients[j] -= lr * m_hat / (v_hat.sqrt() + epsilon);
        // }

        // // for j in 0..3 {
        // //     coefficients[j] -= gradient[j] * 0.01;
        // // }

        // // 誤差が十分小さくなったら終了する。
        // if delta_e.x < 1e-3 {
        //     break;
        // }

        // rs.push(delta_e);

        // if i == ITERATION - 1 {
        //     if delta_e.x > 15.0 {
        //         println!("=== {delta_e} ===");
        //         println!("    {coefficients:?}");
        //         println!("    {gradient:?}");
        //         // println!("delta E history: {rs:?}\n");
        //     }
        // }
    }

    coefficients
}

/// delta Eを計算する関数を与えて、
/// RgbからSigmoidPolynomialの多項式の係数を引くための事前計算テーブルをコンパイル時に生成する。
pub fn init_table<G: ColorGamut>(
    z_nodes: &mut [f32; TABLE_SIZE],
    table: &mut [[[[[f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
) {
    // 精度を0と1の付近に割り振るために、zの非線形なマッピングを計算する。
    const fn z_mapping(zi: usize) -> f64 {
        const fn smoothstep(x: f64) -> f64 {
            x * x * (3.0 - 2.0 * x)
        }
        smoothstep(smoothstep(zi as f64 / (TABLE_SIZE - 1) as f64))
    }
    for i in 0..TABLE_SIZE {
        z_nodes[i] = z_mapping(i) as f32;
    }

    // 事前にXYZからRGBに変換するための重みのテーブルを作っておく。
    let (lambda_table, rgb_table): (Vec<_>, Vec<_>) = (0..N_CIE_FINE_SAMPLES)
        .into_par_iter()
        .map(|i| {
            // lambda_tableとxyz_tableを計算する。
            let lambda = LAMBDA_MIN as f64 + H * i as f64;
            let x = cie_interpret(&CIE_X, lambda);
            let y = cie_interpret(&CIE_Y, lambda);
            let z = cie_interpret(&CIE_Z, lambda);
            let illuminant = cie_interpret(&CIE_D65, lambda);

            // シンプソンの3/8公式で重みを計算する。
            let mut weight = 3.0 / 8.0 * H;
            if i == 0 || i == N_CIE_FINE_SAMPLES - 1 {
                // do nothing
            } else if (i - 1) % 3 == 2 {
                weight *= 2.0;
            } else {
                weight *= 3.0;
            }

            let xyz = glam::dvec3(x, y, z) * illuminant * weight;
            // let xyz = glam::dvec3(x, y, z) * weight;
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
            let z = z_nodes[zi] as f64;
            let y = yi as f64 / (TABLE_SIZE - 1) as f64 * z;
            let x = xi as f64 / (TABLE_SIZE - 1) as f64 * z;
            let rgb = match max_component {
                0 => glam::dvec3(z, x, y),
                1 => glam::dvec3(y, z, x),
                2 => glam::dvec3(x, y, z),
                _ => unreachable!(),
            };

            // 係数を計算する。
            // let cs = calculate_coefficients::<G>(rgb, &lambda_table, &xyz_table);
            // let c0 = LAMBDA_MIN;
            // let c1 = 1.0 / (LAMBDA_MAX - LAMBDA_MIN);
            // let a = cs[0];
            // let b = cs[1];
            // let c = cs[2];
            // let out0 = (a * c1.sqrt()) as f32;
            // let out1 = (b * c1 - 2.0 * a * c0 * c1.sqrt()) as f32;
            // let out2 = (c - b * c0 * c1 + a * (c0 * c1).sqrt()) as f32;
            // [out0, out1, out2]
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
            table[max_component][zi][yi][xi] = [item[0] as f32, item[1] as f32, item[2] as f32];
        });

    // {
    //     let max_component = 1;
    //     let zi = 10;
    //     let yi = 3;
    //     let xi = 3;

    //     // RGBの成分を計算する。
    //     let z = z_nodes[zi] as f64;
    //     let y = yi as f64 / (TABLE_SIZE - 1) as f64 * z;
    //     let x = xi as f64 / (TABLE_SIZE - 1) as f64 * z;
    //     let rgb = match max_component {
    //         0 => glam::dvec3(z, x, y),
    //         1 => glam::dvec3(y, z, x),
    //         2 => glam::dvec3(x, y, z),
    //         _ => unreachable!(),
    //     };

    //     // 係数を計算する。
    //     // let cs = calculate_coefficients::<G>(rgb, &lambda_table, &xyz_table);
    //     // let c0 = LAMBDA_MIN;
    //     // let c1 = 1.0 / (LAMBDA_MAX - LAMBDA_MIN);
    //     // let a = cs[0];
    //     // let b = cs[1];
    //     // let c = cs[2];
    //     // let out0 = (a * c1.sqrt()) as f32;
    //     // let out1 = (b * c1 - 2.0 * a * c0 * c1.sqrt()) as f32;
    //     // let out2 = (c - b * c0 * c1 + a * (c0 * c1).sqrt()) as f32;
    //     // [out0, out1, out2]
    //     let cs0 = calculate_coefficients::<G>(rgb, &lambda_table, &rgb_table);

    //     // RGBの成分を計算する。
    //     let z = z_nodes[zi + 1] as f64;
    //     let y = yi as f64 / (TABLE_SIZE - 1) as f64 * z;
    //     let x = xi as f64 / (TABLE_SIZE - 1) as f64 * z;
    //     let rgb = match max_component {
    //         0 => glam::dvec3(z, x, y),
    //         1 => glam::dvec3(y, z, x),
    //         2 => glam::dvec3(x, y, z),
    //         _ => unreachable!(),
    //     };
    //     let cs1 = calculate_coefficients::<G>(rgb, &lambda_table, &rgb_table);

    //     let dz = 0.78318906;

    //     let mut cs = [0.0; 3];
    //     for i in 0..3 {
    //         cs[i] = (1.0 - dz) * cs0[i] + dz * cs1[i];
    //     }
    //     println!("cs={:?}", cs);
    // }
}
