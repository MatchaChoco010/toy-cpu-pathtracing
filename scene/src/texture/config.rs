//! テクスチャ設定とスペクトラム変換タイプ定義。

/// スペクトラム変換のタイプ。
#[derive(Debug, Clone, Copy)]
pub enum SpectrumType {
    /// アルベド（反射率）スペクトラム用。
    Albedo,
    /// 光源（照明）スペクトラム用。
    Illuminant,
    /// 制限なしスペクトラム用。
    Unbounded,
}
