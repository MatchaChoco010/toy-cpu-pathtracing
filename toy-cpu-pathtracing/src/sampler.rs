use rand::prelude::*;

pub trait Sampler: Clone {
    fn start_pixel_sample(&mut self, dimension: u32);
    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> glam::Vec2;
    fn get_2d_pixel(&mut self) -> glam::Vec2;
}
pub trait SamplerFactory: Send + Sync + Clone {
    type Sampler: Sampler;
    fn create_sampler(&self, x: u32, y: u32) -> Self::Sampler;
}

pub struct RandomSampler {
    rng: ThreadRng,
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
    fn start_pixel_sample(&mut self, _dimension: u32) {}

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

pub struct RandomSamplerFactory;
impl RandomSamplerFactory {
    pub fn new() -> Self {
        Self
    }
}
impl Clone for RandomSamplerFactory {
    fn clone(&self) -> Self {
        Self
    }
}
impl SamplerFactory for RandomSamplerFactory {
    type Sampler = RandomSampler;
    fn create_sampler(&self, _x: u32, _y: u32) -> RandomSampler {
        RandomSampler::new()
    }
}
