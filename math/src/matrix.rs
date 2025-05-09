//! 行列の計算を定義するモジュール。
//! glamで定義されていない追加の計算を提供する。

/// glam::Mat3とglam::Mat3の積をコンパイル時に計算する関数。
pub const fn mat3_mul_mat3(m1: glam::Mat3, m2: glam::Mat3) -> glam::Mat3 {
    glam::Mat3::from_cols(
        glam::vec3(
            m1.x_axis.x * m2.x_axis.x + m1.y_axis.x * m2.x_axis.y + m1.z_axis.x * m2.x_axis.z,
            m1.x_axis.y * m2.x_axis.x + m1.y_axis.y * m2.x_axis.y + m1.z_axis.y * m2.x_axis.z,
            m1.x_axis.z * m2.x_axis.x + m1.y_axis.z * m2.x_axis.y + m1.z_axis.z * m2.x_axis.z,
        ),
        glam::vec3(
            m1.x_axis.x * m2.y_axis.x + m1.y_axis.x * m2.y_axis.y + m1.z_axis.x * m2.y_axis.z,
            m1.x_axis.y * m2.y_axis.x + m1.y_axis.y * m2.y_axis.y + m1.z_axis.y * m2.y_axis.z,
            m1.x_axis.z * m2.y_axis.x + m1.y_axis.z * m2.y_axis.y + m1.z_axis.z * m2.y_axis.z,
        ),
        glam::vec3(
            m1.x_axis.x * m2.z_axis.x + m1.y_axis.x * m2.z_axis.y + m1.z_axis.x * m2.z_axis.z,
            m1.x_axis.y * m2.z_axis.x + m1.y_axis.y * m2.z_axis.y + m1.z_axis.y * m2.z_axis.z,
            m1.x_axis.z * m2.z_axis.x + m1.y_axis.z * m2.z_axis.y + m1.z_axis.z * m2.z_axis.z,
        ),
    )
}

/// glam::Mat3とglam::Vec3の積をコンパイル時に計算する関数。
pub const fn mat3_mul_vec3(m: glam::Mat3, v: glam::Vec3) -> glam::Vec3 {
    glam::vec3(
        m.x_axis.x * v.x + m.y_axis.x * v.y + m.z_axis.x * v.z,
        m.x_axis.y * v.x + m.y_axis.y * v.y + m.z_axis.y * v.z,
        m.x_axis.z * v.x + m.y_axis.z * v.y + m.z_axis.z * v.z,
    )
}

/// glam::Mat3の逆行列をコンパイル時に計算する関数。
pub const fn mat3_inverse(m: glam::Mat3) -> glam::Mat3 {
    let det = m.x_axis.x * (m.y_axis.y * m.z_axis.z - m.z_axis.y * m.y_axis.z)
        - m.y_axis.x * (m.x_axis.y * m.z_axis.z - m.z_axis.y * m.x_axis.z)
        + m.z_axis.x * (m.x_axis.y * m.y_axis.z - m.y_axis.y * m.x_axis.z);
    let inv_det = 1.0 / det;

    glam::Mat3::from_cols(
        glam::vec3(
            (m.y_axis.y * m.z_axis.z - m.z_axis.y * m.y_axis.z) * inv_det,
            -(m.x_axis.y * m.z_axis.z - m.z_axis.y * m.x_axis.z) * inv_det,
            (m.x_axis.y * m.y_axis.z - m.y_axis.y * m.x_axis.z) * inv_det,
        ),
        glam::vec3(
            -(m.y_axis.x * m.z_axis.z - m.z_axis.x * m.y_axis.z) * inv_det,
            (m.x_axis.x * m.z_axis.z - m.z_axis.x * m.x_axis.z) * inv_det,
            -(m.x_axis.x * m.y_axis.z - m.y_axis.x * m.x_axis.z) * inv_det,
        ),
        glam::vec3(
            (m.y_axis.x * m.z_axis.y - m.z_axis.x * m.y_axis.y) * inv_det,
            -(m.x_axis.x * m.z_axis.y - m.z_axis.x * m.x_axis.y) * inv_det,
            (m.x_axis.x * m.y_axis.y - m.y_axis.x * m.x_axis.y) * inv_det,
        ),
    )
}

/// glam::Mat3のLUP分解（LU分解+ピボット行列）をコンパイル時に計算する関数。
pub const fn lup_decompose(m: glam::Mat3, epsilon: f32) -> (glam::Mat3, glam::Mat3, [usize; 3]) {
    let mut a = m.to_cols_array(); // [f32; 9]、列優先（column-major）
    let mut p = [0, 1, 2];
    let mut l = [0.0; 9];
    let mut u = [0.0; 9];

    let mut i = 0;
    while i < 3 {
        // ピボット選択
        let mut max_row = i;
        let mut max_val = a[i + 0 * 3].abs();

        let mut k = i + 1;
        while k < 3 {
            let v = a[i + k * 3].abs();
            if v > max_val {
                max_val = v;
                max_row = k;
            }
            k += 1;
        }

        // 特異行列の検出
        if max_val < epsilon {
            panic!("Matrix is singular or nearly singular");
        }

        // 行交換
        if max_row != i {
            let tmp = p[i];
            p[i] = p[max_row];
            p[max_row] = tmp;

            let mut j = 0;
            while j < 3 {
                let idx1 = j + i * 3;
                let idx2 = j + max_row * 3;
                let temp = a[idx1];
                a[idx1] = a[idx2];
                a[idx2] = temp;
                j += 1;
            }
        }

        // LU 分解処理
        let mut j = 0;
        while j < 3 {
            if j < i {
                l[j + i * 3] = 0.0;
            } else if j == i {
                l[j + i * 3] = 1.0;
            } else {
                l[j + i * 3] = a[i + j * 3] / a[i + i * 3];
            }
            u[i + j * 3] = if j < i { 0.0 } else { a[i + j * 3] };
            j += 1;
        }

        let mut j = i + 1;
        while j < 3 {
            let factor = a[i + j * 3] / a[i + i * 3];
            let mut k = i;
            while k < 3 {
                a[k + j * 3] -= factor * a[k + i * 3];
                k += 1;
            }
            j += 1;
        }

        i += 1;
    }

    (
        glam::Mat3::from_cols_array(&l),
        glam::Mat3::from_cols_array(&u),
        p,
    )
}

/// LUP分解の結果を使って連立方程式Ax = bの解xを解く関数。
pub const fn lup_solve(l: glam::Mat3, u: glam::Mat3, p: [usize; 3], b: glam::Vec3) -> glam::Vec3 {
    let l = l.to_cols_array();
    let u = u.to_cols_array();
    let b = b.to_array();

    // P * b の適用
    let pb0 = b[p[0]];
    let pb1 = b[p[1]];
    let pb2 = b[p[2]];

    // 前進代入：L * y = P * b
    // L は単位対角、下三角のみ意味がある
    let y0 = pb0;
    let y1 = pb1 - l[3] * y0; // l[3] = L[1][0]
    let y2 = pb2 - l[6] * y0 - l[7] * y1; // l[6] = L[2][0], l[7] = L[2][1]

    // 後退代入：U * x = y
    let x2 = y2 / u[8]; // u[8] = U[2][2]
    let x1 = (y1 - u[5] * x2) / u[4]; // u[4] = U[1][1], u[5] = U[1][2]
    let x0 = (y0 - u[1] * x1 - u[2] * x2) / u[0]; // u[0] = U[0][0], u[1]=U[0][1], u[2]=U[0][2]

    glam::vec3(x0, x1, x2)
}
