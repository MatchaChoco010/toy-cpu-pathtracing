//! 数学関連のモジュール。
//! NaNを発生させない数学関数や、
//! 座標系を区別するベクトルや点、法線、レイ、変換行列などを定義する。

mod bounds;
mod coordinate_system;
mod normal;
mod point;
mod ray;
mod safe_math;
mod transform;
mod vector;

pub use bounds::*;
pub use coordinate_system::*;
pub use normal::*;
pub use point::*;
pub use ray::*;
pub use safe_math::*;
pub use transform::*;
pub use vector::*;

/// glam::Mat3の拡張メソッド。
pub trait Mat3Extensions {
    /// Vector3を使用してlook_to_rhマトリックスを作成する。
    fn look_to_rh_from_vectors<C: CoordinateSystem>(
        direction: &Vector3<C>,
        up: &Vector3<C>,
    ) -> Self;
}

impl Mat3Extensions for glam::Mat3 {
    #[inline(always)]
    fn look_to_rh_from_vectors<C: CoordinateSystem>(
        direction: &Vector3<C>,
        up: &Vector3<C>,
    ) -> Self {
        glam::Mat3::look_to_rh(direction.to_vec3(), up.to_vec3())
    }
}
