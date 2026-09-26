//! 2D-функции языка над графическим пространством.
//!
//! Сигнатуры — из `docs/lang/functions.md`. Возвращаемые значения как в
//! оригинале: дескриптор или 0 при ошибке, 1/0 для успеха.

use super::{Brush, Dib, Font, Gfx, Handle, Object, Pen, Shape, Space, TextPart, WINDOW_BORDER, WINDOW_FRAME, WORKSPACE_ON_SCREEN};
use crate::runtime::value::Value;

fn f(args: &[Value], i: usize) -> f64 {
    args.get(i).map(|v| v.as_float()).unwrap_or(0.0)
}

fn h(args: &[Value], i: usize) -> Handle {
    f(args, i).max(0.0) as Handle
}

fn s(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.as_string()).unwrap_or_default()
}

fn handle(v: Handle) -> Value {
    Value::Handle(v as f64)
}

fn num(v: f64) -> Value {
    Value::Float(v)
}

fn ok(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}

/// Вызывает 2D/оконную функцию. `None` — не наша функция.
pub fn call(name: &str, args: &[Value], gfx: &mut Gfx) -> Option<Value> {
    let lower = name.to_ascii_lowercase();
    let v = match lower.as_str() {
        // ── окна ──────────────────────────────────────────────────────────
        "getwindowspace" => handle(gfx.window_space(&s(args, 0)).unwrap_or(0)),
        "iswindowexist" => ok(gfx.window_space(&s(args, 0)).is_some()),
        "openschemewindow" => handle(open_scheme_window(gfx, &s(args, 0), &s(args, 1))),
        "loadspacewindow" => {
            let window = s(args, 0);
            let file = s(args, 1);
            let pic = if file.is_empty() { None } else { gfx.load_picture_file(&file) };
            // трёхмерные пространства рисунка получают номера раньше листа
            let map = pic.as_ref().filter(|_| gfx.window_space(&window).is_none()).map(|p| gfx.create_spaces3d(p)).unwrap_or_default();
            let sp = gfx.open_window(&window);
            if let Some(space) = gfx.space_mut(sp) {
                space.source_file = file.clone();
            }
            if !file.is_empty() {
                if let Some(pic) = pic {
                    gfx.space_mut(sp).unwrap().load(&pic);
                    gfx.bind_views(sp, &map);
                    gfx.resolve_dibs(sp);
                    gfx.fit_client(sp);
                }
            }
            handle(sp)
        }
        "createwindow" => handle(gfx.open_window(&s(args, 0))),
        // CreateWindowEx(окно, родитель, источник, x, y, ширина, высота, стиль):
        // источник — имя имиджа (окно его схемы, как OpenSchemeWindow) или
        // файл .vdr; пустая строка — пустое окно (сверено в Wine: L3, LGSpaceEx)
        "createwindowex" => {
            let (window, source) = (s(args, 0), s(args, 2));
            // окно с таким именем уже есть — оно и возвращается, без
            // изменений, но номер листа всё равно расходуется (сверено в
            // Wine, tools/verify/reopen.txt)
            if let Some(existing) = gfx.window_space(&window) {
                gfx.next_space = gfx.next_space.max(1) + 1;
                return Some(handle(existing));
            }
            let sp = if source.to_lowercase().ends_with(".vdr") {
                let sp = gfx.open_window(&window);
                if let Some(pic) = gfx.load_picture_file(&source) {
                    gfx.space_mut(sp).unwrap().load(&pic);
                    gfx.resolve_dibs(sp);
                }
                sp
            } else {
                open_scheme_window(gfx, &window, &source)
            };
            let (w, hh) = (f(args, 5), f(args, 6));
            // дочернее окно без рамки встаёт в клиентскую область родителя:
            // x, y — в координатах листа родителя (сверено в Wine: L3,
            // окно «OSC» в «test»: 85 + 4 + (240 − 120) = 209)
            let parent = s(args, 1);
            let child = s(args, 7).to_uppercase().contains("WS_CHILD");
            let offset = window_space(gfx, &parent).map(|p| (WINDOW_BORDER.0 + f(args, 3) - p.origin.0, WINDOW_BORDER.1 + f(args, 4) - p.origin.1));
            if let Some(space) = gfx.space_mut(sp) {
                if w > 0.0 && hh > 0.0 {
                    space.client = (w, hh);
                }
                if let (true, Some(offset)) = (child, offset) {
                    space.child_of = Some((parent, offset));
                }
            }
            handle(sp)
        }
        "closewindow" => ok(gfx.close_window(&s(args, 0))),
        "getclientwidth" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.0).unwrap_or(0.0)),
        "getclientheight" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.1).unwrap_or(0.0)),
        // размеры и положение окна — целые пиксели, дробь отбрасывается;
        // начало листа — округление вниз (сверено в Wine, intsize.txt)
        "setclientsize" => {
            let (w, hh) = (f(args, 1).trunc(), f(args, 2).trunc());
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.client = (w, hh)).is_some())
        }
        "showwindow" => {
            let mode = f(args, 1);
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.visible = mode != 0.0).is_some())
        }
        "setwindoworg" => {
            let org = (f(args, 1).trunc(), f(args, 2).trunc());
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.org = org).is_some())
        }
        // SetWindowPos(окно, x, y, ширина, высота): размеры — всего окна
        "setwindowpos" => {
            let (org, size) = ((f(args, 1).trunc(), f(args, 2).trunc()), (f(args, 3).trunc(), f(args, 4).trunc()));
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| {
                sp.org = org;
                sp.client = ((size.0 - WINDOW_FRAME.0).max(0.0), (size.1 - WINDOW_FRAME.1).max(0.0));
            }).is_some())
        }
        "setwindowtitle" | "setwindowprop" | "bringwindowtotop"
        | "setwindowtransparent" | "setwindowtransparentcolor" => ok(window_space(gfx, &s(args, 0)).is_some()),
        // положение — в экранных координатах, размер — с рамкой и заголовком
        "getwindoworgx" => num(window_screen_org(gfx, &s(args, 0), 0).map(|o| o.0).unwrap_or(0.0)),
        "getwindoworgy" => num(window_screen_org(gfx, &s(args, 0), 0).map(|o| o.1).unwrap_or(0.0)),
        "getwindowwidth" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.0 + if sp.child_of.is_some() { 0.0 } else { WINDOW_FRAME.0 }).unwrap_or(0.0)),
        "getwindowheight" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.1 + if sp.child_of.is_some() { 0.0 } else { WINDOW_FRAME.1 }).unwrap_or(0.0)),
        // GetWindowProp(name, prop): "hwnd" и прочие свойства ОС нам недоступны
        // GetWindowProp(окно, "Classname" | "Filename") — из чего открыто окно
        "getwindowprop" => Value::Str(window_space(gfx, &s(args, 0)).map(|sp| match s(args, 1).to_ascii_lowercase().as_str() {
            "classname" => sp.source_class.clone(),
            "filename" => sp.source_file.clone(),
            _ => String::new(),
        }).unwrap_or_default()),
        "getwindowname" => Value::Str(
            gfx.space(h(args, 0)).map(|sp| sp.window.clone()).unwrap_or_default(),
        ),

        // ── пространство ──────────────────────────────────────────────────
        "getspaceorgx" | "getspaceorg2dx" => num(gfx.space(h(args, 0)).map(|sp| sp.origin.0).unwrap_or(0.0)),
        "getspaceorgy" | "getspaceorg2dy" => num(gfx.space(h(args, 0)).map(|sp| sp.origin.1).unwrap_or(0.0)),
        // SetSpaceOrg — прежнее имя (EDS_IND: окна графиков сдвигаются по
        // объекту-рамке, снимок оригинала)
        "setspaceorg2d" | "setspaceorg" => {
            let (x, y) = (f(args, 1).floor(), f(args, 2).floor());
            ok(gfx.space_mut(h(args, 0)).map(|sp| sp.origin = (x, y)).is_some())
        }
        "setscalespace2d" => {
            let sc = f(args, 1);
            ok(gfx.space_mut(h(args, 0)).map(|sp| sp.scale = (sc, sc)).is_some())
        }
        "getscalespace2d" => num(gfx.space(h(args, 0)).map(|sp| sp.scale.0).unwrap_or(0.0)),
        "emptyspace2d" => ok(gfx.space_mut(h(args, 0)).map(|sp| {
            let (handle, window) = (sp.handle, sp.window.clone());
            *sp = Space::new(handle, &window);
        }).is_some()),
        "lockspace2d" | "setspacerect2d" => ok(gfx.space(h(args, 0)).is_some()),

        // ── объекты: поиск и свойства ─────────────────────────────────────
        // имена работают и для трёхмерных пространств (их дескрипторы общие)
        // GetObject2dByName(HSpace, HGroup, имя): в группе, если она задана
        "getobject2dbyname" => {
            let name = s(args, 2);
            let group = h(args, 1);
            handle(gfx.space(h(args, 0)).and_then(|sp| if group != 0 { sp.find_in_group(group, &name) } else { sp.find_by_name(&name) })
                .or_else(|| super::api3d::find_by_name(gfx, h(args, 0), &name))
                .unwrap_or(0))
        }
        "getobjectname2d" => Value::Str(
            object(gfx, args).map(|o| o.name.clone()).or_else(|| super::api3d::object_name(gfx, h(args, 0), h(args, 1))).unwrap_or_default(),
        ),
        "setobjectname2d" => {
            let name = s(args, 2);
            ok(object_mut(gfx, args).map(|o| o.name = name.clone()).is_some() || super::api3d::set_name(gfx, h(args, 0), h(args, 1), &name))
        }
        "getobjecttype2d" => num(object(gfx, args).map(|o| match o.shape {
            Shape::Group { .. } => 3.0,
            Shape::Polyline { .. } => 20.0,
            Shape::Bitmap { masked: false, .. } => 21.0,
            Shape::Bitmap { masked: true, .. } => 22.0,
            Shape::Text { .. } => 23.0,
            Shape::Control { .. } => 26.0,
            Shape::View3d { .. } => 24.0,
            Shape::Unknown => 0.0,
        }).unwrap_or(0.0)),
        "getobjectorg2dx" => num(object(gfx, args).map(|o| o.x).unwrap_or(0.0)),
        "getobjectorg2dy" => num(object(gfx, args).map(|o| o.y).unwrap_or(0.0)),
        "getobjectwidth2d" => num(object(gfx, args).map(|o| o.w).unwrap_or(0.0)),
        "getobjectheight2d" => num(object(gfx, args).map(|o| o.h).unwrap_or(0.0)),
        "getobjectangle2d" => num(object(gfx, args).map(|o| o.angle).unwrap_or(0.0)),
        "getobjectparent2d" => handle(object(gfx, args).and_then(|o| o.parent).unwrap_or(0)),
        "setobjectorg2d" => {
            let (x, y) = (f(args, 2), f(args, 3));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| sp.move_object(h(args, 1), x, y)))
        }
        "setobjectsize2d" => {
            let (w, hh) = (f(args, 2), f(args, 3));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| sp.resize_object(h(args, 1), w, hh)))
        }
        "rotateobject2d" => {
            let (cx, cy, angle) = (f(args, 2), f(args, 3), f(args, 4));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| rotate(sp, h(args, 1), cx, cy, angle)))
        }
        "hideobject2d" => ok(object_mut(gfx, args).map(|o| o.visible = false).is_some() || super::api3d::set_visible(gfx, h(args, 0), h(args, 1), false)),
        "setshowobject2d" => {
            let show = f(args, 2) != 0.0;
            ok(object_mut(gfx, args).map(|o| o.visible = show).is_some() || super::api3d::set_visible(gfx, h(args, 0), h(args, 1), show))
        }
        "getshowobject2d" => ok(object(gfx, args).is_some_and(|o| o.visible)),
        "showobject2d" => ok(object_mut(gfx, args).map(|o| o.visible = true).is_some() || super::api3d::set_visible(gfx, h(args, 0), h(args, 1), true)),
        "deleteobject2d" => ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| sp.delete_object(h(args, 1))) || super::api3d::delete(gfx, h(args, 0), h(args, 1))),
        // группа места в Z-порядке не имеет: «наверх» для неё ничего не
        // делает и даёт 0, «вниз» — ничего не делает и даёт 1 (сверено в
        // Wine, tools/verify/totop.txt)
        "objecttotop2d" => ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
            if sp.objects.get(&h(args, 1)).is_some_and(|o| o.is_group()) {
                return false;
            }
            sp.to_top(h(args, 1));
            true
        })),
        "objecttobottom2d" => ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
            if !sp.objects.get(&h(args, 1)).is_some_and(|o| o.is_group()) {
                sp.to_bottom(h(args, 1));
            }
            true
        })),
        // место в плоском Z-списке простых объектов; у группы места нет — 0
        // (сверено в Wine, tools/verify/zorder.txt)
        "getzorder2d" => num(gfx.space(h(args, 0))
            .and_then(|sp| sp.zorder.iter().position(|z| *z == h(args, 1)))
            .map(|i| i as f64 + 1.0)
            .unwrap_or(0.0)),
        "setzorder2d" => {
            let pos = f(args, 2).max(1.0) as usize - 1;
            let obj = h(args, 1);
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
                if !sp.zorder.contains(&obj) {
                    return false;
                }
                sp.z_move(obj, pos);
                true
            }))
        }
        "getobjectfrompoint2d" | "getobjectfrompoint2dex" => {
            let (x, y) = (f(args, 1), f(args, 2));
            handle(gfx.space(h(args, 0)).and_then(|sp| sp.object_at(x, y)).unwrap_or(0))
        }
        "getnextobject2d" => {
            // перечисление: 0 → первый в Z-порядке, иначе следующий
            let cur = h(args, 1);
            handle(gfx.space(h(args, 0)).and_then(|sp| {
                if cur == 0 {
                    sp.zorder.first().copied()
                } else {
                    sp.zorder.iter().position(|z| *z == cur).and_then(|i| sp.zorder.get(i + 1)).copied()
                }
            }).unwrap_or(0))
        }
        "setobjectattribute2d" | "getobjectattribute2d" => num(0.0),
        "setobjectlayer2d" => {
            let layer = f(args, 2) as u32;
            ok(object_mut(gfx, args).map(|o| o.layer = layer & 31).is_some())
        }
        "getobjectlayer2d" => num(object(gfx, args).map(|o| o.layer as f64).unwrap_or(0.0)),

        // ── линии ─────────────────────────────────────────────────────────
        "createline2d" | "createpolyline2d" => {
            let (pen, brush) = (h(args, 1), h(args, 2));
            let mut points = Vec::new();
            let mut i = 3;
            while i + 1 < args.len() {
                points.push((f(args, i), f(args, i + 1)));
                i += 2;
            }
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                let hh = sp.add_object(Object::new(0, 0.0, 0.0, 0.0, 0.0, Shape::Polyline { pen, brush, points }));
                sp.refit_polyline(hh);
                hh
            }).unwrap_or(0))
        }
        "addpoint2d" => {
            let (index, x, y) = (f(args, 2) as i64, f(args, 3), f(args, 4));
            ok(with_points(gfx, args, |pts| {
                let at = if index < 0 || index as usize > pts.len() { pts.len() } else { index as usize };
                pts.insert(at, (x, y));
            }))
        }
        "delpoint2d" => {
            let index = f(args, 2) as usize;
            ok(with_points(gfx, args, |pts| {
                if index < pts.len() {
                    pts.remove(index);
                }
            }))
        }
        "setvectorpoint2d" => {
            let (index, x, y) = (f(args, 2) as usize, f(args, 3), f(args, 4));
            ok(with_points(gfx, args, |pts| {
                if let Some(p) = pts.get_mut(index) {
                    *p = (x, y);
                }
            }))
        }
        "getvectorpoint2dx" => num(point(gfx, args).map(|p| p.0).unwrap_or(0.0)),
        "getvectorpoint2dy" => num(point(gfx, args).map(|p| p.1).unwrap_or(0.0)),
        "getvectornumpoints2d" => num(object(gfx, args).map(|o| match &o.shape {
            Shape::Polyline { points, .. } => points.len() as f64,
            _ => 0.0,
        }).unwrap_or(0.0)),
        "getpenobject2d" => handle(object(gfx, args).map(|o| match &o.shape {
            Shape::Polyline { pen, .. } => *pen,
            _ => 0,
        }).unwrap_or(0)),
        "getbrushobject2d" => handle(object(gfx, args).map(|o| match &o.shape {
            Shape::Polyline { brush, .. } => *brush,
            _ => 0,
        }).unwrap_or(0)),
        "setpenobject2d" => {
            let pen = h(args, 2);
            ok(object_mut(gfx, args).map(|o| if let Shape::Polyline { pen: p, .. } = &mut o.shape { *p = pen }).is_some())
        }
        "setbrushobject2d" => {
            let brush = h(args, 2);
            ok(object_mut(gfx, args).map(|o| if let Shape::Polyline { brush: b, .. } = &mut o.shape { *b = brush }).is_some())
        }

        // ── карандаши и кисти ─────────────────────────────────────────────
        "createpen2d" => handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_pen(Pen {
            style: f(args, 1) as u16,
            width: f(args, 2) as u16,
            color: f(args, 3) as u32,
            rop: f(args, 4) as u16,
        })).unwrap_or(0)),
        "createbrush2d" => handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_brush(Brush {
            style: f(args, 1) as u16,
            hatch: f(args, 2) as u16,
            color: f(args, 3) as u32,
            dib: h(args, 4),
            rop: f(args, 5) as u16,
        })).unwrap_or(0)),
        "setpencolor2d" => { let c = f(args, 2) as u32; ok(pen_mut(gfx, args).map(|p| p.color = c).is_some()) }
        "setpenwidth2d" => { let w = f(args, 2) as u16; ok(pen_mut(gfx, args).map(|p| p.width = w).is_some()) }
        "setpenstyle2d" => { let st = f(args, 2) as u16; ok(pen_mut(gfx, args).map(|p| p.style = st).is_some()) }
        "setpenrop2d" => { let r = f(args, 2) as u16; ok(pen_mut(gfx, args).map(|p| p.rop = r).is_some()) }
        "getpencolor2d" => num(pen(gfx, args).map(|p| p.color as f64).unwrap_or(0.0)),
        "getpenwidth2d" => num(pen(gfx, args).map(|p| p.width as f64).unwrap_or(0.0)),
        "getpenstyle2d" => num(pen(gfx, args).map(|p| p.style as f64).unwrap_or(0.0)),
        "setbrushcolor2d" => { let c = f(args, 2) as u32; ok(brush_mut(gfx, args).map(|b| b.color = c).is_some()) }
        "setbrushstyle2d" => { let st = f(args, 2) as u16; ok(brush_mut(gfx, args).map(|b| b.style = st).is_some()) }
        "setbrushhatch2d" => { let ht = f(args, 2) as u16; ok(brush_mut(gfx, args).map(|b| b.hatch = ht).is_some()) }
        "setbrushrop2d" => { let r = f(args, 2) as u16; ok(brush_mut(gfx, args).map(|b| b.rop = r).is_some()) }
        "getbrushcolor2d" => num(brush(gfx, args).map(|b| b.color as f64).unwrap_or(0.0)),
        "getbrushhatch2d" => num(brush(gfx, args).map(|b| b.hatch as f64).unwrap_or(0.0)),
        "getbrushrop2d" => num(brush(gfx, args).map(|b| b.rop as f64).unwrap_or(0.0)),
        "getpenrop2d" => num(pen(gfx, args).map(|p| p.rop as f64).unwrap_or(0.0)),
        "getbrushstyle2d" => num(brush(gfx, args).map(|b| b.style as f64).unwrap_or(0.0)),
        "deletetool2d" => {
            let tool = h(args, 2);
            ok(gfx.space_mut(h(args, 0)).map(|sp| {
                sp.pens.remove(&tool);
                sp.brushes.remove(&tool);
                sp.fonts.remove(&tool);
                sp.strings.remove(&tool);
                sp.texts.remove(&tool);
                sp.dibs.remove(&tool);
            }).is_some())
        }

        // ── шрифты, строки, тексты ────────────────────────────────────────
        "createfont2d" => {
            let flags = f(args, 3) as u32;
            handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_font(Font {
                face: s(args, 1),
                height: f(args, 2) as i32,
                weight: if flags & 8 != 0 { 700 } else { 400 },
                italic: flags & 1 != 0,
                underline: flags & 2 != 0,
            })).unwrap_or(0))
        }
        "createstring2d" => { let t = s(args, 1); handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_string(t)).unwrap_or(0)) }
        "setstring2d" => {
            let t = s(args, 2);
            let hs = h(args, 1);
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| sp.strings.get_mut(&hs).map(|x| *x = t).is_some()))
        }
        "getstring2d" => Value::Str(gfx.space(h(args, 0)).and_then(|sp| sp.strings.get(&h(args, 1)).cloned()).unwrap_or_default()),
        "createtext2d" => {
            let part = TextPart { font: h(args, 1), string: h(args, 2), fg: f(args, 3) as u32, bg: f(args, 4) as u32 };
            handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_text(vec![part])).unwrap_or(0))
        }
        "settext2d" => {
            // SetText2d(HSpace, HText, HFont, HString, fg, bg) или с номером фрагмента
            let (ht, rest) = (h(args, 1), &args[2.min(args.len())..]);
            let (index, r) = if rest.len() >= 5 { (f(rest, 0) as usize, &rest[1..]) } else { (0, rest) };
            let part = TextPart { font: h(r, 0), string: h(r, 1), fg: f(r, 2) as u32, bg: f(r, 3) as u32 };
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| match sp.texts.get_mut(&ht) {
                Some(parts) => {
                    if index < parts.len() { parts[index] = part } else { parts.push(part) }
                    true
                }
                None => false,
            }))
        }
        "gettextobject2d" => handle(object(gfx, args).map(|o| match &o.shape {
            Shape::Text { text } => *text,
            _ => 0,
        }).unwrap_or(0)),
        "gettextstring2d" | "gettextfont2d" | "gettextfgcolor2d" | "gettextbkcolor2d" => {
            let part = gfx.space(h(args, 0)).and_then(|sp| sp.texts.get(&h(args, 1))).and_then(|p| p.first()).cloned();
            match (lower.as_str(), part) {
                ("gettextstring2d", Some(p)) => handle(p.string),
                ("gettextfont2d", Some(p)) => handle(p.font),
                ("gettextfgcolor2d", Some(p)) => num(p.fg as f64),
                ("gettextbkcolor2d", Some(p)) => num(p.bg as f64),
                _ => num(0.0),
            }
        }
        "createrastertext2d" => {
            let (text, x, y, angle) = (h(args, 1), f(args, 2), f(args, 3), f(args, 4));
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                let (w, hh) = text_extent(sp, text);
                let mut obj = Object::new(0, x, y, w, hh, Shape::Text { text });
                obj.angle = angle;
                sp.add_object(obj)
            }).unwrap_or(0))
        }

        // ── растры ────────────────────────────────────────────────────────
        "createdib2d" | "createdoubledib2d" => {
            let file = s(args, 1);
            let data = gfx.find_file(&file).and_then(|p| std::fs::read(p).ok());
            handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_dib(Dib::new(data.unwrap_or_default(), Vec::new(), Some(file)))).unwrap_or(0))
        }
        "createbitmap2d" | "createdoublebitmap2d" => {
            let (dib, x, y) = (h(args, 1), f(args, 2), f(args, 3));
            let masked = lower == "createdoublebitmap2d";
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                let (w, hh) = sp.dibs.get(&dib).map(|d| (d.width as f64, d.height as f64)).unwrap_or((0.0, 0.0));
                sp.add_object(Object::new(0, x, y, w, hh, Shape::Bitmap { dib, src: (0.0, 0.0, w, hh), masked }))
            }).unwrap_or(0))
        }
        "setbitmapsrcrect2d" | "setbitmapsrcrect" => {
            let rect = (f(args, 2), f(args, 3), f(args, 4), f(args, 5));
            ok(object_mut(gfx, args).map(|o| if let Shape::Bitmap { src, .. } = &mut o.shape { *src = rect }).is_some())
        }
        "getdibobject2d" | "getddibobject2d" => handle(object(gfx, args).map(|o| match &o.shape {
            Shape::Bitmap { dib, .. } => *dib,
            _ => 0,
        }).unwrap_or(0)),
        "createobjectfromfile2d" => {
            let (file, x, y, flags) = (s(args, 1), f(args, 2), f(args, 3), f(args, 4) as u32);
            let pic = gfx.load_picture_file(&file);
            let space = h(args, 0);
            let inserted = match (gfx.space_mut(space), pic) {
                (Some(sp), Some(pic)) => insert_picture(sp, &pic, x, y, flags & 0x8000 != 0),
                _ => 0,
            };
            gfx.resolve_dibs(space);
            handle(inserted)
        }

        // ── группы ────────────────────────────────────────────────────────
        "creategroup2d" => {
            let children: Vec<Handle> = (1..args.len()).map(|i| h(args, i)).filter(|c| *c != 0).collect();
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                let g = sp.add_object(Object::new(0, 0.0, 0.0, 0.0, 0.0, Shape::Group { children: children.clone() }));
                // объекты остаются на своих местах в Z-порядке
                for c in &children {
                    if let Some(o) = sp.objects.get_mut(c) {
                        o.parent = Some(g);
                    }
                }
                sp.update_group_bounds(g);
                g
            }).unwrap_or(0))
        }
        "addgroupitem2d" => {
            let (g, item) = (h(args, 1), h(args, 2));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
                let Some(Shape::Group { children }) = sp.objects.get_mut(&g).map(|o| &mut o.shape) else { return false };
                if !children.contains(&item) {
                    children.push(item);
                }
                if let Some(o) = sp.objects.get_mut(&item) {
                    o.parent = Some(g);
                }
                sp.update_group_bounds(g);
                true
            }))
        }
        "delgroupitem2d" => {
            let (g, item) = (h(args, 1), h(args, 2));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
                let Some(Shape::Group { children }) = sp.objects.get_mut(&g).map(|o| &mut o.shape) else { return false };
                children.retain(|c| *c != item);
                if let Some(o) = sp.objects.get_mut(&item) {
                    o.parent = None;
                }
                true
            }))
        }
        "getgroupitemscount2d" => num(object(gfx, args).map(|o| match &o.shape {
            Shape::Group { children } => children.len() as f64,
            _ => 0.0,
        }).unwrap_or(0.0)),
        "getgroupitem2d" => {
            let i = f(args, 2) as usize;
            handle(object(gfx, args).and_then(|o| match &o.shape {
                Shape::Group { children } => children.get(i).copied(),
                _ => None,
            }).unwrap_or(0))
        }

        // ── интерфейсные элементы ─────────────────────────────────────────
        "setcontroltext2d" => {
            let t = s(args, 2);
            ok(object_mut(gfx, args).map(|o| if let Shape::Control { text, .. } = &mut o.shape { *text = t }).is_some())
        }
        "getcontroltext2d" => Value::Str(object(gfx, args).and_then(|o| match &o.shape {
            Shape::Control { text, .. } => Some(text.clone()),
            _ => None,
        }).unwrap_or_default()),
        // как EnableWindow: 1, если элемент был отключён (сверено в Wine,
        // tools/verify/enable.txt)
        "enablecontrol2d" => {
            let e = f(args, 2) != 0.0;
            ok(object_mut(gfx, args).is_some_and(|o| match &mut o.shape {
                Shape::Control { enabled, .. } => {
                    let was_disabled = !*enabled;
                    *enabled = e;
                    was_disabled
                }
                _ => false,
            }))
        }
        "checkdlgbutton2d" => {
            let c = f(args, 2) != 0.0;
            ok(object_mut(gfx, args).map(|o| if let Shape::Control { checked, .. } = &mut o.shape { *checked = c }).is_some())
        }
        "isdlgbuttonchecked2d" => num(object(gfx, args).and_then(|o| match &o.shape {
            Shape::Control { checked, .. } => Some(if *checked { 1.0 } else { 0.0 }),
            _ => None,
        }).unwrap_or(0.0)),
        "getcontrolstyle2d" => num(object(gfx, args).and_then(|o| match &o.shape {
            Shape::Control { style, .. } => Some(*style as f64),
            _ => None,
        }).unwrap_or(0.0)),
        "setcontrolstyle2d" | "setcontrolfont2d" => ok(object(gfx, args).is_some()),
        // CreateControlObject2d(HSpace, ClassName, Text, Style, x, y, w, h)
        "createcontrolobject2d" => {
            let (class, text, style) = (s(args, 1), s(args, 2), f(args, 3) as u32);
            let (x, y, w, hh) = (f(args, 4), f(args, 5), f(args, 6), f(args, 7));
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                sp.add_object(Object::new(0, x, y, w, hh, Shape::Control { class, caption: text.clone(), style, text, checked: false, enabled: true }))
            }).unwrap_or(0))
        }

        // ── шрифты и тексты ───────────────────────────────────────────────
        "createfont2dpt" => {
            let flags = f(args, 3) as u32;
            handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_font(Font {
                face: s(args, 1),
                height: -(f(args, 2) * 96.0 / 72.0).round() as i32,
                weight: if flags & 8 != 0 { 700 } else { 400 },
                italic: flags & 1 != 0,
                underline: flags & 2 != 0,
            })).unwrap_or(0))
        }
        "getfontname2d" => Value::Str(gfx.space(h(args, 0)).and_then(|sp| sp.fonts.get(&h(args, 1))).map(|f| f.face.clone()).unwrap_or_default()),
        "getfontsize2d" => num(gfx.space(h(args, 0)).and_then(|sp| sp.fonts.get(&h(args, 1))).map(|f| (f.height.abs() as f64 * 72.0 / 96.0).round()).unwrap_or(0.0)),
        "getfontstyle2d" => num(gfx.space(h(args, 0)).and_then(|sp| sp.fonts.get(&h(args, 1))).map(|f| {
            (if f.italic { 1.0 } else { 0.0 }) + (if f.underline { 2.0 } else { 0.0 }) + (if f.weight >= 700 { 8.0 } else { 0.0 })
        }).unwrap_or(0.0)),
        "setfontname2d" => {
            let face = s(args, 2);
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.fonts.get_mut(&h(args, 1))).map(|f| f.face = face).is_some())
        }
        "setfontsize2d" => {
            let pt = f(args, 2);
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.fonts.get_mut(&h(args, 1))).map(|f| f.height = -(pt * 96.0 / 72.0).round() as i32).is_some())
        }
        "setfontstyle2d" => {
            let st = f(args, 2) as u32;
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.fonts.get_mut(&h(args, 1))).map(|f| {
                f.italic = st & 1 != 0;
                f.underline = st & 2 != 0;
                f.weight = if st & 8 != 0 { 700 } else { 400 };
            }).is_some())
        }
        "getfontlist" => Value::Handle(0.0),
        "gettextcount2d" => num(gfx.space(h(args, 0)).and_then(|sp| sp.texts.get(&h(args, 1))).map(|t| t.len() as f64).unwrap_or(0.0)),
        "settextstring2d" | "settextfont2d" | "settextfgcolor2d" | "settextbkcolor2d" => {
            let idx = f(args, 2) as usize;
            let v = f(args, 3);
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.texts.get_mut(&h(args, 1))).and_then(|parts| parts.get_mut(idx)).map(|p| match lower.as_str() {
                "settextstring2d" => p.string = v as Handle,
                "settextfont2d" => p.font = v as Handle,
                "settextfgcolor2d" => p.fg = v as u32,
                _ => p.bg = v as u32,
            }).is_some())
        }
        // AddText2d(HSpace, HText, index, HFont, HString, fg, bg)
        "addtext2d" => {
            let idx = f(args, 2) as usize;
            let part = super::TextPart { font: h(args, 3), string: h(args, 4), fg: f(args, 5) as u32, bg: f(args, 6) as u32 };
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.texts.get_mut(&h(args, 1))).map(|parts| parts.insert(idx.min(parts.len()), part)).is_some())
        }
        "removetext2d" => {
            let idx = f(args, 2) as usize;
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.texts.get_mut(&h(args, 1))).is_some_and(|parts| {
                if idx < parts.len() { parts.remove(idx); true } else { false }
            }))
        }
        "gettextbkcolor" | "gettextfgcolor" => Value::Color(0.0),

        // ── инструменты и растры ─────────────────────────────────────────
        "gettoolref2d" => num(1.0),
        "getnexttool2d" => {
            let kind = f(args, 1) as u32;
            let cur = h(args, 2);
            handle(gfx.space(h(args, 0)).map(|sp| {
                let keys: Vec<Handle> = match kind {
                    1 => sp.pens.keys().copied().collect(),
                    2 => sp.brushes.keys().copied().collect(),
                    3 => sp.dibs.keys().copied().collect(),
                    5 => sp.fonts.keys().copied().collect(),
                    6 => sp.strings.keys().copied().collect(),
                    7 => sp.texts.keys().copied().collect(),
                    _ => Vec::new(),
                };
                if cur == 0 { keys.first().copied().unwrap_or(0) } else { keys.iter().skip_while(|k| **k != cur).nth(1).copied().unwrap_or(0) }
            }).unwrap_or(0))
        }
        "getbrushdib2d" => handle(gfx.space(h(args, 0)).and_then(|sp| sp.brushes.get(&h(args, 1))).map(|b| b.dib).unwrap_or(0)),
        "setbrushdib2d" => {
            let dib = h(args, 2);
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.brushes.get_mut(&h(args, 1))).map(|b| b.dib = dib).is_some())
        }
        "setbkbrush2d" | "getbkbrush2d" | "setcrdsystem2d" | "setrgncreatemode" | "setlinearrows2d" | "setpoints2d" => ok(gfx.space(h(args, 0)).is_some()),
        // гиперссылка объекта: mode (-1 снять, 0 окно, 3 ничего), target, window,
        // object, effect; неуказанные аргументы не меняются
        "sethyperjump2d" => {
            let mode = f(args, 2) as i32;
            let given = |i: usize| (args.len() > i).then(|| s(args, i));
            ok(object_mut(gfx, args).map(|o| {
                if mode < 0 { o.hyper = None; } else {
                    let mut hy = o.hyper.clone().unwrap_or_default();
                    hy.mode = mode;
                    if let Some(v) = given(3) { hy.target = v; }
                    if let Some(v) = given(4) { hy.window = v; }
                    if let Some(v) = given(5) { hy.object = v; }
                    if let Some(v) = given(6) { hy.effect = v; }
                    o.hyper = Some(hy);
                }
            }).is_some())
        }
        "gethyperjump2d" => num(object(gfx, args).and_then(|o| o.hyper.as_ref()).map(|h| h.mode as f64).unwrap_or(-1.0)),
        "setspacelayers2d" => {
            let mask = f(args, 1) as i64 as u32;
            ok(gfx.space_mut(h(args, 0)).map(|sp| sp.layers = mask).is_some())
        }
        "getspacelayers2d" => num(gfx.space(h(args, 0)).map(|sp| sp.layers as f64).unwrap_or(0.0)),
        "getrgncreatemode" => num(0.0),
        "createrdib2d" | "createrdoubledib2d" => {
            let file = s(args, 1);
            let data = gfx.find_file(&file).and_then(|p| std::fs::read(p).ok());
            handle(gfx.space_mut(h(args, 0)).map(|sp| sp.add_dib(Dib::new(data.unwrap_or_default(), Vec::new(), Some(file)))).unwrap_or(0))
        }
        "setdibobject2d" | "setddibobject2d" => {
            let dib = h(args, 2);
            ok(object_mut(gfx, args).map(|o| if let Shape::Bitmap { dib: d, .. } = &mut o.shape { *d = dib }).is_some())
        }
        "setrdib2d" | "setrdoubledib2d" => {
            let file = s(args, 2);
            let data = gfx.find_file(&file).and_then(|p| std::fs::read(p).ok());
            ok(gfx.space_mut(h(args, 0)).and_then(|sp| sp.dibs.get_mut(&h(args, 1))).map(|d| {
                *d = Dib::new(data.unwrap_or_default(), Vec::new(), Some(file));
            }).is_some())
        }
        "getrdib2d" | "getrdoubledib2d" => Value::Str(gfx.space(h(args, 0)).and_then(|sp| sp.dibs.get(&h(args, 1))).and_then(|d| d.file.clone()).unwrap_or_default()),

        // ── объекты: порядок, группы, буфер обмена ────────────────────────
        "isobjectsintersect2d" => {
            let (a, b) = (h(args, 1), h(args, 2));
            ok(gfx.space(h(args, 0)).and_then(|sp| Some((sp.objects.get(&a)?, sp.objects.get(&b)?))).is_some_and(|(p, q)| {
                p.x < q.x + q.w && q.x < p.x + p.w && p.y < q.y + q.h && q.y < p.y + p.h
            }))
        }
        "setcurrentobject2d" => ok(object(gfx, args).is_some()),
        "getcurrentobject2d" => handle(0),
        "lockobject2d" => ok(object(gfx, args).is_some()),
        "getlastprimary2d" => handle(gfx.last_primary),
        "copytoclipboard2d" => {
            let copied = gfx.space(h(args, 0)).and_then(|sp| {
                let o = sp.objects.get(&h(args, 1))?.clone();
                let (pen, brush) = match &o.shape {
                    Shape::Polyline { pen, brush, .. } => (sp.pens.get(pen).cloned(), sp.brushes.get(brush).cloned()),
                    _ => (None, None),
                };
                Some((o, pen, brush))
            });
            gfx.clipboard = copied;
            ok(gfx.clipboard.is_some())
        }
        // PasteFromClipboard2d(HSpace, x, y, flags)
        "pastefromclipboard2d" => {
            let (x, y) = (f(args, 1), f(args, 2));
            let Some((mut o, pen, brush)) = gfx.clipboard.clone() else { return Some(handle(0)) };
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                if let Shape::Polyline { pen: p, brush: b, .. } = &mut o.shape {
                    if let Some(pen) = pen { *p = sp.add_pen(pen); }
                    if let Some(brush) = brush { *b = sp.add_brush(brush); }
                }
                o.x = x;
                o.y = y;
                o.parent = None;
                sp.add_object(o)
            }).unwrap_or(0))
        }
        "getbottomobject2d" => handle(gfx.space(h(args, 0)).and_then(|sp| sp.zorder.first().copied()).unwrap_or(0)),
        "gettopobject2d" => handle(gfx.space(h(args, 0)).and_then(|sp| sp.zorder.last().copied()).unwrap_or(0)),
        "getupperobject2d" | "getlowerobject2d" => {
            let cur = h(args, 1);
            handle(gfx.space(h(args, 0)).and_then(|sp| {
                let i = sp.zorder.iter().position(|z| *z == cur)?;
                if lower == "getupperobject2d" { sp.zorder.get(i + 1).copied() } else { i.checked_sub(1).and_then(|k| sp.zorder.get(k).copied()) }
            }).unwrap_or(0))
        }
        "getobjectfromzorder2d" => {
            let n = f(args, 1).max(1.0) as usize - 1;
            handle(gfx.space(h(args, 0)).and_then(|sp| sp.zorder.get(n).copied()).unwrap_or(0))
        }
        "swapobject2d" => {
            let (a, b) = (h(args, 1), h(args, 2));
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
                match (sp.zorder.iter().position(|z| *z == a), sp.zorder.iter().position(|z| *z == b)) {
                    (Some(i), Some(j)) => { sp.zorder.swap(i, j); true }
                    _ => false,
                }
            }))
        }
        "deletegroup2d" => {
            let g = h(args, 1);
            ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| {
                // группа распадается, её объекты остаются на своих местах
                let Some(Shape::Group { children }) = sp.objects.get(&g).map(|o| o.shape.clone()) else { return false };
                for c in &children {
                    if let Some(o) = sp.objects.get_mut(c) { o.parent = None; }
                }
                if let Some(o) = sp.objects.get_mut(&g) {
                    o.shape = Shape::Group { children: Vec::new() };
                }
                sp.delete_object(g)
            }))
        }
        "getgroupitemsnum2d" => num(object(gfx, args).map(|o| match &o.shape { Shape::Group { children } => children.len() as f64, _ => 0.0 }).unwrap_or(0.0)),
        "isgroupcontainobject2d" => {
            let item = h(args, 2);
            ok(object(gfx, args).is_some_and(|o| matches!(&o.shape, Shape::Group { children } if children.contains(&item))))
        }
        "setgroupitem2d" => {
            let (idx, item) = (f(args, 2) as usize, h(args, 3));
            ok(object_mut(gfx, args).is_some_and(|o| match &mut o.shape {
                Shape::Group { children } if idx < children.len() => { children[idx] = item; true }
                _ => false,
            }))
        }
        "setgroupitems2d" => {
            let items: Vec<Handle> = (2..args.len()).map(|i| h(args, i)).filter(|c| *c != 0).collect();
            ok(object_mut(gfx, args).is_some_and(|o| match &mut o.shape {
                Shape::Group { children } => { *children = items; true }
                _ => false,
            }))
        }

        // ── списки (LISTBOX/COMBOBOX): строки контрола через перевод строки ──
        "lbaddstring" => {
            let t = s(args, 2);
            num(object_mut(gfx, args).and_then(|o| match &mut o.shape {
                Shape::Control { text, .. } => {
                    if !text.is_empty() { text.push('\n'); }
                    text.push_str(&t);
                    Some(text.lines().count() as f64 - 1.0)
                }
                _ => None,
            }).unwrap_or(-1.0))
        }
        "lbclearlist" => ok(object_mut(gfx, args).map(|o| if let Shape::Control { text, .. } = &mut o.shape { text.clear() }).is_some()),
        "lbdeletestring" => {
            let i = f(args, 2) as usize;
            ok(object_mut(gfx, args).is_some_and(|o| match &mut o.shape {
                Shape::Control { text, .. } => {
                    let mut lines: Vec<&str> = text.lines().collect();
                    if i < lines.len() { lines.remove(i); *text = lines.join("\n"); true } else { false }
                }
                _ => false,
            }))
        }
        "lbgetstring" => {
            let i = f(args, 2) as usize;
            Value::Str(object(gfx, args).and_then(|o| match &o.shape { Shape::Control { text, .. } => text.lines().nth(i).map(str::to_string), _ => None }).unwrap_or_default())
        }
        "lbfindstring" | "lbfindstringexact" => {
            let needle = s(args, 3).to_lowercase();
            let from = f(args, 2).max(0.0) as usize;
            num(object(gfx, args).and_then(|o| match &o.shape {
                Shape::Control { text, .. } => text.lines().enumerate().skip(from).find(|(_, l)| {
                    let l = l.to_lowercase();
                    if lower == "lbfindstringexact" { l == needle } else { l.starts_with(&needle) }
                }).map(|(i, _)| i as f64),
                _ => None,
            }).unwrap_or(-1.0))
        }
        // выбранная строка списка хранится в старшем байте стиля контрола
        "lbgetcaretindex" | "lbgetselindex" => num(object(gfx, args).map(|o| match &o.shape {
            Shape::Control { style, text, .. } if !text.is_empty() => (*style >> 24) as f64,
            _ => -1.0,
        }).unwrap_or(-1.0)),
        "lbsetcaretindex" | "lbsetselindex" => {
            let i = f(args, 2).clamp(0.0, 255.0) as u32;
            ok(object_mut(gfx, args).map(|o| if let Shape::Control { style, .. } = &mut o.shape { *style = (*style & 0x00ff_ffff) | (i << 24) }).is_some())
        }
        "lbgetcount" => num(object(gfx, args).map(|o| match &o.shape { Shape::Control { text, .. } => text.lines().count() as f64, _ => 0.0 }).unwrap_or(0.0)),
        "lbinsertstring" => {
            let (t, at) = (s(args, 2), f(args, 3));
            num(object_mut(gfx, args).and_then(|o| match &mut o.shape {
                Shape::Control { text, .. } => {
                    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
                    let i = if at < 0.0 { lines.len() } else { (at as usize).min(lines.len()) };
                    lines.insert(i, t.clone());
                    *text = lines.join("\n");
                    Some(i as f64)
                }
                _ => None,
            }).unwrap_or(-1.0))
        }
        "addcontroltext2d" => {
            let t = s(args, 2);
            ok(object_mut(gfx, args).map(|o| if let Shape::Control { text, .. } = &mut o.shape { text.push_str(&t) }).is_some())
        }
        // цвет текста контрола рисует страница; ядру достаточно принять вызов
        "setcontroltextcolor2d" => ok(object(gfx, args).is_some()),
        "getcontroltextlength2d" => num(object(gfx, args).map(|o| match &o.shape { Shape::Control { text, .. } => text.chars().count() as f64, _ => 0.0 }).unwrap_or(0.0)),
        "setcontrolfocus2d" | "dbsetcontroltable" => ok(object(gfx, args).is_some()),

        // ── прозрачность и растр по пикселям ─────────────────────────────
        "setobjectalpha2d" => {
            let a = f(args, 2).clamp(0.0, 255.0) as u8;
            ok(object_mut(gfx, args).map(|o| o.alpha = a).is_some())
        }
        "getobjectsize2dx" | "getactualwidth2d" => num(object(gfx, args).map(|o| o.w).unwrap_or(0.0)),
        "getobjectsize2dy" | "getactualheight2d" => num(object(gfx, args).map(|o| o.h).unwrap_or(0.0)),
        "getschemeobject" | "framegetpos2d" => num(0.0),
        "audiosetvolume" | "audiosettone" | "beginwritevideo2d" | "endwritevideo2d" | "writevideoframe2d" | "closevideo" | "videocompressdialog" | "saverectarea2d" => ok(true),
        // сенсор Kinect и сетевые объекты: устройств нет
        "nui_init" | "nui_initinstance" | "nui_createinstance" | "nui_getdevicecount" | "registernetobject" | "initanalyzer" => num(0.0),
        "getwindowtitle" => Value::Str(gfx.window_space(&s(args, 0)).and_then(|h| gfx.space(h)).map(|sp| sp.window.clone()).unwrap_or_default()),
        "iswindowvisible" => ok(gfx.window_space(&s(args, 0)).and_then(|h| gfx.space(h)).is_some_and(|sp| sp.visible)),
        "isiconic" => num(0.0),
        "setwindowsize" => {
            let (w, hh) = (f(args, 1).trunc(), f(args, 2).trunc());
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.client = (w, hh)).is_some())
        }
        "getprojectprop" => Value::Str(String::new()),
        "setprojectprop" => ok(true),
        "removetexture" => ok(true),
        "getobjectalpha2d" => num(object(gfx, args).map(|o| o.alpha as f64).unwrap_or(255.0)),
        "getdibpixel2d" | "getddibpixel2d" => {
            let (x, y) = (f(args, 2) as i64, f(args, 3) as i64);
            let v = gfx.space_mut(h(args, 0)).and_then(|sp| sp.dibs.get_mut(&h(args, 1))).and_then(|d| d.pixel(x, y));
            Value::Color(v.unwrap_or(0) as f64)
        }
        "setdibpixel2d" | "setddibpixel2d" => {
            let (x, y, c) = (f(args, 2) as i64, f(args, 3) as i64, f(args, 4) as u32);
            let done = gfx.space_mut(h(args, 0)).and_then(|sp| sp.dibs.get_mut(&h(args, 1))).and_then(|d| d.set_pixel(x, y, c));
            ok(done.is_some())
        }

        // ── экран и рабочая область ───────────────────────────────────────
        // значения эталонного окружения, в котором сняты размеры окон
        // (tools/verify/windows.txt)
        "getscreenwidth" | "getworkareawidth" => num(1920.0),
        "getscreenheight" | "getworkareaheight" => num(1080.0),
        "getworkareax" => num(0.0),
        "getworkareay" => num(32.0),
        "getprojectdirectory" => Value::Str(gfx.project_dir.display().to_string()),

        // ── принимаем без действия: градиенты, движок, строка состояния, звук ──
        "setbrushpoints2d" | "setbrushcolors2d" | "setspacerenderengine2d" | "setstatustext" | "setlogstring2d"
        | "videodialog" => ok(true),
        "system" => num(0.0),

        _ => return None,
    };
    Some(v)
}

/// Габариты объекта (для `GetActualSize2d`).
pub fn object_size(gfx: &Gfx, args: &[Value]) -> Option<(f64, f64)> {
    object(gfx, args).map(|o| (o.w, o.h))
}

/// Прямоугольник источника растра (для `GetBitmapSrcRect2d`).
pub fn bitmap_src(gfx: &Gfx, args: &[Value]) -> Option<(f64, f64, f64, f64)> {
    match &object(gfx, args)?.shape {
        Shape::Bitmap { src, .. } => Some(*src),
        _ => None,
    }
}

fn window_space<'a>(gfx: &'a Gfx, name: &str) -> Option<&'a Space> {
    gfx.window_space(name).and_then(|h| gfx.space(h))
}

/// Левый верхний угол окна на экране; дочернее окно — от угла родителя.
fn window_screen_org(gfx: &Gfx, name: &str, depth: u32) -> Option<(f64, f64)> {
    let sp = window_space(gfx, name)?;
    match &sp.child_of {
        Some((parent, off)) if depth < 16 => window_screen_org(gfx, parent, depth + 1).map(|p| (p.0 + off.0, p.1 + off.1)),
        _ => Some((sp.org.0 + WORKSPACE_ON_SCREEN.0, sp.org.1 + WORKSPACE_ON_SCREEN.1)),
    }
}

fn window_space_mut<'a>(gfx: &'a mut Gfx, name: &str) -> Option<&'a mut Space> {
    let h = gfx.window_space(name)?;
    gfx.space_mut(h)
}

fn object<'a>(gfx: &'a Gfx, args: &[Value]) -> Option<&'a Object> {
    gfx.space(h(args, 0))?.objects.get(&h(args, 1))
}

fn object_mut<'a>(gfx: &'a mut Gfx, args: &[Value]) -> Option<&'a mut Object> {
    gfx.space_mut(h(args, 0))?.objects.get_mut(&h(args, 1))
}

fn pen<'a>(gfx: &'a Gfx, args: &[Value]) -> Option<&'a Pen> {
    gfx.space(h(args, 0))?.pens.get(&h(args, 1))
}

fn pen_mut<'a>(gfx: &'a mut Gfx, args: &[Value]) -> Option<&'a mut Pen> {
    gfx.space_mut(h(args, 0))?.pens.get_mut(&h(args, 1))
}

fn brush<'a>(gfx: &'a Gfx, args: &[Value]) -> Option<&'a Brush> {
    gfx.space(h(args, 0))?.brushes.get(&h(args, 1))
}

fn brush_mut<'a>(gfx: &'a mut Gfx, args: &[Value]) -> Option<&'a mut Brush> {
    gfx.space_mut(h(args, 0))?.brushes.get_mut(&h(args, 1))
}

fn point(gfx: &Gfx, args: &[Value]) -> Option<(f64, f64)> {
    match &object(gfx, args)?.shape {
        Shape::Polyline { points, .. } => points.get(f(args, 2) as usize).copied(),
        _ => None,
    }
}

fn with_points(gfx: &mut Gfx, args: &[Value], edit: impl FnOnce(&mut Vec<(f64, f64)>)) -> bool {
    let Some(sp) = gfx.space_mut(h(args, 0)) else { return false };
    let obj = h(args, 1);
    let Some(Shape::Polyline { points, .. }) = sp.objects.get_mut(&obj).map(|o| &mut o.shape) else { return false };
    edit(points);
    sp.refit_polyline(obj);
    if let Some(parent) = sp.objects.get(&obj).and_then(|o| o.parent) {
        sp.update_group_bounds(parent);
    }
    true
}

fn rotate(sp: &mut Space, obj: Handle, cx: f64, cy: f64, angle: f64) -> bool {
    let (sin, cos) = angle.sin_cos();
    let children = match sp.objects.get_mut(&obj) {
        Some(o) => {
            o.angle += angle;
            match &mut o.shape {
                Shape::Polyline { points, .. } => {
                    for p in points.iter_mut() {
                        let (dx, dy) = (p.0 - cx, p.1 - cy);
                        *p = (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos);
                    }
                    Vec::new()
                }
                Shape::Group { children } => children.clone(),
                _ => {
                    // прочие объекты поворачиваются как точка-начало
                    let (dx, dy) = (o.x - cx, o.y - cy);
                    o.x = cx + dx * cos - dy * sin;
                    o.y = cy + dx * sin + dy * cos;
                    Vec::new()
                }
            }
        }
        None => return false,
    };
    sp.refit_polyline(obj);
    for c in children {
        rotate(sp, c, cx, cy, angle);
    }
    sp.update_group_bounds(obj);
    true
}

/// Оценка габаритов текста: точных метрик шрифта в ядре нет.
fn text_extent(sp: &Space, text: Handle) -> (f64, f64) {
    let Some(parts) = sp.texts.get(&text) else { return (0.0, 0.0) };
    let mut width = 0.0;
    let mut height: f64 = 0.0;
    for p in parts {
        let h = sp.fonts.get(&p.font).map(|f| f.height.abs() as f64).unwrap_or(12.0).max(1.0);
        let chars = sp.strings.get(&p.string).map(|s| s.chars().count()).unwrap_or(0) as f64;
        width += chars * h * 0.55;
        height = height.max(h);
    }
    (width, height)
}

/// Вставляет содержимое рисунка в пространство как одну группу, возвращает
/// её дескриптор (или единственный объект, если он один).
pub fn insert_picture(sp: &mut Space, pic: &super::Picture, x: f64, y: f64, move_to: bool) -> Handle {
    // рисунок из нескольких объектов получает обёртку по тому же правилу,
    // что и рисунки детей в окне схемы (сверено в Wine: VIDEO, NumberView —
    // группа 1, текст 2)
    let pic = with_wrapper(pic);
    let (top, zorder) = insert_objects(sp, &pic);
    sp.zorder.extend(zorder);
    let Some(&root) = top.first() else { return 0 };
    if matches!(sp.objects.get(&root).map(|o| &o.shape), Some(Shape::Group { .. })) {
        sp.update_group_bounds(root);
    }
    if move_to {
        sp.move_object(root, x, y);
    }
    root
}

/// Рисунок из нескольких объектов верхнего уровня получает обёртку — группу
/// внутри рисунка с наименьшим свободным в нём номером; дальше объекты и
/// она получают номера пространства по порядку своих номеров.
fn with_wrapper(pic: &super::Picture) -> super::Picture {
    use crate::formats::vdr::{Object as VObject, ObjectKind};
    let tops: Vec<u16> = pic.objects.iter().filter(|o| !pic.objects.iter().any(|g| matches!(&g.kind, ObjectKind::Group { children } if children.contains(&o.handle)))).map(|o| o.handle).collect();
    let mut pic = pic.clone();
    if tops.len() > 1 {
        let used: std::collections::BTreeSet<u16> = pic.objects.iter().map(|o| o.handle).collect();
        let w = (1..=u16::MAX).find(|h| !used.contains(h)).unwrap_or(u16::MAX);
        pic.objects.push(VObject { handle: w, name: String::new(), flags: 0, kind: ObjectKind::Group { children: tops } });
    }
    pic
}

/// Объекты и инструменты рисунка — в пространство с новыми дескрипторами
/// (наименьшими свободными). Возвращает объекты верхнего уровня и простые
/// объекты в Z-порядке рисунка; в Z-порядок пространства их дописывает
/// вызывающий.
pub fn insert_objects(sp: &mut Space, pic: &super::Picture) -> (Vec<Handle>, Vec<Handle>) {
    let mut sub = Space::new(0, "");
    sub.load(pic);
    // инструменты переносятся с новыми дескрипторами
    let mut pens = std::collections::HashMap::new();
    let mut brushes = std::collections::HashMap::new();
    let mut fonts = std::collections::HashMap::new();
    let mut strings = std::collections::HashMap::new();
    let mut texts = std::collections::HashMap::new();
    let mut dibs = std::collections::HashMap::new();
    for (k, v) in &sub.pens { pens.insert(*k, sp.add_pen(v.clone())); }
    for (k, v) in &sub.brushes {
        let mut b = v.clone();
        b.dib = *dibs.entry(b.dib).or_insert_with(|| sub.dibs.get(&b.dib).map(|d| sp.add_dib(d.clone())).unwrap_or(0));
        brushes.insert(*k, sp.add_brush(b));
    }
    for (k, v) in &sub.fonts { fonts.insert(*k, sp.add_font(v.clone())); }
    for (k, v) in &sub.strings { strings.insert(*k, sp.add_string(v.clone())); }
    for (k, v) in &sub.dibs { dibs.entry(*k).or_insert_with(|| sp.add_dib(v.clone())); }
    for (k, v) in &sub.texts {
        let parts = v.iter().map(|p| TextPart {
            fg: p.fg,
            bg: p.bg,
            font: fonts.get(&p.font).copied().unwrap_or(0),
            string: strings.get(&p.string).copied().unwrap_or(0),
        }).collect();
        texts.insert(*k, sp.add_text(parts));
    }
    // объекты: новые дескрипторы (наименьшие свободные, по порядку старых),
    // ссылки переписываются
    let mut map = std::collections::HashMap::new();
    let mut order: Vec<Handle> = sub.objects.keys().copied().collect();
    order.sort();
    let mut taken: std::collections::BTreeSet<Handle> = sp.objects.keys().copied().collect();
    let mut next: Handle = 1;
    for old in &order {
        while taken.contains(&next) {
            next += 1;
        }
        taken.insert(next);
        map.insert(*old, next);
    }
    let mut top = Vec::new();
    for old in &order {
        let mut o = sub.objects[old].clone();
        o.handle = map[old];
        o.parent = o.parent.and_then(|p| map.get(&p).copied());
        o.shape = match o.shape {
            Shape::Polyline { pen, brush, points } => Shape::Polyline {
                pen: pens.get(&pen).copied().unwrap_or(0),
                brush: brushes.get(&brush).copied().unwrap_or(0),
                points,
            },
            Shape::Bitmap { dib, src, masked } => Shape::Bitmap { dib: dibs.get(&dib).copied().unwrap_or(0), src, masked },
            Shape::Text { text } => Shape::Text { text: texts.get(&text).copied().unwrap_or(0) },
            Shape::Group { children } => Shape::Group { children: children.iter().filter_map(|c| map.get(c).copied()).collect() },
            other => other,
        };
        if o.parent.is_none() {
            top.push(o.handle);
        }
        sp.objects.insert(o.handle, o);
    }
    let zorder: Vec<Handle> = sub.zorder.iter().filter_map(|z| map.get(z).copied()).collect();
    (top, zorder)
}

/// Схема в окне: на месте значка каждого дочернего имиджа (группа-элемент
/// схемы с handle экземпляра) оригинал показывает рисунок этого имиджа —
/// объекты вставляются в группу с наименьшими свободными номерами, по
/// порядку детей схемы. Вглубь не идёт: рисунок ребёнка — его собственный,
/// значков его схемы в нём нет. Установлено по снимку оригинала
/// («Солнечная система»: текст NumberView получает номера 16, 21, 26, а
/// GetObject2dByName(HSpace, _HObject, "text") находит его внутри значка).
/// Окно схемы имиджа (OpenSchemeWindow): рисунок схемы с рисунками детей.
fn open_scheme_window(gfx: &mut Gfx, window: &str, class: &str) -> Handle {
    let window = window.to_string();
    let class = class.to_string();
    // OpenSchemeWindow на уже открытое окно пересоздаёт лист: новый
    // дескриптор, прежние объекты пропадают (сверено в Wine, reopen.txt)
    if gfx.window_space(&window).is_some() {
        gfx.close_window(&window);
    }
    let pic = if class.is_empty() { None } else { gfx.pictures.get(&class.to_lowercase()).cloned() };
    let map = pic.as_ref().map(|p| gfx.create_spaces3d(p)).unwrap_or_default();
    let sp = gfx.open_window(&window);
    if let Some(space) = gfx.space_mut(sp) {
        space.source_class = class.clone();
    }
    if !class.is_empty() {
        gfx.hyper_current.insert(window.clone(), class.clone());
        if let Some(pic) = pic {
            gfx.space_mut(sp).unwrap().load(&pic);
            gfx.bind_views(sp, &map);
            embed_children(gfx, sp, &class);
            gfx.resolve_dibs(sp);
            gfx.fit_client(sp);
        }
        // «Параметры листа → Окно»: заданный размер и маска слоёв
        if let Some(sheet) = gfx.sheets.get(&class.to_lowercase()).cloned() {
            let space = gfx.space_mut(sp).unwrap();
            if sheet.window_size == "fixed" && sheet.window_wh.0 > 0.0 && sheet.window_wh.1 > 0.0 {
                space.client = sheet.window_wh;
            }
            space.window_size = sheet.window_size.clone();
            space.window_style = sheet.window_style.clone();
            space.layers = sheet.layers;
        }
    }
    sp
}

fn embed_children(gfx: &mut Gfx, sp: Handle, class: &str) {
    let mut children = gfx.class_children.get(&crate::lang::fold(class)).cloned().unwrap_or_default();
    // рисунки раздаются по имиджам: сначала все экземпляры имиджа, который
    // встретился в схеме первым, потом следующего (сверено в Wine: DIFF,
    // tools/verify/diffgroups.txt)
    let mut first: Vec<String> = Vec::new();
    for (_, c) in &children {
        let c = crate::lang::fold(c);
        if !first.contains(&c) {
            first.push(c);
        }
    }
    children.sort_by_key(|(_, c)| first.iter().position(|f| *f == crate::lang::fold(c)));
    for (handle, child_class) in children {
        let group = handle as Handle;
        let Some(pic) = gfx.scheme_pictures.get(&crate::lang::fold(&child_class)).cloned() else { continue };
        let Some(space) = gfx.space_mut(sp) else { return };
        match space.objects.get(&group) {
            Some(o) if matches!(o.shape, Shape::Group { .. }) && o.scheme_element => {}
            _ => continue,
        }
        // рисунок из одного объекта ложится в группу значка как есть; из
        // нескольких — в обёртку (сверено в Wine: «Солнечная система»,
        // tools/verify/zorder.txt)
        let single = pic.objects.iter().filter(|o| !pic.objects.iter().any(|g| matches!(&g.kind, crate::formats::vdr::ObjectKind::Group { children } if children.contains(&o.handle)))).count() == 1;
        // обёртка — группа внутри рисунка с наименьшим свободным в нём номером;
        // затем объекты рисунка вместе с ней получают номера окна по порядку
        // своих номеров: у NumberView (объекты 2, 3) обёртка раньше объектов,
        // у VSlider1 (объекты 1–13) — после (сверено в Wine: «Солнечная
        // система», L1, tools/verify/embedorder.txt)
        let pic = with_wrapper(&pic);
        let (top, zorder) = insert_objects(space, &pic);
        if top.is_empty() {
            continue;
        }
        let (wrapper, top) = if single {
            (None, top)
        } else {
            let w = top[0];
            let inner = match space.objects.get(&w).map(|o| &o.shape) {
                Some(Shape::Group { children }) => children.clone(),
                _ => Vec::new(),
            };
            (Some(w), inner)
        };
        // габарит ломаных рисунка пересчитывается по точкам: в файле он
        // с запасом в единицу (сверено в Wine: BALLS, круг 31 → 30)
        for h in &zorder {
            if let Some(o) = space.objects.get_mut(h) {
                if let Shape::Polyline { points, .. } = &o.shape {
                    if let Some(&(x0, y0)) = points.first() {
                        let (mut a, mut b, mut c, mut d) = (x0, y0, x0, y0);
                        for &(x, y) in points {
                            a = a.min(x);
                            b = b.min(y);
                            c = c.max(x);
                            d = d.max(y);
                        }
                        (o.x, o.y, o.w, o.h) = (a, b, c - a, d - b);
                    }
                }
            }
        }
        // простые объекты рисунка — поверх всего, что уже есть в окне
        space.zorder.extend(zorder);
        let holder = wrapper.unwrap_or(group);
        for c in &top {
            if let Some(o) = space.objects.get_mut(c) {
                o.parent = Some(holder);
            }
        }
        let placed = match wrapper {
            Some(w) => {
                if let Some(o) = space.objects.get_mut(&w) {
                    o.shape = Shape::Group { children: top.clone() };
                    o.parent = Some(group);
                }
                space.update_group_bounds(w);
                w
            }
            None => top[0],
        };
        // рисунок — на место значка: левый верхний угол к углу значка
        let icon_at = match space.objects.get(&group).map(|o| o.shape.clone()) {
            Some(Shape::Group { children }) => children.iter().filter_map(|c| space.objects.get(c)).map(|o| (o.x, o.y)).next(),
            _ => None,
        };
        if let Some((x, y)) = icon_at {
            space.move_object(placed, x, y);
        }
        // значок под рисунком сжимается до 1×1: габарит группы — это рисунок
        // (сверено в Wine: DIFF и BALLS, tools/verify/diffhit.txt)
        if let Some(Shape::Group { children }) = space.objects.get(&group).map(|o| o.shape.clone()) {
            for c in children {
                if let Some(o) = space.objects.get_mut(&c) {
                    o.w = 1.0;
                    o.h = 1.0;
                }
            }
        }
        if let Some(Object { shape: Shape::Group { children }, scheme_element, .. }) = space.objects.get_mut(&group) {
            children.push(placed);
            // группа видна, собственный значок внутри остаётся скрытым
            *scheme_element = false;
        }
        space.update_group_bounds(group);
    }
}
