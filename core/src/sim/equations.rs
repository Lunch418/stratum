//! Решатель уравнений: операторы `a = b` и списки неизвестных `? x, y`
//! всех экземпляров образуют одну систему (связанные переменные — общие
//! ячейки). Решается методом Ньютона с численным якобианом; для
//! неквадратной системы — нормальные уравнения (наименьшие квадраты).

use crate::lang::{Expr, Stmt};

/// Уравнения и неизвестные одного текста, собранные при компиляции.
#[derive(Clone, Default)]
pub struct ClassEquations {
    pub equations: Vec<(Expr, Expr)>,
    pub unknowns: Vec<String>,
}

pub fn collect(body: &[Stmt]) -> ClassEquations {
    fn walk(body: &[Stmt], out: &mut ClassEquations) {
        for st in body {
            match st {
                Stmt::Equation { left, right } => out.equations.push((left.clone(), right.clone())),
                Stmt::Unknowns(names) => out.unknowns.extend(names.iter().map(|n| n.to_lowercase())),
                Stmt::If { then_body, else_body, .. } => {
                    walk(then_body, out);
                    walk(else_body, out);
                }
                Stmt::While { body, .. } | Stmt::DoUntil { body, .. } => walk(body, out),
                Stmt::Switch { arms, default } => {
                    for a in arms {
                        walk(&a.body, out);
                    }
                    walk(default, out);
                }
                _ => {}
            }
        }
    }
    let mut out = ClassEquations::default();
    walk(body, &mut out);
    out
}

/// Решает `J·dx = -r` по методу наименьших квадратов; `jac` — m×n построчно.
pub fn solve_step(jac: &[Vec<f64>], residual: &[f64]) -> Option<Vec<f64>> {
    let m = residual.len();
    let n = jac.first().map(|r| r.len()).unwrap_or(0);
    if m == 0 || n == 0 {
        return None;
    }
    // A = Jᵀ J, b = -Jᵀ r
    let mut a = vec![vec![0.0; n + 1]; n];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = (0..m).map(|k| jac[k][i] * jac[k][j]).sum();
        }
        a[i][n] = -(0..m).map(|k| jac[k][i] * residual[k]).sum::<f64>();
        // регуляризация: вырожденные направления не двигаются
        a[i][i] += 1e-12;
    }
    for col in 0..n {
        let pivot = (col..n).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap())?;
        if a[pivot][col].abs() < 1e-18 {
            return None;
        }
        a.swap(col, pivot);
        let p = a[col][col];
        for j in col..=n {
            a[col][j] /= p;
        }
        for r in 0..n {
            if r != col {
                let f = a[r][col];
                if f != 0.0 {
                    for j in col..=n {
                        a[r][j] -= f * a[col][j];
                    }
                }
            }
        }
    }
    Some((0..n).map(|i| a[i][n]).collect())
}
