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
                "{{\"root\":{},\"dir\":{},\"classes\":[{}]}}",
                json_string(&p.project.root),
                json_string(&p.dir.display().to_string()),
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
            s.project.classes[i].text = body.to_string();
            s.dirty = true;
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
            s.project.classes[i].vars = vars;
            s.dirty = true;
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
            if let Some(ch) = s.project.classes[i].children.iter_mut().find(|c| c.handle == h) {
                ch.x = x.parse().unwrap_or(ch.x);
                ch.y = y.parse().unwrap_or(ch.y);
                s.dirty = true;
            }
            json("{\"ok\":true}".into())
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
