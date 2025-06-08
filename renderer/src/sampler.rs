//! モンテカルロレンダリングに使う乱数を生成するサンプラーを定義するモジュール。
//! サンプラーによっては決定論的な低食い違い量列のサンプルを生成するものもある。
//! その場合は準モンテカルロ法ということになる。

mod random_sampler;
pub use random_sampler::RandomSampler;

mod z_sobol_sampler;
pub use z_sobol_sampler::ZSobolSampler;

pub mod sobol_matrices;

pub trait Sampler: Clone {
    fn new(spp: u32, resolution: glam::UVec2, seed: u32) -> Self
    where
        Self: Sized;
    fn start_pixel_sample(&mut self, p: glam::UVec2, sample_index: u32);
    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> glam::Vec2;
    fn get_2d_pixel(&mut self) -> glam::Vec2;
}
