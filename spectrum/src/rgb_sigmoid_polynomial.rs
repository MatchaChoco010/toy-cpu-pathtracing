//! RGBをシグモイドを掛けた多項式を利用してスペクトルを評価し、
//! スペクトルの波長成分を計算するモジュール。

use std::marker::PhantomData;

use color::{
    Color, ColorAces2065_1, ColorAcesCg, ColorAdobeRGB, ColorDisplayP3, ColorImpl, ColorP3D65,
    ColorRec709, ColorRec2020, ColorSrgb,
    eotf::{Eotf, Gamma2_2, Gamma2_6, GammaRec709, GammaSrgb, Linear},
    gamut::{
        ColorGamut, GamutAces2065_1, GamutAcesCg, GamutAdobeRgb, GamutDciP3D65, GamutRec2020,
        GamutSrgb,
    },
    tone_map::NoneToneMap,
};

use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN};

///作成するテーブルの配列のサイズ。
// const TABLE_SIZE: usize = 64;
const TABLE_SIZE: usize = 16;

/// シグモイド関数。
fn sigmoid(x: f32) -> f32 {
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }
    0.5 + x / (2.0 * (1.0 + x * x).sqrt())
}

fn exp(x: f32) -> f32 {
    if x.is_infinite() {
        return 0.0;
    }
    x.exp()
}

/// 多項式を計算する関数。
fn evaluate_polynomial(t: f32, coefficients: &[f32]) -> f32 {
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

/// RgbからSigmoidPolynomialを引くための事前計算テーブルの構造体。
struct RgbToSpectrumTable<G: ColorGamut, E: Eotf> {
    table: [[[[[f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3],
    z_nodes: [f32; TABLE_SIZE],
    _color_space: PhantomData<ColorImpl<G, NoneToneMap, E>>,
}
impl<G: ColorGamut, E: Eotf> RgbToSpectrumTable<G, E> {
    /// テーブルから多項式の係数を取得する。
    fn get(&self, color: ColorImpl<G, NoneToneMap, E>) -> [f32; 3] {
        /// 線形補間を行う関数。
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }

        // RGBの成分を取得。
        let rgb = color.rgb().max(glam::Vec3::splat(0.0));
        if rgb.max_element() > 1.0 {
            panic!("RGB elements must be in the range [0, 1]");
        }

        // RGBの成分が均一の場合は特別に定数関数になるように返す。
        if rgb.x == rgb.y && rgb.y == rgb.z {
            return [0.0, 0.0, (rgb.x - 0.5) / (rgb.x * (1.0 - rgb.x).sqrt())];
            // return [(rgb.x - 0.5) / (rgb.x * (1.0 - rgb.x).sqrt()), 0.0, 0.0];
        }

        println!("eotf color={:?}", color.rgb());
        let color = color.invert_eotf();
        println!("invert eotf color={:?}", color.rgb());
        let rgb = color.rgb().max(glam::Vec3::splat(0.0));

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
        let dz = (z as f32 - self.z_nodes[zi]) / (self.z_nodes[zi + 1] - self.z_nodes[zi]);

        println!("zi={zi}, xi={xi}, yi={yi}, dz={dz}, dx={dx}, dy={dy}");
        println!("z={z}");
        println! {"z_nodes[zi]={}", self.z_nodes[zi]};
        println! {"z_nodes[zi+1]={}", self.z_nodes[zi + 1]};

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
        println! {"cs={:?}", cs};
        cs
    }
}

/// RGBからシグモイドを掛けた多項式でフィッティングしたスペクトルを保持し、
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
        sigmoid(evaluate_polynomial(lambda, &[self.c0, self.c1, self.c2]))
        // exp(evaluate_polynomial(lambda, &[self.c0, self.c1, self.c2]))
    }

    /// SigmoidPolynomialの最大値を評価する。
    pub fn max_value(&self) -> f32 {
        let mut result = self.value(LAMBDA_MIN).max(self.value(LAMBDA_MAX));
        let lambda = -self.c1 / (2.0 * self.c0);
        let lambda = lambda * (LAMBDA_MAX - LAMBDA_MIN) + LAMBDA_MIN;
        if lambda >= LAMBDA_MIN && lambda <= LAMBDA_MAX {
            result = result.max(self.value(lambda));
        }
        result
    }
}

// ビルドスクリプトで生成したテーブルを読み込む。
include!(concat!(env!("OUT_DIR"), "/spectrum_table.rs"));
