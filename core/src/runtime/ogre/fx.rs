//! Системы частиц (.particle), скрипты наложений (.overlay), шрифт 5x7.

use super::*;

const MAX_PARTICLES: usize = 5000;
const MAX_TEMPLATES: usize = 512;

#[derive(Debug, Clone, Default)]
pub struct Emitter {
    pub kind: String,
    pub rate: f64,
    pub angle: f64,
    pub dir: V3,
    pub vel: (f64, f64),
    pub ttl: (f64, f64),
    pub col: ([f64; 4], [f64; 4]),
    pub pos: V3,
    pub size: V3,
}

#[derive(Debug, Clone, Default)]
pub struct Affector {
    pub kind: String,
    pub v: [f64; 4],
}

#[derive(Debug, Clone, Default)]
pub struct PTemplate {
    pub material: String,
    pub w: f64,
    pub h: f64,
    pub quota: usize,
    pub emitters: Vec<Emitter>,
    pub affectors: Vec<Affector>,
}

#[derive(Debug, Clone, Default)]
pub struct Particle {
    pub pos: V3,
    pub vel: V3,
    pub age: f64,
    pub ttl: f64,
    pub col: [f64; 4],
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Default)]
pub struct PState {
    pub parts: Vec<Particle>,
    acc: f64,
    rng: u64,
}

fn rnd(s: &mut u64) -> f64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    (*s >> 11) as f64 / (1u64 << 53) as f64
}

fn nums(args: &[&str]) -> Vec<f64> {
    args.iter().filter_map(|a| a.parse::<f64>().ok()).filter(|v| v.is_finite()).collect()
}

impl Ogre {
    pub(super) fn parse_particles(&mut self, text: &str) {
        let t = tokenize(text);
        let mut i = 0;
        while i < t.len() {
            if t[i] != "particle_system" || i + 1 >= t.len() {
                i += 1;
                continue;
            }
            let name = t[i + 1].to_ascii_lowercase();
            let mut j = i + 2;
            while j < t.len() && t[j] != "{" {
                j += 1;
            }
            let end = skip_block(&t, j);
            if j < t.len() && self.ptemplates.len() < MAX_TEMPLATES {
                let body = &t[j + 1..end.saturating_sub(1).max(j + 1)];
                self.ptemplates.insert(name, Arc::new(parse_template(body)));
            }
            i = end.max(i + 1);
        }
    }

    /// Шаг всех систем частиц на `dt` секунд.
    pub(super) fn step_particles(&mut self, dt: f64) {
        let ids: Vec<u32> = self.objs.iter().filter(|(_, o)| o.kind == Kind::ParticleSystem && o.parent != 0).map(|(i, _)| *i).collect();
        self.pstate.retain(|k, _| self.objs.contains_key(k));
        for id in ids {
            if self.pf(id, "movable.visible")[0] == 0.0 {
                continue;
            }
            let Some(tpl) = self.objs[&id].st.get("particlesystem.template").and_then(|n| self.ptemplates.get(&n.to_ascii_lowercase())).cloned() else { continue };
            let (np, nr, ns) = self.derived(self.objs[&id].parent);
            let st = self.pstate.entry(id).or_insert_with(|| PState { parts: Vec::new(), acc: 0.0, rng: 0x9E37_79B9_7F4A_7C15 ^ id as u64 });
            let quota = tpl.quota.clamp(1, MAX_PARTICLES);
            for e in &tpl.emitters {
                st.acc += e.rate.max(0.0) * dt;
                while st.acc >= 1.0 {
                    st.acc -= 1.0;
                    if st.parts.len() >= quota {
                        st.acc = st.acc.min(1.0);
                        break;
                    }
                    let r = &mut st.rng;
                    let off = if e.kind == "box" { [(rnd(r) - 0.5) * e.size[0], (rnd(r) - 0.5) * e.size[1], (rnd(r) - 0.5) * e.size[2]] } else { [0.0; 3] };
                    let lp = [e.pos[0] + off[0], e.pos[1] + off[1], e.pos[2] + off[2]];
                    let wp = qrot(nr, [lp[0] * ns[0], lp[1] * ns[1], lp[2] * ns[2]]);
                    // направление: конус с углом `angle` (градусы)
                    let d = unit(e.dir);
                    let d = if len(d) == 0.0 { [0.0, 1.0, 0.0] } else { d };
                    let helper = if d[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
                    let u = unit(cross(d, helper));
                    let v = cross(d, u);
                    let ang = e.angle.to_radians() * rnd(r);
                    let az = rnd(r) * std::f64::consts::TAU;
                    let dir = [0, 1, 2].map(|k| d[k] * ang.cos() + (u[k] * az.cos() + v[k] * az.sin()) * ang.sin());
                    let speed = e.vel.0 + (e.vel.1 - e.vel.0) * rnd(r);
                    let wd = qrot(nr, dir);
                    let c = [0, 1, 2, 3].map(|k| e.col.0[k] + (e.col.1[k] - e.col.0[k]) * rnd(r));
                    st.parts.push(Particle {
                        pos: [np[0] + wp[0], np[1] + wp[1], np[2] + wp[2]],
                        vel: [wd[0] * speed, wd[1] * speed, wd[2] * speed],
                        age: 0.0,
                        ttl: (e.ttl.0 + (e.ttl.1 - e.ttl.0) * rnd(r)).max(0.01),
                        col: c,
                        w: tpl.w,
                        h: tpl.h,
                    });
                }
            }
            for p in st.parts.iter_mut() {
                for a in &tpl.affectors {
                    match a.kind.as_str() {
                        "linearforce" => {
                            for k in 0..3 {
                                p.vel[k] += a.v[k] * dt;
                            }
                        }
                        "colourfader" => {
                            for k in 0..4 {
                                p.col[k] = (p.col[k] + a.v[k] * dt).clamp(0.0, 1.0);
                            }
                        }
                        "scaler" => {
                            p.w = (p.w + a.v[0] * dt).max(0.0);
                            p.h = (p.h + a.v[0] * dt).max(0.0);
                        }
                        _ => {}
                    }
                }
                for k in 0..3 {
                    p.pos[k] += p.vel[k] * dt;
                }
                p.age += dt;
            }
            st.parts.retain(|p| p.age < p.ttl);
        }
    }

    /// Наложения из скрипта .overlay: `Имя { zorder N  container Panel(Имя) { ... } }`.
    pub(super) fn parse_overlays(&mut self, text: &str) {
        let t = tokenize(text);
        let mut i = 0;
        while i + 1 < t.len() {
            if t[i + 1] != "{" || matches!(t[i].as_str(), "container" | "element") {
                i += 1;
                continue;
            }
            let name = t[i].clone();
            let end = skip_block(&t, i + 1);
            let body = &t[i + 2..end.saturating_sub(1).max(i + 2)];
            if !self.overlays.contains_key(&name.to_ascii_lowercase()) && self.overlays.len() < MAX_TEMPLATES {
                let ov = self.add(Obj::new(Kind::Overlay, &clean_name(&name), 0));
                if ov != 0 {
                    self.overlays.insert(name.to_ascii_lowercase(), ov);
                    self.fill_overlay(ov, body, 0);
                }
            }
            i = end.max(i + 1);
        }
    }

    fn fill_overlay(&mut self, owner: u32, t: &[String], depth: usize) {
        let mut i = 0;
        while i < t.len() {
            let w = t[i].as_str();
            if w == "zorder" && i + 1 < t.len() && self.objs[&owner].kind == Kind::Overlay {
                if let Ok(z) = t[i + 1].parse::<f64>() {
                    self.objs.get_mut(&owner).unwrap().f.insert("overlay.zorder".into(), vec![z]);
                }
                i += 2;
                continue;
            }
            if (w == "container" || w == "element") && depth < 16 {
                // заголовок: Тип(Имя) или Тип (Имя) до "{"
                let mut j = i + 1;
                let mut head = String::new();
                while j < t.len() && t[j] != "{" {
                    head.push_str(&t[j]);
                    j += 1;
                }
                let (ty, nm) = match (head.find('('), head.rfind(')')) {
                    (Some(a), Some(b)) if a < b => (head[..a].to_ascii_lowercase(), head[a + 1..b].to_string()),
                    _ => (head.to_ascii_lowercase(), String::new()),
                };
                let end = skip_block(t, j);
                let body = &t[(j + 1).min(t.len())..end.saturating_sub(1).max((j + 1).min(t.len()))];
                let key = nm.to_ascii_lowercase();
                if !nm.is_empty() && !self.elements.contains_key(&key) && self.elements.len() < MAX_OBJECTS / 2 {
                    let e = self.add(Obj::new(Kind::Element, &clean_name(&nm), 0));
                    if e != 0 {
                        self.elements.insert(key, e);
                        let o = self.objs.get_mut(&e).unwrap();
                        o.container = w == "container";
                        o.st.insert("overlayelement.type".into(), ty);
                        o.parent = owner;
                        self.objs.get_mut(&owner).unwrap().children.push(e);
                        self.fill_element(e, body);
                        self.fill_overlay(e, body, depth + 1);
                    }
                }
                i = end.max(i + 1);
                continue;
            }
            i += 1;
        }
    }

    fn fill_element(&mut self, e: u32, t: &[String]) {
        // только свойства верхнего уровня блока (вложенные блоки пропускаются)
        let mut i = 0;
        let mut depth: usize = 0;
        while i < t.len() {
            let w = t[i].as_str();
            if w == "{" {
                depth += 1;
            } else if w == "}" {
                depth = depth.saturating_sub(1);
            }
            if depth > 0 || w == "{" || w == "}" {
                i += 1;
                continue;
            }
            let mut args = Vec::new();
            let mut j = i + 1;
            while j < t.len() && args.len() < 6 && !is_overlay_key(&t[j]) && t[j] != "{" && t[j] != "}" {
                args.push(t[j].as_str());
                j += 1;
            }
            let n = nums(&args);
            let o = self.objs.get_mut(&e).unwrap();
            match w {
                "metrics_mode" => {
                    o.f.insert("overlayelement.metricsmode".into(), vec![if args.first() == Some(&"pixels") { 1.0 } else { 0.0 }]);
                }
                "left" if !n.is_empty() => {
                    let p = o.f.entry("overlayelement.position".into()).or_insert_with(|| vec![0.0, 0.0]);
                    p[0] = n[0];
                }
                "top" if !n.is_empty() => {
                    let p = o.f.entry("overlayelement.position".into()).or_insert_with(|| vec![0.0, 0.0]);
                    p[1] = n[0];
                }
                "width" if !n.is_empty() => {
                    let p = o.f.entry("overlayelement.size".into()).or_insert_with(|| vec![0.0, 0.0]);
                    p[0] = n[0];
                }
                "height" if !n.is_empty() => {
                    let p = o.f.entry("overlayelement.size".into()).or_insert_with(|| vec![0.0, 0.0]);
                    p[1] = n[0];
                }
                "material" if !args.is_empty() => {
                    o.st.insert("overlayelement.materialname".into(), clean_name(args[0]));
                }
                "caption" => {
                    o.st.insert("overlayelement.caption".into(), args.join(" ").chars().take(MAX_TEXT).collect());
                }
                "horz_align" => {
                    let a = match args.first().copied() {
                        Some("center") => 1.0,
                        Some("right") => 2.0,
                        _ => 0.0,
                    };
                    o.f.entry("overlayelement.alignment".into()).or_insert_with(|| vec![0.0, 0.0])[0] = a;
                }
                "vert_align" => {
                    let a = match args.first().copied() {
                        Some("center") => 1.0,
                        Some("bottom") => 2.0,
                        _ => 0.0,
                    };
                    o.f.entry("overlayelement.alignment".into()).or_insert_with(|| vec![0.0, 0.0])[1] = a;
                }
                "colour" if n.len() >= 3 => {
                    o.f.insert("overlayelement.colour".into(), vec![n[0], n[1], n[2], n.get(3).copied().unwrap_or(1.0)]);
                }
                "char_height" | "font_name" | "alignment" if !args.is_empty() => {
                    if o.params.len() < MAX_PARAMS {
                        o.params.push((w.to_string(), args[0].to_string()));
                    }
                }
                _ => {}
            }
            i = j.max(i + 1);
        }
    }
}

fn is_overlay_key(w: &str) -> bool {
    matches!(w, "metrics_mode" | "left" | "top" | "width" | "height" | "material" | "caption" | "horz_align" | "vert_align" | "colour" | "char_height" | "font_name" | "alignment" | "container" | "element" | "zorder")
}

fn parse_template(t: &[String]) -> PTemplate {
    let mut tpl = PTemplate { w: 100.0, h: 100.0, quota: 10, ..Default::default() };
    let mut i = 0;
    while i < t.len() {
        let w = t[i].as_str();
        if (w == "emitter" || w == "affector") && i + 1 < t.len() {
            let kind = t[i + 1].to_ascii_lowercase();
            let mut j = i + 2;
            while j < t.len() && t[j] != "{" {
                j += 1;
            }
            let end = skip_block(t, j);
            let body = &t[(j + 1).min(t.len())..end.saturating_sub(1).max((j + 1).min(t.len()))];
            if w == "emitter" && tpl.emitters.len() < 16 {
                tpl.emitters.push(parse_emitter(&kind, body));
            } else if tpl.affectors.len() < 16 {
                tpl.affectors.push(parse_affector(&kind, body));
            }
            i = end.max(i + 1);
            continue;
        }
        let val = t.get(i + 1).map(|s| s.as_str()).unwrap_or("");
        match w {
            "material" => tpl.material = clean_name(val),
            "particle_width" => tpl.w = val.parse().unwrap_or(100.0),
            "particle_height" => tpl.h = val.parse().unwrap_or(100.0),
            "quota" => tpl.quota = val.parse().unwrap_or(10),
            _ => {}
        }
        i += 1;
    }
    tpl
}

fn props(t: &[String]) -> Vec<(String, Vec<f64>)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < t.len() {
        let mut j = i + 1;
        let mut v = Vec::new();
        while j < t.len() && t[j].parse::<f64>().is_ok() && v.len() < 4 {
            v.push(t[j].parse::<f64>().unwrap_or(0.0));
            j += 1;
        }
        let nonfinite = v.iter().any(|x| !x.is_finite());
        if !nonfinite && out.len() < 64 {
            out.push((t[i].to_ascii_lowercase(), v));
        }
        i = j.max(i + 1);
    }
    out
}

fn parse_emitter(kind: &str, t: &[String]) -> Emitter {
    let mut e = Emitter { kind: kind.to_string(), rate: 10.0, dir: [0.0, 1.0, 0.0], vel: (1.0, 1.0), ttl: (5.0, 5.0), col: ([1.0; 4], [1.0; 4]), ..Default::default() };
    let rgba = |v: &[f64]| [v.first().copied().unwrap_or(1.0), v.get(1).copied().unwrap_or(1.0), v.get(2).copied().unwrap_or(1.0), v.get(3).copied().unwrap_or(1.0)];
    for (k, v) in props(t) {
        let x = v.first().copied().unwrap_or(0.0);
        match k.as_str() {
            "angle" => e.angle = x,
            "emission_rate" => e.rate = x,
            "time_to_live" => e.ttl = (x, x),
            "time_to_live_min" => e.ttl.0 = x,
            "time_to_live_max" => e.ttl.1 = x,
            "velocity" => e.vel = (x, x),
            "velocity_min" => e.vel.0 = x,
            "velocity_max" => e.vel.1 = x,
            "direction" if v.len() >= 3 => e.dir = [v[0], v[1], v[2]],
            "position" if v.len() >= 3 => e.pos = [v[0], v[1], v[2]],
            "colour" => e.col = (rgba(&v), rgba(&v)),
            "colour_range_start" => e.col.0 = rgba(&v),
            "colour_range_end" => e.col.1 = rgba(&v),
            "width" => e.size[0] = x,
            "height" => e.size[1] = x,
            "depth" => e.size[2] = x,
            _ => {}
        }
    }
    e
}

fn parse_affector(kind: &str, t: &[String]) -> Affector {
    let mut a = Affector { kind: kind.to_string(), ..Default::default() };
    for (k, v) in props(t) {
        let x = v.first().copied().unwrap_or(0.0);
        match (kind, k.as_str()) {
            ("linearforce", "force_vector") if v.len() >= 3 => a.v = [v[0], v[1], v[2], 0.0],
            ("colourfader", "red") => a.v[0] = x,
            ("colourfader", "green") => a.v[1] = x,
            ("colourfader", "blue") => a.v[2] = x,
            ("colourfader", "alpha") => a.v[3] = x,
            ("scaler", "rate") => a.v[0] = x,
            _ => {}
        }
    }
    a
}

/// Шрифт 5x7: 95 знаков ASCII 32..126, по 5 столбцов (младший бит - верх).
pub const FONT5X7: [u8; 475] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5F, 0x00, 0x00, 0x00, 0x07, 0x00, 0x07, 0x00, 0x14, 0x7F, 0x14, 0x7F, 0x14, 0x24, 0x2A, 0x7F, 0x2A, 0x12, 0x23, 0x13, 0x08, 0x64, 0x62,
    0x36, 0x49, 0x55, 0x22, 0x50, 0x00, 0x05, 0x03, 0x00, 0x00, 0x00, 0x1C, 0x22, 0x41, 0x00, 0x00, 0x41, 0x22, 0x1C, 0x00, 0x08, 0x2A, 0x1C, 0x2A, 0x08, 0x08, 0x08, 0x3E, 0x08, 0x08,
    0x00, 0x50, 0x30, 0x00, 0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x60, 0x60, 0x00, 0x00, 0x20, 0x10, 0x08, 0x04, 0x02, 0x3E, 0x51, 0x49, 0x45, 0x3E, 0x00, 0x42, 0x7F, 0x40, 0x00,
    0x42, 0x61, 0x51, 0x49, 0x46, 0x21, 0x41, 0x45, 0x4B, 0x31, 0x18, 0x14, 0x12, 0x7F, 0x10, 0x27, 0x45, 0x45, 0x45, 0x39, 0x3C, 0x4A, 0x49, 0x49, 0x30, 0x01, 0x71, 0x09, 0x05, 0x03,
    0x36, 0x49, 0x49, 0x49, 0x36, 0x06, 0x49, 0x49, 0x29, 0x1E, 0x00, 0x36, 0x36, 0x00, 0x00, 0x00, 0x56, 0x36, 0x00, 0x00, 0x08, 0x14, 0x22, 0x41, 0x00, 0x14, 0x14, 0x14, 0x14, 0x14,
    0x00, 0x41, 0x22, 0x14, 0x08, 0x02, 0x01, 0x51, 0x09, 0x06, 0x32, 0x49, 0x79, 0x41, 0x3E, 0x7E, 0x11, 0x11, 0x11, 0x7E, 0x7F, 0x49, 0x49, 0x49, 0x36, 0x3E, 0x41, 0x41, 0x41, 0x22,
    0x7F, 0x41, 0x41, 0x22, 0x1C, 0x7F, 0x49, 0x49, 0x49, 0x41, 0x7F, 0x09, 0x09, 0x09, 0x01, 0x3E, 0x41, 0x49, 0x49, 0x7A, 0x7F, 0x08, 0x08, 0x08, 0x7F, 0x00, 0x41, 0x7F, 0x41, 0x00,
    0x20, 0x40, 0x41, 0x3F, 0x01, 0x7F, 0x08, 0x14, 0x22, 0x41, 0x7F, 0x40, 0x40, 0x40, 0x40, 0x7F, 0x02, 0x0C, 0x02, 0x7F, 0x7F, 0x04, 0x08, 0x10, 0x7F, 0x3E, 0x41, 0x41, 0x41, 0x3E,
    0x7F, 0x09, 0x09, 0x09, 0x06, 0x3E, 0x41, 0x51, 0x21, 0x5E, 0x7F, 0x09, 0x19, 0x29, 0x46, 0x46, 0x49, 0x49, 0x49, 0x31, 0x01, 0x01, 0x7F, 0x01, 0x01, 0x3F, 0x40, 0x40, 0x40, 0x3F,
    0x1F, 0x20, 0x40, 0x20, 0x1F, 0x3F, 0x40, 0x38, 0x40, 0x3F, 0x63, 0x14, 0x08, 0x14, 0x63, 0x07, 0x08, 0x70, 0x08, 0x07, 0x61, 0x51, 0x49, 0x45, 0x43, 0x00, 0x7F, 0x41, 0x41, 0x00,
    0x02, 0x04, 0x08, 0x10, 0x20, 0x00, 0x41, 0x41, 0x7F, 0x00, 0x04, 0x02, 0x01, 0x02, 0x04, 0x40, 0x40, 0x40, 0x40, 0x40, 0x00, 0x01, 0x02, 0x04, 0x00, 0x20, 0x54, 0x54, 0x54, 0x78,
    0x7F, 0x48, 0x44, 0x44, 0x38, 0x38, 0x44, 0x44, 0x44, 0x20, 0x38, 0x44, 0x44, 0x48, 0x7F, 0x38, 0x54, 0x54, 0x54, 0x18, 0x08, 0x7E, 0x09, 0x01, 0x02, 0x08, 0x14, 0x54, 0x54, 0x3C,
    0x7F, 0x08, 0x04, 0x04, 0x78, 0x00, 0x44, 0x7D, 0x40, 0x00, 0x20, 0x40, 0x44, 0x3D, 0x00, 0x00, 0x7F, 0x10, 0x28, 0x44, 0x00, 0x41, 0x7F, 0x40, 0x00, 0x7C, 0x04, 0x18, 0x04, 0x78,
    0x7C, 0x08, 0x04, 0x04, 0x78, 0x38, 0x44, 0x44, 0x44, 0x38, 0x7C, 0x14, 0x14, 0x14, 0x08, 0x08, 0x14, 0x14, 0x18, 0x7C, 0x7C, 0x08, 0x04, 0x04, 0x08, 0x48, 0x54, 0x54, 0x54, 0x20,
    0x04, 0x3F, 0x44, 0x40, 0x20, 0x3C, 0x40, 0x40, 0x20, 0x7C, 0x1C, 0x20, 0x40, 0x20, 0x1C, 0x3C, 0x40, 0x30, 0x40, 0x3C, 0x44, 0x28, 0x10, 0x28, 0x44, 0x0C, 0x50, 0x50, 0x50, 0x3C,
    0x44, 0x64, 0x54, 0x4C, 0x44, 0x00, 0x08, 0x36, 0x41, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x00, 0x41, 0x36, 0x08, 0x00, 0x08, 0x08, 0x2A, 0x1C, 0x08,
];

/// Рисует текст через `put(x, y)` для каждого закрашенного пикселя; высота знака `px` пикселей.
/// Возвращает ширину строки в пикселях.
pub fn draw_text(text: &str, x: f64, y: f64, px: f64, mut put: impl FnMut(i64, i64)) -> f64 {
    let scale = (px / 7.0).max(1.0);
    let adv = 6.0 * scale;
    let mut cx = x;
    for ch in text.chars().take(512) {
        let c = ch as u32;
        let idx = if (32..127).contains(&c) { (c - 32) as usize } else { 31 };
        let cyr = cyr_glyph(ch);
        for col in 0..5 {
            let bits = match cyr {
                Some(g) => g[col],
                None => FONT5X7[idx * 5 + col],
            };
            for row in 0..7 {
                if bits >> row & 1 == 1 {
                    let (x0, y0) = (cx + col as f64 * scale, y + row as f64 * scale);
                    for dy in 0..scale.ceil() as i64 {
                        for dx in 0..scale.ceil() as i64 {
                            put(x0 as i64 + dx, y0 as i64 + dy);
                        }
                    }
                }
            }
        }
        cx += adv;
    }
    cx - x
}

pub fn text_width(text: &str, px: f64) -> f64 {
    text.chars().take(512).count() as f64 * 6.0 * (px / 7.0).max(1.0)
}

/// Кириллица для шрифта 5x7: заглавные формы, строчные рисуются так же.
/// Буквы, похожие на латинские, берутся из латинской таблицы.
fn cyr_glyph(ch: char) -> Option<[u8; 5]> {
    let c = ch as u32;
    let up = match c {
        0x430..=0x44f => c - 0x20,
        0x451 => 0x401,
        _ => c,
    };
    let rows: [&str; 7] = match up {
        0x410 | 0x412 | 0x415 | 0x41a | 0x41c | 0x41d | 0x41e | 0x420 | 0x421 | 0x422 | 0x423 | 0x425 => {
            let l = match up {
                0x410 => 'A',
                0x412 => 'B',
                0x415 => 'E',
                0x41a => 'K',
                0x41c => 'M',
                0x41d => 'H',
                0x41e => 'O',
                0x420 => 'P',
                0x421 => 'C',
                0x422 => 'T',
                0x423 => 'Y',
                _ => 'X',
            };
            let i = (l as usize - 32) * 5;
            let mut g = [0u8; 5];
            g.copy_from_slice(&FONT5X7[i..i + 5]);
            return Some(g);
        }
        0x411 => ["#####", "#....", "####.", "#...#", "#...#", "#...#", "####."],
        0x413 => ["#####", "#....", "#....", "#....", "#....", "#....", "#...."],
        0x414 => ["..##.", ".#.#.", "#..#.", "#..#.", "#####", "#...#", "#...#"],
        0x416 => ["#.#.#", "#.#.#", ".###.", "..#..", ".###.", "#.#.#", "#.#.#"],
        0x417 => ["####.", "....#", "...#.", "..##.", "....#", "#...#", ".###."],
        0x418 => ["#...#", "#...#", "#..##", "#.#.#", "##..#", "#...#", "#...#"],
        0x419 => [".#.#.", "#...#", "#..##", "#.#.#", "##..#", "#...#", "#...#"],
        0x41b => ["..###", ".#..#", ".#..#", ".#..#", ".#..#", "#...#", "#...#"],
        0x41f => ["#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#...#"],
        0x424 => ["..#..", ".###.", "#.#.#", "#.#.#", ".###.", "..#..", "..#.."],
        0x426 => ["#..#.", "#..#.", "#..#.", "#..#.", "#..#.", "#####", "....#"],
        0x427 => ["#...#", "#...#", "#...#", ".####", "....#", "....#", "....#"],
        0x428 => ["#.#.#", "#.#.#", "#.#.#", "#.#.#", "#.#.#", "#.#.#", "#####"],
        0x429 => ["#.#.#", "#.#.#", "#.#.#", "#.#.#", "#.#.#", "#####", "....#"],
        0x42a => ["##...", ".#...", ".###.", ".#..#", ".#..#", ".#..#", ".###."],
        0x42b => ["#...#", "#...#", "#...#", "###.#", "#.#.#", "#.#.#", "###.#"],
        0x42c => ["#....", "#....", "####.", "#...#", "#...#", "#...#", "####."],
        0x42d => ["####.", "....#", "....#", ".####", "....#", "....#", "####."],
        0x42e => ["#.##.", "#.#.#", "#.#.#", "###.#", "#.#.#", "#.#.#", "#.##."],
        0x42f => [".####", "#...#", "#...#", ".####", "..#.#", ".#..#", "#...#"],
        0x401 => [".#.#.", "#####", "#....", "####.", "#....", "#....", "#####"],
        _ => return None,
    };
    let mut g = [0u8; 5];
    for (r, row) in rows.iter().enumerate() {
        for (col, b) in row.bytes().enumerate().take(5) {
            if b == b'#' {
                g[col] |= 1 << r;
            }
        }
    }
    Some(g)
}

// ---- композиторы: цели и проходы ------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct CPass {
    /// clear, render_scene, render_quad (остальные виды игнорируются)
    pub kind: String,
    pub material: String,
    pub inputs: Vec<(usize, String)>,
    pub clear: [f64; 3],
}

#[derive(Debug, Default, Clone)]
pub struct CTarget {
    pub name: String,
    pub previous: bool,
    pub passes: Vec<CPass>,
}

#[derive(Debug, Default, Clone)]
pub struct CompDef {
    pub targets: Vec<CTarget>,
    pub output: CTarget,
}

fn parse_target(name: &str, b: &[String]) -> CTarget {
    let mut t = CTarget { name: name.to_ascii_lowercase(), ..Default::default() };
    let mut i = 0;
    while i < b.len() {
        match b[i].as_str() {
            "input" if b.get(i + 1).is_some_and(|x| x == "previous") => {
                t.previous = true;
                i += 2;
            }
            "pass" => {
                let kind = b.get(i + 1).cloned().unwrap_or_default();
                let mut j = i + 1;
                while j < b.len() && b[j] != "{" {
                    j += 1;
                }
                let end = skip_block(b, j).min(b.len());
                let body = if j < end { &b[j + 1..end.saturating_sub(1).max(j + 1)] } else { &b[0..0] };
                let mut p = CPass { kind: kind.clone(), ..Default::default() };
                let mut k = 0;
                while k < body.len() {
                    match body[k].as_str() {
                        "material" if k + 1 < body.len() => {
                            p.material = body[k + 1].clone();
                            k += 2;
                        }
                        "input" if k + 2 < body.len() => {
                            if let Ok(n) = body[k + 1].parse::<usize>() {
                                if p.inputs.len() < 16 {
                                    p.inputs.push((n, body[k + 2].to_ascii_lowercase()));
                                }
                            }
                            k += 3;
                        }
                        "colour_value" | "color_value" => {
                            for c in 0..3 {
                                p.clear[c] = body.get(k + 1 + c).and_then(|x| x.parse().ok()).unwrap_or(0.0);
                            }
                            k += 5;
                        }
                        _ => k += 1,
                    }
                }
                if t.passes.len() < 32 {
                    t.passes.push(p);
                }
                i = end.max(i + 1);
            }
            _ => i += 1,
        }
    }
    t
}

/// Разбирает все `compositor` из скрипта (берётся первая техника).
pub fn parse_compositors(t: &[String], out: &mut BTreeMap<String, Arc<CompDef>>) {
    let mut i = 0;
    while i < t.len() {
        if t[i] == "compositor" && i + 1 < t.len() {
            let mut j = i + 2;
            while j < t.len() && t[j] != "{" {
                j += 1;
            }
            if j >= t.len() {
                break;
            }
            let end = skip_block(t, j);
            let body = &t[j + 1..end.saturating_sub(1).max(j + 1)];
            let name = t[i + 1].to_ascii_lowercase();
            if out.len() < 256 && !out.contains_key(&name) {
                let mut def = CompDef::default();
                if let Some(tp) = body.iter().position(|w| w == "technique") {
                    let mut k = tp + 1;
                    while k < body.len() && body[k] != "{" {
                        k += 1;
                    }
                    let tend = skip_block(body, k).min(body.len());
                    let tb = if k < tend { &body[k + 1..tend.saturating_sub(1).max(k + 1)] } else { &body[0..0] };
                    let mut m = 0;
                    while m < tb.len() {
                        match tb[m].as_str() {
                            "target" | "target_output" => {
                                let out_t = tb[m] == "target_output";
                                let nm = if out_t { String::new() } else { tb.get(m + 1).cloned().unwrap_or_default() };
                                let mut b = m + 1;
                                while b < tb.len() && tb[b] != "{" {
                                    b += 1;
                                }
                                let e = skip_block(tb, b).min(tb.len());
                                let inner = if b < e { &tb[b + 1..e.saturating_sub(1).max(b + 1)] } else { &tb[0..0] };
                                let t2 = parse_target(&nm, inner);
                                if out_t {
                                    def.output = t2;
                                } else if def.targets.len() < 16 {
                                    def.targets.push(t2);
                                }
                                m = e.max(m + 1);
                            }
                            _ => m += 1,
                        }
                    }
                }
                out.insert(name, Arc::new(def));
            }
            i = end;
        } else {
            i += 1;
        }
    }
}
