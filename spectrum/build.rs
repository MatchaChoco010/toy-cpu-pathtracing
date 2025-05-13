use std::env;
use std::fs::File;
use std::io::Write;

use color::gamut::*;
use rgb_to_spec::{TABLE_SIZE, init_table};

/// 可視光の波長の範囲の最小値 (nm)。
pub const LAMBDA_MIN: f64 = 360.0;
/// 可視光の波長の範囲の最大値 (nm)。
pub const LAMBDA_MAX: f64 = 830.0;

/// ファイルにテーブルの定義を書き出す。
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
impl From<{color_name}<NoneToneMap>> for RgbSigmoidPolynomial<{color_name}<NoneToneMap>> {{
    fn from(color: {color_name}<NoneToneMap>) -> Self {{
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
    init_table::<GamutSrgb>(&mut z_nodes, &mut table);
    write_table::<GamutSrgb>(
        &mut file,
        "ColorSrgb",
        "GamutSrgb",
        "GammaSrgb",
        &z_nodes,
        &table,
    )?;

    // Rec. 709
    write_table::<GamutSrgb>(
        &mut file,
        "ColorRec709",
        "GamutSrgb",
        "GammaRec709",
        &z_nodes,
        &table,
    )?;

    // Display P3
    init_table::<GamutDciP3D65>(&mut z_nodes, &mut table);
    write_table::<GamutDciP3D65>(
        &mut file,
        "ColorDisplayP3",
        "GamutDciP3D65",
        "GammaSrgb",
        &z_nodes,
        &table,
    )?;

    // // P3-D65
    write_table::<GamutDciP3D65>(
        &mut file,
        "ColorP3D65",
        "GamutDciP3D65",
        "Gamma2_6",
        &z_nodes,
        &table,
    )?;

    // Adobe RGB
    init_table::<GamutAdobeRgb>(&mut z_nodes, &mut table);
    write_table::<GamutAdobeRgb>(
        &mut file,
        "ColorAdobeRGB",
        "GamutAdobeRgb",
        "Gamma2_2",
        &z_nodes,
        &table,
    )?;

    // Rec. 2020
    init_table::<GamutRec2020>(&mut z_nodes, &mut table);
    write_table::<GamutRec2020>(
        &mut file,
        "ColorRec2020",
        "GamutRec2020",
        "GammaRec709",
        &z_nodes,
        &table,
    )?;

    // ACEScg
    init_table::<GamutAcesCg>(&mut z_nodes, &mut table);
    write_table::<GamutAcesCg>(
        &mut file,
        "ColorAcesCg",
        "GamutAcesCg",
        "Linear",
        &z_nodes,
        &table,
    )?;

    // ACES2065-1
    init_table::<GamutAces2065_1>(&mut z_nodes, &mut table);
    write_table::<GamutAces2065_1>(
        &mut file,
        "ColorAces2065_1",
        "GamutAces2065_1",
        "Linear",
        &z_nodes,
        &table,
    )?;

    // build.rsがdirtyなときだけ実行するようにする。
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
