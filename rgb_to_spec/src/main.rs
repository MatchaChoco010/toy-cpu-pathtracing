mod data;
mod init;

use init::TABLE_SIZE;

fn main() {
    // 配列を用意する。
    let mut z_nodes = [0.0; TABLE_SIZE];
    let mut table = [[[[[0.0; 3]; TABLE_SIZE]; TABLE_SIZE]; TABLE_SIZE]; 3];

    // エラーが出ずに呼び出せることを確認していく。

    println!("GamutSrgb");
    init::init_table::<color::gamut::GamutSrgb>(&mut z_nodes, &mut table);

    println!("GamutDciP3D65");
    init::init_table::<color::gamut::GamutDciP3D65>(&mut z_nodes, &mut table);

    println!("GamutAdobeRgb");
    init::init_table::<color::gamut::GamutAdobeRgb>(&mut z_nodes, &mut table);

    println!("GamutRec2020");
    init::init_table::<color::gamut::GamutRec2020>(&mut z_nodes, &mut table);

    println!("GamutAcesCg");
    init::init_table::<color::gamut::GamutAcesCg>(&mut z_nodes, &mut table);

    println!("GamutAces2065_1");
    init::init_table::<color::gamut::GamutAces2065_1>(&mut z_nodes, &mut table);
}
