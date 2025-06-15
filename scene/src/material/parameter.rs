//! マテリアルパラメータ定義。

use crate::texture::{FloatTexture, NormalTexture, RgbTexture, SpectrumType, TextureSample};
use glam::Vec2;
use math::{Normal, ShadingTangent};
use spectrum::Spectrum;
use std::sync::Arc;

/// スペクトラムパラメータ。
#[derive(Clone)]
pub enum SpectrumParameter {
    /// 定数値。
    Constant(Spectrum),
    /// テクスチャから取得。
    Texture {
        texture: Arc<RgbTexture>,
        spectrum_type: SpectrumType,
    },
}

impl SpectrumParameter {
    /// UV座標でスペクトラムをサンプリングする。
    pub fn sample(&self, uv: Vec2) -> Spectrum {
        match self {
            SpectrumParameter::Constant(spectrum) => spectrum.clone(),
            SpectrumParameter::Texture {
                texture,
                spectrum_type,
            } => texture.sample_spectrum(uv, *spectrum_type),
        }
    }

    pub fn sample_raw(&self, uv: Vec2) -> [f32; 3] {
        match self {
            SpectrumParameter::Constant(_spectrum) => [1.0, 0.0, 1.0],
            SpectrumParameter::Texture { texture, .. } => texture.sample(uv),
        }
    }

    /// 定数値からSpectrumParameterを作成する。
    pub fn constant(spectrum: Spectrum) -> Self {
        SpectrumParameter::Constant(spectrum)
    }

    /// テクスチャからSpectrumParameterを作成する。
    pub fn texture(texture: Arc<RgbTexture>, spectrum_type: SpectrumType) -> Self {
        SpectrumParameter::Texture {
            texture,
            spectrum_type,
        }
    }
}

/// Float パラメータ。
#[derive(Clone)]
pub enum FloatParameter {
    /// 定数値。
    Constant(f32),
    /// テクスチャから取得。
    Texture(Arc<FloatTexture>),
}

impl FloatParameter {
    /// UV座標でFloat値をサンプリングする。
    pub fn sample(&self, uv: Vec2) -> f32 {
        match self {
            FloatParameter::Constant(value) => *value,
            FloatParameter::Texture(texture) => texture.sample(uv),
        }
    }

    /// 定数値からFloatParameterを作成する。
    pub fn constant(value: f32) -> Self {
        FloatParameter::Constant(value)
    }

    /// テクスチャからFloatParameterを作成する。
    pub fn texture(texture: Arc<FloatTexture>) -> Self {
        FloatParameter::Texture(texture)
    }
}

/// Normal パラメータ。
#[derive(Clone)]
pub enum NormalParameter {
    /// ノーマルマップなし（ジオメトリノーマルを使用）。
    None,
    /// ノーマルマップから取得。
    Texture { texture: Arc<NormalTexture> },
}

impl NormalParameter {
    /// UV座標でノーマルをサンプリングする。
    /// ジオメトリノーマルとの合成は呼び出し側で行う。
    pub fn sample(&self, uv: Vec2) -> Option<Normal<ShadingTangent>> {
        match self {
            NormalParameter::None => None,
            NormalParameter::Texture { texture } => Some(texture.sample_normal(uv)),
        }
    }

    /// ノーマルマップなしのNormalParameterを作成する。
    pub fn none() -> Self {
        NormalParameter::None
    }

    /// テクスチャからNormalParameterを作成する。
    pub fn texture(texture: Arc<NormalTexture>) -> Self {
        NormalParameter::Texture { texture }
    }
}
