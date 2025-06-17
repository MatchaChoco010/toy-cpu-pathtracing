//! ガラスマテリアル実装。

use std::sync::Arc;

use math::{Normal, ShadingTangent, Transform, Vector3};
use spectrum::{SampledWavelengths, presets};

use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample, NormalParameter,
    SurfaceInteraction, SurfaceMaterial, material::bsdf::DielectricBsdf,
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
/// 完全鏡面反射・透過を行う誘電体マテリアル。
pub struct GlassMaterial {
    /// ガラスの種類
    glass_type: GlassType,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// Thin Filmフラグ
    thin_film: bool,
}

impl GlassMaterial {
    /// 新しいGlassMaterialを作成する。
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
        // ガラスの光学特性を取得
        let eta = self.get_eta(lambda);

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

        // 誘電体BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let dielectric_bsdf = DielectricBsdf::new(eta, self.thin_film);
        let bsdf_result = match dielectric_bsdf.sample(&wo_normalmap, uv, lambda) {
            Some(result) => result,
            None => {
                // BSDFサンプリング失敗の場合
                return MaterialSample::failed(normal_map);
            }
        };

        // 結果をシェーディングタンジェント空間に変換して返す
        let wi_shading = &transform_inv * &bsdf_result.wi;

        // 幾何学的制約チェック（誘電体なので反射と透過両方が可能）
        let geometry_normal = shading_point.normal;
        let wi_cos_geometric = geometry_normal.dot(wi_shading);
        let wo_cos_geometric = geometry_normal.dot(wo);

        // 反射の場合は同じ側、透過の場合は反対側
        let is_reflection = wi_cos_geometric.signum() == wo_cos_geometric.signum();
        let is_transmission = wi_cos_geometric.signum() != wo_cos_geometric.signum();

        if !(is_reflection || is_transmission) {
            return MaterialSample::failed(normal_map);
        }

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
        let eta = self.get_eta(lambda);

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
        let dielectric_bsdf = DielectricBsdf::new(eta, self.thin_film);
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
        let eta = self.get_eta(lambda);

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
        let dielectric_bsdf = DielectricBsdf::new(eta, self.thin_film);
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
