//! RgbSpectrumTableのテーブル配列をバイナリデータとして出力するためのビルドスクリプト。

use std::env;
use std::fs::File;
use std::io::Write;

use color::gamut::*;

mod data;
mod init;

use init::{TABLE_SIZE, init_table};

/// 可視光の波長の範囲の最小値 (nm)。
pub const LAMBDA_MIN: f64 = 360.0;
/// 可視光の波長の範囲の最大値 (nm)。
pub const LAMBDA_MAX: f64 = 830.0;

/// テーブルをバイナリファイルに書き出す。
fn write_binary_table(
    file_name: &str,
    z_nodes: &[f32],
    table: &Vec<Vec<Vec<Vec<[f32; 3]>>>>,
    out_dir: &str,
) -> anyhow::Result<()> {
    let path = format!("{out_dir}/{file_name}");
    let mut file = File::create(path)?;

    // z_nodesを書き込み (TABLE_SIZE個のf32)
    for &value in z_nodes {
        file.write_all(&value.to_le_bytes())?;
    }

    // tableを書き込み (3 × TABLE_SIZE³ × 3個のf32)
    for max_component in 0..3 {
        for zi in 0..TABLE_SIZE {
            for yi in 0..TABLE_SIZE {
                for xi in 0..TABLE_SIZE {
                    for component in 0..3 {
                        file.write_all(&table[max_component][zi][yi][xi][component].to_le_bytes())?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;

    // Vecを用意する。
    let mut z_nodes = vec![0.0; TABLE_SIZE];
    let mut table = vec![vec![vec![vec![[0.0_f32; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3];

    // sRGB
    init_table::<GamutSrgb>(&mut z_nodes, &mut table);
    write_binary_table("srgb_table.bin", &z_nodes, &table, &out_dir)?;

    // // Rec. 709は同じガマットなのでsRGBと同じテーブルを使用
    // write_binary_table("rec709_table.bin", &z_nodes, &table, &out_dir)?;

    // // Display P3
    // init_table::<GamutDciP3D65>(&mut z_nodes, &mut table);
    // write_binary_table("displayp3_table.bin", &z_nodes, &table, &out_dir)?;

    // // P3-D65は同じガマットなのでDisplay P3と同じテーブルを使用
    // write_binary_table("p3d65_table.bin", &z_nodes, &table, &out_dir)?;

    // // Adobe RGB
    // init_table::<GamutAdobeRgb>(&mut z_nodes, &mut table);
    // write_binary_table("adobergb_table.bin", &z_nodes, &table, &out_dir)?;

    // // Rec. 2020
    // init_table::<GamutRec2020>(&mut z_nodes, &mut table);
    // write_binary_table("rec2020_table.bin", &z_nodes, &table, &out_dir)?;

    // // ACEScg
    // init_table::<GamutAcesCg>(&mut z_nodes, &mut table);
    // write_binary_table("acescg_table.bin", &z_nodes, &table, &out_dir)?;

    // // ACES2065-1
    // init_table::<GamutAces2065_1>(&mut z_nodes, &mut table);
    // write_binary_table("aces2065_1_table.bin", &z_nodes, &table, &out_dir)?;

    // build/*.rsがdirtyなときだけ実行するようにする。
    println!("cargo:rerun-if-changed=build");

    Ok(())
}
