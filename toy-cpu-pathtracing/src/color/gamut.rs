//! 色域を表すトレイトと、いくつかの色域の実装。

/// 色域を表すトレイト。
pub trait ColorGamut: Clone {
    /// 新しく色域を生成する。
    fn new() -> Self;

    /// 色域をXYZからRGBに変換する。
    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3;

    /// 色域をRGBからXYZに変換する。
    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3;
}

fn xy_to_xyz(xy: glam::Vec2) -> glam::Vec3 {
    let z = 1.0 - xy.x - xy.y;
    glam::Vec3::new(xy.x, xy.y, z)
}

/// RGBからXYZに変換する行列を計算する。
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
pub struct SrgbGamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for SrgbGamut {
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// DCI-P3 D65の色域を表す。
#[derive(Clone)]
pub struct DciP3D65Gamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for DciP3D65Gamut {
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// Adobe RGBの色域を表す。
#[derive(Clone)]
pub struct AdobeRgbGamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for AdobeRgbGamut {
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// ITU-R BT.2020の色域を表す。
#[derive(Clone)]
pub struct ItuRBt2020Gamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for ItuRBt2020Gamut {
    /// ITU-R BT.2020の色域を生成する。
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// ACEScgの色域を表す。
/// AP-1。
#[derive(Clone)]
pub struct AcesCgGamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for AcesCgGamut {
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// ACES2065-1の色域を表す。
/// AP-0。
#[derive(Clone)]
pub struct Aces2065_1Gamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl ColorGamut for Aces2065_1Gamut {
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

    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}
