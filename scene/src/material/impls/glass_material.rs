//! ガラスマテリアル実装。

use std::sync::Arc;

use math::{Normal, ShadingTangent, Transform, Vector3};
use spectrum::{SampledWavelengths, presets};

use crate::{
    BsdfSurfaceMaterial, FloatParameter, Material, MaterialEvaluationResult, MaterialSample,
    NormalParameter, SurfaceInteraction, SurfaceMaterial, material::bsdf::DielectricBsdf,
};

/// ガラスの種類を表す列挙型。
#[derive(Debug, Clone, Copy)]
pub enum GlassType {
    /// BK7ガラス
    Bk7,
    /// BAF10ガラス
    Baf10,
    /// FK51Aガラス
    Fk51a,
    /// LASF9ガラス
    Lasf9,
    /// SF5ガラス
    Sf5,
    /// SF10ガラス
    Sf10,
    /// SF11ガラス
    Sf11,
}

/// ガラスマテリアル。
/// roughnessパラメータに応じて完全鏡面反射・透過またはマイクロファセット反射・透過を行う誘電体マテリアル。
pub struct GlassMaterial {
    /// ガラスの種類
    glass_type: GlassType,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// Thin Filmフラグ
    thin_film: bool,
    /// 表面の粗さパラメータ
    roughness: FloatParameter,
}

impl GlassMaterial {
    /// 新しいGlassMaterialを作成する（完全鏡面）。
    ///
    /// # Arguments
    /// - `glass_type` - ガラスの種類
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    pub fn new(glass_type: GlassType, normal: NormalParameter, thin_film: bool) -> Material {
        Arc::new(Self {
            glass_type,
            normal,
            thin_film,
            roughness: FloatParameter::constant(0.0),
        })
    }

    /// 新しいGlassMaterialを作成する（roughnessパラメータ付き）。
    ///
    /// # Arguments
    /// - `glass_type` - ガラスの種類
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    /// - `roughness` - 表面の粗さパラメータ（0.0で完全鏡面）
    pub fn new_with_roughness(
        glass_type: GlassType,
        normal: NormalParameter,
        thin_film: bool,
        roughness: FloatParameter,
    ) -> Material {
        Arc::new(Self {
            glass_type,
            normal,
            thin_film,
            roughness,
        })
    }

    /// ガラスの屈折率を取得する。
    fn get_eta(&self, lambda: &SampledWavelengths) -> spectrum::SampledSpectrum {
        let spectrum = match self.glass_type {
            GlassType::Bk7 => presets::glass_bk7_eta(),
            GlassType::Baf10 => presets::glass_baf10_eta(),
            GlassType::Fk51a => presets::glass_fk51a_eta(),
            GlassType::Lasf9 => presets::glass_lasf9_eta(),
            GlassType::Sf5 => presets::glass_sf5_eta(),
            GlassType::Sf10 => presets::glass_sf10_eta(),
            GlassType::Sf11 => presets::glass_sf11_eta(),
        };
        spectrum.sample(lambda)
    }
}
impl SurfaceMaterial for GlassMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for GlassMaterial {
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &mut SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<ShadingTangent>,
    ) -> MaterialSample {
        let eta = self.get_eta(lambda);
        // 屈折率が波長依存の場合は最初の波長以外を打ち切る
        if !eta.is_constant() {
            lambda.terminate_secondary();
        }

        // ガラスの光学特性を取得
        let eta = eta.value(0); // 単一波長での屈折率を使用
        let eta = if eta == 0.0 {
            // 屈折率が0の場合は無効な値なので、デフォルトの1.0を使用
            1.0
        } else {
            eta
        };

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);
        let transform_inv = transform.inverse();

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;

        // roughnessパラメータをサンプリング
        let roughness_value = self.roughness.sample(shading_point.uv);

        // 誘電体BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf =
            DielectricBsdf::new_with_roughness(eta, entering, self.thin_film, roughness_value);
        // ucとして追加のランダム値を生成（uvから派生）
        let uc = (uv.x * 73.0 + uv.y * 37.0).fract();
        let bsdf_result = match dielectric_bsdf.sample(&wo_normalmap, uv, uc) {
            Some(result) => result,
            None => {
                // BSDFサンプリング失敗の場合
                return MaterialSample::failed(normal_map);
            }
        };

        // 結果をシェーディングタンジェント空間に変換して返す
        let wi_shading = &transform_inv * &bsdf_result.wi;

        MaterialSample::new(
            bsdf_result.f,
            wi_shading,
            bsdf_result.pdf,
            bsdf_result.sample_type,
            normal_map,
        )
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<ShadingTangent>,
    ) -> MaterialEvaluationResult {
        // ガラスの光学特性を取得
        let eta = self.get_eta(lambda).value(0); // 単一波長での屈折率を使用
        let eta = if eta == 0.0 {
            // 屈折率が0の場合は無効な値なので、デフォルトの1.0を使用
            1.0
        } else {
            eta
        };

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // 誘電体BSDF評価（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(eta, entering, self.thin_film);
        let f = dielectric_bsdf.evaluate(&wo_normalmap, &wi_normalmap);

        MaterialEvaluationResult {
            f,
            pdf: 1.0, // 単一BSDFなので選択確率は1.0
            normal: normal_map,
        }
    }

    fn pdf(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<ShadingTangent>,
        wi: &Vector3<ShadingTangent>,
        shading_point: &SurfaceInteraction<ShadingTangent>,
    ) -> f32 {
        // ガラスの光学特性を取得
        let eta = self.get_eta(lambda).value(0); // 単一波長での屈折率を使用
        let eta = if eta == 0.0 {
            // 屈折率が0の場合は無効な値なので、デフォルトの1.0を使用
            1.0
        } else {
            eta
        };

        // 法線マップから法線を取得（ない場合はデフォルトのZ+法線）
        let normal_map = self
            .normal
            .sample(shading_point.uv)
            .unwrap_or_else(|| Normal::new(0.0, 0.0, 1.0));

        // シェーディングタンジェント空間からノーマルマップタンジェント空間への変換
        let transform = Transform::from_normal_map(&normal_map);

        // ベクトルをノーマルマップタンジェント空間に変換
        let wo_normalmap = &transform * wo;
        let wi_normalmap = &transform * wi;

        // 誘電体BSDF PDF計算（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(eta, entering, self.thin_film);
        dielectric_bsdf.pdf(&wo_normalmap, &wi_normalmap)
    }

    fn sample_albedo_spectrum(
        &self,
        _uv: glam::Vec2,
        _lambda: &SampledWavelengths,
    ) -> spectrum::SampledSpectrum {
        // ガラスの場合、アルベドは1.0（損失なし）
        spectrum::SampledSpectrum::constant(1.0)
    }
}
