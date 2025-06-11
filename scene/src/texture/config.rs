//! テクスチャ設定とスペクトラム変換タイプ定義。

use std::path::PathBuf;

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

/// サポートされている色域。
#[derive(Debug, Clone, Copy)]
pub enum SupportedGamut {
    /// sRGB色域。
    SRgb,
    /// Display P3色域。
    DisplayP3,
    /// Adobe RGB色域。
    AdobeRgb,
    /// Rec. 2020色域。
    Rec2020,
}

/// テクスチャの設定。
#[derive(Debug, Clone)]
pub struct TextureConfig {
    /// テクスチャファイルのパス。
    pub path: PathBuf,
    /// ガンマ補正が適用されているかどうか。
    pub gamma_corrected: bool,
    /// 色域設定（RGBテクスチャのみ）。
    pub gamut: SupportedGamut,
}

impl TextureConfig {
    /// 新しいテクスチャ設定を作成する。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            gamma_corrected: true,       // デフォルトはsRGB
            gamut: SupportedGamut::SRgb, // デフォルトはsRGB色域
        }
    }

    /// ガンマ補正設定を変更する。
    pub fn with_gamma_corrected(mut self, gamma_corrected: bool) -> Self {
        self.gamma_corrected = gamma_corrected;
        self
    }

    /// 色域設定を変更する。
    pub fn with_gamut(mut self, gamut: SupportedGamut) -> Self {
        self.gamut = gamut;
        self
    }
}
