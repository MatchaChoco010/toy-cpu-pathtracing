//! RGBをシグモイドを掛けた多項式を利用してスペクトルを評価し、
//! スペクトルの波長成分を計算するモジュール。

use std::marker::PhantomData;

use color::eotf::{Eotf, Gamma2_2, Gamma2_6, GammaRec709, GammaSrgb, Linear};
use color::gamut::{
    Aces2065_1Gamut, AcesCgGamut, AdobeRgbGamut, ColorGamut, DciP3D65Gamut, Rec2020Gamut, SrgbGamut,
};
use color::tone_map::NoneToneMap;
use color::{
    Aces2065_1Color, AcesCgColor, AdobeRGBColor, Color, ColorTrait, DisplayP3Color, P3D65Color,
    Rec709Color, Rec2020Color, SrgbColor,
};
use math::square_root;

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

#[cfg(not(feature = "fast-analysis"))]
use color::gamut::xyz_to_lab;
#[cfg(not(feature = "fast-analysis"))]
use math::{lup_decompose, lup_solve};

#[cfg(not(feature = "fast-analysis"))]
const N_CIE_SAMPLES: usize = 95;
#[cfg(not(feature = "fast-analysis"))]
const N_CIE_FINE_SAMPLES: usize = (N_CIE_SAMPLES - 1) * 3 + 1;
#[cfg(not(feature = "fast-analysis"))]
const H: f32 = (LAMBDA_MAX - LAMBDA_MIN) / (N_CIE_FINE_SAMPLES - 1) as f32;

#[cfg(not(feature = "fast-analysis"))]
const CIE_X: [f32; N_CIE_SAMPLES] = [
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

#[cfg(not(feature = "fast-analysis"))]
const CIE_Y: [f32; N_CIE_SAMPLES] = [
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

#[cfg(not(feature = "fast-analysis"))]
const CIE_Z: [f32; N_CIE_SAMPLES] = [
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

#[cfg(not(feature = "fast-analysis"))]
const fn N(x: f32) -> f32 {
    x / 10566.864005283874576
}

#[cfg(not(feature = "fast-analysis"))]
const CIE_D65: [f32; N_CIE_SAMPLES] = [
    N(46.6383),
    N(49.3637),
    N(52.0891),
    N(51.0323),
    N(49.9755),
    N(52.3118),
    N(54.6482),
    N(68.7015),
    N(82.7549),
    N(87.1204),
    N(91.486),
    N(92.4589),
    N(93.4318),
    N(90.057),
    N(86.6823),
    N(95.7736),
    N(104.865),
    N(110.936),
    N(117.008),
    N(117.41),
    N(117.812),
    N(116.336),
    N(114.861),
    N(115.392),
    N(115.923),
    N(112.367),
    N(108.811),
    N(109.082),
    N(109.354),
    N(108.578),
    N(107.802),
    N(106.296),
    N(104.79),
    N(106.239),
    N(107.689),
    N(106.047),
    N(104.405),
    N(104.225),
    N(104.046),
    N(102.023),
    N(100.0),
    N(98.1671),
    N(96.3342),
    N(96.0611),
    N(95.788),
    N(92.2368),
    N(88.6856),
    N(89.3459),
    N(90.0062),
    N(89.8026),
    N(89.5991),
    N(88.6489),
    N(87.6987),
    N(85.4936),
    N(83.2886),
    N(83.4939),
    N(83.6992),
    N(81.863),
    N(80.0268),
    N(80.1207),
    N(80.2146),
    N(81.2462),
    N(82.2778),
    N(80.281),
    N(78.2842),
    N(74.0027),
    N(69.7213),
    N(70.6652),
    N(71.6091),
    N(72.979),
    N(74.349),
    N(67.9765),
    N(61.604),
    N(65.7448),
    N(69.8856),
    N(72.4863),
    N(75.087),
    N(69.3398),
    N(63.5927),
    N(55.0054),
    N(46.4182),
    N(56.6118),
    N(66.8054),
    N(65.0941),
    N(63.3828),
    N(63.8434),
    N(64.304),
    N(61.8779),
    N(59.4519),
    N(55.7054),
    N(51.959),
    N(54.6998),
    N(57.4406),
    N(58.8765),
    N(60.3125),
];

/// CIEの波長列を線形保管してサンプルする関数。
#[cfg(not(feature = "fast-analysis"))]
const fn cie_interpret(values: &[f32; N_CIE_SAMPLES], lambda: f32) -> f32 {
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
const fn sigmoid(x: f32) -> f32 {
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }
    0.5 + x / (2.0 * square_root(1.0 + x * x))
}

/// 多項式を計算する関数。
const fn evaluate_polynomial(t: f32, coefficients: &[f32]) -> f32 {
    if coefficients.len() == 1 {
        return coefficients[0];
    }
    let (&c, cs) = coefficients.split_first().unwrap();
    t * evaluate_polynomial(t, cs) + c
}

/// 線形補間を行う関数。
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// テーブルの配列サイズ。
const TABLE_SIZE: usize = 2;

/// RgbからSigmoidPolynomialを引くための事前計算テーブルの構造体。
struct RgbToSpectrumTable<G: ColorGamut, E: Eotf> {
    table: [[[[[f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
    z_nodes: [f32; TABLE_SIZE],
    _color_space: PhantomData<Color<G, NoneToneMap, E>>,
}
impl<G: ColorGamut, E: Eotf> RgbToSpectrumTable<G, E> {
    /// rust-analyzerの実行時に重たいコンパイル時計算をスキップするために、
    /// コンパイル時計算をスキップして空のダミーのテーブルを返す関数。
    #[cfg(feature = "fast-analysis")]
    const fn dummy() -> Self {
        Self {
            table: [[[[[0.0; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
            z_nodes: [0.0; TABLE_SIZE],
            _color_space: PhantomData,
        }
    }

    /// テーブルから多項式の係数を取得する。
    fn get(&self, color: Color<G, NoneToneMap, E>) -> [f32; 3] {
        // RGBの成分を取得。
        let rgb = color.rgb().max(glam::Vec3::splat(0.0));

        // RGBの成分が均一の場合は特別に定数関数になるように返す。
        if rgb.x == rgb.y && rgb.y == rgb.z {
            return [0.0, 0.0, (rgb.x - 0.5) / (rgb.x * (1.0 - rgb.x).sqrt())];
        }

        // RGBの最大成分を元にマップし直す。
        let max_component = rgb.max_position();
        let z = rgb[max_component];
        let x = rgb[(max_component + 1) % 3] * (TABLE_SIZE as f32 - 1.0) / z;
        let y = rgb[(max_component + 2) % 3] * (TABLE_SIZE as f32 - 1.0) / z;

        // 係数補間用のインデックスとオフセットを計算する。
        let xi = (x as usize).min(TABLE_SIZE - 2);
        let yi = (y as usize).min(TABLE_SIZE - 2);
        let zi = (0..TABLE_SIZE).find(|&i| self.z_nodes[i] < z).unwrap();
        let dx = x - xi as f32;
        let dy = y - yi as f32;
        let dz = (z as f32 - self.z_nodes[zi]) / (self.z_nodes[zi + 1] - self.z_nodes[zi]);

        // シグモイド多項式の係数を補間して計算する。
        let mut cs = [0.0; 3];
        for i in 0..3 {
            // シグモイド多項式の係数を参照するラムダを定義する。
            let co = |dx: usize, dy: usize, dz: usize| {
                self.table[max_component][zi + dz][yi + dy][xi + dx][i]
            };

            // シグモイド多項式の係数cを線形補間する。
            cs[i] = lerp(
                lerp(
                    lerp(co(0, 0, 0), co(1, 0, 0), dx),
                    lerp(co(0, 1, 0), co(1, 1, 0), dx),
                    dy,
                ),
                lerp(
                    lerp(co(0, 0, 1), co(1, 0, 1), dx),
                    lerp(co(0, 1, 1), co(1, 1, 1), dx),
                    dy,
                ),
                dz,
            );
        }
        cs
    }
}
macro_rules! impl_rgb_to_spectrum_table {
    ($gamut:ty, $eotf:ty) => {
        impl RgbToSpectrumTable<$gamut, $eotf> {
            /// 多項式の係数と目標のRGBを渡して、
            /// そのシグモイドを掛けた多項式で求まる色とのLab空間での差を計算する。
            #[cfg(not(feature = "fast-analysis"))]
            const fn eval_residual(
                coefficients: [f32; 3],
                rgb: glam::Vec3,
                lambda_table: &[f32; N_CIE_FINE_SAMPLES],
                xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
            ) -> glam::Vec3 {
                let mut i = 0;
                let mut out_xyz = [0.0; 3];
                while i < N_CIE_FINE_SAMPLES {
                    let lambda = lambda_table[i];

                    // 多項式の計算
                    let value = lambda * lambda * coefficients[0]
                        + lambda * coefficients[1]
                        + coefficients[2];

                    // シグモイドの計算。
                    let sigmoid_value = sigmoid(value);

                    // スペクトルをXYZに変換する。
                    out_xyz[0] += xyz_table[0][i] * sigmoid_value;
                    out_xyz[1] += xyz_table[1][i] * sigmoid_value;
                    out_xyz[2] += xyz_table[2][i] * sigmoid_value;

                    i += 1;
                }

                // XYZからLabに変換する。
                let out_lab = xyz_to_lab(glam::vec3(out_xyz[0], out_xyz[1], out_xyz[2]));
                let rgb_lab = <$gamut>::new().rgb_to_lab(rgb);

                // Lab空間での差を計算する。
                glam::vec3(
                    rgb_lab.x - out_lab.x,
                    rgb_lab.y - out_lab.y,
                    rgb_lab.z - out_lab.z,
                )
            }

            /// 多項式の係数と目標のRGBを渡して、係数を前後にずらしたときの残差から
            /// ヤコビアンを計算する。
            #[cfg(not(feature = "fast-analysis"))]
            const fn eval_jacobian(
                coefficients: [f32; 3],
                rgb: glam::Vec3,
                lambda_table: &[f32; N_CIE_FINE_SAMPLES],
                xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
            ) -> glam::Mat3 {
                const RGB2SPEC_EPSILON: f32 = 1e-4;

                let mut tmp;
                let mut jacobian = [0.0; 9];

                let mut i = 0;
                while i < 3 {
                    // 係数を小さく少しずらして評価する。
                    tmp = [coefficients[0], coefficients[1], coefficients[2]];
                    tmp[i] -= RGB2SPEC_EPSILON;
                    let r0 = Self::eval_residual(tmp, rgb, lambda_table, xyz_table).to_array();

                    // 係数を大きく少しずらして評価する。
                    tmp = [coefficients[0], coefficients[1], coefficients[2]];
                    tmp[i] += RGB2SPEC_EPSILON;
                    let r1 = Self::eval_residual(tmp, rgb, lambda_table, xyz_table).to_array();

                    let mut j = 0;
                    while j < 9 {
                        if j / 3 == i {
                            jacobian[j] = (r1[i] - r0[i]) / (2.0 * RGB2SPEC_EPSILON);
                        }
                        j += 1;
                    }

                    i += 1;
                }

                glam::Mat3::from_cols_array(&jacobian)
            }

            /// 渡されたRGBに対してガウス・ニュートン法で係数を計算する。
            #[cfg(not(feature = "fast-analysis"))]
            const fn calculate_coefficients(
                rgb: glam::Vec3,
                lambda_table: &[f32; N_CIE_FINE_SAMPLES],
                xyz_table: &[[f32; N_CIE_FINE_SAMPLES]; 3],
            ) -> [f32; 3] {
                const ITERATION: usize = 15;

                let mut coefficients = [0.0; 3];

                let mut i = 0;
                while i < ITERATION {
                    let residual = Self::eval_residual(coefficients, rgb, lambda_table, xyz_table);
                    let jacobian = Self::eval_jacobian(coefficients, rgb, lambda_table, xyz_table);

                    let (l, u, p) = lup_decompose(jacobian, 1e-15);

                    let x = lup_solve(l, u, p, residual).to_array();

                    let mut r = 0.0;
                    let mut j = 0;
                    while j < 3 {
                        coefficients[j] -= x[j];
                        let residual = residual.to_array();
                        r += residual[j] * residual[j];
                        j += 1;
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

                    i += 1;
                }

                coefficients
            }

            /// delta Eを計算する関数を与えて、
            /// RgbからSigmoidPolynomialの多項式の係数を引くための事前計算テーブルをコンパイル時に生成する。
            #[cfg(not(feature = "fast-analysis"))]
            const fn init() -> Self {
                // zの非線形なマッピングを計算する。
                const fn z_mapping(zi: usize) -> f64 {
                    const fn smoothstep(x: f64) -> f64 {
                        x * x * (3.0 - 2.0 * x)
                    }
                    smoothstep(smoothstep(zi as f64 / (TABLE_SIZE - 1) as f64))
                }
                let mut z_nodes = [0.0; TABLE_SIZE];
                let mut i = 0;
                while i < TABLE_SIZE {
                    z_nodes[i] = z_mapping(i) as f32;
                    i += 1;
                }

                // 事前にXYZからRGBに変換するための重みのテーブルを作っておく。
                let mut lambda_table = [0.0; N_CIE_FINE_SAMPLES];
                let mut xyz_table = [[0.0; N_CIE_FINE_SAMPLES]; 3];
                let mut i = 0;
                while i < N_CIE_FINE_SAMPLES {
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

                    i += 1;
                }

                let mut table = [[[[[0.0; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3];
                let mut max_component = 0;
                while max_component < 3 {
                    let mut zi = 0;
                    while zi < TABLE_SIZE {
                        let mut yi = 0;
                        while yi < TABLE_SIZE {
                            let mut xi = 0;
                            while xi < TABLE_SIZE {
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
                                    Self::calculate_coefficients(rgb, &lambda_table, &xyz_table);

                                xi += 1;
                            }
                            yi += 1;
                        }
                        zi += 1;
                    }
                    max_component += 1;
                }

                Self {
                    table,
                    z_nodes,
                    _color_space: PhantomData,
                }
            }
        }
    };
}
impl_rgb_to_spectrum_table!(SrgbGamut, GammaSrgb);
impl_rgb_to_spectrum_table!(DciP3D65Gamut, GammaSrgb);
impl_rgb_to_spectrum_table!(DciP3D65Gamut, Gamma2_6);
impl_rgb_to_spectrum_table!(AdobeRgbGamut, Gamma2_2);
impl_rgb_to_spectrum_table!(SrgbGamut, GammaRec709);
impl_rgb_to_spectrum_table!(Rec2020Gamut, GammaRec709);
impl_rgb_to_spectrum_table!(AcesCgGamut, Linear);
impl_rgb_to_spectrum_table!(Aces2065_1Gamut, Linear);

/// RGBからシグモイドを掛けた多項式でフィッティングしたスペクトルを保持し、
/// 波長に対するスペクトルの値を引くことができる構造体。
#[derive(Clone)]
pub struct RgbSigmoidPolynomial<C: ColorTrait + Clone> {
    c0: f32,
    c1: f32,
    c2: f32,
    _color_space: PhantomData<C>,
}
impl<C: ColorTrait> RgbSigmoidPolynomial<C> {
    /// SigmoidPolynomialの係数を指定して生成する。
    fn new(c0: f32, c1: f32, c2: f32) -> Self {
        Self {
            c0,
            c1,
            c2,
            _color_space: PhantomData,
        }
    }

    /// SigmoidPolynomialの特定の波長における値を評価する。
    pub fn value(&self, lambda: f32) -> f32 {
        sigmoid(evaluate_polynomial(lambda, &[self.c0, self.c1, self.c2]))
    }

    /// SigmoidPolynomialの最大値を評価する。
    pub fn max_value(&self) -> f32 {
        let mut result = self.value(LAMBDA_MIN).max(self.value(LAMBDA_MAX));
        let lambda = -self.c1 / (2.0 * self.c0);
        if lambda >= LAMBDA_MIN && lambda <= LAMBDA_MAX {
            result = result.max(self.value(lambda));
        }
        result
    }
}
impl From<SrgbColor> for RgbSigmoidPolynomial<SrgbColor> {
    fn from(color: SrgbColor) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<SrgbGamut, GammaSrgb> =
            RgbToSpectrumTable::<SrgbGamut, GammaSrgb>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<SrgbGamut, GammaSrgb> =
            RgbToSpectrumTable::<SrgbGamut, GammaSrgb>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<DisplayP3Color> for RgbSigmoidPolynomial<DisplayP3Color> {
    fn from(color: DisplayP3Color) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<DciP3D65Gamut, GammaSrgb> =
            RgbToSpectrumTable::<DciP3D65Gamut, GammaSrgb>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<DciP3D65Gamut, GammaSrgb> =
            RgbToSpectrumTable::<DciP3D65Gamut, GammaSrgb>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<P3D65Color> for RgbSigmoidPolynomial<P3D65Color> {
    fn from(color: P3D65Color) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<DciP3D65Gamut, Gamma2_6> =
            RgbToSpectrumTable::<DciP3D65Gamut, Gamma2_6>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<DciP3D65Gamut, Gamma2_6> =
            RgbToSpectrumTable::<DciP3D65Gamut, Gamma2_6>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<AdobeRGBColor> for RgbSigmoidPolynomial<AdobeRGBColor> {
    fn from(color: AdobeRGBColor) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<AdobeRgbGamut, Gamma2_2> =
            RgbToSpectrumTable::<AdobeRgbGamut, Gamma2_2>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<AdobeRgbGamut, Gamma2_2> =
            RgbToSpectrumTable::<AdobeRgbGamut, Gamma2_2>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<Rec709Color> for RgbSigmoidPolynomial<Rec709Color> {
    fn from(color: Rec709Color) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<SrgbGamut, GammaRec709> =
            RgbToSpectrumTable::<SrgbGamut, GammaRec709>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<SrgbGamut, GammaRec709> =
            RgbToSpectrumTable::<SrgbGamut, GammaRec709>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<Rec2020Color> for RgbSigmoidPolynomial<Rec2020Color> {
    fn from(color: Rec2020Color) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<Rec2020Gamut, GammaRec709> =
            RgbToSpectrumTable::<Rec2020Gamut, GammaRec709>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<Rec2020Gamut, GammaRec709> =
            RgbToSpectrumTable::<Rec2020Gamut, GammaRec709>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<AcesCgColor> for RgbSigmoidPolynomial<AcesCgColor> {
    fn from(color: AcesCgColor) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<AcesCgGamut, Linear> =
            RgbToSpectrumTable::<AcesCgGamut, Linear>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<AcesCgGamut, Linear> =
            RgbToSpectrumTable::<AcesCgGamut, Linear>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
impl From<Aces2065_1Color> for RgbSigmoidPolynomial<Aces2065_1Color> {
    fn from(color: Aces2065_1Color) -> Self {
        // RgbからSigmoidPolynomialを引くための事前計算テーブル。
        #[allow(long_running_const_eval)]
        #[cfg(not(feature = "fast-analysis"))]
        const TABLE: RgbToSpectrumTable<Aces2065_1Gamut, Linear> =
            RgbToSpectrumTable::<Aces2065_1Gamut, Linear>::init();
        #[cfg(feature = "fast-analysis")]
        const TABLE: RgbToSpectrumTable<Aces2065_1Gamut, Linear> =
            RgbToSpectrumTable::<Aces2065_1Gamut, Linear>::dummy();

        let [c0, c1, c2] = TABLE.get(color);
        Self::new(c0, c1, c2)
    }
}
