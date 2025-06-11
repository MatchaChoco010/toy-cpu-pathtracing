use rand::prelude::*;

use crate::sampler::Sampler;

pub struct RandomSampler {
    rng: ThreadRng,
}
impl Default for RandomSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSampler {
    pub fn new() -> Self {
        Self { rng: rand::rng() }
    }
}
impl Clone for RandomSampler {
    fn clone(&self) -> Self {
        Self { rng: rand::rng() }
    }
}
impl Sampler for RandomSampler {
    fn new(_spp: u32, _resolution: glam::UVec2, _seed: u32) -> Self {
        Self::new()
    }

    fn start_pixel_sample(&mut self, _p: glam::UVec2, _sample_index: u32) {}

    fn get_1d(&mut self) -> f32 {
        self.rng.random()
    }

    fn get_2d(&mut self) -> glam::Vec2 {
        let (u, v) = self.rng.random();
        glam::vec2(u, v)
    }

    fn get_2d_pixel(&mut self) -> glam::Vec2 {
        self.get_2d()
    }
}
