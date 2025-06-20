//! プラスチックマテリアル実装。

use std::sync::Arc;

use math::{Normal, ShadingTangent, Transform, Vector3};
use spectrum::SampledWavelengths;

use crate::{
    BsdfSurfaceMaterial, Material, MaterialEvaluationResult, MaterialSample, NormalParameter,
    SurfaceInteraction, SurfaceMaterial, material::bsdf::DielectricBsdf,
};

/// プラスチックマテリアル。
/// 定数屈折率の誘電体マテリアル。
pub struct PlasticMaterial {
    /// 屈折率（定数値）
    eta: f32,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// Thin Filmフラグ
    thin_film: bool,
}
impl PlasticMaterial {
    /// 新しいPlasticMaterialを作成する。
    ///
    /// # Arguments
    /// - `eta` - 屈折率（定数値）
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    pub fn new(eta: f32, normal: NormalParameter, thin_film: bool) -> Material {
        Arc::new(Self {
            eta,
            normal,
            thin_film,
        })
    }

    /// 一般的なプラスチック用のPlasticMaterialを作成する（屈折率 1.5）。
    ///
    /// # Arguments
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    pub fn new_generic(normal: NormalParameter, thin_film: bool) -> Material {
        Self::new(1.5, normal, thin_film)
    }

    /// アクリル用のPlasticMaterialを作成する（屈折率 1.49）。
    ///
    /// # Arguments
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    pub fn new_acrylic(normal: NormalParameter, thin_film: bool) -> Material {
        Self::new(1.49, normal, thin_film)
    }

    /// ポリカーボネート用のPlasticMaterialを作成する（屈折率 1.58）。
    ///
    /// # Arguments
    /// - `normal` - ノーマルマップパラメータ
    /// - `thin_film` - Thin Filmフラグ
    pub fn new_polycarbonate(normal: NormalParameter, thin_film: bool) -> Material {
        Self::new(1.58, normal, thin_film)
    }

    /// 定数屈折率をスペクトラムに変換する。
    fn get_eta(&self, _lambda: &SampledWavelengths) -> spectrum::SampledSpectrum {
        spectrum::SampledSpectrum::constant(self.eta)
    }
}

impl SurfaceMaterial for PlasticMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_bsdf_material(&self) -> Option<&dyn BsdfSurfaceMaterial> {
        Some(self)
    }
}
impl BsdfSurfaceMaterial for PlasticMaterial {
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

        // プラスチックの光学特性を取得
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

        // 誘電体BSDFサンプリング（ノーマルマップタンジェント空間で実行）
        let entering = shading_point.normal.dot(wo) > 0.0;
        let dielectric_bsdf = DielectricBsdf::new(eta, entering, self.thin_film);
        let bsdf_result = match dielectric_bsdf.sample(&wo_normalmap, uv) {
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
        // プラスチックの光学特性を取得
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
        // プラスチックの光学特性を取得
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
        // プラスチックの場合、アルベドは1.0（損失なし）
        spectrum::SampledSpectrum::constant(1.0)
    }
}
