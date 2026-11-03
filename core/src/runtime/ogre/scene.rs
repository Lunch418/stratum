//! Scene_Load: сцены формата dotScene (XML: environment, nodes, entity,
//! light, camera, plane, вложенные node).

use super::*;

const MAX_XML: usize = 8 << 20;
const MAX_ELEMENTS: usize = 100_000;
const MAX_DEPTH_XML: usize = 64;

#[derive(Debug, Default)]
struct El {
    name: String,
    attrs: Vec<(String, String)>,
    kids: Vec<El>,
}

impl El {
    fn a(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, v)| v.as_str())
    }
    fn f(&self, k: &str, d: f64) -> f64 {
        self.a(k).and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite()).unwrap_or(d)
    }
    fn kid(&self, n: &str) -> Option<&El> {
        self.kids.iter().find(|k| k.name.eq_ignore_ascii_case(n))
    }
    fn v3(&self, n: &str) -> Option<V3> {
        self.kid(n).map(|k| [k.f("x", 0.0), k.f("y", 0.0), k.f("z", 0.0)])
    }
    fn rgb(&self, n: &str) -> Option<V3> {
        self.kid(n).map(|k| [k.f("r", 0.0), k.f("g", 0.0), k.f("b", 0.0)])
    }
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

fn parse_xml(text: &str) -> Option<El> {
    let b = text.as_bytes();
    let mut p = 0;
    let mut stack: Vec<El> = vec![El::default()];
    let mut count = 0usize;
    while p < b.len() {
        if b[p] != b'<' {
            p += 1;
            continue;
        }
        if text[p..].starts_with("<!--") {
            p += text[p..].find("-->")? + 3;
            continue;
        }
        if text[p..].starts_with("<?") || text[p..].starts_with("<!") {
            p += text[p..].find('>')? + 1;
            continue;
        }
        let end = p + text[p..].find('>')?;
        // атрибуты могут содержать '>' только в кавычках: простой случай не поддерживаем
        let inner = &text[p + 1..end];
        p = end + 1;
        if let Some(name) = inner.strip_prefix('/') {
            let _ = name;
            if stack.len() < 2 {
                return None;
            }
            let done = stack.pop()?;
            stack.last_mut()?.kids.push(done);
            continue;
        }
        let selfclose = inner.ends_with('/');
        let inner = inner.trim_end_matches('/');
        let mut parts = inner.splitn(2, |c: char| c.is_whitespace());
        let name = parts.next()?.to_string();
        let rest = parts.next().unwrap_or("");
        let mut el = El { name, ..Default::default() };
        let rb = rest.as_bytes();
        let mut i = 0;
        while i < rb.len() {
            while i < rb.len() && rb[i].is_ascii_whitespace() {
                i += 1;
            }
            let ks = i;
            while i < rb.len() && rb[i] != b'=' && !rb[i].is_ascii_whitespace() {
                i += 1;
            }
            let key = &rest[ks..i];
            while i < rb.len() && (rb[i] == b'=' || rb[i].is_ascii_whitespace()) {
                i += 1;
            }
            if i >= rb.len() || (rb[i] != b'"' && rb[i] != b'\'') {
                break;
            }
            let q = rb[i];
            i += 1;
            let vs = i;
            while i < rb.len() && rb[i] != q {
                i += 1;
            }
            if i > rb.len() {
                break;
            }
            if !key.is_empty() && el.attrs.len() < 64 {
                el.attrs.push((key.to_string(), unescape(&rest[vs..i.min(rest.len())])));
            }
            i += 1;
        }
        count += 1;
        if count > MAX_ELEMENTS || stack.len() > MAX_DEPTH_XML {
            return None;
        }
        if selfclose {
            stack.last_mut()?.kids.push(el);
        } else {
            stack.push(el);
        }
    }
    if stack.len() != 1 {
        return None;
    }
    let root = stack.pop()?;
    root.kids.into_iter().next()
}

impl Ogre {
    fn call_in(&mut self, fx: &mut Effects, name: &str, args: &[Value]) -> Value {
        let Some(sig) = sigs().get(name) else { return Value::Float(0.0) };
        let mut outs = Vec::new();
        self.run(fx, name, sig, args, &mut outs)
    }

    /// Загружает сцену dotScene в сцену `sc`. Возвращает число созданных объектов.
    pub(super) fn load_scene(&mut self, fx: &mut Effects, sc: u32, file: &str) -> usize {
        let Some(p) = self.find_resource(fx, file) else {
            self.fail(format!("Scene_Load: файл '{file}' не найден"));
            return 0;
        };
        match std::fs::metadata(&p) {
            Ok(m) if m.len() <= MAX_XML as u64 => {}
            _ => {
                self.fail("Scene_Load: файл слишком велик".into());
                return 0;
            }
        }
        let Ok(bytes) = std::fs::read(&p) else { return 0 };
        let Some(root) = parse_xml(&String::from_utf8_lossy(&bytes)) else {
            self.fail(format!("Scene_Load: '{file}' - неверный XML"));
            return 0;
        };
        if !root.name.eq_ignore_ascii_case("scene") {
            self.fail("Scene_Load: корневой элемент не scene".into());
            return 0;
        }
        let mut made = 0usize;
        if let Some(env) = root.kid("environment") {
            if let Some(a) = env.rgb("colourAmbient") {
                self.call_in(fx, "scene_setambientlight", &[h(sc), fl(a[0]), fl(a[1]), fl(a[2])]);
            }
            if let Some(f) = env.kid("fog") {
                let mode = match f.a("mode").unwrap_or("none").to_ascii_lowercase().as_str() {
                    "linear" => 3.0,
                    "exp" => 1.0,
                    "exp2" => 2.0,
                    _ => 0.0,
                };
                let c = f.rgb("colour").unwrap_or([0.5; 3]);
                self.call_in(fx, "scene_setfog", &[h(sc), fl(mode), fl(c[0]), fl(c[1]), fl(c[2]), fl(f.f("expDensity", 0.001)), fl(f.f("linearStart", 0.0)), fl(f.f("linearEnd", 1.0))]);
            }
        }
        let rootnode = self.root_node(sc);
        if let Some(nodes) = root.kid("nodes") {
            for n in nodes.kids.iter().filter(|k| k.name.eq_ignore_ascii_case("node")) {
                made += self.load_node(fx, sc, rootnode, n, 0);
            }
        }
        made
    }

    fn load_node(&mut self, fx: &mut Effects, sc: u32, parent: u32, n: &El, depth: usize) -> usize {
        if depth > MAX_DEPTH_XML {
            return 0;
        }
        let name = n.a("name").unwrap_or("");
        let id = self.call_in(fx, "scenenode_create", &[h(sc), Value::Str(name.into())]).as_float() as u32;
        if id == 0 {
            return 0;
        }
        let mut made = 1;
        self.call_in(fx, "node_addchild", &[h(parent), h(id)]);
        if let Some(p) = n.v3("position") {
            self.call_in(fx, "node_setposition", &[h(id), fl(p[0]), fl(p[1]), fl(p[2])]);
        }
        if let Some(s) = n.v3("scale") {
            self.call_in(fx, "node_setscale", &[h(id), fl(s[0]), fl(s[1]), fl(s[2])]);
        }
        if let Some(r) = n.kid("rotation").or_else(|| n.kid("quaternion")) {
            let q = [r.f("qw", 1.0), r.f("qx", 0.0), r.f("qy", 0.0), r.f("qz", 0.0)];
            self.call_in(fx, "node_setrotationquaternion", &[h(id), fl(q[0]), fl(q[1]), fl(q[2]), fl(q[3])]);
        }
        for k in &n.kids {
            match k.name.to_ascii_lowercase().as_str() {
                "node" => made += self.load_node(fx, sc, id, k, depth + 1),
                "entity" => {
                    let e = self.call_in(fx, "entity_create", &[h(sc), Value::Str(k.a("name").unwrap_or("").into()), Value::Str(k.a("meshFile").unwrap_or("").into())]).as_float() as u32;
                    if e != 0 {
                        made += 1;
                        self.call_in(fx, "scenenode_attachobject", &[h(id), h(e)]);
                        if let Some(subs) = k.kid("subentities") {
                            for s in subs.kids.iter().filter(|s| s.name.eq_ignore_ascii_case("subentity")) {
                                if let Some(m) = s.a("materialName") {
                                    self.call_in(fx, "entity_setmaterial", &[h(e), fl(s.f("index", 0.0)), Value::Str(m.into())]);
                                }
                            }
                        }
                        if k.a("castShadows").is_some_and(|v| v == "false") {
                            self.call_in(fx, "movable_setcastshadows", &[h(e), fl(0.0)]);
                        }
                    }
                }
                "light" => {
                    let l = self.call_in(fx, "light_create", &[h(sc), Value::Str(k.a("name").unwrap_or("").into())]).as_float() as u32;
                    if l != 0 {
                        made += 1;
                        let ty = match k.a("type").unwrap_or("point").to_ascii_lowercase().as_str() {
                            "directional" => 1.0,
                            "spot" | "spotlight" => 2.0,
                            _ => 0.0,
                        };
                        self.call_in(fx, "light_settype", &[h(l), fl(ty)]);
                        if let Some(c) = k.rgb("colourDiffuse") {
                            self.call_in(fx, "light_setdiffusecolour", &[h(l), fl(c[0]), fl(c[1]), fl(c[2])]);
                        }
                        if let Some(c) = k.rgb("colourSpecular") {
                            self.call_in(fx, "light_setspecularcolour", &[h(l), fl(c[0]), fl(c[1]), fl(c[2])]);
                        }
                        if let Some(a) = k.kid("lightAttenuation") {
                            let q = if a.a("quadric").is_some() { a.f("quadric", 0.0) } else { a.f("quadratic", 0.0) };
                            self.call_in(fx, "light_setattenuation", &[h(l), fl(a.f("range", 100000.0)), fl(a.f("constant", 1.0)), fl(a.f("linear", 0.0)), fl(q)]);
                        }
                        if let Some(r) = k.kid("lightRange") {
                            self.call_in(fx, "light_setspotlightrange", &[h(l), fl(r.f("inner", 0.5)), fl(r.f("outer", 0.8)), fl(r.f("falloff", 1.0))]);
                        }
                        if k.f("power", 1.0) != 1.0 {
                            self.call_in(fx, "light_setpowerscale", &[h(l), fl(k.f("power", 1.0))]);
                        }
                        let target = if let Some(d) = k.v3("normal").or_else(|| k.v3("direction")) {
                            let sub = self.call_in(fx, "scenenode_create", &[h(sc), Value::Str(String::new())]).as_float() as u32;
                            if sub != 0 {
                                self.call_in(fx, "node_addchild", &[h(id), h(sub)]);
                                let q = look_rotation(d);
                                self.call_in(fx, "node_setrotationquaternion", &[h(sub), fl(q[0]), fl(q[1]), fl(q[2]), fl(q[3])]);
                                sub
                            } else {
                                id
                            }
                        } else {
                            id
                        };
                        self.call_in(fx, "scenenode_attachobject", &[h(target), h(l)]);
                    }
                }
                "camera" => {
                    let c = self.call_in(fx, "camera_create", &[h(sc), Value::Str(k.a("name").unwrap_or("").into())]).as_float() as u32;
                    if c != 0 {
                        made += 1;
                        if k.f("fov", 0.0) > 0.0 {
                            self.call_in(fx, "camera_setfov", &[h(c), fl(k.f("fov", 0.78))]);
                        }
                        if let Some(cl) = k.kid("clipping") {
                            self.call_in(fx, "camera_setnearclipdistance", &[h(c), fl(cl.f("near", 100.0))]);
                            self.call_in(fx, "camera_setfarclipdistance", &[h(c), fl(cl.f("far", 100000.0))]);
                        }
                        self.call_in(fx, "scenenode_attachobject", &[h(id), h(c)]);
                    }
                }
                "plane" => {
                    if let Some(e) = self.make_plane(fx, sc, k) {
                        made += 1;
                        self.call_in(fx, "scenenode_attachobject", &[h(id), h(e)]);
                    }
                }
                _ => {}
            }
        }
        made
    }

    fn make_plane(&mut self, fx: &mut Effects, sc: u32, k: &El) -> Option<u32> {
        let name = k.a("name")?.to_string();
        let (w, hh) = (k.f("width", 100.0), k.f("height", 100.0));
        let xs = (k.f("xSegments", 1.0) as usize).clamp(1, 256);
        let ys = (k.f("ySegments", 1.0) as usize).clamp(1, 256);
        let n = unit(k.v3("normal").unwrap_or([0.0, 1.0, 0.0]));
        let up = unit(k.v3("upVector").unwrap_or([0.0, 0.0, -1.0]));
        if len(n) == 0.0 || len(up) == 0.0 || !(w.is_finite() && hh.is_finite()) {
            return None;
        }
        let xa = unit(cross(up, n));
        let ya = up;
        let d = k.f("distance", 0.0);
        let (ut, vt) = (k.f("uTile", 1.0), k.f("vTile", 1.0));
        let mut sm = SubMesh { material: k.a("material").unwrap_or("").to_string(), op: 4, ..Default::default() };
        for j in 0..=ys {
            for i in 0..=xs {
                let (fx_, fy) = (i as f64 / xs as f64, j as f64 / ys as f64);
                let v = [0, 1, 2].map(|c| (fx_ - 0.5) * w * xa[c] + (fy - 0.5) * hh * ya[c] - n[c] * d);
                sm.verts.push([v[0] as f32, v[1] as f32, v[2] as f32]);
                sm.nrm.push([n[0] as f32, n[1] as f32, n[2] as f32]);
                sm.uv.push([(fx_ * ut) as f32, (1.0 - fy * vt) as f32]);
            }
        }
        let row = xs + 1;
        for j in 0..ys {
            for i in 0..xs {
                let a = (j * row + i) as u32;
                let (b, c, e) = (a + 1, a + row as u32, a + row as u32 + 1);
                for t in [[a, b, e], [a, e, c]] {
                    let (p0, p1, p2) = (sm.verts[t[0] as usize], sm.verts[t[1] as usize], sm.verts[t[2] as usize]);
                    let f = cross(
                        [(p1[0] - p0[0]) as f64, (p1[1] - p0[1]) as f64, (p1[2] - p0[2]) as f64],
                        [(p2[0] - p0[0]) as f64, (p2[1] - p0[1]) as f64, (p2[2] - p0[2]) as f64],
                    );
                    if dot(f, n) >= 0.0 { sm.idx.extend_from_slice(&t) } else { sm.idx.extend_from_slice(&[t[0], t[2], t[1]]) }
                }
            }
        }
        let mut mesh = Mesh { min: [f32::MAX; 3], max: [f32::MIN; 3], ..Default::default() };
        for v in &sm.verts {
            for c in 0..3 {
                mesh.min[c] = mesh.min[c].min(v[c]);
                mesh.max[c] = mesh.max[c].max(v[c]);
            }
        }
        mesh.subs.push(sm);
        let key = format!("plane:{}:{}", sc, name.to_ascii_lowercase());
        self.meshes.insert(key.clone(), Arc::new(mesh));
        let e = self.call_in(fx, "entity_create", &[h(sc), Value::Str(name), Value::Str(key)]).as_float() as u32;
        (e != 0).then_some(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_parser_basics_and_limits() {
        let r = parse_xml("<?xml version=\"1.0\"?><!-- c --><scene a=\"1\"><nodes><node name=\"x &amp; y\"><position x=\"1\"/></node></nodes></scene>").unwrap();
        assert_eq!(r.name, "scene");
        let n = &r.kid("nodes").unwrap().kids[0];
        assert_eq!(n.a("name"), Some("x & y"));
        assert_eq!(n.v3("position"), Some([1.0, 0.0, 0.0]));
        assert!(parse_xml("<a><b></a>").is_none());
        assert!(parse_xml("<a>").is_none());
        let deep = "<a>".repeat(200) + &"</a>".repeat(200);
        assert!(parse_xml(&deep).is_none());
        assert!(parse_xml("").is_none());
    }
}
