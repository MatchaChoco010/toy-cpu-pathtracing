pub static SRGB_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/srgb_table.bin"
));
pub static REC709_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/srgb_table.bin" // Rec. 709の色域はsRGBと同じなので、同じテーブルを使用
));
pub static DISPLAYP3_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/dcip3d65_table.bin"
));
pub static P3D65_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/dcip3d65_table.bin" // DCI-P3の色域はDisplay P3と同じなので、同じテーブルを使用
));
pub static ADOBERGB_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/adobergb_table.bin"
));
pub static REC2020_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/rec2020_table.bin"
));
pub static ACESCG_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/acescg_table.bin"
));
pub static ACES2065_1_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tables/aces2065_1_table.bin"
));
