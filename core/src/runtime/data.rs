//! Матрицы и динамические массивы — структуры данных языка.
//!
//! Матрицы (справка, «Матрицы»): нумеруются пользователем, положительные
//! номера постоянны, неположительный номер при создании даёт новый
//! отрицательный; индексы строк и столбцов задаются диапазонами. Все функции
//! выполняются только при `Flag > 0`.
//!
//! Динамические массивы («Динамические массивы»): создаются `new()`, элементы
//! добавляются `vInsert` с типом FLOAT/STRING/HANDLE или именем имиджа
//! (структура — поля с именами переменных).

use super::value::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct Matrix {
    pub min_i: i64,
    pub max_i: i64,
    pub min_j: i64,
    pub max_j: i64,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn new(min_i: i64, max_i: i64, min_j: i64, max_j: i64) -> Option<Matrix> {
        if max_i < min_i || max_j < min_j {
            return None;
        }
        let cells = ((max_i - min_i + 1) * (max_j - min_j + 1)) as usize;
        if cells > 4_000_000 {
            return None;
        }
        Some(Matrix { min_i, max_i, min_j, max_j, data: vec![0.0; cells] })
    }

    pub fn rows(&self) -> usize {
        (self.max_i - self.min_i + 1) as usize
    }

    pub fn cols(&self) -> usize {
        (self.max_j - self.min_j + 1) as usize
    }

    fn index(&self, i: i64, j: i64) -> Option<usize> {
        if i < self.min_i || i > self.max_i || j < self.min_j || j > self.max_j {
            return None;
        }
        Some(((i - self.min_i) as usize) * self.cols() + (j - self.min_j) as usize)
    }

    pub fn get(&self, i: i64, j: i64) -> f64 {
        self.index(i, j).map(|k| self.data[k]).unwrap_or(0.0)
    }

    pub fn set(&mut self, i: i64, j: i64, v: f64) -> bool {
        match self.index(i, j) {
            Some(k) => {
                self.data[k] = v;
                true
            }
            None => false,
        }
    }

    fn at(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols() + c]
    }

    fn at_mut(&mut self, r: usize, c: usize) -> &mut f64 {
        let cols = self.cols();
        &mut self.data[r * cols + c]
    }
}

#[derive(Debug, Default, Clone)]
pub struct Matrices {
    pub items: BTreeMap<i64, Matrix>,
    next_temp: i64,
}

impl Matrices {
    /// Создаёт матрицу; неположительный номер даёт новый временный.
    pub fn create(&mut self, q: i64, m: Matrix) -> i64 {
        let q = if q > 0 { q } else {
            self.next_temp -= 1;
            self.next_temp
        };
        self.items.insert(q, m);
        q
    }

    pub fn get(&self, q: i64) -> Option<&Matrix> {
        self.items.get(&q)
    }

    pub fn get_mut(&mut self, q: i64) -> Option<&mut Matrix> {
        self.items.get_mut(&q)
    }

    /// Кладёт результат операции в матрицу `q` (или в новую временную).
    pub fn put_result(&mut self, q: i64, m: Matrix) -> i64 {
        self.create(q, m)
    }
}

/// Округление номера строки/столбца «до ближайшего целого», как в справке.
pub fn idx(v: f64) -> i64 {
    v.round() as i64
}

/// Операция над двумя матрицами поэлементно; результат в q3.
pub fn elementwise(ms: &mut Matrices, q1: i64, q2: i64, q3: i64, op: impl Fn(f64, f64) -> f64) -> i64 {
    let (Some(a), Some(b)) = (ms.get(q1), ms.get(q2)) else { return 0 };
    if a.rows() != b.rows() || a.cols() != b.cols() {
        return 0;
    }
    let mut r = a.clone();
    for (k, v) in r.data.iter_mut().enumerate() {
        *v = op(*v, b.data[k]);
    }
    ms.put_result(q3, r)
}

pub fn multiply(ms: &mut Matrices, q1: i64, q2: i64, q3: i64) -> i64 {
    let (Some(a), Some(b)) = (ms.get(q1), ms.get(q2)) else { return 0 };
    if a.cols() != b.rows() {
        return 0;
    }
    let mut r = Matrix::new(a.min_i, a.max_i, b.min_j, b.max_j).unwrap();
    for i in 0..a.rows() {
        for j in 0..b.cols() {
            let mut s = 0.0;
            for k in 0..a.cols() {
                s += a.at(i, k) * b.at(k, j);
            }
            *r.at_mut(i, j) = s;
        }
    }
    ms.put_result(q3, r)
}

pub fn transpose(ms: &mut Matrices, q1: i64, q2: i64) -> i64 {
    let Some(a) = ms.get(q1) else { return 0 };
    let mut r = Matrix::new(a.min_j, a.max_j, a.min_i, a.max_i).unwrap();
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            *r.at_mut(j, i) = a.at(i, j);
        }
    }
    ms.put_result(q2, r)
}

/// Определитель методом Гаусса.
pub fn determinant(m: &Matrix) -> f64 {
    let n = m.rows();
    if n != m.cols() || n == 0 {
        return 0.0;
    }
    let mut a: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| m.at(i, j)).collect()).collect();
    let mut det = 1.0;
    for col in 0..n {
        let pivot = (col..n).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap()).unwrap();
        if a[pivot][col].abs() < 1e-300 {
            return 0.0;
        }
        if pivot != col {
            a.swap(pivot, col);
            det = -det;
        }
        det *= a[col][col];
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= f * a[col][k];
            }
        }
    }
    det
}

/// Обратная матрица методом Гаусса–Жордана; `None`, если вырождена.
pub fn inverse(m: &Matrix) -> Option<Matrix> {
    let n = m.rows();
    if n != m.cols() || n == 0 {
        return None;
    }
    let mut a: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row: Vec<f64> = (0..n).map(|j| m.at(i, j)).collect();
            row.extend((0..n).map(|j| if i == j { 1.0 } else { 0.0 }));
            row
        })
        .collect();
    for col in 0..n {
        let pivot = (col..n).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap())?;
        if a[pivot][col].abs() < 1e-300 {
            return None;
        }
        a.swap(pivot, col);
        let p = a[col][col];
        for k in 0..2 * n {
            a[col][k] /= p;
        }
        for row in 0..n {
            if row != col {
                let f = a[row][col];
                for k in 0..2 * n {
                    a[row][k] -= f * a[col][k];
                }
            }
        }
    }
    let mut r = m.clone();
    for i in 0..n {
        for j in 0..n {
            *r.at_mut(i, j) = a[i][n + j];
        }
    }
    Some(r)
}

/// Сортировка по строке/столбцу `n`: 1,2 — по вертикали (переставляются
/// строки по столбцу n), 3,4 — по горизонтали; чётные — по убыванию.
pub fn sort(m: &mut Matrix, n: i64, kind: i64) -> bool {
    let (rows, cols) = (m.rows(), m.cols());
    match kind {
        1 | 2 => {
            let Some(c) = (n >= m.min_j && n <= m.max_j).then(|| (n - m.min_j) as usize) else { return false };
            let mut order: Vec<usize> = (0..rows).collect();
            order.sort_by(|&a, &b| m.at(a, c).partial_cmp(&m.at(b, c)).unwrap_or(std::cmp::Ordering::Equal));
            if kind == 2 {
                order.reverse();
            }
            let old = m.data.clone();
            for (dst, src) in order.iter().enumerate() {
                m.data[dst * cols..(dst + 1) * cols].copy_from_slice(&old[src * cols..(src + 1) * cols]);
            }
        }
        3 | 4 => {
            let Some(r) = (n >= m.min_i && n <= m.max_i).then(|| (n - m.min_i) as usize) else { return false };
            let mut order: Vec<usize> = (0..cols).collect();
            order.sort_by(|&a, &b| m.at(r, a).partial_cmp(&m.at(r, b)).unwrap_or(std::cmp::Ordering::Equal));
            if kind == 4 {
                order.reverse();
            }
            let old = m.clone();
            for i in 0..rows {
                for (dst, src) in order.iter().enumerate() {
                    *m.at_mut(i, dst) = old.at(i, *src);
                }
            }
        }
        _ => return false,
    }
    true
}

/// Текстовый формат `.mat` для MSaveAs/MLoad: первая строка — границы,
/// далее строки матрицы.
pub fn to_text(m: &Matrix) -> String {
    let mut s = format!("{} {} {} {}\n", m.min_i, m.max_i, m.min_j, m.max_j);
    for i in 0..m.rows() {
        let row: Vec<String> = (0..m.cols()).map(|j| super::value::format_number(m.at(i, j))).collect();
        s.push_str(&row.join(" "));
        s.push('\n');
    }
    s
}

pub fn from_text(text: &str) -> Option<Matrix> {
    let mut lines = text.lines();
    let head: Vec<i64> = lines.next()?.split_whitespace().filter_map(|t| t.parse().ok()).collect();
    if head.len() != 4 {
        return None;
    }
    let mut m = Matrix::new(head[0], head[1], head[2], head[3])?;
    for (i, line) in lines.enumerate().take(m.rows()) {
        for (j, t) in line.split_whitespace().enumerate().take(m.cols()) {
            *m.at_mut(i, j) = t.parse().unwrap_or(0.0);
        }
    }
    Some(m)
}

/// Элемент динамического массива: тип и поля (для базовых типов — одно
/// поле с пустым именем).
#[derive(Debug, Clone, Default)]
pub struct Element {
    pub type_name: String,
    pub fields: HashMap<String, Value>,
}

impl Element {
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(&field.to_lowercase())
    }

    pub fn set(&mut self, field: &str, value: Value) {
        self.fields.insert(field.to_lowercase(), value);
    }
}

#[derive(Debug, Default, Clone)]
pub struct Arrays {
    pub items: HashMap<u32, Vec<Element>>,
    next: u32,
}

impl Arrays {
    pub fn new_array(&mut self) -> u32 {
        self.next += 1;
        self.items.insert(self.next, Vec::new());
        self.next
    }

    pub fn delete(&mut self, h: u32) -> bool {
        self.items.remove(&h).is_some()
    }

    pub fn get(&self, h: u32) -> Option<&Vec<Element>> {
        self.items.get(&h)
    }

    pub fn get_mut(&mut self, h: u32) -> Option<&mut Vec<Element>> {
        self.items.get_mut(&h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_indices_use_the_given_ranges() {
        let mut m = Matrix::new(1, 2, 1, 3).unwrap();
        assert!(m.set(2, 3, 5.0));
        assert!(!m.set(3, 1, 1.0));
        assert_eq!(m.get(2, 3), 5.0);
        assert_eq!((m.rows(), m.cols()), (2, 3));
    }

    #[test]
    fn multiply_and_invert() {
        let mut ms = Matrices::default();
        let mut a = Matrix::new(0, 1, 0, 1).unwrap();
        a.data = vec![4.0, 7.0, 2.0, 6.0];
        let q = ms.create(1, a.clone());
        let inv = inverse(&a).unwrap();
        let qi = ms.create(2, inv);
        let r = multiply(&mut ms, q, qi, 3);
        let m = ms.get(r).unwrap();
        assert!((m.get(0, 0) - 1.0).abs() < 1e-12 && m.get(0, 1).abs() < 1e-12);
        assert!((determinant(&a) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn temporary_matrices_get_negative_numbers() {
        let mut ms = Matrices::default();
        assert_eq!(ms.create(0, Matrix::new(0, 0, 0, 0).unwrap()), -1);
        assert_eq!(ms.create(-5, Matrix::new(0, 0, 0, 0).unwrap()), -2);
    }

    #[test]
    fn text_round_trip() {
        let mut m = Matrix::new(-1, 0, 2, 3).unwrap();
        m.set(0, 3, 2.5);
        let back = from_text(&to_text(&m)).unwrap();
        assert_eq!(back.get(0, 3), 2.5);
        assert_eq!((back.min_i, back.max_j), (-1, 3));
    }
}
