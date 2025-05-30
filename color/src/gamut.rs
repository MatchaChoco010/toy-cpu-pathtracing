//! 色域を表すトレイトと、いくつかの色域の実装。

/// 色域を表すトレイト。
pub trait ColorGamut: Sync + Send + Clone {
    /// 色域の構造体を作成する。
    fn new() -> Self;

    /// 色をXYZからこの色域のRGBに変換する行列を得る。
    fn xyz_to_rgb(&self) -> glam::Mat3;

    /// この色域の色をRGBからXYZに変換する行列を得る。
    fn rgb_to_xyz(&self) -> glam::Mat3;
}

fn xyy_to_xyz(xy: glam::Vec2, y: f32) -> glam::Vec3 {
    if xy.y == 0.0 {
        return glam::Vec3::ZERO;
    }
    let x = xy.x * y / xy.y;
    let z = (1.0 - xy.x - xy.y) * y / xy.y;
    glam::vec3(x, y, z)
}

pub(crate) fn xy_to_xyz(xy: glam::Vec2) -> glam::Vec3 {
    xyy_to_xyz(xy, 1.0)
}

/// RGBのxy色度とホワイトポイントのxy色度からXYZに変換する行列を計算する。
fn rgb_to_xyz(r_xy: glam::Vec2, g_xy: glam::Vec2, b_xy: glam::Vec2, w: glam::Vec2) -> glam::Mat3 {
    let r = xy_to_xyz(r_xy);
    let g = xy_to_xyz(g_xy);
    let b = xy_to_xyz(b_xy);
    let w = xy_to_xyz(w);

    let rgb = glam::Mat3::from_cols(r, g, b);

    let c = rgb.inverse() * w;
    rgb * glam::Mat3::from_diagonal(c)
}

/// sRGBの色域を表す。
#[derive(Clone)]
pub struct GamutSrgb {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutSrgb {
    /// sRGBの色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.6400, 0.3300);
        let g_xy = glam::vec2(0.3000, 0.6000);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}

/// DCI-P3 D65の色域を表す。
#[derive(Clone)]
pub struct GamutDciP3D65 {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutDciP3D65 {
    /// DCI-P3 D65の色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.6800, 0.3200);
        let g_xy = glam::vec2(0.2650, 0.6900);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}

/// Adobe RGBの色域を表す。
#[derive(Clone)]
pub struct GamutAdobeRgb {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutAdobeRgb {
    /// Adobe RGBの色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.6400, 0.3300);
        let g_xy = glam::vec2(0.2100, 0.7100);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}

/// Rec. 2020の色域を表す。
#[derive(Clone)]
pub struct GamutRec2020 {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutRec2020 {
    /// Rec. 2020の色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.7080, 0.2920);
        let g_xy = glam::vec2(0.1700, 0.7970);
        let b_xy = glam::vec2(0.1310, 0.0460);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}

/// ACEScgの色域を表す。
/// AP-1。
#[derive(Clone)]
pub struct GamutAcesCg {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutAcesCg {
    /// ACEScgの色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.7130, 0.2930);
        let g_xy = glam::vec2(0.1650, 0.8300);
        let b_xy = glam::vec2(0.1280, 0.0440);
        let w_xy = glam::vec2(0.32168, 0.33767);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}

/// ACES2065-1の色域を表す。
/// AP-0。
#[derive(Clone)]
pub struct GamutAces2065_1 {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for GamutAces2065_1 {
    /// ACES2065-1の色域を生成する。
    fn new() -> Self {
        let r_xy = glam::vec2(0.73470, 0.26530);
        let g_xy = glam::vec2(0.00000, 1.00000);
        let b_xy = glam::vec2(0.00010, -0.07700);
        let w_xy = glam::vec2(0.32168, 0.33767);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = rgb_to_xyz.inverse();
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    fn xyz_to_rgb(&self) -> glam::Mat3 {
        self.xyz_to_rgb
    }

    fn rgb_to_xyz(&self) -> glam::Mat3 {
        self.rgb_to_xyz
    }
}
