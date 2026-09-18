//! JSON-API для IDE: состав проекта, тексты и переменные имиджей, схемы,
//! иконки, правки. Всё сериализуется вручную — внешних крейтов нет.

use super::{json_string, Shared};
use crate::formats::{cls, vdr};
use crate::gfx::{svg, Space};
use crate::lang;
use std::sync::{Arc, Mutex};

pub struct Response {
    pub status: &'static str,
    pub mime: &'static str,
    pub body: String,
}

fn json(body: String) -> Response {
    Response { status: "200 OK", mime: "application/json; charset=utf-8", body }
}

fn error(status: &'static str, message: &str) -> Response {
    Response { status, mime: "application/json; charset=utf-8", body: format!("{{\"error\":{}}}", json_string(message)) }
}

fn num(v: f64) -> String {
    if v.is_finite() { format!("{v}") } else { "0".into() }
}

fn class_json(c: &cls::Class, library: bool, model: Option<&lang::Model>) -> String {
    let vars: Vec<String> = c
        .vars
        .iter()
        .map(|v| {
            format!(
                "{{\"name\":{},\"type\":{},\"default\":{},\"description\":{},\"local\":{},\"flags\":{}}}",
                json_string(&v.name),
                json_string(&v.var_type),
                json_string(&v.default),
                json_string(&v.description),
                v.is_local(),
                v.flags
            )
        })
        .collect();
    let children: Vec<String> = c
        .children
        .iter()
        .map(|ch| {
            format!(
                "{{\"handle\":{},\"class\":{},\"name\":{},\"x\":{},\"y\":{}}}",
                ch.handle,
                json_string(&ch.class_name),
                json_string(&ch.name),
                num(ch.x),
                num(ch.y)
            )
        })
        .collect();
    let links: Vec<String> = c
        .links
        .iter()
        .map(|l| {
            let pairs: Vec<String> = l.vars.iter().map(|(a, b)| format!("[{},{}]", json_string(a), json_string(b))).collect();
            format!(
                "{{\"handle\":{},\"source\":{},\"target\":{},\"vars\":[{}]}}",
                l.handle,
                l.source,
                l.target,
                pairs.join(",")
            )
        })
        .collect();
    // переменные, объявленные только в тексте
    let declared: Vec<String> = model
        .map(|m| {
            m.declarations
                .iter()
                .flat_map(|d| d.names.iter().map(move |n| format!("{{\"name\":{},\"type\":{}}}", json_string(n), json_string(&d.var_type))))
                .collect()
        })
        .unwrap_or_default();
    format!(
        "{{\"name\":{},\"library\":{},\"description\":{},\"vars\":[{}],\"declared\":[{}],\"text\":{},\"children\":[{}],\"links\":[{}],\"hasIcon\":{},\"hasScheme\":{},\"hasImage\":{},\"source\":{}}}",
        json_string(&c.name),
        library,
        json_string(&c.description),
        vars.join(","),
        declared.join(","),
        json_string(&c.text),
        children.join(","),
        links.join(","),
        c.icon.is_some() || c.icon_file.is_some(),
        c.scheme.is_some(),
        c.image.is_some(),
        json_string(&c.source)
    )
}

fn parse_error_json(e: &lang::ParseError) -> String {
    format!("{{\"line\":{},\"column\":{},\"message\":{}}}", e.line, e.column, json_string(&e.message))
}

/// Иконка или графика имиджа как SVG.
fn picture_svg(blob: &[u8], name: &str) -> Option<String> {
    let pic = vdr::parse(blob, name).ok()?;
    let mut sp = Space::new(1, name);
    sp.load(&pic);
    // иконка: показываем всё, включая элементы схемы
    for o in sp.objects.values_mut() {
        o.scheme_element = false;
    }
    fit(&mut sp);
    Some(svg::render(&sp))
}

/// Подгоняет окно пространства под содержимое.
fn fit(sp: &mut Space) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for o in sp.objects.values() {
        if !matches!(o.shape, crate::gfx::Shape::Group { .. }) {
            x0 = x0.min(o.x);
            y0 = y0.min(o.y);
            x1 = x1.max(o.x + o.w);
            y1 = y1.max(o.y + o.h);
        }
    }
    if x0 < x1 && y0 < y1 {
        sp.origin = (x0, y0);
        sp.client = (x1 - x0, y1 - y0);
    }
}

pub fn handle(method: &str, path: &str, query: &str, body: &str, shared: &Arc<Mutex<Shared>>) -> Response {
    let rest = path.strip_prefix("/api/").unwrap_or("");
    let parts: Vec<&str> = rest.split('/').collect();
    match (method, parts.as_slice()) {
        ("GET", ["project"]) => {
            let s = shared.lock().unwrap();
            let p = &s.project;
            let classes: Vec<String> = p
                .classes
                .iter()
                .enumerate()
                .map(|(i, c)| class_json(c, i >= p.own_classes, s.models.get(&c.name.to_lowercase())))
                .collect();
            json(format!(
                "{{\"root\":{},\"dir\":{},\"native\":{},\"unsaved\":{},\"canUndo\":{},\"canRedo\":{},\"classes\":[{}]}}",
                json_string(&p.project.root),
                json_string(&p.dir.display().to_string()),
                crate::formats::native::is_native(&p.dir),
                s.unsaved,
                !s.history.is_empty(),
                !s.future.is_empty(),
                classes.join(",")
            ))
        }
        ("GET", ["class", name]) => {
            let name = super::url_decode(name);
            let s = shared.lock().unwrap();
            match s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) {
                Some(i) => json(class_json(&s.project.classes[i], i >= s.project.own_classes, s.models.get(&name.to_lowercase()))),
                None => error("404 Not Found", "нет такого имиджа"),
            }
        }
        ("GET", ["icon", name]) => {
            let name = super::url_decode(name);
            let s = shared.lock().unwrap();
            let Some(c) = s.project.class(&name) else { return error("404 Not Found", "нет такого имиджа") };
            // иконка либо встроена в .cls, либо это .vdr-файл, либо номер
            // ячейки в наборе .dbm
            let gfx = &s.sim.effects.gfx;
            let mut svg = c.icon.as_deref().and_then(|b| picture_svg(b, &name));
            if svg.is_none() {
                if let Some(file) = &c.icon_file {
                    if file.to_lowercase().ends_with(".vdr") {
                        svg = gfx.load_picture_file(file).and_then(|pic| {
                            let mut sp = Space::new(1, &name);
                            sp.load(&pic);
                            fit(&mut sp);
                            Some(svg::render(&sp))
                        });
                    } else if let Some(path) = gfx.find_file(file) {
                        svg = std::fs::read(path).ok().and_then(|bmp| sheet_icon(&bmp, c.icon_index.unwrap_or(0)));
                    }
                }
            }
            Response { status: "200 OK", mime: "image/svg+xml", body: svg.unwrap_or_else(|| default_icon(&name)) }
        }
        ("GET", ["scheme", name]) => {
            // графика листа схемы (без иконок имиджей — их рисует IDE сама)
            let name = super::url_decode(name);
            let s = shared.lock().unwrap();
            let Some(c) = s.project.class(&name) else { return error("404 Not Found", "нет такого имиджа") };
            let blob = c.image.as_deref().or(c.scheme.as_deref());
            match blob.and_then(|b| vdr::parse(b, &name).ok()) {
                Some(pic) => {
                    let mut sp = Space::new(1, &name);
                    sp.load(&pic);
                    let bounds = bounds_json(&sp);
                    Response { status: "200 OK", mime: "application/json; charset=utf-8", body: format!("{{\"bounds\":{bounds},\"svg\":{}}}", json_string(&svg::render(&sp))) }
                }
                None => json("{\"bounds\":null,\"svg\":\"\"}".into()),
            }
        }
        ("POST", ["class", name, "text"]) => {
            let name = super::url_decode(name);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            s.project.classes[i].text = body.to_string();
            match lang::parse(body) {
                Ok(m) => {
                    s.models.insert(name.to_lowercase(), m);
                    json("{\"ok\":true}".into())
                }
                Err(e) => json(format!("{{\"ok\":false,\"error\":{}}}", parse_error_json(&e))),
            }
        }
        ("POST", ["class", name, "vars"]) => {
            // тело: строки вида имя\tтип\tпо умолчанию\tописание
            let name = super::url_decode(name);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            let mut vars = Vec::new();
            for line in body.lines().filter(|l| !l.trim().is_empty()) {
                let f: Vec<&str> = line.split('\t').collect();
                vars.push(cls::Variable {
                    name: f.first().unwrap_or(&"").to_string(),
                    var_type: f.get(1).unwrap_or(&"FLOAT").to_string(),
                    default: f.get(2).unwrap_or(&"").to_string(),
                    description: f.get(3).unwrap_or(&"").to_string(),
                    flags: f.get(4).and_then(|v| v.parse().ok()).unwrap_or(0x20000),
                });
            }
            s.remember();
            s.project.classes[i].vars = vars;
            json("{\"ok\":true}".into())
        }
        ("POST", ["child", "move"]) => {
            // class, handle, x, y в query
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h), Some(x), Some(y)) = (get("class"), get("handle"), get("x"), get("y")) else {
                return error("400 Bad Request", "нужны class, handle, x, y");
            };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            let h: u16 = h.parse().unwrap_or(0);
            if s.project.classes[i].children.iter().any(|c| c.handle == h) {
                s.remember();
                let ch = s.project.classes[i].children.iter_mut().find(|c| c.handle == h).unwrap();
                ch.x = x.parse().unwrap_or(ch.x);
                ch.y = y.parse().unwrap_or(ch.y);
            }
            json("{\"ok\":true}".into())
        }
        // добавить имидж на схему: class (схема), child (класс), x, y, name
        ("POST", ["child", "add"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(child)) = (get("class"), get("child")) else {
                return error("400 Bad Request", "нужны class и child");
            };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            if s.project.classes.iter().all(|c| !c.name.eq_ignore_ascii_case(&child)) {
                return error("404 Not Found", "нет такого класса для вставки");
            }
            if i >= s.project.own_classes {
                return error("403 Forbidden", "библиотечный имидж не редактируется");
            }
            s.remember();
            let cls = &mut s.project.classes[i];
            let handle = cls.children.iter().map(|c| c.handle).max().unwrap_or(0) + 1;
            cls.children.push(cls::Child {
                class_name: child,
                handle,
                name: get("name").unwrap_or_default(),
                x: get("x").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                y: get("y").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                flags: 0,
            });
            json(format!("{{\"ok\":true,\"handle\":{handle}}}"))
        }
        ("POST", ["child", "remove"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h)) = (get("class"), get("handle")) else {
                return error("400 Bad Request", "нужны class и handle");
            };
            let h: u16 = h.parse().unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            cls.children.retain(|c| c.handle != h);
            // связи с удалённым экземпляром теряют смысл
            cls.links.retain(|l| l.source != h && l.target != h);
            json("{\"ok\":true}".into())
        }
        ("POST", ["child", "rename"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h)) = (get("class"), get("handle")) else {
                return error("400 Bad Request", "нужны class и handle");
            };
            let h: u16 = h.parse().unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            if let Some(ch) = s.project.classes[i].children.iter_mut().find(|c| c.handle == h) {
                ch.name = get("name").unwrap_or_default();
            }
            json("{\"ok\":true}".into())
        }
        // связь: class, source, target, handle (0 — новая); тело — пары «a\tb» построчно;
        // без пар связь удаляется
        ("POST", ["link", "set"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(src), Some(dst)) = (get("class"), get("source"), get("target")) else {
                return error("400 Bad Request", "нужны class, source, target");
            };
            let source: u16 = src.parse().unwrap_or(0);
            let target: u16 = dst.parse().unwrap_or(0);
            let handle: u16 = get("handle").and_then(|v| v.parse().ok()).unwrap_or(0);
            let vars: Vec<(String, String)> = body
                .lines()
                .filter_map(|l| {
                    let (a, b) = l.split_once('\t')?;
                    (!a.trim().is_empty() && !b.trim().is_empty()).then(|| (a.trim().to_string(), b.trim().to_string()))
                })
                .collect();
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            let existing = cls.links.iter().position(|l| l.handle == handle && handle != 0);
            let result_handle = match (existing, vars.is_empty()) {
                (Some(k), true) => {
                    cls.links.remove(k);
                    0
                }
                (Some(k), false) => {
                    cls.links[k].source = source;
                    cls.links[k].target = target;
                    cls.links[k].vars = vars;
                    handle
                }
                (None, true) => 0,
                (None, false) => {
                    let h = cls.links.iter().map(|l| l.handle).max().unwrap_or(0) + 1;
                    cls.links.push(cls::Link { source, target, handle: h, flags: 0, vars });
                    h
                }
            };
            json(format!("{{\"ok\":true,\"handle\":{result_handle}}}"))
        }
        ("POST", ["link", "remove"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h)) = (get("class"), get("handle")) else {
                return error("400 Bad Request", "нужны class и handle");
            };
            let h: u16 = h.parse().unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            s.project.classes[i].links.retain(|l| l.handle != h);
            json("{\"ok\":true}".into())
        }
        // новый пустой имидж проекта
        ("POST", ["class", "new"]) => {
            let Some(name) = super::param(query, "name").map(super::url_decode).filter(|n| !n.trim().is_empty()) else {
                return error("400 Bad Request", "нужно имя");
            };
            let name = name.trim().to_string();
            let mut s = shared.lock().unwrap();
            if s.project.classes.iter().any(|c| c.name.eq_ignore_ascii_case(&name)) {
                return error("409 Conflict", "имидж с таким именем уже есть");
            }
            s.remember();
            let n = s.project.own_classes;
            s.project.classes.insert(n, cls::Class { name: name.clone(), version: 0x3003, ..Default::default() });
            s.project.own_classes += 1;
            if let Ok(m) = lang::parse("") {
                s.models.insert(name.to_lowercase(), m);
            }
            json("{\"ok\":true}".into())
        }
        // наблюдение: index экземпляра + var
        ("POST", ["trace", "add"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(index), Some(var)) = (get("index").and_then(|v| v.parse::<usize>().ok()), get("var")) else {
                return error("400 Bad Request", "нужны index и var");
            };
            let mut s = shared.lock().unwrap();
            if index >= s.sim.instances().len() {
                return error("404 Not Found", "нет такого экземпляра");
            }
            if let Some(t) = s.traces.iter().find(|t| t.instance == index && t.var.eq_ignore_ascii_case(&var)) {
                return json(format!("{{\"ok\":true,\"id\":{}}}", t.id));
            }
            let id = s.next_trace;
            s.next_trace += 1;
            s.traces.push(super::Trace { id, instance: index, var, points: Default::default() });
            s.sample_traces();
            json(format!("{{\"ok\":true,\"id\":{id}}}"))
        }
        ("POST", ["trace", "remove"]) => {
            let id: u32 = super::param(query, "id").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = shared.lock().unwrap();
            s.traces.retain(|t| t.id != id);
            json("{\"ok\":true}".into())
        }
        ("GET", ["traces"]) => {
            let s = shared.lock().unwrap();
            // since=такт — отдать только новые точки
            let since: u64 = super::param(query, "since").and_then(|v| v.parse().ok()).unwrap_or(0);
            let items: Vec<String> = s
                .traces
                .iter()
                .map(|t| {
                    let path = s.sim.instances().get(t.instance).map(|i| i.path.clone()).unwrap_or_default();
                    let pts: Vec<String> = t.points.iter().filter(|p| p.0 >= since).map(|p| format!("[{},{}]", p.0, num(p.1))).collect();
                    format!(
                        "{{\"id\":{},\"index\":{},\"path\":{},\"var\":{},\"points\":[{}]}}",
                        t.id, t.instance, json_string(&path), json_string(&t.var), pts.join(",")
                    )
                })
                .collect();
            json(format!("{{\"tick\":{},\"traces\":[{}]}}", s.sim.tick_number(), items.join(",")))
        }
        // переименовать имидж проекта: обновляются ссылки детей и корень
        ("POST", ["class", name, "rename"]) => {
            let name = super::url_decode(name);
            let Some(to) = super::param(query, "to").map(super::url_decode).map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) else {
                return error("400 Bad Request", "нужно новое имя to");
            };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            if i >= s.project.own_classes {
                return error("403 Forbidden", "библиотечный имидж не переименовывается");
            }
            if !to.eq_ignore_ascii_case(&name) && s.project.classes.iter().any(|c| c.name.eq_ignore_ascii_case(&to)) {
                return error("409 Conflict", "имидж с таким именем уже есть");
            }
            s.remember();
            let own = s.project.own_classes;
            s.project.classes[i].name = to.clone();
            for c in &mut s.project.classes[..own] {
                for ch in &mut c.children {
                    if ch.class_name.eq_ignore_ascii_case(&name) {
                        ch.class_name = to.clone();
                    }
                }
            }
            if s.project.project.root.eq_ignore_ascii_case(&name) {
                s.project.project.root = to.clone();
            }
            if let Some(st) = &mut s.project.state {
                for im in &mut st.images {
                    if im.class_name.eq_ignore_ascii_case(&name) {
                        im.class_name = to.clone();
                    }
                }
            }
            if let Some(m) = s.models.remove(&name.to_lowercase()) {
                s.models.insert(to.to_lowercase(), m);
            }
            json("{\"ok\":true}".into())
        }
        ("POST", ["class", name, "delete"]) => {
            let name = super::url_decode(name);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            if i >= s.project.own_classes {
                return error("403 Forbidden", "библиотечный имидж не удаляется");
            }
            if s.project.project.root.eq_ignore_ascii_case(&name) {
                return error("409 Conflict", "корневой имидж удалить нельзя");
            }
            let own = s.project.own_classes;
            let users: Vec<String> = s.project.classes[..own]
                .iter()
                .filter(|c| c.children.iter().any(|ch| ch.class_name.eq_ignore_ascii_case(&name)))
                .map(|c| c.name.clone())
                .collect();
            if !users.is_empty() {
                return error("409 Conflict", &format!("имидж используется на схемах: {}", users.join(", ")));
            }
            s.remember();
            s.project.classes.remove(i);
            s.project.own_classes -= 1;
            s.models.remove(&name.to_lowercase());
            json("{\"ok\":true}".into())
        }
        // точки останова: index или class + expr
        ("POST", ["breakpoint", "add"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let Some(expr) = get("expr").filter(|e| !e.trim().is_empty()) else {
                return error("400 Bad Request", "нужно выражение expr");
            };
            let index = get("index").and_then(|v| v.parse::<usize>().ok());
            let class = get("class").filter(|c| !c.is_empty());
            if index.is_none() && class.is_none() {
                return error("400 Bad Request", "нужен index или class");
            }
            let mut s = shared.lock().unwrap();
            let id = s.next_breakpoint;
            s.next_breakpoint += 1;
            s.breakpoints.push(super::Breakpoint { id, index, class, expr: expr.trim().to_string(), enabled: true });
            json(format!("{{\"ok\":true,\"id\":{id}}}"))
        }
        ("POST", ["breakpoint", "remove"]) => {
            let id: u32 = super::param(query, "id").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = shared.lock().unwrap();
            s.breakpoints.retain(|b| b.id != id);
            json("{\"ok\":true}".into())
        }
        ("POST", ["breakpoint", "toggle"]) => {
            let id: u32 = super::param(query, "id").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = shared.lock().unwrap();
            if let Some(b) = s.breakpoints.iter_mut().find(|b| b.id == id) {
                b.enabled = !b.enabled;
            }
            json("{\"ok\":true}".into())
        }
        ("GET", ["breakpoints"]) => {
            let s = shared.lock().unwrap();
            let items: Vec<String> = s
                .breakpoints
                .iter()
                .map(|b| {
                    let path = b.index.and_then(|i| s.sim.instances().get(i)).map(|i| i.path.clone()).unwrap_or_default();
                    format!(
                        "{{\"id\":{},\"index\":{},\"path\":{},\"class\":{},\"expr\":{},\"enabled\":{}}}",
                        b.id,
                        b.index.map(|i| i.to_string()).unwrap_or("null".into()),
                        json_string(&path),
                        json_string(b.class.as_deref().unwrap_or("")),
                        json_string(&b.expr),
                        b.enabled
                    )
                })
                .collect();
            json(format!("[{}]", items.join(",")))
        }
        // вычислить выражение в контексте экземпляра (окно наблюдения)
        ("GET", ["eval", index]) => {
            let Ok(i) = index.parse::<usize>() else { return error("400 Bad Request", "нужен номер экземпляра") };
            let Some(expr) = super::param(query, "expr").map(super::url_decode) else {
                return error("400 Bad Request", "нужно выражение expr");
            };
            let mut s = shared.lock().unwrap();
            match s.sim.eval_in(i, &expr) {
                Ok(v) => json(format!("{{\"ok\":true,\"value\":{}}}", json_string(&v.to_string()))),
                Err(e) => json(format!("{{\"ok\":false,\"error\":{}}}", json_string(&e))),
            }
        }
        // профиль последнего такта: время текста каждого экземпляра
        ("GET", ["profile"]) => {
            let s = shared.lock().unwrap();
            let mut rows: Vec<(usize, u64)> = s.sim.profile.iter().copied().enumerate().filter(|(_, t)| *t > 0).collect();
            rows.sort_by(|a, b| b.1.cmp(&a.1));
            let total: u64 = rows.iter().map(|r| r.1).sum();
            let items: Vec<String> = rows
                .iter()
                .take(40)
                .filter_map(|(i, t)| {
                    let inst = s.sim.instances().get(*i)?;
                    Some(format!("{{\"index\":{i},\"path\":{},\"class\":{},\"ns\":{t}}}", json_string(&inst.path), json_string(&inst.class_name)))
                })
                .collect();
            json(format!("{{\"tick\":{},\"total\":{total},\"items\":[{}]}}", s.sim.tick_number(), items.join(",")))
        }
        // справка: docs/help/topics/*.md (собирается локально из SC3.HLP)
        ("GET", ["help"]) => {
            let q = super::param(query, "q").map(super::url_decode).unwrap_or_default().to_lowercase();
            let Some(dir) = help_dir() else { return json("[]".into()) };
            let mut names: Vec<String> = std::fs::read_dir(dir)
                .map(|rd| rd.filter_map(|e| e.ok()).filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string())).collect())
                .unwrap_or_default();
            names.retain(|n| q.is_empty() || n.to_lowercase().contains(&q));
            names.sort_by_key(|n| (!n.to_lowercase().starts_with(&q), n.to_lowercase()));
            names.truncate(50);
            json(format!("[{}]", names.iter().map(|n| json_string(n)).collect::<Vec<_>>().join(",")))
        }
        ("GET", ["help", topic]) => {
            let topic = super::url_decode(topic);
            let Some(dir) = help_dir() else { return error("404 Not Found", "справка не собрана: make help") };
            let found = std::fs::read_dir(&dir).ok().and_then(|rd| {
                rd.filter_map(|e| e.ok().map(|e| e.path()))
                    .find(|p| p.file_stem().is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case(&topic)))
            });
            match found.and_then(|p| std::fs::read_to_string(&p).ok().map(|t| (p, t))) {
                Some((p, text)) => json(format!(
                    "{{\"topic\":{},\"markdown\":{}}}",
                    json_string(&p.file_stem().unwrap().to_string_lossy()),
                    json_string(&text)
                )),
                None => error("404 Not Found", "нет такой темы"),
            }
        }
        ("POST", ["undo"]) | ("POST", ["redo"]) => {
            let mut s = shared.lock().unwrap();
            let done = if parts[0] == "undo" { s.undo() } else { s.redo() };
            json(format!("{{\"ok\":{done},\"canUndo\":{},\"canRedo\":{}}}", !s.history.is_empty(), !s.future.is_empty()))
        }
        // сохранить проект в родном формате: в его папку или в ?dir=
        ("POST", ["save"]) => {
            let mut s = shared.lock().unwrap();
            let dir = match super::param(query, "dir").map(super::url_decode).filter(|d| !d.is_empty()) {
                Some(d) => std::path::PathBuf::from(d),
                None => s.project.dir.clone(),
            };
            if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| crate::formats::native::save(&dir, &s.project)) {
                return error("500 Internal Server Error", &e.to_string());
            }
            s.project.dir = dir.clone();
            s.unsaved = false;
            json(format!("{{\"ok\":true,\"dir\":{}}}", json_string(&dir.display().to_string())))
        }
        // экспорт в Stratum 2000 (project.spj + .cls) в папку ?dir=
        ("POST", ["export"]) => {
            let Some(dir) = super::param(query, "dir").map(super::url_decode).filter(|d| !d.is_empty()) else {
                return error("400 Bad Request", "нужна папка dir");
            };
            let s = shared.lock().unwrap();
            let dir = std::path::PathBuf::from(dir);
            if let Err(e) = crate::formats::native::export_stratum2000(&dir, &s.project) {
                return error("500 Internal Server Error", &e.to_string());
            }
            json(format!("{{\"ok\":true,\"dir\":{},\"classes\":{}}}", json_string(&dir.display().to_string()), s.project.own_classes))
        }
        ("GET", ["instances"]) => {
            let s = shared.lock().unwrap();
            let items: Vec<String> = s
                .sim
                .instances()
                .iter()
                .enumerate()
                .map(|(i, inst)| {
                    format!(
                        "{{\"index\":{i},\"path\":{},\"name\":{},\"class\":{},\"parent\":{},\"handle\":{}}}",
                        json_string(&inst.path),
                        json_string(&inst.name),
                        json_string(&inst.class_name),
                        inst.parent.map(|p| p.to_string()).unwrap_or("null".into()),
                        inst.handle
                    )
                })
                .collect();
            json(format!("[{}]", items.join(",")))
        }
        ("GET", ["watch", index]) => {
            let s = shared.lock().unwrap();
            let Ok(i) = index.parse::<usize>() else { return error("400 Bad Request", "нужен номер экземпляра") };
            if i >= s.sim.instances().len() {
                return error("404 Not Found", "нет такого экземпляра");
            }
            let inst = &s.sim.instances()[i];
            let vars: Vec<String> = inst
                .var_names()
                .iter()
                .filter_map(|n| s.sim.value(i, n).map(|v| format!("[{},{}]", json_string(n), json_string(&v.to_string()))))
                .collect();
            json(format!("{{\"path\":{},\"vars\":[{}]}}", json_string(&inst.path), vars.join(",")))
        }
        ("POST", ["set", index]) => {
            // query: var, value
            let mut s = shared.lock().unwrap();
            let Ok(i) = index.parse::<usize>() else { return error("400 Bad Request", "нужен номер экземпляра") };
            let (Some(var), Some(value)) = (super::param(query, "var").map(super::url_decode), super::param(query, "value").map(super::url_decode)) else {
                return error("400 Bad Request", "нужны var и value");
            };
            let v = match value.parse::<f64>() {
                Ok(f) => crate::runtime::Value::Float(f),
                Err(_) => crate::runtime::Value::Str(value),
            };
            json(format!("{{\"ok\":{}}}", s.sim.set_value(i, &var, v)))
        }
        _ => error("404 Not Found", "нет такого метода"),
    }
}

/// Папка с темами справки: рядом с репозиторием или с исполняемым файлом.
fn help_dir() -> Option<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = ["docs/help/topics", "../docs/help/topics", "../../docs/help/topics", "../../../docs/help/topics"]
        .iter()
        .map(std::path::PathBuf::from)
        .collect();
    if let Ok(exe) = std::env::current_exe() {
        for up in exe.ancestors().take(6) {
            candidates.push(up.join("docs/help/topics"));
        }
    }
    candidates.into_iter().find(|p| p.is_dir())
}

fn bounds_json(sp: &Space) -> String {
    let (mut x0, mut y0, mut x1, mut y1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for o in sp.objects.values() {
        x0 = x0.min(o.x);
        y0 = y0.min(o.y);
        x1 = x1.max(o.x + o.w);
        y1 = y1.max(o.y + o.h);
    }
    format!("{{\"x\":{},\"y\":{},\"w\":{},\"h\":{}}}", num(x0), num(y0), num(x1 - x0), num(y1 - y0))
}

/// Ячейка 32×32 из набора иконок `.dbm` (BMP-лист 7×N).
fn sheet_icon(bmp: &[u8], index: u16) -> Option<String> {
    let (w, h) = crate::gfx::bmp_size(bmp);
    if w < 32 || h < 32 {
        return None;
    }
    let cols = w / 32;
    let (cx, cy) = ((index as u32 % cols) * 32, (index as u32 / cols) * 32);
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"32\" height=\"32\" viewBox=\"{cx} {cy} 32 32\">\
         <image width=\"{w}\" height=\"{h}\" xlink:href=\"data:image/bmp;base64,{}\"/></svg>",
        svg::base64(bmp)
    ))
}

fn default_icon(name: &str) -> String {
    let initial: String = name.chars().take(2).collect();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"32\" height=\"32\" viewBox=\"0 0 32 32\">\
         <rect x=\"1\" y=\"1\" width=\"30\" height=\"30\" rx=\"4\" fill=\"#e8ecf3\" stroke=\"#9aa3b5\"/>\
         <text x=\"16\" y=\"21\" font-size=\"12\" text-anchor=\"middle\" font-family=\"sans-serif\" fill=\"#3b4252\">{}</text></svg>",
        initial.replace('<', "&lt;")
    )
}
