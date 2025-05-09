//! 色域を表すトレイトと、いくつかの色域の実装。

use math::{cube_root, mat3_inverse, mat3_mul_mat3, mat3_mul_vec3};

/// 色域を表すトレイト。
pub trait ColorGamut: Sync + Send + Clone {
    /// 色域をXYZからRGBに変換する。
    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3;

    /// 色域をRGBからXYZに変換する。
    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3;
}

const fn xyy_to_xyz(xy: glam::Vec2, y: f32) -> glam::Vec3 {
    if xy.y == 0.0 {
        return glam::Vec3::ZERO;
    }
    let x = xy.x * y;
    let z = (1.0 - xy.x - xy.y) * y / xy.y;
    glam::Vec3::new(x, y, z)
}

const fn xy_to_xyz(xy: glam::Vec2) -> glam::Vec3 {
    xyy_to_xyz(xy, 1.0)
}

/// RGBからXYZに変換する行列を計算する。
const fn rgb_to_xyz(
    r_xy: glam::Vec2,
    g_xy: glam::Vec2,
    b_xy: glam::Vec2,
    w: glam::Vec2,
) -> glam::Mat3 {
    let r = xy_to_xyz(r_xy);
    let g = xy_to_xyz(g_xy);
    let b = xy_to_xyz(b_xy);
    let w = xy_to_xyz(w);

    let rgb = glam::Mat3::from_cols(r, g, b);

    let c = mat3_mul_vec3(mat3_inverse(rgb), w);

    mat3_mul_mat3(rgb, glam::Mat3::from_diagonal(c))
}

/// XYZからLabに変換する関数。
pub const fn xyz_to_lab(xyz: glam::Vec3) -> glam::Vec3 {
    const fn f(t: f32) -> f32 {
        if t > 0.008856 {
            cube_root(t)
        } else {
            (t * 903.3 + 16.0) / 116.0
        }
    }
    let w = glam::vec2(0.34567, 0.35850);
    let w_xyz = xy_to_xyz(w);

    let xr = xyz.x / w_xyz.x;
    let yr = xyz.y / w_xyz.y;
    let zr = xyz.z / w_xyz.z;

    let l = 116.0 * f(yr) - 16.0;
    let a = 500.0 * (f(xr) - f(yr));
    let b = 200.0 * (f(yr) - f(zr));

    glam::vec3(l, a, b)
}

/// RGBからCIE Labに変換する関数。
const fn lab(rgb: glam::Vec3, rgb_to_xyz: glam::Mat3) -> glam::Vec3 {
    const fn f(t: f32) -> f32 {
        if t > 0.008856 {
            cube_root(t)
        } else {
            (903.3 * t + 16.0) / 116.0
        }
    }
    // RGBをXYZに変換する。
    let xyz = mat3_mul_vec3(rgb_to_xyz, rgb);

    // D50白色点のXYZ値を取得する。
    let w = glam::vec2(0.34567, 0.35850);
    let w_xyz = xy_to_xyz(w);

    // Xr, Yr, Zrを計算する。
    let xr = xyz.x / w_xyz.x;
    let yr = xyz.y / w_xyz.y;
    let zr = xyz.z / w_xyz.z;

    // Xr, Yr, Zrを使ってL*, a*, b*を計算する。
    let l = 116.0 * f(yr) - 16.0;
    let a = 500.0 * (f(xr) - f(yr));
    let b = 200.0 * (f(yr) - f(zr));

    glam::vec3(l, a, b)
}

/// sRGBの色域を表す。
#[derive(Clone)]
pub struct SrgbGamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl SrgbGamut {
    /// sRGBの色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.6400, 0.3300);
        let g_xy = glam::vec2(0.3000, 0.6000);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// sRGBの色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for SrgbGamut {
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
impl DciP3D65Gamut {
    /// DCI-P3 D65の色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.6800, 0.3200);
        let g_xy = glam::vec2(0.2650, 0.6900);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// DCI-P3 D65の色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for DciP3D65Gamut {
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
impl AdobeRgbGamut {
    /// Adobe RGBの色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.6400, 0.3300);
        let g_xy = glam::vec2(0.2100, 0.7100);
        let b_xy = glam::vec2(0.1500, 0.0600);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// Adobe RGBの色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for AdobeRgbGamut {
    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}

/// Rec. 2020の色域を表す。
#[derive(Clone)]
pub struct Rec2020Gamut {
    xyz_to_rgb: glam::Mat3,
    rgb_to_xyz: glam::Mat3,
}
impl Rec2020Gamut {
    /// Rec. 2020の色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.7080, 0.2920);
        let g_xy = glam::vec2(0.1700, 0.7970);
        let b_xy = glam::vec2(0.1310, 0.0460);
        let w_xy = glam::vec2(0.3127, 0.3290);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// Rec. 2020の色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for Rec2020Gamut {
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
impl AcesCgGamut {
    /// ACEScgの色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.7130, 0.2930);
        let g_xy = glam::vec2(0.1650, 0.8300);
        let b_xy = glam::vec2(0.1280, 0.0440);
        let w_xy = glam::vec2(0.32168, 0.33767);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// ACEScgの色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for AcesCgGamut {
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
impl Aces2065_1Gamut {
    /// ACES2065-1の色域を生成する。
    pub const fn new() -> Self {
        let r_xy = glam::vec2(0.73470, 0.26530);
        let g_xy = glam::vec2(0.00000, 1.00000);
        let b_xy = glam::vec2(0.00010, -0.07700);
        let w_xy = glam::vec2(0.32168, 0.33767);
        let rgb_to_xyz = rgb_to_xyz(r_xy, g_xy, b_xy, w_xy);
        let xyz_to_rgb = mat3_inverse(rgb_to_xyz);
        Self {
            xyz_to_rgb,
            rgb_to_xyz,
        }
    }

    /// ACES2065-1の色をCIE Labに変換する。
    pub const fn rgb_to_lab(&self, rgb: glam::Vec3) -> glam::Vec3 {
        lab(rgb, self.rgb_to_xyz)
    }
}
impl ColorGamut for Aces2065_1Gamut {
    fn xyz_to_rgb(&self, xyz: glam::Vec3) -> glam::Vec3 {
        self.xyz_to_rgb * xyz
    }

    fn rgb_to_xyz(&self, rgb: glam::Vec3) -> glam::Vec3 {
        self.rgb_to_xyz * rgb
    }
}
