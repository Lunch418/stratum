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
                "{{\"handle\":{},\"source\":{},\"target\":{},\"vars\":[{}],\"style\":{},\"pad\":{}}}",
                l.handle,
                l.source,
                l.target,
                pairs.join(","),
                crate::formats::native::link_style_json(&l.style).compact(),
                l.pad
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
    let pads: Vec<String> = c.pads.iter().map(|p| format!("{{\"id\":{},\"x\":{},\"y\":{}}}", p.id, num(p.x), num(p.y))).collect();
    format!(
        "{{\"name\":{},\"library\":{},\"description\":{},\"vars\":[{}],\"declared\":[{}],\"text\":{},\"children\":[{}],\"links\":[{}],\"pads\":[{}],\"hasIcon\":{},\"hasScheme\":{},\"hasImage\":{},\"source\":{},\"flags\":{},\"sheet\":{}}}",
        json_string(&c.name),
        library,
        json_string(&c.description),
        vars.join(","),
        declared.join(","),
        json_string(&c.text),
        children.join(","),
        links.join(","),
        pads.join(","),
        c.icon.is_some() || c.icon_file.is_some(),
        c.scheme.is_some(),
        c.image.is_some(),
        json_string(&c.source),
        c.flags.unwrap_or(0),
        crate::formats::native::sheet_json(c.sheet.as_ref().unwrap_or(&Default::default())).compact()
    )
}

/// Свободный дескриптор на листе: не занят ни блоком, ни связью, ни площадкой.
fn free_handle(c: &cls::Class) -> u16 {
    let used = c.children.iter().map(|x| x.handle).chain(c.links.iter().map(|l| l.handle)).chain(c.pads.iter().map(|p| p.id));
    used.max().unwrap_or(0).saturating_add(1).max(1)
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
                "{{\"root\":{},\"dir\":{},\"empty\":{},\"native\":{},\"unsaved\":{},\"canUndo\":{},\"canRedo\":{},\"classes\":[{}]}}",
                json_string(&p.project.root),
                json_string(&p.dir.display().to_string()),
                s.empty,
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
                        svg = gfx.load_picture_file(file).map(|pic| {
                            let mut sp = Space::new(1, &name);
                            sp.load(&pic);
                            fit(&mut sp);
                            svg::render(&sp)
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
                Some(mut pic) => {
                    // растры контактных площадок IDE рисует сама
                    let is_pad = |h: u16| c.pads.iter().any(|p| p.id == h);
                    pic.zorder.retain(|h| !is_pad(*h));
                    pic.objects.retain(|o| !is_pad(o.handle));
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
                    // проверка компилятором в правилах Stratum 2000
                    let check = crate::formats::native::check_text(&s.project, &s.project.classes[i], &m);
                    // на ходу: работающая модель получает новый текст без сброса
                    let live = s.sim.hot_swap_text(&name, m.clone());
                    s.models.insert(name.to_lowercase(), m);
                    let compile_error = match check {
                        Ok(()) => "null".to_string(),
                        Err(e) => format!("{{\"line\":{},\"message\":{}}}", e.line, json_string(&e.message)),
                    };
                    json(format!("{{\"ok\":true,\"live\":{live},\"compileError\":{compile_error}}}"))
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
        // групповой перенос одним шагом отмены: тело — строки «handle\tx\ty»
        ("POST", ["child", "moveall"]) => {
            let Some(class) = super::param(query, "class").map(super::url_decode) else {
                return error("400 Bad Request", "нужен class");
            };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            for line in body.lines() {
                let f: Vec<&str> = line.split('\t').collect();
                let (Some(h), Some(x), Some(y)) = (f.first().and_then(|v| v.parse::<u16>().ok()), f.get(1).and_then(|v| v.parse::<f64>().ok()), f.get(2).and_then(|v| v.parse::<f64>().ok())) else { continue };
                if let Some(ch) = cls.children.iter_mut().find(|c| c.handle == h) {
                    ch.x = x;
                    ch.y = y;
                }
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
        // «Заменить другим…»: экземпляр получает другой класс, handle, имя и
        // положение сохраняются; пары связей с исчезнувшими переменными снимаются
        ("POST", ["child", "replace"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h), Some(child)) = (get("class"), get("handle"), get("child")) else {
                return error("400 Bad Request", "нужны class, handle и child");
            };
            let h: u16 = h.parse().unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            let Some(new_vars) = s.project.class(&child).map(|c| c.vars.iter().map(|v| v.name.to_lowercase()).collect::<Vec<_>>()) else {
                return error("404 Not Found", "нет такого класса для замены");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            if let Some(ch) = cls.children.iter_mut().find(|c| c.handle == h) {
                ch.class_name = child;
            }
            let mut dropped = 0;
            for l in &mut cls.links {
                let before = l.vars.len();
                if l.source == h {
                    l.vars.retain(|(a, _)| new_vars.contains(&a.to_lowercase()));
                }
                if l.target == h {
                    l.vars.retain(|(_, b)| new_vars.contains(&b.to_lowercase()));
                }
                dropped += before - l.vars.len();
            }
            cls.links.retain(|l| !l.vars.is_empty());
            json(format!("{{\"ok\":true,\"droppedPairs\":{dropped}}}"))
        }
        // «Конвертировать в один имидж»: выбранные блоки уходят в новый имидж,
        // внутренние связи — вместе с ними, внешние идут через переменные
        // нового имиджа. Тело — handles построчно; query: class, name
        ("POST", ["child", "merge"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(name)) = (get("class"), get("name").filter(|n| !n.trim().is_empty())) else {
                return error("400 Bad Request", "нужны class и name");
            };
            let picked: Vec<u16> = body.lines().filter_map(|l| l.trim().parse().ok()).collect();
            if picked.is_empty() {
                return error("400 Bad Request", "выберите хотя бы один блок");
            }
            let mut s = shared.lock().unwrap();
            if s.project.classes.iter().any(|c| c.name.eq_ignore_ascii_case(&name)) {
                return error("409 Conflict", "имидж с таким именем уже есть");
            }
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            // типы переменных детей — для сквозных переменных нового имиджа
            let var_type = |s: &super::Shared, child_class: &str, var: &str| -> String {
                s.project.class(child_class).and_then(|c| c.vars.iter().find(|v| v.name.eq_ignore_ascii_case(var))).map(|v| v.var_type.clone()).unwrap_or_else(|| "FLOAT".into())
            };
            s.remember();
            let parent = s.project.classes[i].clone();
            let inside = |h: u16| picked.contains(&h);
            let moved: Vec<cls::Child> = parent.children.iter().filter(|c| inside(c.handle)).cloned().collect();
            let (cx, cy) = (moved.iter().map(|c| c.x).sum::<f64>() / moved.len() as f64, moved.iter().map(|c| c.y).sum::<f64>() / moved.len() as f64);
            let mut new_cls = cls::Class { name: name.clone(), version: 0x3003, ..Default::default() };
            new_cls.children = moved.iter().map(|c| cls::Child { x: c.x - cx, y: c.y - cy, ..c.clone() }).collect();
            let new_handle = parent.children.iter().map(|c| c.handle).max().unwrap_or(0) + 1;
            let mut parent_links = Vec::new();
            let mut next_inner = 1u16;
            for l in &parent.links {
                match (inside(l.source), inside(l.target)) {
                    (true, true) => {
                        new_cls.links.push(cls::Link { handle: next_inner, ..l.clone() });
                        next_inner += 1;
                    }
                    (false, false) => parent_links.push(l.clone()),
                    (src_in, _) => {
                        // внешняя связь: переменная-прокладка в новом имидже
                        let inner = if src_in { l.source } else { l.target };
                        let inner_class = moved.iter().find(|c| c.handle == inner).map(|c| c.class_name.clone()).unwrap_or_default();
                        let mut outer_pairs = Vec::new();
                        for (a, b) in &l.vars {
                            let (inner_var, outer_var) = if src_in { (a, b) } else { (b, a) };
                            if !new_cls.vars.iter().any(|v| v.name.eq_ignore_ascii_case(inner_var)) {
                                new_cls.vars.push(cls::Variable { name: inner_var.clone(), description: format!("из {inner_class}"), default: String::new(), var_type: var_type(&s, &inner_class, inner_var), flags: 0 });
                            }
                            // внутри: сам имидж (handle 0) ↔ блок
                            new_cls.links.push(cls::Link { source: 0, target: inner, handle: next_inner, flags: 0, vars: vec![(inner_var.clone(), inner_var.clone())], ..Default::default() });
                            next_inner += 1;
                            outer_pairs.push(if src_in { (inner_var.clone(), outer_var.clone()) } else { (outer_var.clone(), inner_var.clone()) });
                        }
                        let mut nl = l.clone();
                        if src_in { nl.source = new_handle } else { nl.target = new_handle }
                        nl.vars = outer_pairs;
                        parent_links.push(nl);
                    }
                }
            }
            let cls_mut = &mut s.project.classes[i];
            cls_mut.children.retain(|c| !inside(c.handle));
            cls_mut.children.push(cls::Child { class_name: name.clone(), handle: new_handle, name: String::new(), x: cx, y: cy, flags: 0 });
            cls_mut.links = parent_links;
            let n = s.project.own_classes;
            s.project.classes.insert(n, new_cls);
            s.project.own_classes += 1;
            if let Ok(m) = lang::parse("") {
                s.models.insert(name.to_lowercase(), m);
            }
            json(format!("{{\"ok\":true,\"handle\":{new_handle}}}"))
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
            // контактная площадка со стороны самого имиджа (handle 0)
            let pad: Option<u16> = get("pad").and_then(|v| v.parse().ok());
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
                    if let Some(p) = pad {
                        cls.links[k].pad = p;
                    }
                    handle
                }
                (None, true) => 0,
                (None, false) => {
                    // дескрипторы связей, блоков и площадок в оригинале — из одного
                    // пространства объектов листа: новый не должен совпасть ни с одним
                    let h = free_handle(cls);
                    cls.links.push(cls::Link { source, target, handle: h, flags: 0, vars, pad: pad.unwrap_or(0), ..Default::default() });
                    h
                }
            };
            json(format!("{{\"ok\":true,\"handle\":{result_handle}}}"))
        }
        // контактная площадка: добавить (class, x, y), передвинуть (class, id, x, y),
        // удалить (class, id) — вместе со связями, которые к ней подходят
        ("POST", ["pad", op]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let Some(class) = get("class") else { return error("400 Bad Request", "нужен class") };
            let f = |k: &str| get(k).and_then(|v| v.parse::<f64>().ok()).filter(|v| v.is_finite());
            let id: u16 = get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            if *op != "add" && !s.project.classes[i].pads.iter().any(|p| p.id == id) {
                return error("404 Not Found", "нет такой площадки");
            }
            match *op {
                "add" => {
                    s.remember();
                    let cls = &mut s.project.classes[i];
                    let id = free_handle(cls);
                    cls.pads.push(cls::Pad { id, x: f("x").unwrap_or(0.0), y: f("y").unwrap_or(0.0) });
                    json(format!("{{\"ok\":true,\"id\":{id}}}"))
                }
                "move" => {
                    s.remember();
                    let (x, y) = (f("x"), f("y"));
                    if let Some(p) = s.project.classes[i].pads.iter_mut().find(|p| p.id == id) {
                        p.x = x.unwrap_or(p.x);
                        p.y = y.unwrap_or(p.y);
                    }
                    json("{\"ok\":true}".into())
                }
                "remove" => {
                    s.remember();
                    let cls = &mut s.project.classes[i];
                    cls.pads.retain(|p| p.id != id);
                    let before = cls.links.len();
                    cls.links.retain(|l| !(l.pad == id && (l.source == 0 || l.target == 0)));
                    json(format!("{{\"ok\":true,\"links\":{}}}", before - cls.links.len()))
                }
                _ => error("404 Not Found", "неизвестная операция"),
            }
        }
        // оформление связи: class, handle; тело — JSON стиля
        ("POST", ["link", "style"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(class), Some(h)) = (get("class"), get("handle")) else {
                return error("400 Bad Request", "нужны class и handle");
            };
            let h: u16 = h.parse().unwrap_or(0);
            let Ok(j) = crate::formats::json::parse(body) else { return error("400 Bad Request", "тело — JSON") };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            if let Some(l) = s.project.classes[i].links.iter_mut().find(|l| l.handle == h) {
                l.style = crate::formats::native::link_style_from(&j);
            }
            json("{\"ok\":true}".into())
        }
        // параметры листа имиджа; тело — JSON (см. native::sheet_json)
        ("POST", ["class", name, "sheet"]) => {
            let name = super::url_decode(name);
            let Ok(j) = crate::formats::json::parse(body) else { return error("400 Bad Request", "тело — JSON") };
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let sheet = crate::formats::native::sheet_from(&j);
            s.project.classes[i].sheet = if sheet == Default::default() { None } else { Some(sheet) };
            json("{\"ok\":true}".into())
        }
        // свойства имиджа: description и flags в query
        ("POST", ["class", name, "props"]) => {
            let name = super::url_decode(name);
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            if let Some(d) = get("description") {
                cls.description = d;
            }
            if let Some(f) = get("flags").and_then(|v| v.parse::<u32>().ok()) {
                cls.flags = Some(f);
            }
            json("{\"ok\":true}".into())
        }
        // порядок вычислений: тело — handles детей построчно в новом порядке
        ("POST", ["child", "reorder"]) => {
            let Some(class) = super::param(query, "class").map(super::url_decode) else {
                return error("400 Bad Request", "нужен class");
            };
            let order: Vec<u16> = body.lines().filter_map(|l| l.trim().parse().ok()).collect();
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&class)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            let mut rest = std::mem::take(&mut cls.children);
            let mut sorted = Vec::with_capacity(rest.len());
            for h in order {
                if let Some(k) = rest.iter().position(|c| c.handle == h) {
                    sorted.push(rest.remove(k));
                }
            }
            sorted.extend(rest);
            cls.children = sorted;
            json("{\"ok\":true}".into())
        }
        // свойства проекта (.spj «f»): GET — список, POST — key + int|text в query
        ("GET", ["project", "properties"]) => {
            let s = shared.lock().unwrap();
            let items: Vec<String> = s
                .project
                .project
                .properties
                .iter()
                .map(|p| match &p.value {
                    crate::formats::project::PropertyValue::Int(i) => format!("{{\"key\":{},\"int\":{i}}}", json_string(&p.key)),
                    crate::formats::project::PropertyValue::Text(t) => format!("{{\"key\":{},\"text\":{}}}", json_string(&p.key), json_string(t)),
                })
                .collect();
            json(format!("[{}]", items.join(",")))
        }
        ("POST", ["project", "properties"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let Some(key) = get("key") else { return error("400 Bad Request", "нужен key") };
            let value = match (get("int"), get("text")) {
                (Some(i), _) => i.parse::<i64>().ok().map(|i| crate::formats::project::PropertyValue::Int(i as u32)),
                (None, Some(t)) => Some(crate::formats::project::PropertyValue::Text(t)),
                _ => None,
            };
            let mut s = shared.lock().unwrap();
            s.remember();
            let props = &mut s.project.project.properties;
            props.retain(|p| !p.key.eq_ignore_ascii_case(&key));
            if let Some(value) = value {
                props.push(crate::formats::project::Property { key, value });
            }
            s.apply_project_options();
            json("{\"ok\":true}".into())
        }
        // матрицы модели («Matrix editor» оригинала): список и содержимое
        ("GET", ["matrices"]) => {
            let s = shared.lock().unwrap();
            let items: Vec<String> = s
                .sim
                .effects
                .matrices
                .items
                .iter()
                .map(|(q, m)| format!("{{\"q\":{q},\"minI\":{},\"maxI\":{},\"minJ\":{},\"maxJ\":{}}}", m.min_i, m.max_i, m.min_j, m.max_j))
                .collect();
            json(format!("[{}]", items.join(",")))
        }
        ("GET", ["matrix", q]) => {
            let q: i64 = q.parse().unwrap_or(0);
            let s = shared.lock().unwrap();
            let Some(m) = s.sim.effects.matrices.get(q) else { return error("404 Not Found", "нет такой матрицы") };
            let rows: Vec<String> = (m.min_i..=m.max_i)
                .map(|i| format!("[{}]", (m.min_j..=m.max_j).map(|j| num(m.get(i, j))).collect::<Vec<_>>().join(",")))
                .collect();
            json(format!("{{\"q\":{q},\"minI\":{},\"maxI\":{},\"minJ\":{},\"maxJ\":{},\"rows\":[{}]}}", m.min_i, m.max_i, m.min_j, m.max_j, rows.join(",")))
        }
        // правка ячейки: i, j, value в query
        ("POST", ["matrix", q]) => {
            let q: i64 = q.parse().unwrap_or(0);
            let get = |k: &str| super::param(query, k).and_then(|v| super::url_decode(v).parse::<f64>().ok());
            let (Some(i), Some(j), Some(v)) = (get("i"), get("j"), get("value")) else {
                return error("400 Bad Request", "нужны i, j, value");
            };
            let mut s = shared.lock().unwrap();
            let ok = s.sim.effects.matrices.get_mut(q).is_some_and(|m| m.set(i as i64, j as i64, v));
            json(format!("{{\"ok\":{ok}}}"))
        }
        // библиотека иконок: наборы .dbm рядом с библиотеками и число ячеек 32×32
        ("GET", ["icons"]) => {
            let s = shared.lock().unwrap();
            let gfx = &s.sim.effects.gfx;
            let mut dirs: Vec<std::path::PathBuf> = vec![gfx.project_dir.clone()];
            for lib in &gfx.library_dirs {
                if let Some(parent) = lib.parent() {
                    dirs.push(parent.join("ICONS"));
                    dirs.push(parent.join("data").join("ICONS"));
                }
            }
            let mut sets = Vec::new();
            for d in dirs {
                let Ok(rd) = std::fs::read_dir(&d) else { continue };
                for e in rd.flatten() {
                    let p = e.path();
                    if !p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dbm")) {
                        continue;
                    }
                    let Ok(bmp) = std::fs::read(&p) else { continue };
                    let (w, h) = crate::gfx::bmp_size(&bmp);
                    if w < 32 || h < 32 {
                        continue;
                    }
                    let name = p.file_name().unwrap().to_string_lossy().to_string();
                    sets.push(format!("{{\"file\":{},\"count\":{},\"cols\":{}}}", json_string(&name), (w / 32) * (h / 32), w / 32));
                }
            }
            sets.sort();
            sets.dedup();
            json(format!("[{}]", sets.join(",")))
        }
        ("GET", ["iconsheet"]) => {
            let file = super::param(query, "file").map(super::url_decode).unwrap_or_default();
            let index: u16 = super::param(query, "index").and_then(|v| v.parse().ok()).unwrap_or(0);
            let s = shared.lock().unwrap();
            let Some(path) = s.sim.effects.gfx.find_file(&file) else { return error("404 Not Found", "нет такого набора") };
            // весь лист целиком отдаёт /api/file?name=…; здесь — одна ячейка
            let Some(svg) = std::fs::read(path).ok().and_then(|bmp| sheet_icon(&bmp, index)) else { return error("404 Not Found", "нет ячейки") };
            Response { status: "200 OK", mime: "image/svg+xml", body: svg }
        }
        // иконка имиджа из набора: file, index; пустой file — встроенная/по умолчанию
        ("POST", ["class", name, "icon"]) => {
            let name = super::url_decode(name);
            let file = super::param(query, "file").map(super::url_decode).unwrap_or_default();
            let index: u16 = super::param(query, "index").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut s = shared.lock().unwrap();
            let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            s.remember();
            let cls = &mut s.project.classes[i];
            if file.is_empty() {
                cls.icon_file = None;
                cls.icon_index = None;
            } else {
                cls.icon_file = Some(file);
                cls.icon_index = Some(index);
                cls.icon = None;
            }
            json("{\"ok\":true}".into())
        }
        // проверка всех собственных имиджей компилятором в правилах Stratum 2000
        ("GET", ["check"]) => {
            let s = shared.lock().unwrap();
            let mut items = Vec::new();
            for c in &s.project.classes[..s.project.own_classes] {
                let Ok(m) = lang::parse(&c.text) else { continue };
                if let Err(e) = crate::formats::native::check_text(&s.project, c, &m) {
                    // имидж с сохранённым байт-кодом оригинал исполнит и так
                    let stored = c.bytecode.is_some() && c.bytecode_text.as_deref() == Some(c.text.as_str());
                    items.push(format!("{{\"class\":{},\"line\":{},\"message\":{},\"stored\":{stored}}}", json_string(&c.name), e.line, json_string(&e.message)));
                }
            }
            json(format!("[{}]", items.join(",")))
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
        // выражение может вызвать функцию с побочным эффектом — только POST
        ("POST", ["eval", index]) => {
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
        // открыть проект: path — папка, .spj или project.json
        ("POST", ["open"]) => {
            let Some(path) = super::param(query, "path").map(super::url_decode).filter(|p| !p.trim().is_empty()) else {
                return error("400 Bad Request", "нужен путь path");
            };
            let mut s = shared.lock().unwrap();
            match s.open(std::path::PathBuf::from(path.trim())) {
                Ok(()) => json(format!("{{\"ok\":true,\"dir\":{}}}", json_string(&s.project.dir.display().to_string()))),
                Err(e) => error("400 Bad Request", &e),
            }
        }
        // новый проект: пустой в памяти или сразу в папке ?dir= с корнем ?root=
        ("POST", ["new"]) => {
            let dir = super::param(query, "dir").map(super::url_decode).filter(|d| !d.trim().is_empty());
            let root = super::param(query, "root").map(super::url_decode).filter(|r| !r.trim().is_empty()).unwrap_or_else(|| "Main".into());
            let mut s = shared.lock().unwrap();
            if let Err(e) = s.open(std::path::PathBuf::new()) {
                return error("500 Internal Server Error", &e);
            }
            s.project.project.root = root.clone();
            if let Some(c) = s.project.classes.first_mut() {
                c.name = root.clone();
            }
            s.models.clear();
            if let Some(dir) = dir {
                let dir = std::path::PathBuf::from(dir.trim());
                if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| crate::formats::native::save(&dir, &s.project)) {
                    return error("500 Internal Server Error", &e.to_string());
                }
                s.project.dir = dir;
                s.empty = false;
            }
            json(format!("{{\"ok\":true,\"root\":{}}}", json_string(&root)))
        }
        // сводка проекта: диалог «Информация» оригинала
        ("GET", ["info"]) => {
            let s = shared.lock().unwrap();
            let p = &s.project;
            let mut functions: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
            let mut equations = 0usize;
            let mut lines = 0usize;
            fn count_calls(e: &crate::lang::Expr, out: &mut std::collections::BTreeMap<String, u32>) {
                use crate::lang::Expr;
                match e {
                    Expr::Call(name, args) => {
                        *out.entry(name.clone()).or_insert(0) += 1;
                        args.iter().for_each(|a| count_calls(a, out));
                    }
                    Expr::Unary(_, a) => count_calls(a, out),
                    Expr::Binary(_, a, b) => {
                        count_calls(a, out);
                        count_calls(b, out);
                    }
                    _ => {}
                }
            }
            fn walk(body: &[crate::lang::Stmt], out: &mut std::collections::BTreeMap<String, u32>, eq: &mut usize) {
                use crate::lang::Stmt;
                for st in body {
                    match st {
                        Stmt::Assign { value, .. } | Stmt::AssignDeferred { value, .. } | Stmt::Expr(value) => count_calls(value, out),
                        Stmt::If { condition, then_body, else_body } => { count_calls(condition, out); walk(then_body, out, eq); walk(else_body, out, eq); }
                        Stmt::While { condition, body } | Stmt::DoUntil { body, condition } => { count_calls(condition, out); walk(body, out, eq); }
                        Stmt::Switch { arms, default } => { for a in arms { count_calls(&a.condition, out); walk(&a.body, out, eq); } walk(default, out, eq); }
                        Stmt::Equation { left, right } => { *eq += 1; count_calls(left, out); count_calls(right, out); }
                        Stmt::Return(Some(v)) => count_calls(v, out),
                        _ => {}
                    }
                }
            }
            for c in &p.classes[..p.own_classes] {
                lines += c.text.lines().count();
                if let Some(m) = s.models.get(&c.name.to_lowercase()) {
                    walk(&m.body, &mut functions, &mut equations);
                }
            }
            let mut fns: Vec<(String, u32)> = functions.into_iter().collect();
            fns.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            let fns_json: Vec<String> = fns.iter().map(|(n, c)| format!("[{},{c}]", json_string(n))).collect();
            let links: usize = p.classes[..p.own_classes].iter().map(|c| c.links.len()).sum();
            let children: usize = p.classes[..p.own_classes].iter().map(|c| c.children.len()).sum();
            let props: Vec<String> = p.project.properties.iter().map(|pr| format!("[{},{}]", json_string(&pr.key), json_string(&match &pr.value { crate::formats::project::PropertyValue::Int(i) => i.to_string(), crate::formats::project::PropertyValue::Text(t) => t.clone() }))).collect();
            json(format!(
                "{{\"root\":{},\"dir\":{},\"classes\":{},\"libraryClasses\":{},\"instances\":{},\"children\":{},\"links\":{},\"equations\":{},\"lines\":{},\"matrices\":{},\"windows\":{},\"functions\":[{}],\"properties\":[{}],\"libraries\":[{}]}}",
                json_string(&p.project.root), json_string(&p.dir.display().to_string()), p.own_classes, p.classes.len() - p.own_classes,
                s.sim.instances().len(), children, links, equations, lines, s.sim.effects.matrices.items.len(), s.sim.effects.gfx.windows.len(),
                fns_json.join(","), props.join(","),
                p.library_dirs.iter().map(|d| json_string(&d.display().to_string())).collect::<Vec<_>>().join(",")
            ))
        }
        // обзор папок для диалога открытия: подпапки и файлы проектов
        ("GET", ["browse"]) => {
            let dir = super::param(query, "dir").map(super::url_decode).filter(|d| !d.is_empty())
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("/")));
            let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
            let Ok(rd) = std::fs::read_dir(&dir) else { return error("404 Not Found", "нет такой папки") };
            // ext=vdr,bmp — показывать файлы с этими расширениями вместо проектов
            let exts: Vec<String> = super::param(query, "ext").map(super::url_decode).unwrap_or_default().split(',').filter(|e| !e.is_empty()).map(|e| e.to_lowercase()).collect();
            let mut entries: Vec<(String, String, &str)> = Vec::new();
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if p.is_dir() {
                    let is_project = p.join("project.json").is_file()
                        || std::fs::read_dir(&p).map(|r| r.filter_map(|x| x.ok()).any(|x| x.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("spj") || x.eq_ignore_ascii_case("prj")))).unwrap_or(false);
                    entries.push((name, p.display().to_string(), if is_project { "project" } else { "dir" }));
                } else if !exts.is_empty() {
                    if p.extension().is_some_and(|x| exts.contains(&x.to_string_lossy().to_lowercase())) {
                        entries.push((name, p.display().to_string(), "file"));
                    }
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("spj") || x.eq_ignore_ascii_case("prj")) || name == "project.json" {
                    entries.push((name, p.display().to_string(), "file"));
                }
            }
            entries.sort_by(|a, b| (a.2 == "file").cmp(&(b.2 == "file")).then(a.0.to_lowercase().cmp(&b.0.to_lowercase())));
            let items: Vec<String> = entries.iter().map(|(n, p, k)| format!("{{\"name\":{},\"path\":{},\"kind\":{}}}", json_string(n), json_string(p), json_string(k))).collect();
            json(format!(
                "{{\"dir\":{},\"parent\":{},\"entries\":[{}]}}",
                json_string(&dir.display().to_string()),
                dir.parent().map(|p| json_string(&p.display().to_string())).unwrap_or("null".into()),
                items.join(",")
            ))
        }
        // графический объект окна модели: по точке (x, y) или по handle
        ("GET", ["object"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let Some(win) = get("win") else { return error("400 Bad Request", "нужно окно win") };
            let s = shared.lock().unwrap();
            let gfx = &s.sim.effects.gfx;
            let Some(space) = gfx.window_space(&win).and_then(|h| gfx.space(h)) else { return error("404 Not Found", "нет такого окна") };
            let handle = match get("handle").and_then(|v| v.parse::<u32>().ok()) {
                Some(h) => Some(h),
                None => {
                    let (x, y) = (get("x").and_then(|v| v.parse().ok()).unwrap_or(0.0), get("y").and_then(|v| v.parse().ok()).unwrap_or(0.0));
                    space.object_at(x, y)
                }
            };
            let Some(o) = handle.and_then(|h| space.objects.get(&h)) else { return json("null".into()) };
            json(object_json(space, o))
        }
        ("POST", ["object", "set"]) => {
            let get = |k: &str| super::param(query, k).map(super::url_decode);
            let (Some(win), Some(handle), Some(field), Some(value)) = (get("win"), get("handle").and_then(|v| v.parse::<u32>().ok()), get("field"), get("value")) else {
                return error("400 Bad Request", "нужны win, handle, field, value");
            };
            let mut s = shared.lock().unwrap();
            let gfx = &mut s.sim.effects.gfx;
            let Some(space) = gfx.window_space(&win).and_then(|h| gfx.space_mut(h)) else { return error("404 Not Found", "нет такого окна") };
            let done = set_object_field(space, handle, &field, &value);
            json(format!("{{\"ok\":{done}}}"))
        }
        // граф зависимостей схемы имиджа: узлы — переменные экземпляров,
        // дуги — связи и присваивания в текстах
        ("GET", ["graph", name]) => {
            let name = super::url_decode(name);
            let s = shared.lock().unwrap();
            let Some(cls) = s.project.classes.iter().find(|c| c.name.eq_ignore_ascii_case(&name)) else {
                return error("404 Not Found", "нет такого имиджа");
            };
            let mut nodes: Vec<String> = Vec::new();
            let mut edges: Vec<String> = Vec::new();
            let mut node = |inst: &str, var: &str| -> String { let id = format!("{inst}.{}", var.to_ascii_lowercase()); if !nodes.contains(&id) { nodes.push(id.clone()); } id };
            // сам имидж — узел «0»
            let mut members: Vec<(u16, String, String)> = vec![(0, "self".into(), cls.name.clone())];
            for ch in &cls.children {
                members.push((ch.handle, format!("#{}", ch.handle), ch.class_name.clone()));
            }
            let labels: Vec<String> = members.iter().map(|(h, id, c)| {
                let inst_name = cls.children.iter().find(|x| x.handle == *h).map(|x| if x.name.is_empty() { c.clone() } else { x.name.clone() }).unwrap_or(c.clone());
                format!("{{\"id\":{},\"label\":{},\"class\":{}}}", json_string(id), json_string(&inst_name), json_string(c))
            }).collect();
            for (h, id, class_name) in &members {
                let _ = h;
                let Some(model) = s.models.get(&class_name.to_lowercase()) else { continue };
                for (a, b) in lang::dependencies(model) {
                    let from = node(id, &a);
                    let to = node(id, &b);
                    edges.push(format!("{{\"from\":{},\"to\":{},\"kind\":\"text\"}}", json_string(&from), json_string(&to)));
                }
            }
            for l in &cls.links {
                let src = if l.source == 0 { "self".to_string() } else { format!("#{}", l.source) };
                let dst = if l.target == 0 { "self".to_string() } else { format!("#{}", l.target) };
                for (a, b) in &l.vars {
                    let from = node(&src, a);
                    let to = node(&dst, b);
                    edges.push(format!("{{\"from\":{},\"to\":{},\"kind\":\"link\"}}", json_string(&from), json_string(&to)));
                }
            }
            let nodes_json: Vec<String> = nodes.iter().map(|n| json_string(n)).collect();
            json(format!("{{\"instances\":[{}],\"nodes\":[{}],\"edges\":[{}]}}", labels.join(","), nodes_json.join(","), edges.join(",")))
        }
        // редактор рисунка/схемы/иконки имиджа
        (m, ["picture", class]) => {
            let class = super::url_decode(class);
            let kind = super::editor::Kind::parse(&super::param(query, "kind").map(super::url_decode).unwrap_or_default());
            let mut s = shared.lock().unwrap();
            match super::editor::handle(m, &class, kind, body, &mut s) {
                Ok(j) => json(j),
                Err((status, msg)) => error(status, &msg),
            }
        }
        // состояние модели: файл .stt, стартовое состояние проекта, по умолчанию
        ("POST", ["state", action]) => {
            let path = super::param(query, "path").map(super::url_decode).unwrap_or_default();
            // class=Имя — только экземпляры этого имиджа («переменные имиджа»)
            let only = super::param(query, "class").map(super::url_decode).filter(|c| !c.is_empty());
            let mut s = shared.lock().unwrap();
            let root = s.project.project.root.clone();
            match *action {
                "save" => {
                    if path.is_empty() {
                        return error("400 Bad Request", "нужен путь path");
                    }
                    let mut st = s.sim.snapshot_state(&root);
                    if let Some(c) = &only {
                        st.images.retain(|i| i.class_name.eq_ignore_ascii_case(c));
                    }
                    match std::fs::write(&path, crate::formats::project::write_state(&st)) {
                        Ok(()) => json(format!("{{\"ok\":true,\"images\":{}}}", st.images.len())),
                        Err(e) => error("500 Internal Server Error", &e.to_string()),
                    }
                }
                "load" => {
                    if path.is_empty() {
                        return error("400 Bad Request", "нужен путь path");
                    }
                    let Ok(data) = std::fs::read(&path) else { return error("404 Not Found", "файл не читается") };
                    match crate::formats::project::parse_state(&data, &path) {
                        Ok(mut st) => {
                            if let Some(c) = &only {
                                st.images.retain(|i| i.class_name.eq_ignore_ascii_case(c));
                            }
                            s.sim.load_state(&st);
                            json(format!("{{\"ok\":true,\"images\":{}}}", st.images.len()))
                        }
                        Err(e) => error("400 Bad Request", &e.to_string()),
                    }
                }
                // текущие значения становятся стартовыми (state.json / _preload.stt при сохранении)
                "keep" => {
                    let st = s.sim.snapshot_state(&root);
                    s.remember();
                    s.project.state = Some(st);
                    json("{\"ok\":true}".into())
                }
                "default" => {
                    s.sim.reset_to_defaults();
                    s.running = false;
                    json("{\"ok\":true}".into())
                }
                _ => error("404 Not Found", "нет такого действия"),
            }
        }
        // поиск по текстам и переменным всех имиджей проекта
        ("GET", ["search"]) => {
            let q = super::param(query, "q").map(super::url_decode).unwrap_or_default();
            let q_low = q.to_lowercase();
            if q_low.trim().is_empty() {
                return json("[]".into());
            }
            let libs = super::param(query, "libs").is_some_and(|v| v == "1");
            let s = shared.lock().unwrap();
            let mut hits: Vec<String> = Vec::new();
            for (i, c) in s.project.classes.iter().enumerate() {
                if i >= s.project.own_classes && !libs {
                    break;
                }
                for (n, line) in c.text.lines().enumerate() {
                    if line.to_lowercase().contains(&q_low) {
                        hits.push(format!("{{\"class\":{},\"line\":{},\"text\":{},\"kind\":\"text\"}}", json_string(&c.name), n + 1, json_string(line.trim())));
                        if hits.len() > 500 {
                            break;
                        }
                    }
                }
                for v in &c.vars {
                    if v.name.to_lowercase().contains(&q_low) || v.description.to_lowercase().contains(&q_low) {
                        hits.push(format!("{{\"class\":{},\"line\":0,\"text\":{},\"kind\":\"var\"}}", json_string(&c.name), json_string(&format!("{} {} = {} {}", v.var_type, v.name, v.default, v.description))));
                    }
                }
                if c.name.to_lowercase().contains(&q_low) {
                    hits.push(format!("{{\"class\":{},\"line\":0,\"text\":\"имидж\",\"kind\":\"class\"}}", json_string(&c.name)));
                }
            }
            json(format!("[{}]", hits.join(",")))
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

/// Меняет поле графического объекта (инспектор и редактор рисунка).
pub(crate) fn set_object_field(space: &mut Space, handle: u32, field: &str, value: &str) -> bool {
    let num: f64 = value.parse().unwrap_or(0.0);
    match field {
        "x" | "y" | "w" | "h" | "angle" | "visible" | "name" | "alpha" => space.objects.get_mut(&handle).map(|o| match field {
            "x" => o.x = num,
            "y" => o.y = num,
            "w" => o.w = num.max(0.0),
            "h" => o.h = num.max(0.0),
            "angle" => o.angle = num,
            "visible" => o.visible = num != 0.0,
            "alpha" => o.alpha = num.clamp(0.0, 255.0) as u8,
            _ => o.name = value.to_string(),
        }).is_some(),
        "pen.color" | "pen.width" | "pen.style" => {
            let pen = space.objects.get(&handle).and_then(|o| match &o.shape { crate::gfx::Shape::Polyline { pen, .. } => Some(*pen), _ => None });
            pen.and_then(|p| space.pens.get_mut(&p)).map(|p| match field {
                "pen.color" => p.color = parse_color(value),
                "pen.width" => p.width = num.max(0.0) as u16,
                _ => p.style = num.max(0.0) as u16,
            }).is_some()
        }
        "brush.color" | "brush.style" => {
            let brush = space.objects.get(&handle).and_then(|o| match &o.shape { crate::gfx::Shape::Polyline { brush, .. } => Some(*brush), _ => None });
            brush.and_then(|b| space.brushes.get_mut(&b)).map(|b| match field {
                "brush.color" => b.color = parse_color(value),
                _ => b.style = num.max(0.0) as u16,
            }).is_some()
        }
        // текст: строка, шрифт и цвета первого фрагмента
        "text" | "font.face" | "font.size" | "font.bold" | "font.italic" | "font.underline" | "text.fg" | "text.bg" => {
            let shape = space.objects.get(&handle).map(|o| o.shape.clone());
            match shape {
                Some(crate::gfx::Shape::Text { text }) => {
                    let Some(parts) = space.texts.get(&text).cloned() else { return false };
                    let Some(first) = parts.first().cloned() else { return false };
                    match field {
                        "text" => {
                            // весь текст — в первый фрагмент, остальные убираем
                            let sh = space.add_string(value.to_string());
                            let mut p0 = first;
                            p0.string = sh;
                            space.texts.insert(text, vec![p0]);
                        }
                        "text.fg" | "text.bg" => {
                            let c = parse_color(value);
                            if let Some(ps) = space.texts.get_mut(&text) {
                                for p in ps { if field == "text.fg" { p.fg = c } else { p.bg = c } }
                            }
                        }
                        _ => {
                            let mut f = space.fonts.get(&first.font).cloned().unwrap_or(crate::gfx::Font { height: -16, weight: 400, italic: false, underline: false, face: "Arial".into() });
                            match field {
                                "font.face" => f.face = value.to_string(),
                                "font.size" => f.height = -(num.max(1.0) * 96.0 / 72.0).round() as i32,
                                "font.bold" => f.weight = if num != 0.0 { 700 } else { 400 },
                                "font.italic" => f.italic = num != 0.0,
                                _ => f.underline = num != 0.0,
                            }
                            let fh = space.add_font(f);
                            if let Some(ps) = space.texts.get_mut(&text) { for p in ps { p.font = fh; } }
                        }
                    }
                    true
                }
                Some(crate::gfx::Shape::Control { .. }) if field == "text" => space.objects.get_mut(&handle).map(|o| if let crate::gfx::Shape::Control { text, caption, .. } = &mut o.shape { *text = value.to_string(); *caption = value.to_string(); }).is_some(),
                _ => false,
            }
        }
        // гипербаза: hyper.mode (-1 — снять ссылку), hyper.target/window/object/effect
        "hyper.mode" | "hyper.target" | "hyper.window" | "hyper.object" | "hyper.effect" => space.objects.get_mut(&handle).map(|o| {
            if field == "hyper.mode" && num < 0.0 {
                o.hyper = None;
                return;
            }
            let h = o.hyper.get_or_insert_with(Default::default);
            match field {
                "hyper.mode" => h.mode = num as i32,
                "hyper.target" => h.target = value.to_string(),
                "hyper.window" => h.window = value.to_string(),
                "hyper.object" => h.object = value.to_string(),
                _ => h.effect = value.to_string(),
            }
        }).is_some(),
        "enabled" | "checked" => space.objects.get_mut(&handle).map(|o| if let crate::gfx::Shape::Control { enabled, checked, .. } = &mut o.shape { if field == "enabled" { *enabled = num != 0.0 } else { *checked = num != 0.0 } }).is_some(),
        "zorder" => {
            let tops = space.top_order();
            let n = (num.max(0.0) as usize).min(tops.len().saturating_sub(1));
            if tops.contains(&handle) {
                space.move_among_tops(handle, n);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// `#rrggbb` или число → COLORREF.
pub(crate) fn parse_color(v: &str) -> u32 {
    if let Some(hex) = v.strip_prefix('#') {
        if let Ok(rgb) = u32::from_str_radix(hex, 16) {
            return ((rgb & 0xFF) << 16) | (rgb & 0xFF00) | (rgb >> 16);
        }
    }
    v.parse::<f64>().unwrap_or(0.0) as u32
}

/// Свойства графического объекта для инспектора.
pub(crate) fn object_json(sp: &Space, o: &crate::gfx::Object) -> String {
    use crate::gfx::Shape;
    let kind = match &o.shape {
        Shape::Polyline { .. } => "polyline",
        Shape::Bitmap { .. } => "bitmap",
        Shape::Text { .. } => "text",
        Shape::Control { .. } => "control",
        Shape::Group { .. } => "group",
        Shape::View3d { .. } => "view3d",
        Shape::Unknown => "unknown",
    };
    let mut extra = String::new();
    match &o.shape {
        Shape::Polyline { pen, brush, points } => {
            if let Some(p) = sp.pens.get(pen) {
                extra.push_str(&format!(",\"pen\":{{\"color\":{},\"width\":{},\"style\":{}}}", json_string(&svg::color(p.color)), p.width, p.style));
            }
            if let Some(b) = sp.brushes.get(brush) {
                extra.push_str(&format!(",\"brush\":{{\"color\":{},\"style\":{}}}", json_string(&svg::color(b.color)), b.style));
            }
            let pts: Vec<String> = points.iter().map(|(x, y)| format!("[{},{}]", num(*x), num(*y))).collect();
            extra.push_str(&format!(",\"points\":[{}]", pts.join(",")));
        }
        Shape::Text { text } => {
            let t: String = sp.texts.get(text).map(|parts| parts.iter().filter_map(|p| sp.strings.get(&p.string).cloned()).collect::<Vec<_>>().join("")).unwrap_or_default();
            extra.push_str(&format!(",\"text\":{}", json_string(&t)));
            // шрифт и цвета первого фрагмента — для правки в инспекторе
            if let Some(first) = sp.texts.get(text).and_then(|p| p.first()) {
                if let Some(f) = sp.fonts.get(&first.font) {
                    extra.push_str(&format!(",\"font\":{{\"face\":{},\"size\":{},\"bold\":{},\"italic\":{},\"underline\":{},\"fg\":{},\"bg\":{}}}",
                        json_string(&f.face), (f.height.abs() as f64 * 72.0 / 96.0).round(), f.weight >= 600, f.italic, f.underline,
                        json_string(&format!("#{:02x}{:02x}{:02x}", first.fg & 255, (first.fg >> 8) & 255, (first.fg >> 16) & 255)),
                        json_string(&format!("#{:02x}{:02x}{:02x}", first.bg & 255, (first.bg >> 8) & 255, (first.bg >> 16) & 255))));
                }
            }
        }
        Shape::Control { class, text, enabled, checked, .. } => extra.push_str(&format!(",\"class\":{},\"text\":{},\"enabled\":{enabled},\"checked\":{checked}", json_string(class), json_string(text))),
        Shape::Group { children } => extra.push_str(&format!(",\"children\":{}", children.len())),
        _ => {}
    }
    if let Some(h) = &o.hyper {
        extra.push_str(&format!(
            ",\"hyper\":{{\"mode\":{},\"target\":{},\"window\":{},\"object\":{},\"effect\":{}}}",
            h.mode, json_string(&h.target), json_string(&h.window), json_string(&h.object), json_string(&h.effect)
        ));
    }
    let z = sp.top_order().iter().position(|&h| h == o.handle).map(|z| z.to_string()).unwrap_or("null".into());
    format!(
        "{{\"handle\":{},\"name\":{},\"kind\":{},\"x\":{},\"y\":{},\"w\":{},\"h\":{},\"angle\":{},\"visible\":{},\"alpha\":{},\"zorder\":{},\"parent\":{}{}}}",
        o.handle, json_string(&o.name), json_string(kind), num(o.x), num(o.y), num(o.w), num(o.h), num(o.angle), o.visible, o.alpha, z,
        o.parent.map(|p| p.to_string()).unwrap_or("null".into()), extra
    )
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
