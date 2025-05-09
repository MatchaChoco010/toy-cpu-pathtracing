//! gbSpectrumTableのテーブル配列を定義するためのビルドスクリプト。

use std::env;
use std::fs::File;
use std::io::Write;

use color::gamut::{
    Aces2065_1Gamut, AcesCgGamut, AdobeRgbGamut, ColorGamut, DciP3D65Gamut, Rec2020Gamut,
    SrgbGamut, xyz_to_lab,
};
use math::{lup_decompose, lup_solve};

/// 可視光の波長の範囲の最小値 (nm)。
const LAMBDA_MIN: f32 = 360.0;
/// 可視光の波長の範囲の最大値 (nm)。
const LAMBDA_MAX: f32 = 830.0;

///作成するテーブルの配列のサイズ。
const TABLE_SIZE: usize = 36;

/// 積分の評価に使うサンプル数。
const N_CIE_FINE_SAMPLES: usize = (N_CIE_SAMPLES - 1) * 3 + 1;

/// サンプルの間隔。
const H: f32 = (LAMBDA_MAX - LAMBDA_MIN) / (N_CIE_FINE_SAMPLES - 1) as f32;

/// CIEの波長のデータのサンプル数。
pub const N_CIE_SAMPLES: usize = 95;

/// CIEのXYZマッチン曲線のXの曲線のスペクトルデータ。
pub const CIE_X: [f32; N_CIE_SAMPLES] = [
    0.000129900000,
    0.000232100000,
    0.000414900000,
    0.000741600000,
    0.001368000000,
    0.002236000000,
    0.004243000000,
    0.007650000000,
    0.014310000000,
    0.023190000000,
    0.043510000000,
    0.077630000000,
    0.134380000000,
    0.214770000000,
    0.283900000000,
    0.328500000000,
    0.348280000000,
    0.348060000000,
    0.336200000000,
    0.318700000000,
    0.290800000000,
    0.251100000000,
    0.195360000000,
    0.142100000000,
    0.095640000000,
    0.057950010000,
    0.032010000000,
    0.014700000000,
    0.004900000000,
    0.002400000000,
    0.009300000000,
    0.029100000000,
    0.063270000000,
    0.109600000000,
    0.165500000000,
    0.225749900000,
    0.290400000000,
    0.359700000000,
    0.433449900000,
    0.512050100000,
    0.594500000000,
    0.678400000000,
    0.762100000000,
    0.842500000000,
    0.916300000000,
    0.978600000000,
    1.026300000000,
    1.056700000000,
    1.062200000000,
    1.045600000000,
    1.002600000000,
    0.938400000000,
    0.854449900000,
    0.751400000000,
    0.642400000000,
    0.541900000000,
    0.447900000000,
    0.360800000000,
    0.283500000000,
    0.218700000000,
    0.164900000000,
    0.121200000000,
    0.087400000000,
    0.063600000000,
    0.046770000000,
    0.032900000000,
    0.022700000000,
    0.015840000000,
    0.011359160000,
    0.008110916000,
    0.005790346000,
    0.004109457000,
    0.002899327000,
    0.002049190000,
    0.001439971000,
    0.000999949300,
    0.000690078600,
    0.000476021300,
    0.000332301100,
    0.000234826100,
    0.000166150500,
    0.000117413000,
    0.000083075270,
    0.000058706520,
    0.000041509940,
    0.000029353260,
    0.000020673830,
    0.000014559770,
    0.000010253980,
    0.000007221456,
    0.000005085868,
    0.000003581652,
    0.000002522525,
    0.000001776509,
    0.000001251141,
];

/// CIEのXYZマッチン曲線のYの曲線のスペクトルデータ。
pub const CIE_Y: [f32; N_CIE_SAMPLES] = [
    0.000003917000,
    0.000006965000,
    0.000012390000,
    0.000022020000,
    0.000039000000,
    0.000064000000,
    0.000120000000,
    0.000217000000,
    0.000396000000,
    0.000640000000,
    0.001210000000,
    0.002180000000,
    0.004000000000,
    0.007300000000,
    0.011600000000,
    0.016840000000,
    0.023000000000,
    0.029800000000,
    0.038000000000,
    0.048000000000,
    0.060000000000,
    0.073900000000,
    0.090980000000,
    0.112600000000,
    0.139020000000,
    0.169300000000,
    0.208020000000,
    0.258600000000,
    0.323000000000,
    0.407300000000,
    0.503000000000,
    0.608200000000,
    0.710000000000,
    0.793200000000,
    0.862000000000,
    0.914850100000,
    0.954000000000,
    0.980300000000,
    0.994950100000,
    1.000000000000,
    0.995000000000,
    0.978600000000,
    0.952000000000,
    0.915400000000,
    0.870000000000,
    0.816300000000,
    0.757000000000,
    0.694900000000,
    0.631000000000,
    0.566800000000,
    0.503000000000,
    0.441200000000,
    0.381000000000,
    0.321000000000,
    0.265000000000,
    0.217000000000,
    0.175000000000,
    0.138200000000,
    0.107000000000,
    0.081600000000,
    0.061000000000,
    0.044580000000,
    0.032000000000,
    0.023200000000,
    0.017000000000,
    0.011920000000,
    0.008210000000,
    0.005723000000,
    0.004102000000,
    0.002929000000,
    0.002091000000,
    0.001484000000,
    0.001047000000,
    0.000740000000,
    0.000520000000,
    0.000361100000,
    0.000249200000,
    0.000171900000,
    0.000120000000,
    0.000084800000,
    0.000060000000,
    0.000042400000,
    0.000030000000,
    0.000021200000,
    0.000014990000,
    0.000010600000,
    0.000007465700,
    0.000005257800,
    0.000003702900,
    0.000002607800,
    0.000001836600,
    0.000001293400,
    0.000000910930,
    0.000000641530,
    0.000000451810,
];

/// CIEのXYZマッチン曲線のZの曲線のスペクトルデータ。
pub const CIE_Z: [f32; N_CIE_SAMPLES] = [
    0.000606100000,
    0.001086000000,
    0.001946000000,
    0.003486000000,
    0.006450001000,
    0.010549990000,
    0.020050010000,
    0.036210000000,
    0.067850010000,
    0.110200000000,
    0.207400000000,
    0.371300000000,
    0.645600000000,
    1.039050100000,
    1.385600000000,
    1.622960000000,
    1.747060000000,
    1.782600000000,
    1.772110000000,
    1.744100000000,
    1.669200000000,
    1.528100000000,
    1.287640000000,
    1.041900000000,
    0.812950100000,
    0.616200000000,
    0.465180000000,
    0.353300000000,
    0.272000000000,
    0.212300000000,
    0.158200000000,
    0.111700000000,
    0.078249990000,
    0.057250010000,
    0.042160000000,
    0.029840000000,
    0.020300000000,
    0.013400000000,
    0.008749999000,
    0.005749999000,
    0.003900000000,
    0.002749999000,
    0.002100000000,
    0.001800000000,
    0.001650001000,
    0.001400000000,
    0.001100000000,
    0.001000000000,
    0.000800000000,
    0.000600000000,
    0.000340000000,
    0.000240000000,
    0.000190000000,
    0.000100000000,
    0.000049999990,
    0.000030000000,
    0.000020000000,
    0.000010000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
    0.000000000000,
];

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

/// 多項式の係数と目標のRGBを渡して、
/// そのシグモイドを掛けた多項式で求まる色とのLab空間での差を計算する。
fn eval_residual<G: ColorGamut>(
    coefficients: [f32; 3],
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
) -> glam::Vec3 {
    let mut out_xyz = [0.0; 3];
    for i in 0..N_CIE_FINE_SAMPLES {
        let lambda = lambda_table[i];

        // 多項式の計算
        let value = lambda * lambda * coefficients[0] + lambda * coefficients[1] + coefficients[2];

        // シグモイドの計算。
        let sigmoid_value = sigmoid(value);

        // スペクトルをXYZに変換する。
        out_xyz[0] += xyz_table[0][i] * sigmoid_value;
        out_xyz[1] += xyz_table[1][i] * sigmoid_value;
        out_xyz[2] += xyz_table[2][i] * sigmoid_value;
    }

    // XYZからLabに変換する。
    let out_lab = xyz_to_lab(glam::vec3(out_xyz[0], out_xyz[1], out_xyz[2]));
    let rgb_lab = G::new().rgb_to_lab(rgb);

    // Lab空間での差を計算する。
    glam::vec3(
        rgb_lab.x - out_lab.x,
        rgb_lab.y - out_lab.y,
        rgb_lab.z - out_lab.z,
    )
}

/// 多項式の係数と目標のRGBを渡して、係数を前後にずらしたときの残差から
/// ヤコビアンを計算する。
fn eval_jacobian<G: ColorGamut>(
    coefficients: [f32; 3],
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
) -> glam::Mat3 {
    const RGB2SPEC_EPSILON: f32 = 1e-4;

    let mut tmp;
    let mut jacobian = [0.0; 9];

    for i in 0..3 {
        // 係数を小さく少しずらして評価する。
        tmp = [coefficients[0], coefficients[1], coefficients[2]];
        tmp[i] -= RGB2SPEC_EPSILON;
        let r0 = eval_residual::<G>(tmp, rgb, lambda_table, xyz_table).to_array();

        // 係数を大きく少しずらして評価する。
        tmp = [coefficients[0], coefficients[1], coefficients[2]];
        tmp[i] += RGB2SPEC_EPSILON;
        let r1 = eval_residual::<G>(tmp, rgb, lambda_table, xyz_table).to_array();

        for j in 0..3 {
            jacobian[j + i * 3] = (r1[j] - r0[j]) / (2.0 * RGB2SPEC_EPSILON);
        }
    }

    glam::Mat3::from_cols_array(&jacobian)
}

/// 渡されたRGBに対してガウス・ニュートン法で係数を計算する。
fn calculate_coefficients<G: ColorGamut>(
    rgb: glam::Vec3,
    lambda_table: &[f32; N_CIE_FINE_SAMPLES],
    xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
) -> [f32; 3] {
    const ITERATION: usize = 1;

    let mut coefficients = [0.0; 3];

    for _ in 0..ITERATION {
        let residual = eval_residual::<G>(coefficients, rgb, lambda_table, xyz_table);
        let jacobian = eval_jacobian::<G>(coefficients, rgb, lambda_table, xyz_table);

        let (l, u, p) = lup_decompose(jacobian, 1e-15);

        let x = lup_solve(l, u, p, residual).to_array();

        let mut r = 0.0;
        for j in 0..3 {
            coefficients[j] -= x[j];
            let residual = residual.to_array();
            r += residual[j] * residual[j];
        }
        let max = coefficients[0].max(coefficients[1]).max(coefficients[2]);

        if max > 200.0 {
            coefficients[0] *= 200.0 / max;
            coefficients[1] *= 200.0 / max;
            coefficients[2] *= 200.0 / max;
        }

        if r < 1e-6 {
            break;
        }
    }

    coefficients
}

/// delta Eを計算する関数を与えて、
/// RgbからSigmoidPolynomialの多項式の係数を引くための事前計算テーブルをコンパイル時に生成する。
fn init_table<G: ColorGamut>(
    z_nodes: &mut [f32; TABLE_SIZE],
    table: &mut [[[[[f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
) {
    // zの非線形なマッピングを計算する。
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
    let mut lambda_table = [0.0; N_CIE_FINE_SAMPLES];
    let mut xyz_table = [[0.0; N_CIE_FINE_SAMPLES]; 3];
    for i in 0..N_CIE_FINE_SAMPLES {
        let lambda = LAMBDA_MIN as f32 + H * i as f32;
        lambda_table[i] = lambda;
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

        xyz_table[0][i] = x * weight;
        xyz_table[1][i] = y * weight;
        xyz_table[2][i] = z * weight;
    }

    for max_component in 0..3 {
        for zi in 0..TABLE_SIZE {
            for yi in 0..TABLE_SIZE {
                for xi in 0..TABLE_SIZE {
                    // RGBの成分を計算する。
                    let z = z_nodes[zi];
                    let y = yi as f32 / (TABLE_SIZE - 1) as f32 * z;
                    let x = xi as f32 / (TABLE_SIZE - 1) as f32 * z;
                    let r;
                    let g;
                    let b;
                    match max_component {
                        0 => {
                            r = z;
                            g = x * z;
                            b = y * z;
                        }
                        1 => {
                            r = y * z;
                            g = z;
                            b = x * z;
                        }
                        2 => {
                            r = x * z;
                            g = y * z;
                            b = z;
                        }
                        _ => unreachable!(),
                    }
                    let rgb = glam::vec3(r, g, b);

                    // テーブルに格納する。
                    table[max_component][zi][yi][xi] =
                        calculate_coefficients::<G>(rgb, &lambda_table, &xyz_table);
                }
            }
        }
    }
}

fn write_table<G: ColorGamut>(
    writer: &mut impl Write,
    color_name: &str,
    gamut_name: &str,
    eotf_name: &str,
    z_nodes: &[f32; TABLE_SIZE],
    table: &[[[[[f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
) -> anyhow::Result<()> {
    // テーブルのサイズを出力する。
    writeln!(
        writer,
        r#"
impl From<{color_name}> for RgbSigmoidPolynomial<{color_name}> {{
    fn from(color: {color_name}) -> Self {{
        const TABLE_VALUES: [[[[[f32; 3]; {TABLE_SIZE}]; {TABLE_SIZE}]; {TABLE_SIZE}]; 3] = [
        "#
    )?;

    for max_component in 0..3 {
        writeln!(writer, "[")?;
        for zi in 0..TABLE_SIZE {
            writeln!(writer, "[")?;
            for yi in 0..TABLE_SIZE {
                writeln!(writer, "[")?;
                for xi in 0..TABLE_SIZE {
                    let mut x = table[max_component][zi][yi][xi][0].to_string();
                    if !x.contains('.') {
                        x.push_str(".0");
                    }
                    let mut y = table[max_component][zi][yi][xi][1].to_string();
                    if !y.contains('.') {
                        y.push_str(".0");
                    }
                    let mut z = table[max_component][zi][yi][xi][2].to_string();
                    if !z.contains('.') {
                        z.push_str(".0");
                    }
                    writeln!(writer, "[{x}, {y}, {z}],",)?;
                }
                writeln!(writer, "],")?;
            }
            writeln!(writer, "],")?;
        }
        writeln!(writer, "],")?;
    }

    writeln!(
        writer,
        r#"
        ];
        const Z_NODES: [f32; {TABLE_SIZE}] = [
        "#
    )?;

    for i in 0..TABLE_SIZE {
        let mut z = z_nodes[i].to_string();
        if !z.contains('.') {
            z.push_str(".0");
        }
        writeln!(writer, "{z},")?;
    }

    writeln!(
        writer,
        r#"
        ];
        const TABLE: RgbToSpectrumTable<{gamut_name}, {eotf_name}> = RgbToSpectrumTable {{
            z_nodes: Z_NODES,
            table: TABLE_VALUES,
            _color_space: PhantomData,
        }};
        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }}
}}
    "#
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // 出力のテーブル用のファイルのパスを構築する。。
    let out_dir = env::var("OUT_DIR")?;
    let table_file = format!("{out_dir}/spectrum_table.rs");

    // テーブルファイルを開く。
    let mut file = File::create(table_file)?;

    // 配列を用意する。
    let mut z_nodes = [0.0; TABLE_SIZE];
    let mut table = [[[[[0.0; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3];

    // sRGB
    init_table::<SrgbGamut>(&mut z_nodes, &mut table);
    write_table::<SrgbGamut>(
        &mut file,
        "SrgbColor",
        "SrgbGamut",
        "GammaSrgb",
        &z_nodes,
        &table,
    )?;

    // Display P3
    init_table::<DciP3D65Gamut>(&mut z_nodes, &mut table);
    write_table::<DciP3D65Gamut>(
        &mut file,
        "DisplayP3Color",
        "DciP3D65Gamut",
        "GammaSrgb",
        &z_nodes,
        &table,
    )?;

    // P3-D65
    init_table::<DciP3D65Gamut>(&mut z_nodes, &mut table);
    write_table::<DciP3D65Gamut>(
        &mut file,
        "P3D65Color",
        "DciP3D65Gamut",
        "Gamma2_6",
        &z_nodes,
        &table,
    )?;

    // Adobe RGB
    init_table::<AdobeRgbGamut>(&mut z_nodes, &mut table);
    write_table::<AdobeRgbGamut>(
        &mut file,
        "AdobeRGBColor",
        "AdobeRgbGamut",
        "Gamma2_2",
        &z_nodes,
        &table,
    )?;

    // Rec. 709
    init_table::<SrgbGamut>(&mut z_nodes, &mut table);
    write_table::<SrgbGamut>(
        &mut file,
        "Rec709Color",
        "SrgbGamut",
        "GammaRec709",
        &z_nodes,
        &table,
    )?;

    // Rec. 2020
    init_table::<Rec2020Gamut>(&mut z_nodes, &mut table);
    write_table::<Rec2020Gamut>(
        &mut file,
        "Rec2020Color",
        "Rec2020Gamut",
        "GammaRec709",
        &z_nodes,
        &table,
    )?;

    // ACEScg
    init_table::<AcesCgGamut>(&mut z_nodes, &mut table);
    write_table::<AcesCgGamut>(
        &mut file,
        "AcesCgColor",
        "AcesCgGamut",
        "Linear",
        &z_nodes,
        &table,
    )?;

    // ACES2065-1
    init_table::<Aces2065_1Gamut>(&mut z_nodes, &mut table);
    write_table::<Aces2065_1Gamut>(
        &mut file,
        "Aces2065_1Color",
        "Aces2065_1Gamut",
        "Linear",
        &z_nodes,
        &table,
    )?;

    Ok(())
}
