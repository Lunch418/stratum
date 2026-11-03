//! Отрисовка сцены Ogre: геометрия объектов (с костной анимацией), свет,
//! туман, текстуры, растеризация, кадр для IDE.

use super::*;

/// Имя окна рендеринга в списке окон модели.
pub const WINDOW_NAME: &str = "Ogre3D";

pub(super) struct Geo {
    pub pos: Vec<V3>,
    pub nrm: Vec<V3>,
    pub uv: Vec<[f32; 2]>,
    pub tris: Vec<[u32; 3]>,
    pub lines: Vec<[u32; 2]>,
    pub material: String,
}

struct MatLook {
    amb: [f64; 4],
    diff: [f64; 4],
    emis: [f64; 4],
    lit: bool,
    cull: u8,
    depth_check: bool,
    depth_write: bool,
    blend: (u8, u8),
    reject: (u8, f64),
    wire: bool,
    tex: Option<Arc<Tex>>,
}

struct Light {
    pos: V3,
    dir: V3,
    kind: u8,
    col: V3,
    atten: [f64; 4],
    spot: [f64; 3],
}

#[derive(Clone, Copy)]
struct Pv {
    p: V3,
    col: [f64; 3],
    uv: [f64; 2],
}

struct Surf<'a> {
    rgb: &'a mut [u8],
    full_w: usize,
    x0: usize,
    y0: usize,
    vw: usize,
    vh: usize,
    depth: Vec<f32>,
}

struct Cam {
    near: f64,
    th: f64,
    aspect: f64,
    ortho: bool,
    fog: Option<[f64; 7]>,
}

impl Frame {
    /// Кадр как BMP 24 бита (снизу вверх).
    pub fn to_bmp(&self) -> Vec<u8> {
        let stride = (self.width * 3).div_ceil(4) * 4;
        let size = 54 + stride * self.height;
        let mut b = Vec::with_capacity(size);
        b.extend_from_slice(b"BM");
        b.extend_from_slice(&(size as u32).to_le_bytes());
        b.extend_from_slice(&[0; 4]);
        b.extend_from_slice(&54u32.to_le_bytes());
        b.extend_from_slice(&40u32.to_le_bytes());
        b.extend_from_slice(&(self.width as u32).to_le_bytes());
        b.extend_from_slice(&(self.height as u32).to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&24u16.to_le_bytes());
        b.extend_from_slice(&[0; 24]);
        for y in (0..self.height).rev() {
            let row = &self.rgb[y * self.width * 3..(y + 1) * self.width * 3];
            for p in row.chunks_exact(3) {
                b.extend_from_slice(&[p[2], p[1], p[0]]);
            }
            b.resize(b.len() + stride - self.width * 3, 0);
        }
        b
    }
}

fn slerp0(q: Q, w: f64) -> Q {
    // from identity to q with weight w (shortest path)
    let q = if q[0] < 0.0 { [-q[0], -q[1], -q[2], -q[3]] } else { q };
    qnorm([1.0 + (q[0] - 1.0) * w, q[1] * w, q[2] * w, q[3] * w])
}

fn sample(keys: &[Key], t: f64) -> Option<(Q, V3)> {
    let first = keys.first()?;
    let last = keys.last()?;
    if t <= first.t || keys.len() == 1 {
        return Some((first.rot, first.pos));
    }
    if t >= last.t {
        return Some((last.rot, last.pos));
    }
    let i = keys.partition_point(|k| k.t <= t).max(1);
    let (a, b) = (&keys[i - 1], &keys[i]);
    let f = if b.t > a.t { (t - a.t) / (b.t - a.t) } else { 0.0 };
    let mut bq = b.rot;
    if a.rot.iter().zip(bq.iter()).map(|(x, y)| x * y).sum::<f64>() < 0.0 {
        bq = [-bq[0], -bq[1], -bq[2], -bq[3]];
    }
    let q = qnorm([0, 1, 2, 3].map(|k| a.rot[k] + (bq[k] - a.rot[k]) * f));
    let p = [0, 1, 2].map(|k| a.pos[k] + (b.pos[k] - a.pos[k]) * f);
    Some((q, p))
}

fn derive_bones(sk: &Skeleton, local: &[(V3, Q)]) -> Vec<(V3, Q)> {
    let n = sk.bones.len();
    let mut out: Vec<Option<(V3, Q)>> = vec![None; n];
    for i in 0..n {
        let mut chain = vec![i];
        let mut cur = i;
        while let Some(p) = sk.bones[cur].parent {
            if out[p].is_some() || chain.len() > n {
                break;
            }
            chain.push(p);
            cur = p;
        }
        for &c in chain.iter().rev() {
            let (lp, lr) = local[c];
            let d = match sk.bones[c].parent.and_then(|p| out[p]) {
                Some((pp, pr)) => {
                    let r = qrot(pr, lp);
                    ([pp[0] + r[0], pp[1] + r[1], pp[2] + r[2]], qnorm(qmul(pr, lr)))
                }
                None => (lp, lr),
            };
            out[c] = Some(d);
        }
    }
    out.into_iter().map(|x| x.unwrap_or(([0.0; 3], [1.0, 0.0, 0.0, 0.0]))).collect()
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}

impl Ogre {
    /// Читает и декодирует очередь текстур (после вызова функции, пока доступна песочница).
    pub(super) fn flush_textures(&mut self, fx: &mut Effects) {
        let names = std::mem::take(&mut self.pending_tex);
        for n in names {
            let key = n.to_ascii_lowercase();
            if self.textures.contains_key(&key) || self.textures.len() >= 128 {
                continue;
            }
            let tex = self.read_file(fx, &n).and_then(|d| image::decode(&n, &d));
            if tex.is_none() {
                self.pending_log.push(format!("Ogre: текстура '{n}' не прочитана (BMP, TGA, PNG, базовый JPEG)"));
            }
            self.textures.insert(key, tex.map(Arc::new));
        }
    }

    pub(super) fn want_texture(&mut self, name: &str) {
        let n = clean_name(name);
        if !n.is_empty() && self.pending_tex.len() < 1000 {
            self.pending_tex.push(n);
        }
    }

    /// Кадры для списка окон IDE: (имя, ширина, высота, SVG с вложенным BMP).
    /// Первое окно называется `Ogre3D`, остальные - `Ogre3D 2` и так далее;
    /// лишние окна без областей рендеринга не показываются.
    pub fn frames_svg(&mut self) -> Vec<(String, f64, f64, String)> {
        let mut out = Vec::new();
        if !self.initialised {
            return out;
        }
        let wins: Vec<u32> = self.objs.iter().filter(|(_, o)| o.kind == Kind::Window).map(|(k, _)| *k).collect();
        self.cached.0.retain(|k, _| wins.contains(k));
        for (n, win) in wins.into_iter().enumerate() {
            if n > 0 && self.viewports_of(win).is_empty() {
                continue;
            }
            let fresh = self.cached.0.get(&win).is_some_and(|c| c.0 == self.frames);
            if !fresh {
                let Some(fr) = self.render(Some(win)) else { continue };
                let svg = format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\
                     <image x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" preserveAspectRatio=\"none\" xlink:href=\"data:image/bmp;base64,{}\"/></svg>",
                    crate::gfx::svg::base64(&fr.to_bmp()),
                    w = fr.width,
                    h = fr.height
                );
                self.cached.0.insert(win, (self.frames, fr.width as f64, fr.height as f64, svg));
            }
            if let Some(c) = self.cached.0.get(&win) {
                let name = if n == 0 { WINDOW_NAME.to_string() } else { format!("{WINDOW_NAME} {}", n + 1) };
                out.push((name, c.1, c.2, c.3.clone()));
            }
        }
        out
    }

    /// Кадр первого окна (для проверок).
    pub fn frame_svg(&mut self) -> Option<(f64, f64, String)> {
        self.frames_svg().into_iter().next().map(|f| (f.1, f.2, f.3))
    }

    /// Сообщение мыши окна Ogre3D (WM_*), координаты в пикселях кадра.
    pub fn mouse_event(&mut self, msg: u32, x: f64, y: f64, keys: u32) {
        if x.is_finite() && y.is_finite() {
            self.cursor = [x, y];
        }
        match msg {
            513 => self.mouse[0] = true,
            514 => self.mouse[0] = false,
            516 => self.mouse[1] = true,
            517 => self.mouse[1] = false,
            519 => self.mouse[2] = true,
            520 => self.mouse[2] = false,
            522 => self.wheel += ((keys >> 16) as u16 as i16) as f64 / 120.0,
            _ => {}
        }
    }

    /// Клавиатура: WM_KEYDOWN/WM_KEYUP (и SYS-варианты).
    pub fn key_event(&mut self, msg: u32, vk: u32) {
        match msg {
            256 | 260 if self.keys.len() < 256 => {
                self.keys.insert(vk);
            }
            257 | 261 => {
                self.keys.remove(&vk);
            }
            _ => {}
        }
    }

    /// Матрицы костей (поворот, перенос) для скиннинга сущности.
    fn skin_mats(&self, id: u32) -> Option<Vec<(Q, V3)>> {
        let o = &self.objs[&id];
        let sk = o.skel.as_ref()?;
        let n = sk.bones.len();
        if n == 0 {
            return None;
        }
        let rest: Vec<(V3, Q)> = sk.bones.iter().map(|b| (b.pos, b.rot)).collect();
        let mut local = rest.clone();
        for (_, st) in self.objs.iter().filter(|(_, s)| s.kind == Kind::AnimState && s.parent == id) {
            let get = |k: &str, d: f64| st.f.get(k).and_then(|v| v.first()).copied().unwrap_or(d);
            let (enabled, weight, t) = (get("animationstate.enabled", 0.0), get("animationstate.weight", 1.0), get("animationstate.timeposition", 0.0));
            if enabled == 0.0 || weight <= 0.0 {
                continue;
            }
            let Some(an) = sk.anims.iter().find(|a| a.name == st.name) else { continue };
            for (bi, keys) in &an.tracks {
                if let Some((q, p)) = sample(keys, t) {
                    let l = &mut local[*bi];
                    l.0 = [l.0[0] + p[0] * weight, l.0[1] + p[1] * weight, l.0[2] + p[2] * weight];
                    l.1 = qnorm(qmul(l.1, slerp0(q, weight.min(1.0))));
                }
            }
        }
        // кости с ручным управлением
        for (_, b) in self.objs.iter().filter(|(_, s)| s.kind == Kind::Bone && s.parent == id) {
            if b.f.get("node.manuallycontrolled").is_some_and(|v| v[0] != 0.0) {
                if let Some(bi) = sk.bones.iter().position(|d| d.name == b.name) {
                    if let Some(p) = b.f.get("node.position") {
                        local[bi].0 = [p[0], p[1], p[2]];
                    }
                    if let Some(r) = b.f.get("node.rot") {
                        local[bi].1 = qnorm([r[0], r[1], r[2], r[3]]);
                    }
                }
            }
        }
        let bind = derive_bones(sk, &rest);
        let cur = derive_bones(sk, &local);
        Some(
            (0..n)
                .map(|i| {
                    let (bp, br) = bind[i];
                    let ibr = qconj(br);
                    let ibp = qrot(ibr, [-bp[0], -bp[1], -bp[2]]);
                    let (cp, cr) = cur[i];
                    let q = qnorm(qmul(cr, ibr));
                    let r = qrot(cr, ibp);
                    (q, [r[0] + cp[0], r[1] + cp[1], r[2] + cp[2]])
                })
                .collect(),
        )
    }

    /// Геометрия объекта в мировых координатах.
    pub(super) fn object_geos(&self, id: u32) -> Vec<Geo> {
        let o = &self.objs[&id];
        if o.parent == 0 {
            return Vec::new();
        }
        let (p, r, s) = self.derived(o.parent);
        let skin = if o.kind == Kind::Entity { self.skin_mats(id) } else { None };
        let mut out = Vec::new();
        let mut push = |verts: &[[f32; 3]], nrm: &[[f32; 3]], uv: &[[f32; 2]], idx: &[u32], op: u16, material: &str, bones: &[Vec<(u16, f32)>]| {
            let mut pos = Vec::with_capacity(verts.len());
            let mut nr = Vec::with_capacity(nrm.len());
            for (i, v) in verts.iter().enumerate() {
                let mut vv = [v[0] as f64, v[1] as f64, v[2] as f64];
                let mut nn = nrm.get(i).map(|n| [n[0] as f64, n[1] as f64, n[2] as f64]);
                if let (Some(sk), Some(bw)) = (&skin, bones.get(i)) {
                    let tw: f64 = bw.iter().map(|b| b.1 as f64).sum();
                    if tw > 0.0 {
                        let (mut acc, mut accn) = ([0.0; 3], [0.0; 3]);
                        for &(b, w) in bw {
                            if let Some((q, t)) = sk.get(b as usize) {
                                let tv = qrot(*q, vv);
                                let f = w as f64 / tw;
                                for k in 0..3 {
                                    acc[k] += (tv[k] + t[k]) * f;
                                }
                                if let Some(n) = nn {
                                    let tn = qrot(*q, n);
                                    for k in 0..3 {
                                        accn[k] += tn[k] * f;
                                    }
                                }
                            }
                        }
                        vv = acc;
                        if nn.is_some() {
                            nn = Some(accn);
                        }
                    }
                }
                let sv = qrot(r, [vv[0] * s[0], vv[1] * s[1], vv[2] * s[2]]);
                pos.push([sv[0] + p[0], sv[1] + p[1], sv[2] + p[2]]);
                if let Some(n) = nn {
                    nr.push(unit(qrot(r, n)));
                }
            }
            let mut tris = Vec::new();
            let mut lines = Vec::new();
            match op {
                2 => lines.extend(idx.chunks_exact(2).map(|c| [c[0], c[1]])),
                3 => lines.extend(idx.windows(2).map(|c| [c[0], c[1]])),
                1 => {}
                _ => tris = tri_list(op, idx),
            }
            let n = pos.len() as u32;
            tris.retain(|t| t.iter().all(|&i| i < n));
            lines.retain(|t| t.iter().all(|&i| i < n));
            out.push(Geo { pos, nrm: if nr.len() == verts.len() { nr } else { Vec::new() }, uv: uv.to_vec(), tris, lines, material: material.to_string() });
        };
        if let Some(m) = &o.mesh {
            for (i, sm) in m.subs.iter().enumerate() {
                let mat = o.sub_mat.get(i).map(|x| x.as_str()).unwrap_or(&sm.material);
                push(&sm.verts, &sm.nrm, &sm.uv, &sm.idx, if sm.op == 0 { 4 } else { sm.op }, mat, &sm.bones);
            }
        }
        if let Some(m) = &o.manual {
            for sec in &m.sections {
                let nz: Vec<[f32; 3]> = if sec.norms.iter().any(|n| n.iter().any(|v| *v != 0.0)) { sec.norms.clone() } else { Vec::new() };
                push(&sec.verts, &nz, &sec.uvs, &sec.idx, sec.op, &sec.material, &[]);
            }
        }
        out
    }

    /// Все треугольники объекта в мировых координатах.
    pub(super) fn world_tris(&self, id: u32) -> Vec<[V3; 3]> {
        let mut out = Vec::new();
        for g in self.object_geos(id) {
            for t in &g.tris {
                out.push([g.pos[t[0] as usize], g.pos[t[1] as usize], g.pos[t[2] as usize]]);
            }
        }
        out
    }

    fn material_look(&self, name: &str) -> MatLook {
        let mut m = MatLook {
            amb: [1.0; 4],
            diff: [0.8, 0.8, 0.8, 1.0],
            emis: [0.0, 0.0, 0.0, 1.0],
            lit: true,
            cull: 2,
            depth_check: true,
            depth_write: true,
            blend: (0, 1),
            reject: (1, 0.0),
            wire: false,
            tex: None,
        };
        let Some(&mid) = self.materials.get(&name.to_ascii_lowercase()) else { return m };
        let Some(pass) = self.objs[&mid].children.first().and_then(|t| self.objs[t].children.first()).copied() else { return m };
        let v = |k: &str| self.pf(pass, k);
        let c4 = |x: Vec<f64>, d: [f64; 4]| -> [f64; 4] { if x.len() >= 4 { [x[0], x[1], x[2], x[3]] } else { d } };
        m.amb = c4(v("pass.ambient"), m.amb);
        m.diff = c4(v("pass.diffuse"), m.diff);
        m.emis = c4(v("pass.selfillumination"), m.emis);
        m.lit = v("pass.lightingenabled")[0] != 0.0;
        m.cull = v("pass.cullingmode")[0] as u8;
        m.depth_check = v("pass.depthcheckenabled")[0] != 0.0;
        m.depth_write = v("pass.depthwriteenabled")[0] != 0.0;
        let b = v("pass.sceneblending");
        m.blend = (b[0] as u8, b[1] as u8);
        let ar = v("pass.alpharejectsettings");
        m.reject = (ar[0] as u8, ar[1]);
        m.wire = v("pass.polygonmode")[0] == 2.0;
        if let Some(tu) = self.objs[&pass].children.first() {
            if let Some(t) = self.objs[tu].st.get("textureunitstate.texture") {
                m.tex = self.textures.get(&t.to_ascii_lowercase()).cloned().flatten();
            }
        }
        m
    }

    /// Рисует все области окна рендеринга (или текстуры) программным
    /// растеризатором: свет, туман, текстуры, буфер глубины.
    pub fn render(&self, window: Option<u32>) -> Option<Frame> {
        let win = window.or_else(|| self.objs.iter().find(|(_, o)| o.kind == Kind::Window).map(|(k, _)| *k))?;
        if !matches!(self.objs.get(&win)?.kind, Kind::Window | Kind::Texture) {
            return None;
        }
        let ws = self.window_size(win);
        let (w, hh) = ((ws[0] as usize).clamp(1, 4096), (ws[1] as usize).clamp(1, 4096));
        let mut rgb = vec![0u8; w * hh * 3];
        for vp in self.viewports_of(win) {
            self.render_viewport(vp, w, hh, &mut rgb);
        }
        Some(Frame { width: w, height: hh, rgb })
    }

    fn lights(&self, scene: u32) -> Vec<Light> {
        let mut v = Vec::new();
        for (id, o) in &self.objs {
            if o.kind != Kind::Light || o.scene != scene || o.parent == 0 || self.pf(*id, "movable.visible")[0] == 0.0 {
                continue;
            }
            let (p, r, _) = self.derived(o.parent);
            let c = self.pf(*id, "light.diffusecolour");
            let pw = self.pf(*id, "light.powerscale")[0];
            let at = self.pf(*id, "light.attenuation");
            let sp = self.pf(*id, "light.spotlightrange");
            v.push(Light {
                pos: p,
                dir: unit(qrot(r, [0.0, 0.0, -1.0])),
                kind: self.pf(*id, "light.type").first().copied().unwrap_or(0.0) as u8,
                col: [c[0] * pw, c[1] * pw, c[2] * pw],
                atten: [at[0], at[1], at[2], at[3]],
                spot: if sp.len() >= 3 { [sp[0], sp[1], sp[2]] } else { [0.5, 0.8, 1.0] },
            });
        }
        v
    }

    fn light_at(&self, lights: &[Light], p: V3, n: V3, two_sided: bool) -> V3 {
        let mut acc = [0.0; 3];
        for l in lights {
            let (ldir, att) = if l.kind == 1 {
                ([-l.dir[0], -l.dir[1], -l.dir[2]], 1.0)
            } else {
                let d = sub(l.pos, p);
                let dist = len(d);
                if dist > l.atten[0] {
                    continue;
                }
                let a = 1.0 / (l.atten[1] + l.atten[2] * dist + l.atten[3] * dist * dist).max(1e-6);
                let u = unit(d);
                let mut spot = 1.0;
                if l.kind == 2 {
                    let cosang = dot([-u[0], -u[1], -u[2]], l.dir).clamp(-1.0, 1.0);
                    // в справке подписи внутреннего и внешнего углов перепутаны: берём меньший как внутренний
                    let (ci, co) = ((l.spot[0].min(l.spot[1]) / 2.0).cos(), (l.spot[0].max(l.spot[1]) / 2.0).cos());
                    spot = if cosang >= ci { 1.0 } else if cosang <= co { 0.0 } else { ((cosang - co) / (ci - co).max(1e-9)).powf(l.spot[2].max(0.0)) };
                }
                (u, a.min(1.0) * spot)
            };
            let nl = dot(n, ldir);
            let d = if two_sided { nl.abs() } else { nl.max(0.0) };
            for k in 0..3 {
                acc[k] += d * att * l.col[k];
            }
        }
        acc
    }

    fn render_viewport(&self, vp: u32, w: usize, hh: usize, rgb: &mut [u8]) {
        let Some(cam) = self.objs[&vp].f.get("camera").map(|c| c[0] as u32) else { return };
        let Some((cp, cr)) = self.camera_pose(cam) else { return };
        let dim = {
            let d = self.pf(vp, "viewport.dimensions");
            if d.len() == 4 { d } else { vec![0.0, 0.0, 1.0, 1.0] }
        };
        let (x0, y0) = ((dim[0].clamp(0.0, 1.0) * w as f64) as usize, (dim[1].clamp(0.0, 1.0) * hh as f64) as usize);
        let vw = ((dim[2].clamp(0.0, 1.0) * w as f64) as usize).min(w - x0.min(w - 1)).max(1);
        let vh = ((dim[3].clamp(0.0, 1.0) * hh as f64) as usize).min(hh - y0.min(hh - 1)).max(1);
        let (x0, y0) = (x0.min(w - 1), y0.min(hh - 1));
        let bg = self.pf(vp, "viewport.backgroundcolour");
        let bgc = [0, 1, 2].map(|i| (bg.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0) * 255.0) as u8);
        for y in y0..y0 + vh {
            for x in x0..x0 + vw {
                rgb[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(&bgc);
            }
        }
        let scene = self.objs[&cam].scene;
        self.draw_sky(scene, cr, cam, rgb, w, (x0, y0, vw, vh));
        let near = self.pf(cam, "camera.nearclipdistance")[0].max(1e-4);
        let th = (self.pf(cam, "camera.fov")[0] / 2.0).tan();
        let fog = self.objs.get(&scene).and_then(|s| s.f.get("scene.fog")).filter(|f| f.len() >= 7 && f[0] != 0.0).map(|f| [f[0], f[1], f[2], f[3], f[4], f[5], f[6]]);
        let camc = Cam { near, th, aspect: vw as f64 / vh as f64, ortho: self.pf(cam, "camera.projectiontype")[0] == 0.0, fog };
        let cam_wire = self.pf(cam, "camera.polygonmode")[0] == 2.0;
        let ambient = self.objs.get(&scene).and_then(|s| s.f.get("scene.ambientlight")).cloned().unwrap_or_else(|| vec![0.5; 3]);
        let lights = self.lights(scene);
        let inv = qconj(cr);
        let mut surf = Surf { rgb, full_w: w, x0, y0, vw, vh, depth: vec![f32::INFINITY; vw * vh] };

        struct Item {
            dist: f64,
            geo: Geo,
            mat: MatLook,
        }
        let mut items: Vec<Item> = Vec::new();
        for (id, o) in &self.objs {
            if o.scene != scene || !matches!(o.kind, Kind::Entity | Kind::ManualObject) || o.parent == 0 || self.pf(*id, "movable.visible")[0] == 0.0 {
                continue;
            }
            for g in self.object_geos(*id) {
                let mat = self.material_look(&g.material);
                let c = g.pos.first().copied().unwrap_or([0.0; 3]);
                items.push(Item { dist: len(sub(c, cp)), geo: g, mat });
            }
        }
        items.sort_by(|a, b| {
            let (ta, tb) = (a.mat.blend != (0, 1), b.mat.blend != (0, 1));
            ta.cmp(&tb).then(if ta { b.dist.partial_cmp(&a.dist).unwrap_or(std::cmp::Ordering::Equal) } else { std::cmp::Ordering::Equal })
        });
        let amb = [ambient.first().copied().unwrap_or(0.5), ambient.get(1).copied().unwrap_or(0.5), ambient.get(2).copied().unwrap_or(0.5)];
        for it in &items {
            let g = &it.geo;
            let m = &it.mat;
            let two = m.cull == 1;
            // цвета вершин: свет считается по мировым координатам
            let vcol: Vec<[f64; 3]> = if g.nrm.len() == g.pos.len() && !g.nrm.is_empty() {
                (0..g.pos.len()).map(|i| self.vertex_color(m, &lights, amb, g.pos[i], g.nrm[i], two)).collect()
            } else {
                Vec::new()
            };
            let to_cam = |v: V3| qrot(inv, sub(v, cp));
            for t in &g.tris {
                let ids = [t[0] as usize, t[1] as usize, t[2] as usize];
                let (a, b, c) = (g.pos[ids[0]], g.pos[ids[1]], g.pos[ids[2]]);
                let fnorm = unit(cross(sub(b, a), sub(c, a)));
                let pvs: Vec<Pv> = ids
                    .iter()
                    .map(|&i| {
                        let col = if vcol.is_empty() { self.vertex_color(m, &lights, amb, g.pos[i], fnorm, two) } else { vcol[i] };
                        let uv = g.uv.get(i).map(|u| [u[0] as f64, u[1] as f64]).unwrap_or([0.0; 2]);
                        Pv { p: to_cam(g.pos[i]), col, uv }
                    })
                    .collect();
                let wire = m.wire || cam_wire;
                for poly in clip_near(&pvs, camc.near) {
                    if wire {
                        for k in 0..poly.len() {
                            draw_line(&mut surf, &camc, poly[k], poly[(k + 1) % poly.len()], m);
                        }
                    } else {
                        for k in 1..poly.len().saturating_sub(1) {
                            draw_tri(&mut surf, &camc, [poly[0], poly[k], poly[k + 1]], m);
                        }
                    }
                }
            }
            for l in &g.lines {
                let mk = |i: u32| {
                    let i = i as usize;
                    Pv { p: to_cam(g.pos[i]), col: if vcol.is_empty() { [m.diff[0], m.diff[1], m.diff[2]] } else { vcol[i] }, uv: [0.0; 2] }
                };
                for poly in clip_near(&[mk(l[0]), mk(l[1])], camc.near) {
                    if poly.len() == 2 {
                        draw_line(&mut surf, &camc, poly[0], poly[1], m);
                    }
                }
            }
        }
        // плоскости (BillboardSet): лицом к камере
        for (id, o) in &self.objs {
            if o.scene != scene || o.kind != Kind::BillboardSet || o.parent == 0 || self.pf(*id, "movable.visible")[0] == 0.0 {
                continue;
            }
            let (np, nr, ns) = self.derived(o.parent);
            let dim = self.pf(*id, "billboardset.defaultdimensions");
            let (hw, hh2) = (dim[0] / 2.0, dim[1] / 2.0);
            let mut m = self.material_look(o.st.get("billboardset.materialname").map(|s| s.as_str()).unwrap_or(""));
            m.lit = false;
            m.cull = 1;
            m.depth_write = false;
            let ids: Vec<u32> = if o.children.is_empty() { vec![*id] } else { o.children.clone() };
            for b in ids {
                let bp = self.pf(b, "billboard.position");
                let bp = if bp.len() == 3 { bp } else { vec![0.0; 3] };
                let wp = qrot(nr, [bp[0] * ns[0], bp[1] * ns[1], bp[2] * ns[2]]);
                let c = to_cam_pt(inv, cp, [np[0] + wp[0], np[1] + wp[1], np[2] + wp[2]]);
                let rot = self.pf(b, "billboard.rotation").first().copied().unwrap_or(0.0);
                let col = self.pf(b, "billboard.colour");
                let col = if col.len() == 4 { col } else { vec![1.0; 4] };
                m.diff = [col[0], col[1], col[2], col[3]];
            draw_cam_quad(&mut surf, &camc, &m, c, hw, hh2, rot);
            }
        }
        // частицы: плоскости лицом к камере
        for (id, o) in &self.objs {
            if o.scene != scene || o.kind != Kind::ParticleSystem {
                continue;
            }
            let (Some(ps), Some(tpl)) = (self.pstate.get(id), o.st.get("particlesystem.template").and_then(|n| self.ptemplates.get(&n.to_ascii_lowercase()))) else { continue };
            let mut m = self.material_look(&tpl.material);
            m.lit = false;
            m.cull = 1;
            m.depth_write = false;
            let mut order: Vec<&fx::Particle> = ps.parts.iter().collect();
            order.sort_by(|a, b| {
                let (da, db) = (len(sub(a.pos, cp)), len(sub(b.pos, cp)));
                db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
            });
            for p in order {
                m.diff = p.col;
                draw_cam_quad(&mut surf, &camc, &m, to_cam_pt(inv, cp, p.pos), p.w / 2.0, p.h / 2.0, 0.0);
            }
        }
        // подписи MovableText
        for (id, o) in &self.objs {
            if o.kind != Kind::MovableText || o.parent == 0 || self.pf(*id, "movable.visible")[0] == 0.0 || self.objs.get(&o.parent).map(|n| n.scene) != Some(scene) {
                continue;
            }
            let cap = o.st.get("movabletext.caption").cloned().unwrap_or_default();
            let (np, nr, ns) = self.derived(o.parent);
            let lt = self.pf(*id, "movabletext.localtranslation");
            let gt = self.pf(*id, "movabletext.globaltranslation");
            let lt = if lt.len() == 3 { lt } else { vec![0.0; 3] };
            let gt = if gt.len() == 3 { gt } else { vec![0.0; 3] };
            let lw = qrot(nr, [lt[0] * ns[0], lt[1] * ns[1], lt[2] * ns[2]]);
            let c = to_cam_pt(inv, cp, [np[0] + lw[0] + gt[0], np[1] + lw[1] + gt[1], np[2] + lw[2] + gt[2]]);
            if c[2] > -camc.near {
                continue;
            }
            let sp = project(&camc, &surf, c);
            let chh = self.pf(*id, "movabletext.characterheight")[0];
            let px = (chh / (2.0 * sp[2] * camc.th) * vh as f64).clamp(1.0, 256.0);
            let al = self.pf(*id, "movabletext.textalignment");
            let (ha, va) = (al.first().copied().unwrap_or(0.0), al.get(1).copied().unwrap_or(0.0));
            let tw = fx::text_width(&cap, px);
            let x = if ha == 1.0 { sp[0] - tw / 2.0 } else { sp[0] };
            let y = match va as i64 {
                1 => sp[1] - px,
                2 => sp[1] - px / 2.0,
                _ => sp[1],
            };
            let col = self.pf(*id, "movabletext.color");
            let col = if col.len() == 4 { col } else { vec![1.0; 4] };
            put_text(&mut surf, &cap, x, y, px, [col[0], col[1], col[2]], col[3]);
        }
        // наложения (Overlay)
        if self.pf(vp, "viewport.overlaysenabled")[0] != 0.0 {
            self.draw_overlays(&mut surf);
        }
    }

    fn draw_sky(&self, scene: u32, cr: Q, cam: u32, rgb: &mut [u8], w: usize, vp: (usize, usize, usize, usize)) {
        let Some(sc) = self.objs.get(&scene) else { return };
        if !sc.f.get("scene.skybox").is_some_and(|v| v[0] != 0.0) {
            return;
        }
        let Some(&mid) = sc.st.get("scene.skybox").and_then(|n| self.materials.get(&n.to_ascii_lowercase())) else { return };
        let Some(pass) = self.objs[&mid].children.first().and_then(|t| self.objs[t].children.first()).copied() else { return };
        let Some(cubic) = self.objs[&pass].children.first().and_then(|tu| self.objs[tu].st.get("textureunitstate.cubic")) else { return };
        let faces: Vec<Option<Arc<Tex>>> = cubic.split('|').map(|n| self.textures.get(&n.to_ascii_lowercase()).cloned().flatten()).collect();
        if faces.len() != 6 || faces.iter().all(|f| f.is_none()) {
            return;
        }
        let th = (self.pf(cam, "camera.fov")[0] / 2.0).tan();
        let (x0, y0, vw, vh) = vp;
        let aspect = vw as f64 / vh as f64;
        for y in 0..vh {
            for x in 0..vw {
                let nx = (x as f64 + 0.5) / vw as f64 * 2.0 - 1.0;
                let ny = 1.0 - (y as f64 + 0.5) / vh as f64 * 2.0;
                let d = qrot(cr, [nx * th * aspect, ny * th, -1.0]);
                let (ax, ay, az) = (d[0].abs(), d[1].abs(), d[2].abs());
                // грани: fr -Z, bk +Z, lf -X, rt +X, up +Y, dn -Y
                let (face, u, v) = if az >= ax && az >= ay {
                    if d[2] < 0.0 { (0, d[0] / az, -d[1] / az) } else { (1, -d[0] / az, -d[1] / az) }
                } else if ax >= ay {
                    if d[0] < 0.0 { (2, -d[2] / ax, -d[1] / ax) } else { (3, d[2] / ax, -d[1] / ax) }
                } else if d[1] > 0.0 {
                    (4, d[0] / ay, -d[2] / ay)
                } else {
                    (5, d[0] / ay, d[2] / ay)
                };
                if let Some(tx) = &faces[face] {
                    let c = texel(tx, (u + 1.0) / 2.0, (v + 1.0) / 2.0);
                    let o = ((y0 + y) * w + x0 + x) * 3;
                    for k in 0..3 {
                        rgb[o + k] = (c[k] * 255.0) as u8;
                    }
                }
            }
        }
    }

    /// Прямоугольник элемента наложения (пиксели области) внутри прямоугольника родителя.
    pub(super) fn element_rect(&self, e: u32, parent: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let mode = self.pf(e, "overlayelement.metricsmode").first().copied().unwrap_or(0.0);
        let pos = self.pf(e, "overlayelement.position");
        let size = self.pf(e, "overlayelement.size");
        let (l, t) = (pos.first().copied().unwrap_or(0.0), pos.get(1).copied().unwrap_or(0.0));
        let (ew, eh) = (size.first().copied().unwrap_or(0.0), size.get(1).copied().unwrap_or(0.0));
        let (pw, ph) = (parent.2, parent.3);
        let (mut x, mut y, w, h) = if mode == 1.0 { (l, t, ew, eh) } else { (l * pw, t * ph, ew * pw, eh * ph) };
        let al = self.pf(e, "overlayelement.alignment");
        let (ha, va) = (al.first().copied().unwrap_or(0.0), al.get(1).copied().unwrap_or(0.0));
        x += match ha as i64 {
            1 => parent.0 + pw / 2.0,
            2 => parent.0 + pw - w,
            _ => parent.0,
        };
        y += match va as i64 {
            1 => parent.1 + ph / 2.0,
            2 => parent.1 + ph - h,
            _ => parent.1,
        };
        (x, y, w, h)
    }

    pub(super) fn find_element(&self, e: u32, parent: (f64, f64, f64, f64), px: f64, py: f64, depth: usize) -> Option<u32> {
        if depth > 16 || self.pf(e, "overlayelement.visible")[0] == 0.0 {
            return None;
        }
        let r = self.element_rect(e, parent);
        for &c in self.objs[&e].children.iter().rev() {
            if let Some(x) = self.find_element(c, r, px, py, depth + 1) {
                return Some(x);
            }
        }
        (px >= r.0 && py >= r.1 && px < r.0 + r.2 && py < r.1 + r.3).then_some(e)
    }

    fn draw_overlays(&self, s: &mut Surf) {
        let mut ovs: Vec<(f64, u32)> = self.objs.iter().filter(|(_, o)| o.kind == Kind::Overlay).map(|(i, _)| (self.pf(*i, "overlay.zorder").first().copied().unwrap_or(0.0), *i)).collect();
        ovs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, ov) in ovs {
            if self.pf(ov, "overlay.visible").first().copied().unwrap_or(0.0) == 0.0 {
                continue;
            }
            let sc = self.pf(ov, "overlay.scale");
            let sr = self.pf(ov, "overlay.scroll");
            let xf = ([sc[0], sc.get(1).copied().unwrap_or(1.0)], [sr.first().copied().unwrap_or(0.0), sr.get(1).copied().unwrap_or(0.0)]);
            for &e in &self.objs[&ov].children {
                self.draw_element(s, e, (0.0, 0.0, s.vw as f64, s.vh as f64), xf, 0);
            }
        }
    }

    fn draw_element(&self, s: &mut Surf, e: u32, parent: (f64, f64, f64, f64), xf: ([f64; 2], [f64; 2]), depth: usize) {
        let o = &self.objs[&e];
        if depth > 16 || self.pf(e, "overlayelement.visible")[0] == 0.0 {
            return;
        }
        let mode = self.pf(e, "overlayelement.metricsmode").first().copied().unwrap_or(0.0);
        let rect = self.element_rect(e, parent);
        let (x, y, w, h) = rect;
        // преобразование наложения: масштаб относительно центра экрана и прокрутка
        let (cx, cy) = (s.vw as f64 / 2.0, s.vh as f64 / 2.0);
        let (fx_, fy_) = (cx + (x - cx) * xf.0[0] + xf.1[0] * 0.5 * s.vw as f64, cy + (y - cy) * xf.0[1] - xf.1[1] * 0.5 * s.vh as f64);
        let (dw, dh) = (w * xf.0[0], h * xf.0[1]);
        let ty = o.st.get("overlayelement.type").map(|v| v.as_str()).unwrap_or("");
        if ty == "textarea" {
            let cap = o.st.get("overlayelement.caption").cloned().unwrap_or_default();
            let chp = o.params.iter().find(|p| p.0 == "char_height").and_then(|p| p.1.parse::<f64>().ok()).unwrap_or(0.03);
            let px = (if mode == 1.0 { chp } else { chp * s.vh as f64 } * xf.0[1]).clamp(1.0, 256.0);
            let c = self.pf(e, "overlayelement.colour");
            put_text(s, &cap, fx_, fy_, px, [c[0], c[1], c[2]], c.get(3).copied().unwrap_or(1.0));
        } else if let Some(mat) = o.st.get("overlayelement.materialname").filter(|m| !m.is_empty()) {
            let mut m = self.material_look(mat);
            m.lit = false;
            m.depth_check = false;
            m.depth_write = false;
            let oc = Cam { near: 1.0, th: 1.0, aspect: 1.0, ortho: false, fog: None };
            let (x1, y1) = ((fx_ + dw).ceil().min(s.vw as f64), (fy_ + dh).ceil().min(s.vh as f64));
            for py in fy_.floor().max(0.0) as usize..y1.max(0.0) as usize {
                for px in fx_.floor().max(0.0) as usize..x1.max(0.0) as usize {
                    let mut col = [m.diff[0], m.diff[1], m.diff[2]];
                    let mut a = m.diff[3];
                    if let Some(tx) = &m.tex {
                        let tc = texel(tx, (px as f64 + 0.5 - fx_) / dw.max(1.0), (py as f64 + 0.5 - fy_) / dh.max(1.0));
                        for k in 0..3 {
                            col[k] *= tc[k];
                        }
                        a *= tc[3];
                    }
                    put(s, &oc, px, py, 0.0, col, a, &m);
                }
            }
        }
        for &c in &o.children {
            self.draw_element(s, c, rect, xf, depth + 1);
        }
    }

    fn vertex_color(&self, m: &MatLook, lights: &[Light], amb: V3, p: V3, n: V3, two: bool) -> [f64; 3] {
        if !m.lit {
            return [1.0, 1.0, 1.0];
        }
        let d = self.light_at(lights, p, n, two);
        [0, 1, 2].map(|k| (m.emis[k] + m.amb[k] * amb[k] * m.diff[k] + m.diff[k] * d[k]).clamp(0.0, 1.0))
    }
}

fn to_cam_pt(inv: Q, cp: V3, v: V3) -> V3 {
    qrot(inv, sub(v, cp))
}

fn clip_near(poly: &[Pv], near: f64) -> Vec<Vec<Pv>> {
    // камера смотрит вдоль -Z: видимо z <= -near
    let inside = |v: &Pv| v.p[2] <= -near;
    let mut out: Vec<Pv> = Vec::new();
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if poly.len() == 2 && i == 1 {
            break;
        }
        if inside(&a) {
            out.push(a);
        }
        if inside(&a) != inside(&b) {
            let t = (-near - a.p[2]) / (b.p[2] - a.p[2]);
            out.push(Pv { p: [a.p[0] + (b.p[0] - a.p[0]) * t, a.p[1] + (b.p[1] - a.p[1]) * t, -near], col: lerp3(a.col, b.col, t), uv: [a.uv[0] + (b.uv[0] - a.uv[0]) * t, a.uv[1] + (b.uv[1] - a.uv[1]) * t] });
        }
    }
    if poly.len() == 2 {
        // отрезок: оставшиеся точки
        let ins: Vec<Pv> = poly.iter().copied().filter(inside).collect();
        if ins.len() == 2 {
            return vec![ins];
        }
        return if out.len() >= 2 { vec![out[..2].to_vec()] } else { Vec::new() };
    }
    if out.len() >= 3 { vec![out] } else { Vec::new() }
}

fn project(c: &Cam, s: &Surf, p: V3) -> [f64; 3] {
    let z = -p[2];
    if c.ortho {
        let hh = c.th * c.near;
        [(p[0] / (hh * c.aspect) * 0.5 + 0.5) * s.vw as f64, (0.5 - p[1] / hh * 0.5) * s.vh as f64, z]
    } else {
        [(p[0] / (z * c.th * c.aspect) * 0.5 + 0.5) * s.vw as f64, (0.5 - p[1] / (z * c.th) * 0.5) * s.vh as f64, z]
    }
}

fn factor(f: u8, src: [f64; 4], dst: [f64; 4]) -> [f64; 3] {
    let one = |v: f64| [v; 3];
    match f {
        0 => one(1.0),
        1 => one(0.0),
        2 => [dst[0], dst[1], dst[2]],
        3 => [src[0], src[1], src[2]],
        4 => [1.0 - dst[0], 1.0 - dst[1], 1.0 - dst[2]],
        5 => [1.0 - src[0], 1.0 - src[1], 1.0 - src[2]],
        6 => one(dst[3]),
        7 => one(src[3]),
        8 => one(1.0 - dst[3]),
        _ => one(1.0 - src[3]),
    }
}

fn put(s: &mut Surf, c: &Cam, x: usize, y: usize, z: f64, col: [f64; 3], a: f64, m: &MatLook) {
    let di = y * s.vw + x;
    if m.depth_check && z as f32 >= s.depth[di] {
        return;
    }
    let ok = match m.reject.0 {
        0 => false,
        1 => true,
        2 => a * 255.0 < m.reject.1,
        3 => a * 255.0 <= m.reject.1,
        4 => (a * 255.0 - m.reject.1).abs() < 0.5,
        5 => (a * 255.0 - m.reject.1).abs() >= 0.5,
        6 => a * 255.0 >= m.reject.1,
        _ => a * 255.0 > m.reject.1,
    };
    if !ok {
        return;
    }
    let mut col = col;
    if let Some(f) = c.fog {
        let k = match f[0] as u8 {
            1 => (-f[4] * z).exp(),
            2 => (-(f[4] * z).powi(2)).exp(),
            _ => ((f[6] - z) / (f[6] - f[5]).max(1e-9)).clamp(0.0, 1.0),
        }
        .clamp(0.0, 1.0);
        col = lerp3([f[1], f[2], f[3]], col, k);
    }
    let o = ((s.y0 + y) * s.full_w + s.x0 + x) * 3;
    let dst = [s.rgb[o] as f64 / 255.0, s.rgb[o + 1] as f64 / 255.0, s.rgb[o + 2] as f64 / 255.0, 1.0];
    let src = [col[0], col[1], col[2], a];
    let (fs, fd) = (factor(m.blend.0, src, dst), factor(m.blend.1, src, dst));
    for k in 0..3 {
        s.rgb[o + k] = ((src[k] * fs[k] + dst[k] * fd[k]).clamp(0.0, 1.0) * 255.0) as u8;
    }
    if m.depth_write {
        s.depth[di] = z as f32;
    }
}

fn texel(t: &Tex, u: f64, v: f64) -> [f64; 4] {
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let x = ((fu * t.w as f64) as usize).min(t.w - 1);
    let y = ((fv * t.h as f64) as usize).min(t.h - 1);
    let p = &t.rgba[(y * t.w + x) * 4..(y * t.w + x) * 4 + 4];
    [p[0] as f64 / 255.0, p[1] as f64 / 255.0, p[2] as f64 / 255.0, p[3] as f64 / 255.0]
}

fn draw_tri(s: &mut Surf, c: &Cam, t: [Pv; 3], m: &MatLook) {
    let p = [project(c, s, t[0].p), project(c, s, t[1].p), project(c, s, t[2].p)];
    let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
    if area.abs() < 1e-9 || !area.is_finite() {
        return;
    }
    // вершины против часовой стрелки на экране (y вниз) дают area < 0
    if (m.cull == 2 && area > 0.0) || (m.cull == 3 && area < 0.0) {
        return;
    }
    let minx = p.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min).floor().max(0.0);
    let maxx = p.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max).ceil().min(s.vw as f64 - 1.0);
    let miny = p.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min).floor().max(0.0);
    let maxy = p.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max).ceil().min(s.vh as f64 - 1.0);
    if !(minx.is_finite() && maxx.is_finite() && miny.is_finite() && maxy.is_finite()) || minx > maxx || miny > maxy {
        return;
    }
    let iz = [1.0 / p[0][2], 1.0 / p[1][2], 1.0 / p[2][2]];
    for y in miny as usize..=maxy as usize {
        for x in minx as usize..=maxx as usize {
            let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
            let w0 = ((p[1][0] - px) * (p[2][1] - py) - (p[2][0] - px) * (p[1][1] - py)) / area;
            let w1 = ((p[2][0] - px) * (p[0][1] - py) - (p[0][0] - px) * (p[2][1] - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let isum = w0 * iz[0] + w1 * iz[1] + w2 * iz[2];
            if isum <= 0.0 {
                continue;
            }
            let z = 1.0 / isum;
            let (a0, a1, a2) = (w0 * iz[0] * z, w1 * iz[1] * z, w2 * iz[2] * z);
            let mut col = [0, 1, 2].map(|k| t[0].col[k] * a0 + t[1].col[k] * a1 + t[2].col[k] * a2);
            let mut alpha = m.diff[3];
            if let Some(tx) = &m.tex {
                let u = t[0].uv[0] * a0 + t[1].uv[0] * a1 + t[2].uv[0] * a2;
                let v = t[0].uv[1] * a0 + t[1].uv[1] * a1 + t[2].uv[1] * a2;
                let tc = texel(tx, u, v);
                for k in 0..3 {
                    col[k] *= tc[k];
                }
                alpha *= tc[3];
            }
            put(s, c, x, y, z, col, alpha, m);
        }
    }
}

fn draw_line(s: &mut Surf, c: &Cam, a: Pv, b: Pv, m: &MatLook) {
    let (pa, pb) = (project(c, s, a.p), project(c, s, b.p));
    let n = ((pb[0] - pa[0]).abs().max((pb[1] - pa[1]).abs()).ceil() as usize).clamp(1, 8192);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let (x, y) = (pa[0] + (pb[0] - pa[0]) * t, pa[1] + (pb[1] - pa[1]) * t);
        if x < 0.0 || y < 0.0 || x >= s.vw as f64 || y >= s.vh as f64 {
            continue;
        }
        let z = 1.0 / (1.0 / pa[2] + (1.0 / pb[2] - 1.0 / pa[2]) * t);
        let col = lerp3(a.col, b.col, t);
        let col = if m.lit { [col[0].max(0.2), col[1].max(0.2), col[2].max(0.2)] } else { col };
        put(s, c, x as usize, y as usize, z - 1e-3, col, m.diff[3], m);
    }
}

fn put_text(s: &mut Surf, text: &str, x: f64, y: f64, px: f64, col: [f64; 3], alpha: f64) {
    let c = [(col[0].clamp(0.0, 1.0) * 255.0) as u8, (col[1].clamp(0.0, 1.0) * 255.0) as u8, (col[2].clamp(0.0, 1.0) * 255.0) as u8];
    let (vw, vh, full, x0, y0) = (s.vw as i64, s.vh as i64, s.full_w, s.x0, s.y0);
    let a = alpha.clamp(0.0, 1.0);
    let rgb = &mut *s.rgb;
    fx::draw_text(text, x, y, px, |px_, py_| {
        if px_ >= 0 && py_ >= 0 && px_ < vw && py_ < vh {
            let o = ((y0 + py_ as usize) * full + x0 + px_ as usize) * 3;
            for k in 0..3 {
                rgb[o + k] = (c[k] as f64 * a + rgb[o + k] as f64 * (1.0 - a)) as u8;
            }
        }
    });
}

fn draw_cam_quad(s: &mut Surf, camc: &Cam, m: &MatLook, c: V3, hw: f64, hh: f64, rot: f64) {
    let (sn, cs) = rot.sin_cos();
    let corner = |dx: f64, dy: f64, u: f64, v: f64| {
        let (rx, ry) = (dx * cs - dy * sn, dx * sn + dy * cs);
        Pv { p: [c[0] + rx, c[1] + ry, c[2]], col: [m.diff[0], m.diff[1], m.diff[2]], uv: [u, v] }
    };
    let q = [corner(-hw, hh, 0.0, 0.0), corner(hw, hh, 1.0, 0.0), corner(hw, -hh, 1.0, 1.0), corner(-hw, -hh, 0.0, 1.0)];
    for tri in [[q[0], q[3], q[2]], [q[0], q[2], q[1]]] {
        for poly in clip_near(&tri, camc.near) {
            for k in 1..poly.len().saturating_sub(1) {
                draw_tri(s, camc, [poly[0], poly[k], poly[k + 1]], m);
            }
        }
    }
}
