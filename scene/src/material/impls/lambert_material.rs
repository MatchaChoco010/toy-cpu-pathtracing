//! 拡散反射（Lambert）マテリアル実装。

use math::{Tangent, Vector3};
use spectrum::{SampledSpectrum, SampledWavelengths};

use crate::{
    BsdfSample, BsdfSurfaceMaterial, Material, NormalParameter, NormalizedLambertBsdf, SceneId, SpectrumParameter, SurfaceInteraction,
    SurfaceMaterial,
};

/// 拡散反射のみを行うLambertマテリアル。
/// テクスチャ対応の反射率とノーマルマップパラメータを持つ。
pub struct LambertMaterial {
    /// 反射率パラメータ
    albedo: SpectrumParameter,
    /// ノーマルマップパラメータ
    normal: NormalParameter,
    /// 内部でBSDF計算を行う構造体
    bsdf: NormalizedLambertBsdf,
}
impl LambertMaterial {
    /// 新しいLambertMaterialを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率パラメータ
    /// - `normal` - ノーマルマップパラメータ
    pub fn new(albedo: SpectrumParameter, normal: NormalParameter) -> Material {
        std::sync::Arc::new(Self {
            albedo,
            normal,
            bsdf: NormalizedLambertBsdf::new(),
        })
    }

    /// ノーマルマップなしのLambertMaterialを作成する。
    ///
    /// # Arguments
    /// - `albedo` - 反射率パラメータ
    pub fn new_simple(albedo: SpectrumParameter) -> Material {
        Self::new(albedo, NormalParameter::none())
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
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> Option<BsdfSample> {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);
        
        // TODO: ノーマルマップがある場合の処理を追加
        // 現在はLambertBSDFはノーマル変更に対応していないため、
        // 将来的に接空間の変換を実装する必要がある
        
        self.bsdf.sample(&albedo, wo, uv)
    }

    fn evaluate(
        &self,
        lambda: &SampledWavelengths,
        wo: &Vector3<Tangent>,
        wi: &Vector3<Tangent>,
        shading_point: &SurfaceInteraction<Id, Tangent>,
    ) -> SampledSpectrum {
        let albedo = self.albedo.sample(shading_point.uv).sample(lambda);
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
