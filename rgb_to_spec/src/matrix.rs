//! 行列の計算を定義するモジュール。
//! LU分解とその結果を使った連立方程式の解法を実装する。

/// glam::DMat3のLUP分解（LU分解+ピボット行列）を計算する関数。
pub fn lup_decompose(m: glam::DMat3, epsilon: f64) -> (glam::DMat3, glam::DMat3, [usize; 3]) {
    let mut a = m.to_cols_array_2d();
    let mut p = [0, 1, 2];

    for i in 0..3 {
        let mut max_a = 0.0;
        let mut i_max = 0;

        for k in i..3 {
            let abs_a = a[k][i].abs();
            if abs_a > max_a {
                max_a = abs_a;
                i_max = k;
            }
        }

        if max_a < epsilon {
            panic!("Matrix is singular or nearly singular: {max_a}, {m:?}");
            // panic!("Matrix is singular or nearly singular");
        }

        // 列交換
        if i_max != i {
            let tmp = p[i];
            p[i] = p[i_max];
            p[i_max] = tmp;

            for col in 0..3 {
                let tmp = a[col];
                a[col] = a[i_max];
                a[i_max] = tmp;
            }
        }

        // LとUの更新
        for j in (i + 1)..3 {
            a[j][i] /= a[i][i];
            for k in (i + 1)..3 {
                a[j][k] -= a[j][i] * a[i][k];
            }
        }
    }

    // LとUを構築
    let mut l = [[0.0; 3]; 3];
    let mut u = [[0.0; 3]; 3];
    for col in 0..3 {
        for row in 0..3 {
            if row > col {
                l[col][row] = a[col][row]; // Lの下三角
            } else if row == col {
                l[col][row] = 1.0; // Lの対角
                u[col][row] = a[col][row]; // Uの対角
            } else {
                u[col][row] = a[col][row]; // Uの上三角
            }
        }
    }

    (
        glam::DMat3::from_cols_array_2d(&l),
        glam::DMat3::from_cols_array_2d(&u),
        p,
    )
}

/// LUP分解の結果を使って連立方程式Ax = bの解xを解く関数。
pub fn lup_solve(l: glam::DMat3, u: glam::DMat3, p: [usize; 3], b: glam::DVec3) -> glam::DVec3 {
    let l = l.to_cols_array_2d();
    let u = u.to_cols_array_2d();
    let b = b.to_array();

    let mut x = [0.0; 3];
    for i in 0..3 {
        x[i] = b[p[i]];
        for k in 0..i {
            x[i] -= l[i][k] * x[k];
        }
    }
    for i in (0..3).rev() {
        for k in (i + 1)..3 {
            x[i] -= u[i][k] * x[k];
        }
        x[i] /= u[i][i];
    }

    glam::dvec3(x[0], x[1], x[2])
}
