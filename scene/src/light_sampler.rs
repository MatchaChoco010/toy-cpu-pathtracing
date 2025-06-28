//! シーンのライトのサンプリングに利用する構造体などを定義するモジュール。

use math::{Bounds, Render};
use spectrum::SampledWavelengths;

use crate::{PrimitiveIndex, SceneId, primitive::PrimitiveRepository};

/// サンプルした光源のPrimitiveIndexとサンプル確率を持つ構造体。
#[derive(Debug, Clone)]
pub struct LightSample<Id: SceneId> {
    pub primitive_index: PrimitiveIndex<Id>,
    pub probability: f32,
}

/// 光源をサンプリングするための構造体。
#[derive(Debug)]
pub struct LightSampler<'a, Id: SceneId> {
    factory: &'a LightSamplerFactory<Id>,
    sample_weight_list: Vec<f32>,
    sample_weight_sum: f32,
    sample_table: Vec<f32>,
}
impl<'a, Id: SceneId> LightSampler<'a, Id> {
    /// 光源をサンプリングする。
    /// ライトがない場合やサンプルする波長でのライトの重みが0の場合はNoneを返す。
    pub fn sample_light(&self, u: f32) -> Option<LightSample<Id>> {
        if self.sample_table.is_empty() || self.sample_weight_sum == 0.0 {
            return None;
        }

        for i in 0..self.sample_table.len() {
            if u < self.sample_table[i] {
                return Some(LightSample {
                    primitive_index: self.factory.light_list[i],
                    probability: self.sample_weight_list[i] / self.sample_weight_sum,
                });
            }
        }
        Some(LightSample {
            primitive_index: self.factory.light_list[self.sample_table.len() - 1],
            probability: self.sample_weight_list[self.sample_table.len() - 1]
                / self.sample_weight_sum,
        })
    }

    /// 指定したPrimitiveIndexの光源をLightSamplerが返す確率を返す。
    pub fn probability(&self, primitive_index: &PrimitiveIndex<Id>) -> f32 {
        if self.sample_table.is_empty() || self.sample_weight_sum == 0.0 {
            return 0.0;
        }
        if let Some(index) = self
            .factory
            .light_list
            .iter()
            .position(|id| id == primitive_index)
        {
            let weight = self.sample_weight_list[index];
            weight / self.sample_weight_sum
        } else {
            0.0
        }
    }

    /// 無限光源のみをサンプリングする。
    pub fn sample_infinite_light(
        &self,
        primitive_repository: &PrimitiveRepository<Id>,
        u: f32,
    ) -> Option<LightSample<Id>> {
        if self.sample_table.is_empty() || self.sample_weight_sum == 0.0 {
            return None;
        }

        // 無限光源のみをフィルタリング
        let mut infinite_weights = vec![];
        let mut infinite_indices = vec![];
        let mut infinite_weight_sum = 0.0;

        for (i, &primitive_index) in self.factory.light_list.iter().enumerate() {
            let primitive = primitive_repository.get(primitive_index);
            if primitive.as_infinite_light().is_some() {
                infinite_weights.push(self.sample_weight_list[i]);
                infinite_indices.push(i);
                infinite_weight_sum += self.sample_weight_list[i];
            }
        }

        if infinite_weight_sum == 0.0 {
            return None;
        }

        // 累積分布を作成
        let mut cumulative = 0.0;
        for i in 0..infinite_weights.len() {
            cumulative += infinite_weights[i];
            if u < cumulative / infinite_weight_sum {
                let original_index = infinite_indices[i];
                return Some(LightSample {
                    primitive_index: self.factory.light_list[original_index],
                    probability: infinite_weights[i] / infinite_weight_sum,
                });
            }
        }

        // フォールバック
        if !infinite_indices.is_empty() {
            let last_index = infinite_indices[infinite_indices.len() - 1];
            Some(LightSample {
                primitive_index: self.factory.light_list[last_index],
                probability: infinite_weights[infinite_weights.len() - 1] / infinite_weight_sum,
            })
        } else {
            None
        }
    }

    /// 指定したPrimitiveIndexの無限光源をLightSamplerが返す確率を返す。
    pub fn probability_infinite_light(
        &self,
        primitive_repository: &PrimitiveRepository<Id>,
        primitive_index: &PrimitiveIndex<Id>,
    ) -> f32 {
        if self.sample_table.is_empty() || self.sample_weight_sum == 0.0 {
            return 0.0;
        }

        // 無限光源の重みの合計を計算
        let mut infinite_weight_sum = 0.0;
        for (i, &idx) in self.factory.light_list.iter().enumerate() {
            let primitive = primitive_repository.get(idx);
            if primitive.as_infinite_light().is_some() {
                infinite_weight_sum += self.sample_weight_list[i];
            }
        }

        if infinite_weight_sum == 0.0 {
            return 0.0;
        }

        // 指定されたプリミティブの重みを取得
        if let Some(index) = self
            .factory
            .light_list
            .iter()
            .position(|id| id == primitive_index)
        {
            let primitive = primitive_repository.get(*primitive_index);
            if primitive.as_infinite_light().is_some() {
                let weight = self.sample_weight_list[index];
                weight / infinite_weight_sum
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// サンプルする波長を設定して、LightSamplerを作成するファクトリ。
#[derive(Clone, Debug)]
pub struct LightSamplerFactory<Id: SceneId> {
    light_list: Vec<PrimitiveIndex<Id>>,
}
impl<Id: SceneId> LightSamplerFactory<Id> {
    /// LightSamplerを構築する。
    /// `primitive_repository`から全てのPrimitiveIndexを取得し、ライトのpreprocessを呼び出す。
    pub fn build(
        primitive_repository: &mut PrimitiveRepository<Id>,
        scene_bounds: &Bounds<Render>,
    ) -> Self {
        // 全てのプリミティブのインデックスを取得する。
        let primitive_index_list = primitive_repository
            .get_all_primitive_indices()
            .collect::<Vec<_>>();

        // ライトのプリミティブを取得しリストに格納しつつ、preprocessを呼び出す。
        let mut light_list = vec![];
        for primitive_index in &primitive_index_list {
            let primitive = primitive_repository.get_mut(*primitive_index);
            if let Some(light) = primitive.as_light_mut() {
                light.preprocess(scene_bounds);
                light_list.push(*primitive_index);
            }
        }
        LightSamplerFactory { light_list }
    }

    /// サンプルする波長を設定し、サンプリングのテーブルを構築してLightSamplerを作成する。
    pub fn create(
        &self,
        primitive_repository: &PrimitiveRepository<Id>,
        lambda: &SampledWavelengths,
    ) -> LightSampler<Id> {
        // ライトの選択確率のウエイトの合計値とウエイトのリストを構築する。
        let mut weight_sum = 0.0;
        let mut sample_weight_list = vec![];
        for primitive_index in &self.light_list {
            let primitive = primitive_repository.get(*primitive_index);
            let light = primitive.as_light().unwrap();
            let weight = light.phi(lambda).average();
            weight_sum += weight;
            sample_weight_list.push(weight);
        }

        // ウエイトの累積和を計算し、サンプリング用のテーブルを構築する。
        let mut sample_table = vec![0.0; sample_weight_list.len()];
        let mut cumulative_sum = 0.0;
        for i in 0..sample_table.len() {
            cumulative_sum += sample_weight_list[i];
            sample_table[i] = cumulative_sum / weight_sum;
        }

        LightSampler {
            factory: self,
            sample_weight_list,
            sample_weight_sum: weight_sum,
            sample_table,
        }
    }
}
