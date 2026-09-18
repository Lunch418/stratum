//! Трёхмерная графика: пространства, объекты из точек и примитивов,
//! камеры, стек систем координат, тела-генераторы и проекция в SVG.
//!
//! Соглашения как у оригинала: матрицы 4×4 применяются к вектору-строке
//! (`p' = p · M`, перенос в четвёртой строке); ось поворота задаётся
//! матрицей 2×3 (точка и направление); цвета — COLORREF.

use super::svg::color;
use super::Handle;
use std::collections::BTreeMap;
use std::fmt::Write;

pub type Vec3 = [f64; 3];
pub type Mat4 = [[f64; 4]; 4];

pub const IDENTITY: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// Флаги примитивов (`AddPrimitive3d`).
pub const PRIM_POLYGON: u32 = 0x08;
pub const PRIM_POLYLINE: u32 = 0x10;

#[derive(Debug, Clone)]
pub struct Prim {
    pub flags: u32,
    pub color: u32,
    pub idx: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Object3d {
    pub handle: Handle,
    pub name: String,
    pub points: Vec<Vec3>,
    pub prims: Vec<Prim>,
    /// Локальные координаты → мировые.
    pub matrix: Mat4,
    pub color: u32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub handle: Handle,
    pub name: String,
    pub pos: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Половина ширины кадра в мировых единицах на расстоянии цели;
    /// 0 — подобрать по габариту сцены при рисовании.
    pub extent: f64,
    /// > 0 — перспектива (условное фокусное расстояние), 0 — ортогональная.
    pub focus: f64,
    pub background: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct Space3d {
    pub handle: Handle,
    /// Двумерное пространство, в котором создаются проекции.
    pub owner: Handle,
    pub objects: BTreeMap<Handle, Object3d>,
    pub cameras: BTreeMap<Handle, Camera>,
    next: Handle,
    /// Текущая система координат (локальная → мировая) и стек.
    pub crd: Mat4,
    pub stack: Vec<Mat4>,
    /// Материалы: дескриптор → (имя, основной цвет).
    pub materials: BTreeMap<Handle, (String, u32)>,
}

// ── векторная арифметика ──────────────────────────────────────────────

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn scale(a: Vec3, k: f64) -> Vec3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn norm(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l < 1e-12 { [0.0, 0.0, 0.0] } else { scale(a, 1.0 / l) }
}

pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}

/// Точка (строка) на матрицу.
pub fn apply(p: Vec3, m: &Mat4) -> Vec3 {
    let w = p[0] * m[0][3] + p[1] * m[1][3] + p[2] * m[2][3] + m[3][3];
    let w = if w.abs() < 1e-12 { 1.0 } else { w };
    [
        (p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0] + m[3][0]) / w,
        (p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1]) / w,
        (p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2] + m[3][2]) / w,
    ]
}

/// Направление (без переноса).
pub fn apply_dir(p: Vec3, m: &Mat4) -> Vec3 {
    [
        p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0],
        p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1],
        p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2],
    ]
}

pub fn translation(t: Vec3) -> Mat4 {
    let mut m = IDENTITY;
    m[3] = [t[0], t[1], t[2], 1.0];
    m
}

/// Поворот вокруг оси через точку `p` с направлением `d` на угол `a`.
pub fn rotation(p: Vec3, d: Vec3, a: f64) -> Mat4 {
    let d = norm(d);
    let (s, c) = a.sin_cos();
    let t = 1.0 - c;
    let (x, y, z) = (d[0], d[1], d[2]);
    let r: Mat4 = [
        [t * x * x + c, t * x * y + s * z, t * x * z - s * y, 0.0],
        [t * x * y - s * z, t * y * y + c, t * y * z + s * x, 0.0],
        [t * x * z + s * y, t * y * z - s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    mul(&mul(&translation(scale(p, -1.0)), &r), &translation(p))
}

/// Обратная матрица (Гаусс–Жордан); для вырожденной — единичная.
pub fn inverse(m: &Mat4) -> Mat4 {
    let mut a = *m;
    let mut inv = IDENTITY;
    for col in 0..4 {
        let mut pivot = col;
        for r in col + 1..4 {
            if a[r][col].abs() > a[pivot][col].abs() {
                pivot = r;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return IDENTITY;
        }
        a.swap(col, pivot);
        inv.swap(col, pivot);
        let p = a[col][col];
        for j in 0..4 {
            a[col][j] /= p;
            inv[col][j] /= p;
        }
        for r in 0..4 {
            if r != col {
                let f = a[r][col];
                for j in 0..4 {
                    a[r][j] -= f * a[col][j];
                    inv[r][j] -= f * inv[col][j];
                }
            }
        }
    }
    inv
}

// ── пространство ──────────────────────────────────────────────────────

impl Space3d {
    pub fn new(handle: Handle, owner: Handle) -> Self {
        Space3d { handle, owner, objects: BTreeMap::new(), cameras: BTreeMap::new(), next: 1, crd: IDENTITY, stack: Vec::new(), materials: BTreeMap::new() }
    }

    fn alloc(&mut self) -> Handle {
        let h = self.next;
        self.next += 1;
        h
    }

    pub fn alloc_public(&mut self) -> Handle {
        self.alloc()
    }

    pub fn add_object(&mut self, mut o: Object3d) -> Handle {
        let h = self.alloc();
        o.handle = h;
        self.objects.insert(h, o);
        h
    }

    pub fn add_camera(&mut self, mut c: Camera) -> Handle {
        let h = self.alloc();
        c.handle = h;
        self.cameras.insert(h, c);
        h
    }

    pub fn find_by_name(&self, name: &str) -> Option<Handle> {
        self.objects
            .values()
            .find(|o| o.name.eq_ignore_ascii_case(name))
            .map(|o| o.handle)
            .or_else(|| self.cameras.values().find(|c| c.name.eq_ignore_ascii_case(name)).map(|c| c.handle))
    }

    pub fn set_name(&mut self, h: Handle, name: &str) -> bool {
        if let Some(o) = self.objects.get_mut(&h) {
            o.name = name.to_string();
            return true;
        }
        if let Some(c) = self.cameras.get_mut(&h) {
            c.name = name.to_string();
            return true;
        }
        false
    }

    /// Преобразование `t`, заданное в текущей системе координат, применяется к объекту.
    pub fn transform_object(&mut self, h: Handle, t: &Mat4) -> bool {
        let crd = self.crd;
        let Some(o) = self.objects.get_mut(&h) else { return false };
        let world_t = mul(&mul(&inverse(&crd), t), &crd);
        o.matrix = mul(&o.matrix, &world_t);
        true
    }

    /// Начало координат объекта — в точку (x, y, z) текущей системы.
    pub fn set_base(&mut self, h: Handle, p: Vec3) -> bool {
        let world = apply(p, &self.crd);
        let Some(o) = self.objects.get_mut(&h) else { return false };
        o.matrix[3] = [world[0], world[1], world[2], 1.0];
        true
    }

    pub fn base(&self, h: Handle) -> Option<Vec3> {
        let o = self.objects.get(&h)?;
        let world = [o.matrix[3][0], o.matrix[3][1], o.matrix[3][2]];
        Some(apply(world, &inverse(&self.crd)))
    }

    /// Мировые точки объекта.
    pub fn world_points(&self, o: &Object3d) -> Vec<Vec3> {
        o.points.iter().map(|p| apply(*p, &o.matrix)).collect()
    }

    /// Габарит всех объектов в мировых координатах: центр и радиус.
    pub fn bounds(&self) -> Option<(Vec3, f64)> {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let mut any = false;
        for o in self.objects.values() {
            for p in self.world_points(o) {
                any = true;
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
        if !any {
            return None;
        }
        let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
        let r = dot(sub(hi, lo), sub(hi, lo)).sqrt() / 2.0;
        Some((c, r.max(1e-6)))
    }
}

impl Camera {
    pub fn looking_from(pos: Vec3) -> Camera {
        let up = if pos[0].abs() < 1e-9 && pos[1].abs() < 1e-9 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
        Camera { handle: 0, name: String::new(), pos, target: [0.0; 3], up, extent: 0.0, focus: 0.0, background: 0xFFFFFF, flags: 0 }
    }

    /// Оси камеры: вправо, вверх, вперёд.
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let f = norm(sub(self.target, self.pos));
        let mut r = cross(f, self.up);
        if dot(r, r) < 1e-12 {
            r = cross(f, [0.0, 1.0, 0.0]);
        }
        let r = norm(r);
        let u = cross(r, f);
        (r, u, f)
    }

    /// Точка в координатах камеры: (x вправо, y вверх, z вглубь).
    pub fn to_view(&self, p: Vec3) -> Vec3 {
        let (r, u, f) = self.basis();
        let d = sub(p, self.pos);
        [dot(d, r), dot(d, u), dot(d, f)]
    }

    pub fn distance(&self) -> f64 {
        dot(sub(self.target, self.pos), sub(self.target, self.pos)).sqrt().max(1e-6)
    }

    /// Экранные координаты внутри кадра `w × h` (начало — левый верх).
    pub fn project(&self, v: Vec3, w: f64, h: f64) -> Option<(f64, f64)> {
        let half = (w.min(h) / 2.0).max(1.0);
        let k = half / self.extent.max(1e-9);
        let (x, y) = if self.focus > 0.0 {
            let z = v[2];
            if z <= 1e-6 {
                return None;
            }
            let d = self.distance();
            (v[0] * k * d / z, v[1] * k * d / z)
        } else {
            (v[0] * k, v[1] * k)
        };
        Some((w / 2.0 + x, h / 2.0 - y))
    }

    /// Поворот камеры вокруг цели относительно её же осей (1 — X экрана,
    /// 2 — Y экрана, 3 — ось взгляда).
    pub fn orbit(&mut self, mode: u32, angle: f64) {
        let (r, u, f) = self.basis();
        let axis = match mode {
            1 => r,
            2 => u,
            _ => f,
        };
        let m = rotation(self.target, axis, angle);
        self.pos = apply(self.pos, &m);
        self.up = norm(apply_dir(self.up, &m));
    }

    pub fn pan(&mut self, dx_px: f64, dy_px: f64, w: f64, h: f64) {
        let half = (w.min(h) / 2.0).max(1.0);
        let per_px = self.extent / half;
        let (r, u, _) = self.basis();
        let shift = add(scale(r, dx_px * per_px), scale(u, dy_px * per_px));
        self.pos = add(self.pos, shift);
        self.target = add(self.target, shift);
    }
}

// ── тела ──────────────────────────────────────────────────────────────

fn quad(o: &mut Object3d, color: u32, a: usize, b: usize, c: usize, d: usize) {
    o.prims.push(Prim { flags: PRIM_POLYGON, color, idx: vec![a, b, c, d] });
}

pub fn empty(color: u32) -> Object3d {
    Object3d { handle: 0, name: String::new(), points: Vec::new(), prims: Vec::new(), matrix: IDENTITY, color, visible: true }
}

/// Параллелепипед с центром в начале координат.
pub fn make_bar(color: u32, sx: f64, sy: f64, sz: f64) -> Object3d {
    let mut o = empty(color);
    let (x, y, z) = (sx / 2.0, sy / 2.0, sz / 2.0);
    o.points = vec![[-x, -y, -z], [x, -y, -z], [x, y, -z], [-x, y, -z], [-x, -y, z], [x, -y, z], [x, y, z], [-x, y, z]];
    quad(&mut o, color, 0, 3, 2, 1);
    quad(&mut o, color, 4, 5, 6, 7);
    quad(&mut o, color, 0, 1, 5, 4);
    quad(&mut o, color, 1, 2, 6, 5);
    quad(&mut o, color, 2, 3, 7, 6);
    quad(&mut o, color, 3, 0, 4, 7);
    o
}

/// Тело вращения вокруг оси Z: профиль (r, z) по кольцам и `seg1`
/// сегментам по окружности; при `closed` торцы закрываются.
fn lathe(color: u32, profile: &[(f64, f64)], seg1: usize, closed: bool) -> Object3d {
    let mut o = empty(color);
    let seg1 = seg1.max(3);
    for &(r, z) in profile {
        for i in 0..seg1 {
            let a = i as f64 / seg1 as f64 * std::f64::consts::TAU;
            o.points.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for ring in 0..profile.len().saturating_sub(1) {
        for i in 0..seg1 {
            let j = (i + 1) % seg1;
            let (a, b) = (ring * seg1 + i, ring * seg1 + j);
            let (c, d) = ((ring + 1) * seg1 + j, (ring + 1) * seg1 + i);
            quad(&mut o, color, a, b, c, d);
        }
    }
    if closed && profile.len() >= 2 {
        if profile[0].0 > 1e-9 {
            o.prims.push(Prim { flags: PRIM_POLYGON, color, idx: (0..seg1).rev().collect() });
        }
        let last = profile.len() - 1;
        if profile[last].0 > 1e-9 {
            o.prims.push(Prim { flags: PRIM_POLYGON, color, idx: (0..seg1).map(|i| last * seg1 + i).collect() });
        }
    }
    o
}

pub fn make_cylinder(color: u32, r1: f64, r2: f64, length: f64, seg1: usize, seg2: usize) -> Object3d {
    let seg2 = seg2.max(1);
    let profile: Vec<(f64, f64)> = (0..=seg2).map(|k| {
        let t = k as f64 / seg2 as f64;
        (r1 + (r2 - r1) * t, length * t)
    }).collect();
    lathe(color, &profile, seg1, true)
}

pub fn make_tube(color: u32, r_out: f64, r_in: f64, length: f64, seg1: usize, seg2: usize) -> Object3d {
    let seg2 = seg2.max(1);
    let mut profile: Vec<(f64, f64)> = (0..=seg2).map(|k| (r_out, length * k as f64 / seg2 as f64)).collect();
    profile.extend((0..=seg2).rev().map(|k| (r_in, length * k as f64 / seg2 as f64)));
    profile.push((r_out, 0.0));
    lathe(color, &profile, seg1, false)
}

pub fn make_sphere(color: u32, r: f64, seg1: usize, seg2: usize) -> Object3d {
    let seg2 = seg2.max(2);
    let profile: Vec<(f64, f64)> = (0..=seg2).map(|k| {
        let a = std::f64::consts::PI * k as f64 / seg2 as f64;
        (r * a.sin(), -r * a.cos())
    }).collect();
    lathe(color, &profile, seg1, false)
}

pub fn make_tore(color: u32, big: f64, small: f64, seg1: usize, seg2: usize) -> Object3d {
    let seg2 = seg2.max(3);
    let profile: Vec<(f64, f64)> = (0..=seg2).map(|k| {
        let a = std::f64::consts::TAU * k as f64 / seg2 as f64;
        (big + small * a.cos(), small * a.sin())
    }).collect();
    lathe(color, &profile, seg1, false)
}

/// Сетка в плоскости XY с центром в начале координат.
pub fn make_grid(color: u32, size_x: f64, count_x: usize, size_y: f64, count_y: usize) -> Object3d {
    let mut o = empty(color);
    let (cx, cy) = (count_x.max(1), count_y.max(1));
    for i in 0..=cx {
        let x = -size_x / 2.0 + size_x * i as f64 / cx as f64;
        let n = o.points.len();
        o.points.push([x, -size_y / 2.0, 0.0]);
        o.points.push([x, size_y / 2.0, 0.0]);
        o.prims.push(Prim { flags: PRIM_POLYLINE, color, idx: vec![n, n + 1] });
    }
    for j in 0..=cy {
        let y = -size_y / 2.0 + size_y * j as f64 / cy as f64;
        let n = o.points.len();
        o.points.push([-size_x / 2.0, y, 0.0]);
        o.points.push([size_x / 2.0, y, 0.0]);
        o.prims.push(Prim { flags: PRIM_POLYLINE, color, idx: vec![n, n + 1] });
    }
    o
}

/// Поверхность по сетке вершин `rows × cols`; цвет ячейки — из `colors`.
pub fn make_surface(points: Vec<Vec3>, rows: usize, cols: usize, colors: &[u32], default: u32) -> Object3d {
    let mut o = empty(default);
    o.points = points;
    for i in 0..rows.saturating_sub(1) {
        for k in 0..cols.saturating_sub(1) {
            let color = colors.get(i * cols + k).copied().unwrap_or(default);
            quad(&mut o, color, i * cols + k, i * cols + k + 1, (i + 1) * cols + k + 1, (i + 1) * cols + k);
        }
    }
    o
}

// ── проекция в SVG ────────────────────────────────────────────────────

struct Drawn {
    depth: f64,
    svg: String,
}

/// Рисует содержимое пространства через камеру в прямоугольник `x y w h`.
/// Алгоритм художника: дальние грани раньше; плоская подсветка по нормали.
pub fn render_view(sp: &Space3d, cam: &Camera, x: f64, y: f64, w: f64, h: f64, out: &mut String) {
    let fitted;
    let cam = if cam.extent > 0.0 {
        cam
    } else {
        let mut c = cam.clone();
        c.extent = auto_extent(sp, cam);
        fitted = c;
        &fitted
    };
    let _ = write!(
        out,
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
        x, y, w, h, color(cam.background)
    );
    let mut drawn: Vec<Drawn> = Vec::new();
    let light = norm([-0.4, 0.6, -0.7]);
    for o in sp.objects.values().filter(|o| o.visible) {
        let world = sp.world_points(o);
        let view: Vec<Vec3> = world.iter().map(|p| cam.to_view(*p)).collect();
        let screen: Vec<Option<(f64, f64)>> = view.iter().map(|v| cam.project(*v, w, h).map(|(px, py)| (px + x, py + y))).collect();
        for prim in &o.prims {
            let pts: Vec<(usize, (f64, f64))> = prim.idx.iter().filter_map(|&i| Some((i, screen.get(i).copied().flatten()?))).collect();
            if pts.is_empty() {
                continue;
            }
            let depth = pts.iter().map(|(i, _)| view[*i][2]).sum::<f64>() / pts.len() as f64;
            let path: String = pts.iter().enumerate().map(|(k, (_, (px, py)))| format!("{}{:.1} {:.1}", if k == 0 { "M" } else { "L" }, px, py)).collect::<Vec<_>>().join(" ");
            let svg = if prim.flags & PRIM_POLYGON != 0 && pts.len() >= 3 {
                let (a, b, c) = (view[pts[0].0], view[pts[1].0], view[pts[2].0]);
                let n = norm(cross(sub(b, a), sub(c, a)));
                let shade = 0.55 + 0.45 * dot(n, light).abs();
                format!("<path d=\"{path} Z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-linejoin=\"round\"/>\n", shaded(prim.color, shade), shaded(prim.color, shade * 0.7))
            } else if prim.flags & PRIM_POLYLINE != 0 || pts.len() >= 2 {
                format!("<path d=\"{path}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1\"/>\n", color(prim.color))
            } else {
                let (px, py) = pts[0].1;
                format!("<circle cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"1.5\" fill=\"{}\"/>\n", color(prim.color))
            };
            drawn.push(Drawn { depth, svg });
        }
        if o.prims.is_empty() {
            for (i, s) in screen.iter().enumerate() {
                if let Some((px, py)) = s {
                    drawn.push(Drawn { depth: view[i][2], svg: format!("<circle cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"1.5\" fill=\"{}\"/>\n", color(o.color)) });
                }
            }
        }
    }
    drawn.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));
    let _ = write!(out, "<clipPath id=\"v3d{}\"><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/></clipPath><g clip-path=\"url(#v3d{})\">\n", sp.handle, x, y, w, h, sp.handle);
    for d in drawn {
        out.push_str(&d.svg);
    }
    out.push_str("</g>\n");
}

/// Масштаб «показать всё» для камеры без заданного масштаба.
pub fn auto_extent(sp: &Space3d, cam: &Camera) -> f64 {
    let mut r: f64 = 0.0;
    for o in sp.objects.values().filter(|o| o.visible) {
        for p in sp.world_points(o) {
            let v = cam.to_view(p);
            r = r.max(v[0].abs()).max(v[1].abs());
        }
    }
    if r > 0.0 { r * 1.15 } else { 200.0 }
}

fn shaded(c: u32, k: f64) -> String {
    let ch = |v: u32| ((v as f64 * k).round().clamp(0.0, 255.0)) as u32;
    format!("#{:02x}{:02x}{:02x}", ch(c & 0xFF), ch((c >> 8) & 0xFF), ch((c >> 16) & 0xFF))
}

/// Попадание точки кадра в примитив: ближайший к камере полигон, содержащий точку.
pub fn hit(sp: &Space3d, cam: &Camera, w: f64, h: f64, px: f64, py: f64) -> Option<(Handle, usize)> {
    let fitted;
    let cam = if cam.extent > 0.0 { cam } else { let mut c = cam.clone(); c.extent = auto_extent(sp, cam); fitted = c; &fitted };
    let mut best: Option<(f64, Handle, usize)> = None;
    for o in sp.objects.values().filter(|o| o.visible) {
        let view: Vec<Vec3> = sp.world_points(o).iter().map(|p| cam.to_view(*p)).collect();
        let screen: Vec<Option<(f64, f64)>> = view.iter().map(|v| cam.project(*v, w, h)).collect();
        for (n, prim) in o.prims.iter().enumerate() {
            let poly: Vec<(f64, f64)> = prim.idx.iter().filter_map(|&i| screen.get(i).copied().flatten()).collect();
            if poly.len() < 3 || !inside(&poly, px, py) {
                continue;
            }
            let depth = prim.idx.iter().filter_map(|&i| view.get(i)).map(|v| v[2]).sum::<f64>() / prim.idx.len().max(1) as f64;
            if best.is_none_or(|b| depth < b.0) {
                best = Some((depth, o.handle, n));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

fn inside(poly: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[(i + n - 1) % n];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            c = !c;
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_about_z_turns_x_into_y() {
        let m = rotation([0.0; 3], [0.0, 0.0, 1.0], std::f64::consts::FRAC_PI_2);
        let p = apply([1.0, 0.0, 0.0], &m);
        assert!((p[0]).abs() < 1e-9 && (p[1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn inverse_undoes_transform() {
        let m = mul(&rotation([1.0, 2.0, 3.0], [1.0, 1.0, 0.0], 0.7), &translation([5.0, -2.0, 1.0]));
        let p = [3.0, 4.0, 5.0];
        let back = apply(apply(p, &m), &inverse(&m));
        for k in 0..3 {
            assert!((back[k] - p[k]).abs() < 1e-9);
        }
    }

    #[test]
    fn bar_has_six_faces_and_projects_inside_frame() {
        let mut sp = Space3d::new(1, 1);
        let h = sp.add_object(make_bar(0xFF0000, 20.0, 20.0, 20.0));
        assert_eq!(sp.objects[&h].prims.len(), 6);
        let mut cam = Camera::looking_from([100.0, 100.0, 100.0]);
        cam.extent = 30.0;
        let mut out = String::new();
        render_view(&sp, &cam, 0.0, 0.0, 200.0, 200.0, &mut out);
        assert!(out.matches("<path").count() == 6);
    }
}
