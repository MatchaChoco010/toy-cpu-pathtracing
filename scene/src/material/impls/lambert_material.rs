//! 拡散反射（Lambert）マテリアル実装。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths, Spectrum};

use crate::{
    BsdfSample, BsdfSurfaceMaterial, Material, NormalizedLambertBsdf, SceneId, SurfaceInteraction,
    SurfaceMaterial,
};

/// 拡散反射のみを行うLambertマテリアル。
/// 単一のRGBパラメータから反射率を設定する。
pub struct LambertMaterial {
    /// 反射率スペクトル
    albedo: Spectrum,
    /// 内部でBSDF計算を行う構造体
    bsdf: NormalizedLambertBsdf,
}
impl LambertMaterial {
    /// 新しいLambertMaterialを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率スペクトル
    pub fn new(albedo: Spectrum) -> Material {
        std::sync::Arc::new(Self {
            albedo,
            bsdf: NormalizedLambertBsdf::new(),
        })
    }
}
impl SurfaceMaterial for LambertMaterial {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl<Id: SceneId> BsdfSurfaceMaterial<Id> for LambertMaterial {
    fn sample(
        &self,
        uv: glam::Vec2,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        _shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample> {
        // テクスチャ座標uvは将来のテクスチャ実装のため保持
        let albedo = self.albedo.sample(lambda);
        self.bsdf.sample(&albedo, wo, uv)
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        _shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> SampledSpectrum {
        let albedo = self.albedo.sample(lambda);
        self.bsdf.evaluate(&albedo, wo, wi)
    }

    fn pdf(
        &self,
        _lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        _shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> f32 {
        self.bsdf.pdf(wo, wi)
    }
}
