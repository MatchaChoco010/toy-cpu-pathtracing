//! RGBをシグモイドを掛けた二次式を利用してスペクトルを評価し、
//! スペクトルの波長成分を計算するモジュール。

use std::marker::PhantomData;
use std::sync::OnceLock;

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorImpl, ColorP3D65,
    ColorRec709, ColorRec2020, ColorSrgb, eotf::Eotf, gamut::ColorGamut, tone_map::NoneToneMap,
};
use rgb_to_spec::*;

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

/// 作成するテーブルの配列のサイズ。
const TABLE_SIZE: usize = 64;

/// シグモイド関数。
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// 係数から二次式を計算する関数。
fn parabolic(t: f32, coefficients: &[f32]) -> f32 {
    t * t * coefficients[0] + t * coefficients[1] + coefficients[2]
}

/// バイナリデータから変換されたテーブル構造体
struct RgbToSpectrumTable {
    z_nodes: Vec<f32>,
    table: Vec<Vec<Vec<Vec<Vec<f32>>>>>,
}

/// バイナリデータからテーブルを読み込む
fn load_table_from_binary(data: &[u8]) -> RgbToSpectrumTable {
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

    RgbToSpectrumTable { z_nodes, table }
}
impl RgbToSpectrumTable {
    /// テーブルから二次式の係数を取得する。
    fn get<G: ColorGamut, E: Eotf>(&self, color: ColorImpl<G, NoneToneMap, E>) -> [f32; 3] {
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

/// RGBからシグモイドを掛けた二次式でフィッティングしたスペクトルを保持し、
/// 波長に対するスペクトルの値を引くことができる構造体。
#[derive(Clone)]
pub struct RgbSigmoidPolynomial<C: Color + Clone> {
    c0: f32,
    c1: f32,
    c2: f32,
    _color_space: PhantomData<C>,
}
impl<C: Color> RgbSigmoidPolynomial<C> {
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
        let lambda = (lambda - LAMBDA_MIN) / (LAMBDA_MAX - LAMBDA_MIN);
        sigmoid(parabolic(lambda, &[self.c0, self.c1, self.c2]))
    }

    /// SigmoidPolynomialの最大値を評価する。
    pub fn max_value(&self) -> f32 {
        let mut result = self.value(LAMBDA_MIN).max(self.value(LAMBDA_MAX));
        let lambda = -self.c1 / (2.0 * self.c0);
        let lambda = lambda * (LAMBDA_MAX - LAMBDA_MIN) + LAMBDA_MIN;
        if (LAMBDA_MIN..=LAMBDA_MAX).contains(&lambda) {
            result = result.max(self.value(lambda));
        }
        result
    }
}

/// 各色空間のテーブルを遅延初期化で取得
macro_rules! get_table {
    ($data:expr, $lock:ident) => {{
        static $lock: OnceLock<RgbToSpectrumTable> = OnceLock::new();
        $lock.get_or_init(|| load_table_from_binary($data))
    }};
}

// 各色空間のFromトレイト実装
impl From<ColorSrgb<NoneToneMap>> for RgbSigmoidPolynomial<ColorSrgb<NoneToneMap>> {
    fn from(color: ColorSrgb<NoneToneMap>) -> Self {
        let table = get_table!(SRGB_DATA, SRGB_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorRec709<NoneToneMap>> for RgbSigmoidPolynomial<ColorRec709<NoneToneMap>> {
    fn from(color: ColorRec709<NoneToneMap>) -> Self {
        let table = get_table!(REC709_DATA, REC709_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorDisplayP3<NoneToneMap>> for RgbSigmoidPolynomial<ColorDisplayP3<NoneToneMap>> {
    fn from(color: ColorDisplayP3<NoneToneMap>) -> Self {
        let table = get_table!(DISPLAYP3_DATA, DISPLAYP3_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorP3D65<NoneToneMap>> for RgbSigmoidPolynomial<ColorP3D65<NoneToneMap>> {
    fn from(color: ColorP3D65<NoneToneMap>) -> Self {
        let table = get_table!(P3D65_DATA, P3D65_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorAdobeRGB<NoneToneMap>> for RgbSigmoidPolynomial<ColorAdobeRGB<NoneToneMap>> {
    fn from(color: ColorAdobeRGB<NoneToneMap>) -> Self {
        let table = get_table!(ADOBERGB_DATA, ADOBERGB_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorRec2020<NoneToneMap>> for RgbSigmoidPolynomial<ColorRec2020<NoneToneMap>> {
    fn from(color: ColorRec2020<NoneToneMap>) -> Self {
        let table = get_table!(REC2020_DATA, REC2020_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorAcesCg<NoneToneMap>> for RgbSigmoidPolynomial<ColorAcesCg<NoneToneMap>> {
    fn from(color: ColorAcesCg<NoneToneMap>) -> Self {
        let table = get_table!(ACESCG_DATA, ACESCG_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}

impl From<ColorAces2065_1<NoneToneMap>> for RgbSigmoidPolynomial<ColorAces2065_1<NoneToneMap>> {
    fn from(color: ColorAces2065_1<NoneToneMap>) -> Self {
        let table = get_table!(ACES2065_1_DATA, ACES2065_1_TABLE);
        let [c0, c1, c2] = table.get(color);
        Self::new(c0, c1, c2)
    }
}
