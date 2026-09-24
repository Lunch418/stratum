//! Функции трёхмерной графики языка: пространства, объекты, камеры,
//! системы координат, тела из `3DTOOLS.TDL`. Работают с матрицами
//! среды (`Matrices`) и возвращают значения по ссылке через `outputs`.

use super::space3d::{self, Camera, Mat4, Prim, Space3d, Vec3, IDENTITY, PRIM_POLYGON};
use super::{Gfx, Handle, Object, Shape};
use crate::runtime::data::{self, Matrices, Matrix};
use crate::runtime::Value;

fn f(args: &[Value], i: usize) -> f64 {
    args.get(i).map(|v| v.as_float()).unwrap_or(0.0)
}
fn h(args: &[Value], i: usize) -> Handle {
    f(args, i).max(0.0) as Handle
}
fn s(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.to_string()).unwrap_or_default()
}
fn num(v: f64) -> Value {
    Value::Float(v)
}
fn handle(v: Handle) -> Value {
    Value::Handle(v as f64)
}
fn ok(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}

/// Матрица среды 4×4 → `Mat4` (индексы с 1, как в `mcreate(…,1,4,1,4)`).
fn mat4_from(m: &Matrix) -> Mat4 {
    let mut r = IDENTITY;
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = m.get(m.min_i + i as i64, m.min_j + j as i64);
        }
    }
    r
}

fn mat4_to(q: i64, m: &Mat4, ms: &mut Matrices) -> i64 {
    let mut out = Matrix::new(1, 4, 1, 4).unwrap();
    for i in 0..4 {
        for j in 0..4 {
            out.set(1 + i as i64, 1 + j as i64, m[i][j]);
        }
    }
    ms.put_result(q, out)
}

/// Ось из матрицы 2×3: точка и направление.
fn axis_from(m: &Matrix) -> (Vec3, Vec3) {
    let p = [m.get(m.min_i, m.min_j), m.get(m.min_i, m.min_j + 1), m.get(m.min_i, m.min_j + 2)];
    let d = [m.get(m.min_i + 1, m.min_j), m.get(m.min_i + 1, m.min_j + 1), m.get(m.min_i + 1, m.min_j + 2)];
    (p, d)
}

/// Точки из матрицы: тройки подряд по строкам (N×3 или сетка N×3M).
fn points_from(m: &Matrix) -> (Vec<Vec3>, usize, usize) {
    let cols = m.cols() / 3;
    let mut pts = Vec::with_capacity(m.rows() * cols);
    for i in 0..m.rows() as i64 {
        for k in 0..cols as i64 {
            pts.push([
                m.get(m.min_i + i, m.min_j + k * 3),
                m.get(m.min_i + i, m.min_j + k * 3 + 1),
                m.get(m.min_i + i, m.min_j + k * 3 + 2),
            ]);
        }
    }
    (pts, m.rows(), cols)
}

/// Проекция (2D-объект) → её пространство и камера.
fn view_of(gfx: &Gfx, space2d: Handle, view: Handle) -> Option<(Handle, Handle)> {
    match gfx.space(space2d)?.objects.get(&view)?.shape {
        Shape::View3d { space, camera } => Some((space, camera)),
        _ => None,
    }
}

fn sp3<'a>(gfx: &'a mut Gfx, args: &[Value], i: usize) -> Option<&'a mut Space3d> {
    gfx.spaces3d.get_mut(&h(args, i))
}

fn view_rect(gfx: &Gfx, space2d: Handle, view: Handle) -> (f64, f64) {
    gfx.space(space2d).and_then(|sp| sp.objects.get(&view)).map(|o| (o.w, o.h)).unwrap_or((100.0, 100.0))
}

pub fn call(name: &str, args: &[Value], gfx: &mut Gfx, ms: &mut Matrices, outputs: &mut Vec<(usize, Value)>) -> Option<Value> {
    let lower = name.to_ascii_lowercase();
    if !lower.contains("3d") {
        return None;
    }
    let v = match lower.as_str() {
        // ── пространства и проекции ──────────────────────────────────────
        "createspace3d" => {
            let owner = h(args, 0);
            if gfx.space(owner).is_none() {
                return Some(handle(0));
            }
            handle(gfx.create_space3d(owner))
        }
        "deletespace3d" => ok(gfx.spaces3d.remove(&h(args, 0)).is_some()),
        "getspace3d" => handle(view_of(gfx, h(args, 0), h(args, 1)).map(|v| v.0).unwrap_or(0)),
        "getactivecamera3d" => handle(view_of(gfx, h(args, 0), h(args, 1)).map(|v| v.1).unwrap_or(0)),
        "switchtocamera3d" => {
            let cam = h(args, 2);
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.objects.get_mut(&h(args, 1))).map(|o| {
                if let Shape::View3d { camera, .. } = &mut o.shape {
                    *camera = cam;
                }
            }).is_some())
        }
        "create3dview2d" | "create3dview" => {
            let (space, camera) = (h(args, 0), h(args, 1));
            let (x, y, w, hh) = (f(args, 2), f(args, 3), f(args, 4), f(args, 5));
            let Some(owner) = gfx.spaces3d.get(&space).map(|s| s.owner) else { return Some(handle(0)) };
            handle(gfx.space_mut(owner).map(|sp| sp.add_object(Object::new(0, x, y, w, hh, Shape::View3d { space, camera }))).unwrap_or(0))
        }

        // ── камеры ────────────────────────────────────────────────────────
        "createdefcamera3d" => {
            let kind = f(args, 1) as u32;
            let mut pos = [0.0; 3];
            if kind & 1 != 0 { pos[0] = 1000.0 }
            if kind & 2 != 0 { pos[0] = -1000.0 }
            if kind & 4 != 0 { pos[1] = 1000.0 }
            if kind & 8 != 0 { pos[1] = -1000.0 }
            if kind & 16 != 0 { pos[2] = 1000.0 }
            if kind & 32 != 0 { pos[2] = -1000.0 }
            if pos == [0.0; 3] {
                pos = [1000.0, 1000.0, 1000.0];
            }
            handle(sp3(gfx, args, 0).map(|s| s.add_camera(Camera::looking_from(pos))).unwrap_or(0))
        }
        // CreateCamera3dEx(hSpace3d, name, flags, render, bkColor, extent, focus, upX, upY, upZ, posX, posY, posZ)
        "createcamera3dex" => {
            let name = s(args, 1);
            let flags = f(args, 2) as u32;
            let background = f(args, 4) as u32;
            let extent = f(args, 5);
            let focus = f(args, 6);
            let up = [f(args, 7), f(args, 8), f(args, 9)];
            let pos = [f(args, 10), f(args, 11), f(args, 12)];
            let pos = if pos == [0.0; 3] { [1000.0, 1000.0, 1000.0] } else { pos };
            let mut cam = Camera::looking_from(pos);
            cam.name = name;
            cam.flags = flags;
            cam.background = background;
            if space3d::dot(up, up) > 1e-9 {
                cam.up = space3d::norm(up);
            }
            if extent > 0.0 {
                cam.extent = extent;
            }
            cam.focus = focus.max(0.0);
            handle(sp3(gfx, args, 0).map(|s| s.add_camera(cam)).unwrap_or(0))
        }
        "setcamerapoint3d" => {
            let p = [f(args, 2), f(args, 3), f(args, 4)];
            ok(sp3(gfx, args, 0).and_then(|s| s.cameras.get_mut(&h(args, 1))).map(|c| c.pos = p).is_some())
        }
        // TransformCamera3d(hSpace2d, hView, mode, p1, p2, p3)
        "transformcamera3d" => {
            let (space2d, view) = (h(args, 0), h(args, 1));
            let mode = f(args, 2) as u32;
            let (p1, p2) = (f(args, 3), f(args, 4));
            let (w, hh) = view_rect(gfx, space2d, view);
            let Some((space, camera)) = view_of(gfx, space2d, view) else { return Some(ok(false)) };
            // масштаб «авто» фиксируется перед изменением
            if let Some(s3) = gfx.spaces3d.get_mut(&space) {
                let auto = s3.cameras.get(&camera).map(|c| if c.extent > 0.0 { 0.0 } else { space3d::auto_extent(s3, c) });
                if let (Some(a), Some(cam)) = (auto, s3.cameras.get_mut(&camera)) {
                    if a > 0.0 {
                        cam.extent = a;
                    }
                }
            }
            let done = gfx.spaces3d.get_mut(&space).and_then(|s| s.cameras.get_mut(&camera)).map(|cam| match mode {
                1..=3 => cam.orbit(mode, p1),
                10 => {
                    if p1 > 0.0 {
                        cam.extent /= p1;
                    }
                }
                40 => cam.pan(p1, -p2, w, hh),
                41 => cam.pan(p1, 0.0, w, hh),
                42 => cam.pan(0.0, -p1, w, hh),
                _ => {}
            });
            ok(done.is_some())
        }
        // FitToCamera3d(hSpace2d, hView, hCamera, fit)
        "fittocamera3d" => {
            let (space2d, view) = (h(args, 0), h(args, 1));
            let fit = f(args, 3);
            let fit = if fit > 0.0 { fit } else { 1.0 };
            let Some((space, camera)) = view_of(gfx, space2d, view) else { return Some(ok(false)) };
            let camera = if h(args, 2) != 0 { h(args, 2) } else { camera };
            let done = gfx.spaces3d.get_mut(&space).and_then(|s| {
                let (center, radius) = s.bounds()?;
                let cam = s.cameras.get_mut(&camera)?;
                let dir = space3d::sub(cam.pos, cam.target);
                cam.target = center;
                cam.pos = space3d::add(center, if space3d::dot(dir, dir) > 1e-9 { dir } else { [radius * 3.0; 3] });
                cam.extent = radius / fit;
                Some(())
            });
            ok(done.is_some())
        }
        "_cameraproc3d" => num(1.0),

        // ── системы координат ────────────────────────────────────────────
        "pushcrdsystem3d" => ok(sp3(gfx, args, 0).map(|s| { let c = s.crd; s.stack.push(c) }).is_some()),
        "popcrdsystem3d" => ok(sp3(gfx, args, 0).map(|s| if let Some(c) = s.stack.pop() { s.crd = c }).is_some()),
        "selectworldcrd3d" | "selectviewcrd3d" => ok(sp3(gfx, args, 0).map(|s| s.crd = IDENTITY).is_some()),
        "selectlocalcrd3d" => {
            let obj = f(args, 1);
            ok(sp3(gfx, args, 0).map(|s| {
                s.crd = if obj <= 0.0 { IDENTITY } else { s.objects.get(&(obj as Handle)).map(|o| o.matrix).unwrap_or(IDENTITY) };
            }).is_some())
        }

        // ── объекты ───────────────────────────────────────────────────────
        "createobject3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::empty(0xFFFFFF))).unwrap_or(0)),
        "duplicateobject3d" => {
            handle(sp3(gfx, args, 0).and_then(|s| {
                let copy = s.objects.get(&h(args, 1))?.clone();
                Some(s.add_object(copy))
            }).unwrap_or(0))
        }
        "makebar3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_bar(f(args, 1) as u32, f(args, 2), f(args, 3), f(args, 4)))).unwrap_or(0)),
        // MakeCylinder3d(hSpace3d, hParent, color, r1, r2, length, seg1, seg2)
        "makecylinder3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_cylinder(f(args, 2) as u32, f(args, 3), f(args, 4), f(args, 5), f(args, 6) as usize, f(args, 7) as usize))).unwrap_or(0)),
        "maketube3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_tube(f(args, 2) as u32, f(args, 3), f(args, 4), f(args, 5), f(args, 6) as usize, f(args, 7) as usize))).unwrap_or(0)),
        // MakeSphere3d(hSpace3d, hParent, color, r, seg1, seg2)
        "makesphere3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_sphere(f(args, 2) as u32, f(args, 3), f(args, 4) as usize, f(args, 5) as usize))).unwrap_or(0)),
        // MakeTore3d(hSpace3d, hParent, color, R, r, angle, seg1, seg2)
        "maketore3d" | "maketoreex3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_tore(f(args, 2) as u32, f(args, 3), f(args, 4), f(args, 6) as usize, f(args, 7) as usize))).unwrap_or(0)),
        // MakeGrid3d(hSpace3d, sizeX, countX, sizeY, countY, color)
        "makegrid3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_grid(f(args, 5) as u32, f(args, 1), f(args, 2) as usize, f(args, 3), f(args, 4) as usize))).unwrap_or(0)),
        // CreateSurface3d(hSpace3d, Mvertex, Mcolors, defColor, [flags])
        "createsurface3d" => {
            let Some(m) = ms.get(data::idx(f(args, 1))) else { return Some(handle(0)) };
            let (pts, rows, cols) = points_from(m);
            let colors: Vec<u32> = ms
                .get(data::idx(f(args, 2)))
                .map(|cm| {
                    let mut out = Vec::new();
                    for i in 0..rows as i64 {
                        for k in 0..cols as i64 {
                            out.push(cm.get(cm.min_i + i, cm.min_j + k) as u32);
                        }
                    }
                    out
                })
                .unwrap_or_default();
            handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::make_surface(pts, rows, cols, &colors, f(args, 3) as u32))).unwrap_or(0))
        }
        "createobjectfromfile3d" | "sweepandextrude3d" => handle(sp3(gfx, args, 0).map(|s| s.add_object(space3d::empty(0xFFFFFF))).unwrap_or(0)),
        "setobjectcolor3d" => {
            let c = f(args, 2) as u32;
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                o.color = c;
                for p in &mut o.prims {
                    p.color = c;
                }
            }).is_some())
        }
        "getobjectcolor3d" => Value::Color(sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))).map(|o| o.color as f64).unwrap_or(0.0)),

        // ── точки и примитивы ────────────────────────────────────────────
        "addpoint3d" => {
            let p = [f(args, 2), f(args, 3), f(args, 4)];
            num(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                o.points.push(p);
                o.points.len() as f64
            }).unwrap_or(0.0))
        }
        "setpoint3d" => {
            let n = f(args, 2) as usize;
            let p = [f(args, 3), f(args, 4), f(args, 5)];
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).and_then(|o| {
                let slot = o.points.get_mut(n.checked_sub(1)?)?;
                *slot = p;
                Some(())
            }).is_some())
        }
        // GetPoint3d(hSpace3d, hObject, num, &x, &y, &z)
        "getpoint3d" => {
            let n = f(args, 2) as usize;
            match sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))).and_then(|o| o.points.get(n.checked_sub(1)?)) {
                Some(p) => {
                    outputs.push((3, num(p[0])));
                    outputs.push((4, num(p[1])));
                    outputs.push((5, num(p[2])));
                    ok(true)
                }
                None => ok(false),
            }
        }
        "delpoint3d" => {
            let n = f(args, 2) as usize;
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).is_some_and(|o| {
                if n == 0 || n > o.points.len() {
                    return false;
                }
                o.points.remove(n - 1);
                for p in &mut o.prims {
                    p.idx.retain(|&i| i != n - 1);
                    for i in &mut p.idx {
                        if *i > n - 1 {
                            *i -= 1;
                        }
                    }
                }
                true
            }))
        }
        "getnumpoints3d" => num(sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))).map(|o| o.points.len() as f64).unwrap_or(0.0)),
        "getnumprimitives3d" => num(sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))).map(|o| o.prims.len() as f64).unwrap_or(0.0)),
        // AddPrimitive3d(hSpace3d, hObject, flags, color, n1, n2, …) — номера точек с 1
        "addprimitive3d" => {
            let flags = f(args, 2) as u32;
            let color = f(args, 3) as u32;
            let idx: Vec<usize> = args.iter().skip(4).map(|v| v.as_float() as usize).filter(|&n| n > 0).map(|n| n - 1).collect();
            num(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                o.prims.push(Prim { flags: if flags == 0 && idx.len() >= 3 { PRIM_POLYGON } else { flags }, color, idx });
                o.prims.len() as f64
            }).unwrap_or(0.0))
        }
        // SetPrimitive3d(hSpace3d, hObject, num, flags, color, n1, …)
        "setprimitive3d" => {
            let n = f(args, 2) as usize;
            let flags = f(args, 3) as u32;
            let color = f(args, 4) as u32;
            let idx: Vec<usize> = args.iter().skip(5).map(|v| v.as_float() as usize).filter(|&k| k > 0).map(|k| k - 1).collect();
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).and_then(|o| {
                let p = o.prims.get_mut(n.checked_sub(1)?)?;
                p.flags = flags;
                p.color = color;
                if !idx.is_empty() {
                    p.idx = idx;
                }
                Some(())
            }).is_some())
        }
        "delprimitive3d" => {
            let n = f(args, 2) as usize;
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).is_some_and(|o| {
                if n == 0 || n > o.prims.len() {
                    return false;
                }
                o.prims.remove(n - 1);
                true
            }))
        }
        // GetObjectPoints3d(hSpace3d, hObject, matrix) → N×3
        "getobjectpoints3d" => {
            let Some(o) = sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))) else { return Some(num(0.0)) };
            let n = o.points.len().max(1) as i64;
            let mut m = Matrix::new(1, n, 1, 3).unwrap();
            for (i, p) in o.points.iter().enumerate() {
                for k in 0..3 {
                    m.set(1 + i as i64, 1 + k as i64, p[k]);
                }
            }
            num(ms.put_result(data::idx(f(args, 2)), m) as f64)
        }
        "setobjectpoints3d" => {
            let Some(m) = ms.get(data::idx(f(args, 2))) else { return Some(ok(false)) };
            let (pts, _, _) = points_from(m);
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                for (i, p) in pts.into_iter().enumerate() {
                    if i < o.points.len() {
                        o.points[i] = p;
                    } else {
                        o.points.push(p);
                    }
                }
            }).is_some())
        }
        // GetColors3d(hSpace3d, hObject, matrix, from, count) → столбец цветов примитивов
        "getcolors3d" => {
            let Some(o) = sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))) else { return Some(num(0.0)) };
            let from = (f(args, 3) as usize).min(o.prims.len());
            let count = if f(args, 4) < 0.0 { o.prims.len() - from } else { (f(args, 4) as usize).min(o.prims.len() - from) };
            let mut m = Matrix::new(1, count.max(1) as i64, 1, 1).unwrap();
            for i in 0..count {
                m.set(1 + i as i64, 1, o.prims[from + i].color as f64);
            }
            num(ms.put_result(data::idx(f(args, 2)), m) as f64)
        }
        "setcolors3d" => {
            let Some(m) = ms.get(data::idx(f(args, 2))).cloned() else { return Some(ok(false)) };
            let from = f(args, 3) as usize;
            let count = f(args, 4);
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                let n = if count < 0.0 { o.prims.len().saturating_sub(from) } else { count as usize };
                for i in 0..n {
                    if let Some(p) = o.prims.get_mut(from + i) {
                        p.color = m.get(m.min_i + i as i64, m.min_j) as u32;
                    }
                }
            }).is_some())
        }

        // ── положение и преобразования ───────────────────────────────────
        "setobjectbase3d" => {
            let p = [f(args, 2), f(args, 3), f(args, 4)];
            ok(sp3(gfx, args, 0).is_some_and(|s| s.set_base(h(args, 1), p)))
        }
        // GetObjectBase3d(hSpace3d, hObject, matrix) → 1×3
        "getobjectbase3d" => match sp3(gfx, args, 0).and_then(|s| s.base(h(args, 1))) {
            Some(p) => {
                let mut m = Matrix::new(1, 1, 1, 3).unwrap();
                for k in 0..3 {
                    m.set(1, 1 + k as i64, p[k]);
                }
                num(ms.put_result(data::idx(f(args, 2)), m) as f64)
            }
            None => num(0.0),
        },
        // GetObjectBase3dM(hSpace3d, hObject, &x, &y, &z)
        "getobjectbase3dm" => match sp3(gfx, args, 0).and_then(|s| s.base(h(args, 1))) {
            Some(p) => {
                outputs.push((2, num(p[0])));
                outputs.push((3, num(p[1])));
                outputs.push((4, num(p[2])));
                ok(true)
            }
            None => ok(false),
        },
        // GetObjectDimension3d(hSpace3d, hObject, matrix) → 1×3 габарит
        "getobjectdimension3d" => match sp3(gfx, args, 0).and_then(|s| s.objects.get(&h(args, 1))) {
            Some(o) => {
                let mut lo = [f64::INFINITY; 3];
                let mut hi = [f64::NEG_INFINITY; 3];
                for p in &o.points {
                    for k in 0..3 {
                        lo[k] = lo[k].min(p[k]);
                        hi[k] = hi[k].max(p[k]);
                    }
                }
                let mut m = Matrix::new(1, 1, 1, 3).unwrap();
                for k in 0..3 {
                    m.set(1, 1 + k as i64, if o.points.is_empty() { 0.0 } else { hi[k] - lo[k] });
                }
                num(ms.put_result(data::idx(f(args, 2)), m) as f64)
            }
            None => num(0.0),
        },
        // GetObjectMatrix3d(matrix, hSpace3d, hObject) — матрица первым аргументом
        "getobjectmatrix3d" => match gfx.spaces3d.get(&h(args, 1)).and_then(|s| s.objects.get(&h(args, 2))) {
            Some(o) => num(mat4_to(data::idx(f(args, 0)), &o.matrix, ms) as f64),
            None => num(0.0),
        },
        "setobjectmatrix3d" => {
            let Some(m) = ms.get(data::idx(f(args, 0))).map(mat4_from) else { return Some(ok(false)) };
            ok(gfx.spaces3d.get_mut(&h(args, 1)).and_then(|s| s.objects.get_mut(&h(args, 2))).map(|o| o.matrix = m).is_some())
        }
        // TransformObject3d(hSpace3d, hObject, matrix4×4) — в текущей системе координат
        "transformobject3d" => {
            let Some(t) = ms.get(data::idx(f(args, 2))).map(mat4_from) else { return Some(ok(false)) };
            ok(sp3(gfx, args, 0).is_some_and(|s| s.transform_object(h(args, 1), &t)))
        }
        // RotateObject3d(hSpace3d, hObject, axis2×3, angle)
        "rotateobject3d" => {
            let Some((p, d)) = ms.get(data::idx(f(args, 2))).map(axis_from) else { return Some(ok(false)) };
            let t = space3d::rotation(p, d, f(args, 3));
            ok(sp3(gfx, args, 0).is_some_and(|s| s.transform_object(h(args, 1), &t)))
        }
        // …Points-варианты меняют сами точки в локальных координатах
        "transformobjectpoints3d" => {
            let Some(t) = ms.get(data::idx(f(args, 2))).map(mat4_from) else { return Some(ok(false)) };
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                for p in &mut o.points {
                    *p = space3d::apply(*p, &t);
                }
            }).is_some())
        }
        "rotateobjectpoints3d" => {
            let Some((p, d)) = ms.get(data::idx(f(args, 2))).map(axis_from) else { return Some(ok(false)) };
            let t = space3d::rotation(p, d, f(args, 3));
            ok(sp3(gfx, args, 0).and_then(|s| s.objects.get_mut(&h(args, 1))).map(|o| {
                for q in &mut o.points {
                    *q = space3d::apply(*q, &t);
                }
            }).is_some())
        }
        // GetObject3dFromPoint2d(hSpace2d, hView, x, y, &hObject, &item)
        "getobject3dfrompoint2d" => {
            let (space2d, view) = (h(args, 0), h(args, 1));
            let Some((space, camera)) = view_of(gfx, space2d, view) else { return Some(ok(false)) };
            let (ox, oy, w, hh) = gfx.space(space2d).and_then(|sp| sp.objects.get(&view)).map(|o| (o.x, o.y, o.w, o.h)).unwrap_or((0.0, 0.0, 1.0, 1.0));
            let found = gfx.spaces3d.get(&space).and_then(|s3| s3.cameras.get(&camera).and_then(|cam| space3d::hit(s3, cam, w, hh, f(args, 2) - ox, f(args, 3) - oy)));
            match found {
                Some((obj, item)) => {
                    outputs.push((4, handle(obj)));
                    outputs.push((5, num(item as f64 + 1.0)));
                    ok(true)
                }
                None => ok(false),
            }
        }

        // ── материалы: без текстур, но цвет (diffuse) ложится на объект ───
        // CreateMaterial3d(hSpace3d, name, file, ambient, diffuse, specular, emittance, shine, transparency, flags)
        "creatematerial3d" => {
            let name = s(args, 1);
            let diffuse = f(args, 4) as u32;
            handle(sp3(gfx, args, 0).map(|s3| {
                let h = s3.alloc_public();
                s3.materials.insert(h, (name, diffuse));
                h
            }).unwrap_or(0))
        }
        "getmaterialbyname3d" => {
            let name = s(args, 1);
            handle(sp3(gfx, args, 0).and_then(|s3| s3.materials.iter().find(|(_, m)| m.0.eq_ignore_ascii_case(&name)).map(|(h, _)| *h)).unwrap_or(0))
        }
        // ApplyTexture3d(hSpace3d, hObject, hFrame, hMaterial, …)
        "applytexture3d" => {
            let mat = h(args, 3);
            ok(sp3(gfx, args, 0).and_then(|s3| {
                let color = s3.materials.get(&mat)?.1;
                let o = s3.objects.get_mut(&h(args, 1))?;
                if color != 0xFFFFFF || o.color == 0 {
                    o.color = color;
                    for p in &mut o.prims {
                        p.color = color;
                    }
                }
                Some(())
            }).is_some())
        }
        "removetexture3d" => ok(true),

        _ => return None,
    };
    Some(v)
}

/// Дескриптор трёхмерного объекта или камеры по имени (для `GetObject2dByName`
/// с дескриптором трёхмерного пространства).
pub fn find_by_name(gfx: &Gfx, space: Handle, name: &str) -> Option<Handle> {
    gfx.spaces3d.get(&space)?.find_by_name(name)
}

pub fn set_name(gfx: &mut Gfx, space: Handle, obj: Handle, name: &str) -> bool {
    gfx.spaces3d.get_mut(&space).is_some_and(|s| s.set_name(obj, name))
}

pub fn set_visible(gfx: &mut Gfx, space: Handle, obj: Handle, visible: bool) -> bool {
    gfx.spaces3d.get_mut(&space).and_then(|s| s.objects.get_mut(&obj)).map(|o| o.visible = visible).is_some()
}

pub fn delete(gfx: &mut Gfx, space: Handle, obj: Handle) -> bool {
    gfx.spaces3d.get_mut(&space).is_some_and(|s| s.objects.remove(&obj).is_some() || s.cameras.remove(&obj).is_some())
}

pub fn object_name(gfx: &Gfx, space: Handle, obj: Handle) -> Option<String> {
    let s = gfx.spaces3d.get(&space)?;
    s.objects.get(&obj).map(|o| o.name.clone()).or_else(|| s.cameras.get(&obj).map(|c| c.name.clone()))
}
