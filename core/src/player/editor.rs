//! Редактор графики имиджа: рисунок, лист схемы или иконка загружаются в
//! отдельное пространство, правки применяются операциями и после каждой
//! операции пишутся обратно в имидж как блок `.vdr` — так они попадают в
//! Undo/Redo проекта и в сохранение.

use super::Shared;
use crate::formats::json::{self, Json};
use crate::formats::{cls, vdr};
use crate::gfx::{svg, Brush, Font, Handle, Object, Pen, Shape, Space, TextPart};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Image,
    Scheme,
    Icon,
}

impl Kind {
    pub fn parse(s: &str) -> Kind {
        match s {
            "scheme" => Kind::Scheme,
            "icon" => Kind::Icon,
            _ => Kind::Image,
        }
    }
}

fn blob(c: &cls::Class, kind: Kind) -> &Option<Vec<u8>> {
    match kind {
        Kind::Image => &c.image,
        Kind::Scheme => &c.scheme,
        Kind::Icon => &c.icon,
    }
}

fn blob_mut(c: &mut cls::Class, kind: Kind) -> &mut Option<Vec<u8>> {
    match kind {
        Kind::Image => &mut c.image,
        Kind::Scheme => &mut c.scheme,
        Kind::Icon => &mut c.icon,
    }
}

/// Пространство с графикой имиджа; пустой блок даёт пустой лист.
pub fn open(c: &cls::Class, kind: Kind) -> Space {
    let mut sp = Space::new(1, &c.name);
    if let Some(b) = blob(c, kind) {
        if let Ok(pic) = vdr::parse(b, &c.name) {
            sp.load(&pic);
        }
    }
    if kind == Kind::Icon {
        for o in sp.objects.values_mut() {
            o.scheme_element = false;
        }
        if sp.client == crate::gfx::DEFAULT_CLIENT {
            sp.client = (32.0, 32.0);
        }
    }
    sp
}

/// Записывает пространство обратно в имидж.
pub fn store(c: &mut cls::Class, kind: Kind, sp: &Space) {
    let pic = sp.to_picture();
    *blob_mut(c, kind) = Some(vdr::write(&pic));
}

fn num(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 { format!("{}", v as i64) } else { format!("{v}") }
}

/// Состояние для страницы: SVG всего листа (включая элементы схемы) и список объектов.
pub fn state_json(sp: &Space, kind: Kind) -> String {
    // лист общий для рисунка и схемы: иконки имиджей и линии связей
    // (элементы схемы) показываются только в режиме схемы
    let mut shown = sp.clone();
    for o in shown.objects.values_mut() {
        if kind == Kind::Scheme || kind == Kind::Icon {
            o.scheme_element = false;
        }
    }
    let editable = |o: &crate::gfx::Object| kind == Kind::Scheme || !o.scheme_element;
    // SVG покрывает и лист, и всё, что лежит за его краями
    let (mut x0, mut y0, mut x1, mut y1) = (sp.origin.0, sp.origin.1, sp.origin.0 + sp.client.0, sp.origin.1 + sp.client.1);
    for o in sp.objects.values() {
        if !matches!(o.shape, Shape::Group { .. }) && editable(o) {
            x0 = x0.min(o.x);
            y0 = y0.min(o.y);
            x1 = x1.max(o.x + o.w);
            y1 = y1.max(o.y + o.h);
        }
    }
    shown.origin = (x0, y0);
    shown.scale = (1.0, 1.0);
    shown.client = ((x1 - x0).max(1.0), (y1 - y0).max(1.0));
    let svg = svg::render(&shown);
    let view = format!("[{},{},{},{}]", num(x0), num(y0), num(shown.client.0), num(shown.client.1));
    let objects: Vec<String> = sp.zorder.iter().filter_map(|h| sp.objects.get(h)).filter(|o| editable(o)).map(|o| super::api::object_json(sp, o)).collect();
    let kind_name = match kind {
        Kind::Image => "image",
        Kind::Scheme => "scheme",
        Kind::Icon => "icon",
    };
    format!(
        "{{\"kind\":\"{kind_name}\",\"origin\":[{},{}],\"client\":[{},{}],\"scale\":{},\"view\":{view},\"svg\":{},\"objects\":[{}]}}",
        num(sp.origin.0), num(sp.origin.1), num(sp.client.0), num(sp.client.1), num(sp.scale.0),
        super::json_string(&svg),
        objects.join(",")
    )
}

fn points_of(j: &Json) -> Vec<(f64, f64)> {
    j.get("points")
        .and_then(Json::as_array)
        .map(|a| a.iter().filter_map(|p| { let p = p.as_array()?; Some((p.first()?.as_f64()?, p.get(1)?.as_f64()?)) }).collect())
        .unwrap_or_default()
}

fn color_of(j: Option<&Json>, default: u32) -> u32 {
    match j.and_then(Json::as_str) {
        Some(hex) => super::api::parse_color(hex),
        None => j.and_then(Json::as_f64).map(|v| v as u32).unwrap_or(default),
    }
}

fn bbox(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (x, y) in points {
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(*x);
        y1 = y1.max(*y);
    }
    if points.is_empty() { (0.0, 0.0, 0.0, 0.0) } else { (x0, y0, x1 - x0, y1 - y0) }
}

/// Полилиния с точками в координатах листа; точки объекта хранятся
/// относительно его начала — как в файлах Stratum.
fn add_polyline(sp: &mut Space, points: Vec<(f64, f64)>, pen: Handle, brush: Handle, closed: bool) -> Handle {
    let mut pts = points;
    if closed && pts.len() > 2 && pts.first() != pts.last() {
        pts.push(pts[0]);
    }
    let (x, y, w, h) = bbox(&pts);
    let rel: Vec<(f64, f64)> = pts.iter().map(|(px, py)| (px - x, py - y)).collect();
    sp.add_object(Object::new(0, x, y, w, h, Shape::Polyline { pen, brush, points: rel }))
}

/// Применяет операцию; возвращает handle затронутого объекта.
pub fn apply(sp: &mut Space, op: &Json) -> Result<Handle, String> {
    let name = op.str_or("op", "");
    let handle = op.num_or("handle", 0.0) as Handle;
    match name.as_str() {
        "add" => {
            let shape = op.str_or("shape", "line");
            let pen_j = op.get("pen");
            let pen = sp.add_pen(Pen {
                color: color_of(pen_j.and_then(|p| p.get("color")), 0),
                width: pen_j.map(|p| p.num_or("width", 1.0)).unwrap_or(1.0) as u16,
                style: pen_j.map(|p| p.num_or("style", 0.0)).unwrap_or(0.0) as u16,
                rop: 0,
            });
            let brush_j = op.get("brush");
            let brush = match brush_j {
                Some(b) if b.get("color").is_some() => sp.add_brush(Brush {
                    color: color_of(b.get("color"), 0xFFFFFF),
                    style: b.num_or("style", 0.0) as u16,
                    hatch: 0,
                    rop: 0,
                    dib: 0,
                }),
                _ => 0,
            };
            let pts = points_of(op);
            let h = match shape.as_str() {
                "text" => {
                    let font = sp.add_font(Font { height: -(op.num_or("size", 12.0) * 96.0 / 72.0).round() as i32, weight: 400, italic: false, underline: false, face: op.str_or("font", "Arial") });
                    let string = sp.add_string(op.str_or("text", "Текст"));
                    let text = sp.add_text(vec![TextPart { fg: color_of(op.get("fg"), 0), bg: color_of(op.get("bg"), 0xFFFFFF), font, string }]);
                    let (x, y) = pts.first().copied().unwrap_or((0.0, 0.0));
                    sp.add_object(Object::new(0, x, y, op.num_or("w", 100.0), op.num_or("h", 20.0), Shape::Text { text }))
                }
                "rect" | "roundrect" | "ellipse" | "arc" => {
                    if pts.len() < 2 {
                        return Err("нужны две точки".into());
                    }
                    let (a, b) = (pts[0], pts[1]);
                    let (x0, y0, x1, y1) = (a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1));
                    let outline: Vec<(f64, f64)> = match shape.as_str() {
                        "rect" => vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
                        "roundrect" => {
                            let r = ((x1 - x0).min(y1 - y0) * 0.2).max(1.0);
                            let mut v = Vec::new();
                            for (cx, cy, a0) in [(x1 - r, y0 + r, -90.0f64), (x1 - r, y1 - r, 0.0), (x0 + r, y1 - r, 90.0), (x0 + r, y0 + r, 180.0)] {
                                for k in 0..=6 {
                                    let t = (a0 + 90.0 * k as f64 / 6.0).to_radians();
                                    v.push((cx + r * t.cos(), cy + r * t.sin()));
                                }
                            }
                            v
                        }
                        "ellipse" => {
                            let (cx, cy, rx, ry) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, (x1 - x0) / 2.0, (y1 - y0) / 2.0);
                            (0..36).map(|k| { let t = k as f64 / 36.0 * std::f64::consts::TAU; (cx + rx * t.cos(), cy + ry * t.sin()) }).collect()
                        }
                        _ => {
                            // дуга: от угла start на sweep градусов внутри прямоугольника
                            let (cx, cy, rx, ry) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, (x1 - x0) / 2.0, (y1 - y0) / 2.0);
                            let start = op.num_or("start", 0.0);
                            let sweep = op.num_or("sweep", 90.0);
                            let n = ((sweep.abs() / 5.0).ceil() as usize).max(2);
                            (0..=n).map(|k| { let t = (start + sweep * k as f64 / n as f64).to_radians(); (cx + rx * t.cos(), cy - ry * t.sin()) }).collect()
                        }
                    };
                    add_polyline(sp, outline, pen, brush, shape != "arc")
                }
                // «Вставка → Формы»: Windows-элемент (EDIT, BUTTON, COMBOBOX, CHECKBOX, RADIOBUTTON, LISTBOX)
                "control" => {
                    let (x, y) = pts.first().copied().unwrap_or((0.0, 0.0));
                    let class = op.str_or("class", "BUTTON").to_uppercase();
                    let (w, h) = match class.as_str() {
                        "LISTBOX" => (120.0, 80.0),
                        "COMBOBOX" => (120.0, 22.0),
                        "CHECKBOX" | "RADIOBUTTON" => (110.0, 18.0),
                        _ => (90.0, 24.0),
                    };
                    let (w, h) = (op.num_or("w", w), op.num_or("h", h));
                    let text = op.str_or("text", &class.to_lowercase());
                    sp.add_object(Object::new(0, x, y, w, h, Shape::Control { class, caption: text.clone(), style: 0, text, checked: false, enabled: true }))
                }
                _ => {
                    if pts.len() < 2 {
                        return Err("нужны хотя бы две точки".into());
                    }
                    add_polyline(sp, pts, pen, brush, op.get("closed").and_then(Json::as_bool).unwrap_or(false))
                }
            };
            if let Some(o) = sp.objects.get_mut(&h) {
                o.name = op.str_or("name", "");
            }
            Ok(h)
        }
        "move" => {
            let (dx, dy) = (op.num_or("dx", 0.0), op.num_or("dy", 0.0));
            move_object(sp, handle, dx, dy);
            Ok(handle)
        }
        "resize" => {
            let o = sp.objects.get(&handle).ok_or("нет объекта")?.clone();
            let (nx, ny, nw, nh) = (op.num_or("x", o.x), op.num_or("y", o.y), op.num_or("w", o.w).max(0.0), op.num_or("h", o.h).max(0.0));
            let (kx, ky) = (if o.w > 0.0 { nw / o.w } else { 1.0 }, if o.h > 0.0 { nh / o.h } else { 1.0 });
            let obj = sp.objects.get_mut(&handle).ok_or("нет объекта")?;
            obj.x = nx;
            obj.y = ny;
            obj.w = nw;
            obj.h = nh;
            if let Shape::Polyline { points, .. } = &mut obj.shape {
                for p in points.iter_mut() {
                    p.0 *= kx;
                    p.1 *= ky;
                }
            }
            Ok(handle)
        }
        "points" => {
            let pts = points_of(op);
            let (x, y, w, h) = bbox(&pts);
            let obj = sp.objects.get_mut(&handle).ok_or("нет объекта")?;
            if let Shape::Polyline { points, .. } = &mut obj.shape {
                *points = pts.iter().map(|(px, py)| (px - x, py - y)).collect();
                obj.x = x;
                obj.y = y;
                obj.w = w;
                obj.h = h;
            }
            Ok(handle)
        }
        "delete" => {
            if !sp.delete_object(handle) {
                return Err("нет объекта".into());
            }
            Ok(0)
        }
        "set" => {
            let field = op.str_or("field", "");
            let value = op.get("value").map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.as_f64().map(num).unwrap_or_default())).unwrap_or_default();
            if !super::api::set_object_field(sp, handle, &field, &value) {
                return Err(format!("не удалось изменить {field}"));
            }
            Ok(handle)
        }
        "group" => {
            let children: Vec<Handle> = op.get("handles").and_then(Json::as_array).map(|a| a.iter().filter_map(|v| v.as_f64()).map(|v| v as Handle).collect()).unwrap_or_default();
            if children.len() < 2 {
                return Err("для группы нужны хотя бы два объекта".into());
            }
            let g = sp.add_object(Object::new(0, 0.0, 0.0, 0.0, 0.0, Shape::Group { children: children.clone() }));
            for c in &children {
                sp.zorder.retain(|z| z != c);
                if let Some(o) = sp.objects.get_mut(c) {
                    o.parent = Some(g);
                }
            }
            fit_group(sp, g);
            Ok(g)
        }
        "ungroup" => {
            let Some(Shape::Group { children }) = sp.objects.get(&handle).map(|o| o.shape.clone()) else { return Err("не группа".into()) };
            let pos = sp.zorder.iter().position(|z| *z == handle).unwrap_or(sp.zorder.len());
            sp.zorder.retain(|z| *z != handle);
            for (i, c) in children.iter().enumerate() {
                if let Some(o) = sp.objects.get_mut(c) {
                    o.parent = None;
                }
                sp.zorder.insert((pos + i).min(sp.zorder.len()), *c);
            }
            sp.objects.remove(&handle);
            Ok(0)
        }
        "zorder" => {
            let to = op.str_or("to", "top");
            let Some(i) = sp.zorder.iter().position(|z| *z == handle) else { return Err("объект не в списке".into()) };
            let h = sp.zorder.remove(i);
            let at = match to.as_str() {
                "top" => sp.zorder.len(),
                "bottom" => 0,
                "up" => (i + 1).min(sp.zorder.len()),
                _ => i.saturating_sub(1),
            };
            sp.zorder.insert(at, h);
            Ok(handle)
        }
        "duplicate" => {
            let mut o = sp.objects.get(&handle).ok_or("нет объекта")?.clone();
            if let Shape::Group { .. } = o.shape {
                return Err("группы пока не дублируются".into());
            }
            o.x += 16.0;
            o.y += 16.0;
            o.parent = None;
            Ok(sp.add_object(o))
        }
        "page" => {
            if let Some(w) = op.get("w").and_then(Json::as_f64) {
                sp.client.0 = w.max(1.0);
            }
            if let Some(h) = op.get("h").and_then(Json::as_f64) {
                sp.client.1 = h.max(1.0);
            }
            if let Some(x) = op.get("ox").and_then(Json::as_f64) {
                sp.origin.0 = x;
            }
            if let Some(y) = op.get("oy").and_then(Json::as_f64) {
                sp.origin.1 = y;
            }
            Ok(0)
        }
        "clear" => {
            *sp = Space::new(1, "");
            Ok(0)
        }
        // растр: новый пустой («Битовая карта») и запись пикселей из битового редактора
        "bitmap" => {
            let (w, h) = (op.num_or("w", 32.0).max(1.0) as u32, op.num_or("h", 32.0).max(1.0) as u32);
            let (x, y) = (op.num_or("x", 0.0), op.num_or("y", 0.0));
            let rgb = vec![255u8; (w * h * 3) as usize];
            let d = sp.add_dib(crate::gfx::Dib::new(crate::gfx::encode_bmp24(w, h, &rgb), Vec::new(), None));
            Ok(sp.add_object(Object::new(0, x, y, w as f64, h as f64, Shape::Bitmap { dib: d, src: (0.0, 0.0, w as f64, h as f64), masked: false })))
        }
        "dibset" => {
            let dib = match sp.objects.get(&handle).map(|o| &o.shape) {
                Some(Shape::Bitmap { dib, .. }) => *dib,
                _ => return Err("объект — не растр".into()),
            };
            let (w, h) = (op.num_or("w", 0.0) as u32, op.num_or("h", 0.0) as u32);
            let hex = op.str_or("rgb", "");
            if hex.len() != (w * h * 3 * 2) as usize {
                return Err("размер rgb не совпадает с w×h".into());
            }
            let rgb: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()).collect();
            let d = sp.dibs.get_mut(&dib).ok_or("нет растра")?;
            d.width = w;
            d.height = h;
            d.pixels = Some(rgb);
            d.dirty = true;
            d.flush();
            // после обрезки («ножницы») объект принимает новый размер
            if op.get("resize").and_then(Json::as_bool).unwrap_or(false) {
                if let Some(o) = sp.objects.get_mut(&handle) {
                    o.w = w as f64;
                    o.h = h as f64;
                    if let Shape::Bitmap { src, .. } = &mut o.shape {
                        *src = (0.0, 0.0, w as f64, h as f64);
                    }
                }
            }
            Ok(handle)
        }
        // вставка из файла («Вставка → Из файла»): .vdr — как группа, .bmp — растр
        "insert" => {
            let file = op.str_or("file", "");
            let (x, y) = (op.num_or("x", 0.0), op.num_or("y", 0.0));
            let data = std::fs::read(&file).map_err(|e| format!("{file}: {e}"))?;
            if file.to_lowercase().ends_with(".vdr") {
                let pic = vdr::parse(&data, &file).map_err(|e| format!("{file}: {e}"))?;
                Ok(crate::gfx::api::insert_picture(sp, &pic, x, y, true))
            } else if data.starts_with(b"BM") {
                let dib = crate::gfx::Dib::new(data, Vec::new(), None);
                let (w, h) = (dib.width as f64, dib.height as f64);
                let d = sp.add_dib(dib);
                Ok(sp.add_object(Object::new(0, x, y, w, h, Shape::Bitmap { dib: d, src: (0.0, 0.0, w, h), masked: false })))
            } else {
                Err("поддерживаются файлы .vdr и .bmp".into())
            }
        }
        other => Err(format!("неизвестная операция {other}")),
    }
}

fn move_object(sp: &mut Space, handle: Handle, dx: f64, dy: f64) {
    let children = match sp.objects.get(&handle).map(|o| o.shape.clone()) {
        Some(Shape::Group { children }) => children,
        _ => Vec::new(),
    };
    if let Some(o) = sp.objects.get_mut(&handle) {
        o.x += dx;
        o.y += dy;
    }
    for c in children {
        move_object(sp, c, dx, dy);
    }
}

fn fit_group(sp: &mut Space, g: Handle) {
    let children = match sp.objects.get(&g).map(|o| o.shape.clone()) {
        Some(Shape::Group { children }) => children,
        _ => return,
    };
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for c in &children {
        if let Some(o) = sp.objects.get(c) {
            x0 = x0.min(o.x);
            y0 = y0.min(o.y);
            x1 = x1.max(o.x + o.w);
            y1 = y1.max(o.y + o.h);
        }
    }
    if let Some(o) = sp.objects.get_mut(&g) {
        if x0 < x1 {
            o.x = x0;
            o.y = y0;
            o.w = x1 - x0;
            o.h = y1 - y0;
        }
    }
}

/// Обработчик маршрутов `/api/picture/{class}`.
pub fn handle(method: &str, class: &str, kind: Kind, body: &str, s: &mut Shared) -> Result<String, (&'static str, String)> {
    let Some(i) = s.project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(class)) else {
        return Err(("404 Not Found", "нет такого имиджа".into()));
    };
    match method {
        "GET" => {
            let sp = open(&s.project.classes[i], kind);
            Ok(state_json(&sp, kind))
        }
        "POST" => {
            if i >= s.project.own_classes {
                return Err(("403 Forbidden", "библиотечный имидж не редактируется".into()));
            }
            let op = json::parse(body).map_err(|e| ("400 Bad Request", e))?;
            // чтение растра — без записи в проект
            if op.str_or("op", "") == "dibget" {
                let mut sp = open(&s.project.classes[i], kind);
                return dib_get(&mut sp, op.num_or("handle", 0.0) as Handle).map_err(|e| ("400 Bad Request", e));
            }
            let ops: Vec<Json> = match &op {
                Json::Array(items) => items.clone(),
                other => vec![other.clone()],
            };
            let mut sp = open(&s.project.classes[i], kind);
            let mut last = 0;
            for o in &ops {
                last = apply(&mut sp, o).map_err(|e| ("400 Bad Request", e))?;
            }
            s.remember();
            store(&mut s.project.classes[i], kind, &sp);
            Ok(format!("{{\"ok\":true,\"handle\":{last},\"state\":{}}}", state_json(&sp, kind)))
        }
        _ => Err(("405 Method Not Allowed", "метод".into())),
    }
}

/// Пиксели растра объекта как hex RGB — для битового редактора.
fn dib_get(sp: &mut Space, handle: Handle) -> Result<String, String> {
    let dib = match sp.objects.get(&handle).map(|o| &o.shape) {
        Some(Shape::Bitmap { dib, .. }) => *dib,
        _ => return Err("объект — не растр".into()),
    };
    let d = sp.dibs.get_mut(&dib).ok_or("нет растра")?;
    d.pixel(0, 0);
    let (w, h) = (d.width, d.height);
    let hex: String = d.pixels.as_deref().unwrap_or(&[]).iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!("{{\"w\":{w},\"h\":{h},\"rgb\":\"{hex}\"}}"))
}
