//! 行列の計算を定義するモジュール。
//! glamで定義されていない追加の計算を提供する。

/// glam::Mat3のLUP分解（LU分解+ピボット行列）を計算する関数。
pub fn lup_decompose(m: glam::Mat3, epsilon: f32) -> (glam::Mat3, glam::Mat3, [usize; 3]) {
    let mut a = m.to_cols_array(); // [f32; 9]、列優先（column-major）
    let mut p = [0, 1, 2];

    for i in 0..3 {
        // ピボット選択
        let mut max_row = i;
        let mut max_val = a[i + 0 * 3].abs();

        for k in 0..3 {
            let v = a[i + k * 3].abs();
            if v > max_val {
                max_val = v;
                max_row = k;
            }
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

            for j in 0..3 {
                let idx1 = j + i * 3;
                let idx2 = j + max_row * 3;
                let temp = a[idx1];
                a[idx1] = a[idx2];
                a[idx2] = temp;
            }
        }

        // LU更新（U上三角とL下三角をa内に同時格納）
        for j in (i + 1)..3 {
            let factor = a[i + j * 3] / a[i + i * 3];
            for k in i..3 {
                a[k + j * 3] -= factor * a[k + i * 3]; // Uの該当要素（上三角）として格納
            }
            a[i + j * 3] = factor; // Lの該当要素（下三角）として格納
        }
    }

    // LとUを構築
    let mut l = [0.0; 9];
    let mut u = [0.0; 9];
    for col in 0..3 {
        for row in 0..3 {
            if row > col {
                l[col * 3 + row] = a[col * 3 + row]; // Lの下三角
            } else if row == col {
                l[col * 3 + row] = 1.0; // Lの対角
                u[col * 3 + row] = a[col * 3 + row]; // Uの対角
            } else {
                u[col * 3 + row] = a[col * 3 + row]; // Uの上三角
            }
        }
    }

    (
        glam::Mat3::from_cols_array(&l),
        glam::Mat3::from_cols_array(&u),
        p,
    )
}

/// LUP分解の結果を使って連立方程式Ax = bの解xを解く関数。
pub fn lup_solve(l: glam::Mat3, u: glam::Mat3, p: [usize; 3], b: glam::Vec3) -> glam::Vec3 {
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
