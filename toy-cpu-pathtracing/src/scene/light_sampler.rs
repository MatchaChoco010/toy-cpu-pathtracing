//! ライトサンプリングに利用する構造体などを定義するモジュール。

use crate::math::{Bounds, Render};
use crate::scene::{PrimitiveIndex, PrimitiveRepository, SceneId};
use crate::spectrum::SampledWavelengths;

/// サンプルした光源のPrimitiveIndexとサンプル確率を持つ構造体。
pub struct LightSample<Id: SceneId> {
    pub primitive_index: PrimitiveIndex<Id>,
    pub probability: f32,
}

/// 光源をサンプリングするための構造体。
#[derive(Debug)]
pub struct LightSampler<Id: SceneId> {
    primitive_index_list: Vec<PrimitiveIndex<Id>>,
    sample_weight_list: Vec<f32>,
    sample_weight_sum: f32,
    sample_table: Vec<f32>,
    initialized: bool,
}
impl<Id: SceneId> LightSampler<Id> {
    /// LightSamplerを構築する。
    /// `primitive_repository`から全てのPrimitiveIndexを取得し、ライトのpreprocessを呼び出す。
    pub fn build(
        primitive_repository: &mut PrimitiveRepository<Id>,
        scene_bounds: &Bounds<Render>,
    ) -> Self {
        let primitive_index_list = primitive_repository
            .get_all_primitive_indices()
            .collect::<Vec<_>>();
        for primitive_index in &primitive_index_list {
            let primitive = primitive_repository.get_mut(*primitive_index);
            if let Some(light) = primitive.as_light_mut() {
                light.preprocess(scene_bounds);
            }
        }
        LightSampler {
            primitive_index_list,
            sample_weight_list: vec![],
            sample_weight_sum: 0.0,
            sample_table: vec![],
            initialized: false,
        }
    }

    /// サンプルする波長を設定し、サンプリングのテーブルを構築する。
    pub fn set_sample_lambda(
        &mut self,
        primitive_repository: &PrimitiveRepository<Id>,
        lambda: &SampledWavelengths,
    ) {
        let mut weight_sum = 0.0;
        let mut sample_weight_list = vec![];
        for primitive_index in &self.primitive_index_list {
            let primitive = primitive_repository.get(*primitive_index);
            if let Some(light) = primitive.as_light() {
                let weight = light.phi(lambda);
                weight_sum += weight;
                sample_weight_list.push(weight);
            }
        }
        let mut sample_table = vec![0.0; self.primitive_index_list.len()];
        let mut cumulative_sum = 0.0;
        for i in 0..sample_table.len() {
            cumulative_sum += sample_weight_list[i];
            sample_table[i] = cumulative_sum / weight_sum;
        }
        self.sample_weight_list = sample_weight_list;
        self.sample_weight_sum = weight_sum;
        self.sample_table = sample_table;
        self.initialized = true;
    }

    /// 光源をサンプリングする。
    /// LightSamplerをcloneした後は、`set_sample_lambda()`を呼び出してから使用する必要がある。
    /// もしcloneした後に`set_sample_lambda()`を呼び出さなかった場合、panicする。
    pub fn sample_light(&self, u: f32) -> LightSample<Id> {
        if !self.initialized {
            panic!("LightSampler is not initialized. Call set_sample_lambda() first.");
        }
        for i in 0..self.sample_table.len() {
            if u < self.sample_table[i] {
                return LightSample {
                    primitive_index: self.primitive_index_list[i],
                    probability: self.sample_table[i],
                };
            }
        }
        LightSample {
            primitive_index: self.primitive_index_list[self.sample_table.len() - 1],
            probability: self.sample_table[self.sample_table.len() - 1],
        }
    }

    /// 指定したPrimitiveIndexの光源をLightSamplerが返す確率を返す。
    /// LightSamplerをcloneした後は、`set_sample_lambda()`を呼び出してから使用する必要がある。
    /// もしcloneした後に`set_sample_lambda()`を呼び出さなかった場合、panicする。
    pub fn probability(&self, primitive_index: &PrimitiveIndex<Id>) -> f32 {
        if !self.initialized {
            panic!("LightSampler is not initialized. Call set_sample_lambda() first.");
        }
        if let Some(index) = self
            .primitive_index_list
            .iter()
            .position(|id| id == primitive_index)
        {
            let weight = self.sample_weight_list[index];
            let probability = weight / self.sample_weight_sum;
            probability
        } else {
            0.0
        }
    }
}
impl<Id: SceneId> Clone for LightSampler<Id> {
    fn clone(&self) -> Self {
        Self {
            primitive_index_list: self.primitive_index_list.clone(),
            sample_weight_list: vec![],
            sample_weight_sum: 0.0,
            sample_table: vec![],
            initialized: false,
        }
    }
}
