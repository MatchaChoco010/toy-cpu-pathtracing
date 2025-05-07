/// フィルタのサンプル結果の構造体。
pub struct FilterSample {
    pub x: f32,
    pub y: f32,
    pub weight: f32,
}

/// フィルタ用のトレイト。
pub trait Filter: Send + Sync + Clone {
    fn sample(&self, uv: glam::Vec2) -> FilterSample;
}

#[derive(Debug, Clone)]
pub struct BoxFilter {
    pub width: f32,
}
impl BoxFilter {
    pub fn new(width: f32) -> Self {
        Self { width }
    }
}
impl Filter for BoxFilter {
    fn sample(&self, uv: glam::Vec2) -> FilterSample {
        let x = uv.x * self.width - self.width * 0.5;
        let y = uv.y * self.width - self.width * 0.5;
        FilterSample { x, y, weight: 1.0 }
    }
}
