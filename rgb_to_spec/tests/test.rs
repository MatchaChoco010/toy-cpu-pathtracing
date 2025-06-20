use rayon::prelude::*;

use color::{eotf::*, gamut::*, tone_map::*, *};

mod cie_data;

/// 多項式のテーブルのサイズ。
pub const TABLE_SIZE: usize = 64;

/// バイナリデータから変換されたテーブル構造体
struct RgbToSpectrumTable {
    z_nodes: Vec<f32>,
    table: Vec<Vec<Vec<Vec<Vec<f32>>>>>,
}
impl RgbToSpectrumTable {
    /// バイナリデータからテーブルを読み込む
    fn load_table_from_binary(data: &[u8]) -> Self {
        let expected_size = TABLE_SIZE * 4 + (3 * TABLE_SIZE * TABLE_SIZE * TABLE_SIZE * 3 * 4);
        assert_eq!(data.len(), expected_size, "Binary data size mismatch");

        let mut offset = 0;

        // z_nodes を読み込み
        let mut z_nodes = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            z_nodes.push(value);
            offset += 4;
        }

        // table を読み込み
        let mut table = Vec::with_capacity(3);
        for _ in 0..3 {
            let mut zi_vec = Vec::with_capacity(TABLE_SIZE);
            for _ in 0..TABLE_SIZE {
                let mut yi_vec = Vec::with_capacity(TABLE_SIZE);
                for _ in 0..TABLE_SIZE {
                    let mut xi_vec = Vec::with_capacity(TABLE_SIZE);
                    for _ in 0..TABLE_SIZE {
                        let mut component_vec = Vec::with_capacity(3);
                        for _ in 0..3 {
                            let value = f32::from_le_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]);
                            component_vec.push(value);
                            offset += 4;
                        }
                        xi_vec.push(component_vec);
                    }
                    yi_vec.push(xi_vec);
                }
                zi_vec.push(yi_vec);
            }
            table.push(zi_vec);
        }

        Self { z_nodes, table }
    }

    /// テーブルから二次式の係数を取得する。
    fn get<G: ColorGamut, E: Eotf>(&self, color: &ColorImpl<G, NoneToneMap, E>) -> [f32; 3] {
        /// 線形補間を行う関数。
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }

        // RGBのEOTFを逆変換してリニアな値にする。
        let color = color.invert_eotf();
        let rgb = color.rgb().max(glam::Vec3::splat(0.0));

        // RGBの最大値が1.0を超えている場合はエラーとする。
        if rgb.max_element() > 1.0 {
            // Debug print before panic to understand which values are causing the issue
            eprintln!(
                "RGB validation error: RGB({:.6}, {:.6}, {:.6}) has max element {:.6} > 1.0",
                rgb.x,
                rgb.y,
                rgb.z,
                rgb.max_element()
            );
            panic!("RGB elements must be in the range [0, 1]");
        }

        // RGBの成分が均一の場合は特別に定数関数になるように返す。
        if rgb.x == rgb.y && rgb.y == rgb.z {
            return [0.0, 0.0, (rgb.x / (1.0 - rgb.x)).ln()];
        }

        // RGBの最大成分を元にマップし直す。
        let max_component = rgb.max_position();
        let z = rgb[max_component];
        let x = rgb[(max_component + 1) % 3] * (TABLE_SIZE as f32 - 1.0) / z;
        let y = rgb[(max_component + 2) % 3] * (TABLE_SIZE as f32 - 1.0) / z;

        // 係数補間用のインデックスとオフセットを計算する。
        let xi = (x as usize).min(TABLE_SIZE - 2);
        let yi = (y as usize).min(TABLE_SIZE - 2);
        let zi = (0..=TABLE_SIZE - 2)
            .find(|&i| self.z_nodes[i + 1] > z)
            .unwrap_or(TABLE_SIZE - 2);
        let dx = x - xi as f32;
        let dy = y - yi as f32;
        let dz = (z - self.z_nodes[zi]) / (self.z_nodes[zi + 1] - self.z_nodes[zi]);

        // シグモイド二次式の係数を補間して計算する。
        let mut cs = [0.0; 3];
        for i in 0..3 {
            // シグモイド二次式の係数を参照するラムダを定義する。
            let co = |dx: usize, dy: usize, dz: usize| {
                self.table[max_component][zi + dz][yi + dy][xi + dx][i]
            };

            // シグモイド二次式の係数cを線形補間する。
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

/// シグモイド関数。
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// 係数から二次式を計算する関数。
fn parabolic(t: f32, coefficients: &[f32]) -> f32 {
    t * t * coefficients[0] + t * coefficients[1] + coefficients[2]
}

/// SigmoidPolynomialの特定の波長における値を評価する。
fn value(lambda: f32, cs: &[f32; 3]) -> f32 {
    let lambda = (lambda - cie_data::LAMBDA_MIN) / (cie_data::LAMBDA_MAX - cie_data::LAMBDA_MIN);
    sigmoid(parabolic(lambda, cs))
}

fn evaluate_spectrum<G: ColorGamut, E: Eotf>(
    data: &[u8],
    color: &ColorImpl<G, NoneToneMap, E>,
) -> ColorImpl<G, NoneToneMap, E> {
    let table = RgbToSpectrumTable::load_table_from_binary(data);
    let cs = table.get(color);

    let mut xyz = glam::Vec3::ZERO;
    let range = 0..=(cie_data::LAMBDA_MAX - cie_data::LAMBDA_MIN) as usize;
    for i in range {
        let lambda = cie_data::LAMBDA_MIN + i as f32;
        xyz.x +=
            value(lambda, &cs) * cie_data::cie_value_x(lambda) * cie_data::cie_value_d65(lambda);
        xyz.y +=
            value(lambda, &cs) * cie_data::cie_value_y(lambda) * cie_data::cie_value_d65(lambda);
        xyz.z +=
            value(lambda, &cs) * cie_data::cie_value_z(lambda) * cie_data::cie_value_d65(lambda);
    }

    let rgb = (G::new().xyz_to_rgb() * xyz).max(glam::Vec3::ZERO);

    let linear_color = ColorImpl::<G, NoneToneMap, Linear>::new(rgb.x, rgb.y, rgb.z);
    linear_color.apply_eotf()
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
fn rgb_to_lab(rgb: glam::Vec3, rgb_to_xyz: glam::Mat3) -> glam::Vec3 {
    fn f(t: f32) -> f32 {
        if t > 0.008856 {
            t.powf(1.0 / 3.0)
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }

    // RGBをXYZに変換する。
    let xyz = rgb_to_xyz * rgb;

    // D65白色点のXYZ値を取得する。
    let w = glam::vec2(0.31270, 0.32900);
    let w_xyz = xy_to_xyz(w);

    // Xr, Yr, Zrを計算する。
    let xr = xyz[0] / w_xyz.x;
    let yr = xyz[1] / w_xyz.y;
    let zr = xyz[2] / w_xyz.z;

    // Xr, Yr, Zrを使ってL*, a*, b*を計算する。
    let l = 116.0 * f(yr) - 16.0;
    let a = 500.0 * (f(xr) - f(yr));
    let b = 200.0 * (f(yr) - f(zr));

    glam::vec3(l, a, b)
}

fn color_match_test<G: ColorGamut, E: Eotf>(data: &[u8]) {
    let error_3 = (0..=(255 / 16))
        .into_par_iter()
        .map(|r| {
            (0..=(255 / 16))
                .map(|g| {
                    (0..=(255 / 16))
                        .map(|b| {
                            let r = r * 16;
                            let g = g * 16;
                            let b = b * 16;
                            let color = ColorImpl::<G, NoneToneMap, E>::new(
                                r as f32 / 255.0,
                                g as f32 / 255.0,
                                b as f32 / 255.0,
                            );
                            let evaluated_color = evaluate_spectrum::<G, E>(data, &color);

                            let linear_color = color.invert_eotf();
                            let color_lab = rgb_to_lab(linear_color.rgb(), G::new().rgb_to_xyz());

                            let evaluated_color_linear = evaluated_color.invert_eotf();
                            let evaluated_color_lab =
                                rgb_to_lab(evaluated_color_linear.rgb(), G::new().rgb_to_xyz());

                            let delta_e = (color_lab - evaluated_color_lab).length();
                            let error = delta_e > 3.0;
                            if error {
                                eprintln!(
                                    "Color mismatch: RGB({:.3}, {:.3}, {:.3}) vs Evaluated RGB({:.3}, {:.3}, {:.3}), Delta E: {:.3}",
                                    color.rgb().x, color.rgb().y, color.rgb().z,
                                    evaluated_color.rgb().x, evaluated_color.rgb().y, evaluated_color.rgb().z,
                                    delta_e
                                );
                            }
                            error
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|x| if x { 1 } else { 0 })
                        .sum::<u32>()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .sum::<u32>()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .sum::<u32>();

    println!(
        "Color match test result: {}/{} violations (delta E > 3)",
        error_3,
        256 / 16 * 256 / 16 * 256 / 16
    );

    let max_cs = (0..=(255 / 16))
        .into_par_iter()
        .map(|r| {
            (0..=(255 / 16))
                .map(|g| {
                    (0..=(255 / 16))
                        .map(|b| {
                            let r = r * 16;
                            let g = g * 16;
                            let b = b * 16;
                            let color = ColorImpl::<G, NoneToneMap, E>::new(
                                r as f32 / 255.0,
                                g as f32 / 255.0,
                                b as f32 / 255.0,
                            );
                            let table = RgbToSpectrumTable::load_table_from_binary(data);

                            table.get(&color)
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .fold(glam::Vec3::ZERO, |acc, cs| {
                            glam::vec3(acc.x.max(cs[0]), acc.y.max(cs[1]), acc.z.max(cs[2]))
                        })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .fold(glam::Vec3::ZERO, |acc, cs| {
                    glam::vec3(acc.x.max(cs.x), acc.y.max(cs.y), acc.z.max(cs.z))
                })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .fold(glam::Vec3::ZERO, |acc, cs| {
            glam::vec3(acc.x.max(cs.x), acc.y.max(cs.y), acc.z.max(cs.z))
        });

    println!(
        "Maximum coefficients: a: {:.6}, b: {:.6}, c: {:.6}",
        max_cs.x, max_cs.y, max_cs.z
    );
}

#[test]
fn color_match_test_srgb() {
    println!("Testing sRGB color match...");
    let data = rgb_to_spec::SRGB_DATA;
    color_match_test::<GamutSrgb, GammaSrgb>(data);
}

#[test]
fn color_match_test_rec709() {
    println!("Testing Rec. 709 color match...");
    let data = rgb_to_spec::SRGB_DATA;
    color_match_test::<GamutSrgb, GammaRec709>(data);
}

#[test]
fn color_match_test_display_p3() {
    println!("Testing Display P3 color match...");
    let data = rgb_to_spec::DISPLAYP3_DATA;
    color_match_test::<GamutDciP3D65, GammaSrgb>(data);
}

#[test]
fn color_match_test_p3d65() {
    println!("Testing P3 D65 color match...");
    let data = rgb_to_spec::P3D65_DATA;
    color_match_test::<GamutDciP3D65, Gamma2_6>(data);
}

#[test]
fn color_match_test_adobe_rgb() {
    println!("Testing Adobe RGB color match...");
    let data = rgb_to_spec::ADOBERGB_DATA;
    color_match_test::<GamutAdobeRgb, Gamma2_2>(data);
}

#[test]
fn color_match_test_rec2020() {
    println!("Testing Rec. 2020 color match...");
    let data = rgb_to_spec::REC2020_DATA;
    color_match_test::<GamutRec2020, GammaRec709>(data);
}

#[test]
fn color_match_test_acescg() {
    println!("Testing ACEScg color match...");
    let data = rgb_to_spec::ACESCG_DATA;
    color_match_test::<GamutAcesCg, Linear>(data);
}

#[test]
fn color_match_test_aces2065_1() {
    println!("Testing ACES2065-1 color match...");
    let data = rgb_to_spec::ACES2065_1_DATA;
    color_match_test::<GamutAces2065_1, Linear>(data);
}
