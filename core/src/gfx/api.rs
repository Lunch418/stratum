//! 2D-функции языка над графическим пространством.
//!
//! Сигнатуры — из `docs/lang/functions.md`. Возвращаемые значения как в
//! оригинале: дескриптор или 0 при ошибке, 1/0 для успеха.

use super::{Brush, Dib, Font, Gfx, Handle, Object, Pen, Shape, Space, TextPart};
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
        "openschemewindow" => {
            let window = s(args, 0);
            let class = s(args, 1);
            let sp = gfx.open_window(&window);
            if !class.is_empty() {
                if let Some(pic) = gfx.pictures.get(&class.to_lowercase()).cloned() {
                    gfx.space_mut(sp).unwrap().load(&pic);
                    gfx.resolve_dibs(sp);
                    gfx.fit_client(sp);
                }
            }
            handle(sp)
        }
        "loadspacewindow" => {
            let window = s(args, 0);
            let file = s(args, 1);
            let sp = gfx.open_window(&window);
            if !file.is_empty() {
                if let Some(pic) = gfx.load_picture_file(&file) {
                    gfx.space_mut(sp).unwrap().load(&pic);
                    gfx.resolve_dibs(sp);
                    gfx.fit_client(sp);
                }
            }
            handle(sp)
        }
        "createwindow" | "createwindowex" => handle(gfx.open_window(&s(args, 0))),
        "closewindow" => ok(gfx.close_window(&s(args, 0))),
        "getclientwidth" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.0).unwrap_or(0.0)),
        "getclientheight" => num(window_space(gfx, &s(args, 0)).map(|sp| sp.client.1).unwrap_or(0.0)),
        "setclientsize" => {
            let (w, hh) = (f(args, 1), f(args, 2));
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.client = (w, hh)).is_some())
        }
        "showwindow" => {
            let mode = f(args, 1);
            ok(window_space_mut(gfx, &s(args, 0)).map(|sp| sp.visible = mode != 0.0).is_some())
        }
        "setwindoworg" | "setwindowpos" | "setwindowtitle" | "setwindowprop" | "bringwindowtotop"
        | "setwindowtransparent" | "setwindowtransparentcolor" => ok(window_space(gfx, &s(args, 0)).is_some()),
        "getwindoworgx" | "getwindoworgy" | "getwindowwidth" | "getwindowheight" => num(0.0),
        // GetWindowProp(name, prop): "hwnd" и прочие свойства ОС нам недоступны
        "getwindowprop" => num(0.0),
        "getwindowname" => Value::Str(
            gfx.space(h(args, 0)).map(|sp| sp.window.clone()).unwrap_or_default(),
        ),

        // ── пространство ──────────────────────────────────────────────────
        "getspaceorgx" | "getspaceorg2dx" => num(gfx.space(h(args, 0)).map(|sp| sp.origin.0).unwrap_or(0.0)),
        "getspaceorgy" | "getspaceorg2dy" => num(gfx.space(h(args, 0)).map(|sp| sp.origin.1).unwrap_or(0.0)),
        "setspaceorg2d" => {
            let (x, y) = (f(args, 1), f(args, 2));
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
        "getobject2dbyname" => {
            let name = s(args, 2);
            handle(gfx.space(h(args, 0)).and_then(|sp| sp.find_by_name(&name)).unwrap_or(0))
        }
        "getobjectname2d" => Value::Str(
            object(gfx, args).map(|o| o.name.clone()).unwrap_or_default(),
        ),
        "setobjectname2d" => {
            let name = s(args, 2);
            ok(object_mut(gfx, args).map(|o| o.name = name).is_some())
        }
        "getobjecttype2d" => num(object(gfx, args).map(|o| match o.shape {
            Shape::Group { .. } => 3.0,
            Shape::Polyline { .. } => 20.0,
            Shape::Bitmap { masked: false, .. } => 21.0,
            Shape::Bitmap { masked: true, .. } => 22.0,
            Shape::Text { .. } => 23.0,
            Shape::Control { .. } => 26.0,
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
        "hideobject2d" => ok(object_mut(gfx, args).map(|o| o.visible = false).is_some()),
        "setshowobject2d" => {
            let show = f(args, 2) != 0.0;
            ok(object_mut(gfx, args).map(|o| o.visible = show).is_some())
        }
        "getshowobject2d" => ok(object(gfx, args).is_some_and(|o| o.visible)),
        "showobject2d" => ok(object_mut(gfx, args).map(|o| o.visible = true).is_some()),
        "deleteobject2d" => ok(gfx.space_mut(h(args, 0)).is_some_and(|sp| sp.delete_object(h(args, 1)))),
        "objecttotop2d" => ok(gfx.space_mut(h(args, 0)).map(|sp| sp.to_top(h(args, 1))).is_some()),
        "objecttobottom2d" => ok(gfx.space_mut(h(args, 0)).map(|sp| sp.to_bottom(h(args, 1))).is_some()),
        "getzorder2d" => num(gfx.space(h(args, 0))
            .and_then(|sp| sp.zorder.iter().position(|z| *z == h(args, 1)))
            .map(|i| i as f64 + 1.0)
            .unwrap_or(0.0)),
        "setzorder2d" => {
            let pos = f(args, 2).max(1.0) as usize - 1;
            ok(gfx.space_mut(h(args, 0)).map(|sp| {
                let obj = h(args, 1);
                sp.zorder.retain(|z| *z != obj);
                let at = pos.min(sp.zorder.len());
                sp.zorder.insert(at, obj);
            }).is_some())
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
        "setobjectattribute2d" | "getobjectattribute2d" | "setobjectlayer2d" | "getobjectlayer2d" => num(0.0),

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
                for c in &children {
                    sp.zorder.retain(|z| z != c);
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
                sp.zorder.retain(|z| *z != item);
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
                sp.zorder.push(item);
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
        "enablecontrol2d" | "checkdlgbutton2d" | "setcontrolstyle2d" | "setcontrolfont2d" => ok(object(gfx, args).is_some()),
        "isdlgbuttonchecked2d" | "getcontrolstyle2d" => num(0.0),
        // CreateControlObject2d(HSpace, ClassName, Text, Style, x, y, w, h)
        "createcontrolobject2d" => {
            let (class, text, style) = (s(args, 1), s(args, 2), f(args, 3) as u32);
            let (x, y, w, hh) = (f(args, 4), f(args, 5), f(args, 6), f(args, 7));
            handle(gfx.space_mut(h(args, 0)).map(|sp| {
                sp.add_object(Object::new(0, x, y, w, hh, Shape::Control { class, caption: text.clone(), style, text }))
            }).unwrap_or(0))
        }

        // ── прозрачность и растр по пикселям ─────────────────────────────
        "setobjectalpha2d" => {
            let a = f(args, 2).clamp(0.0, 255.0) as u8;
            ok(object_mut(gfx, args).map(|o| o.alpha = a).is_some())
        }
        "getobjectsize2dx" | "getactualwidth2d" => num(object(gfx, args).map(|o| o.w).unwrap_or(0.0)),
        "getobjectsize2dy" | "getactualheight2d" => num(object(gfx, args).map(|o| o.h).unwrap_or(0.0)),
        "getschemeobject" | "framegetpos2d" => num(0.0),
        "audiosetvolume" | "audiosettone" | "beginwritevideo2d" | "endwritevideo2d" => ok(true),
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
        "getscreenwidth" => num(1440.0),
        "getscreenheight" => num(900.0),
        "getworkareax" | "getworkareay" => num(0.0),
        "getworkareawidth" => num(1440.0),
        "getworkareaheight" => num(860.0),
        "getprojectdirectory" => Value::Str(gfx.project_dir.display().to_string()),

        // ── принимаем без действия: градиенты, движок, строка состояния, звук ──
        "setbrushpoints2d" | "setbrushcolors2d" | "setspacerenderengine2d" | "setstatustext" | "setlogstring2d"
        | "sndplaysound" | "mcisendstring" | "videodialog" | "setspaceorg" => ok(true),
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
    // объекты: новые дескрипторы, ссылки переписываются
    let mut map = std::collections::HashMap::new();
    let mut order: Vec<Handle> = sub.objects.keys().copied().collect();
    order.sort();
    for old in &order {
        let new = sp.alloc();
        map.insert(*old, new);
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
    let root = if top.len() == 1 {
        let h = top[0];
        sp.zorder.push(h);
        h
    } else {
        let g = sp.add_object(Object::new(0, 0.0, 0.0, 0.0, 0.0, Shape::Group { children: zorder.clone() }));
        for c in &zorder {
            if let Some(o) = sp.objects.get_mut(c) {
                o.parent = Some(g);
            }
        }
        sp.update_group_bounds(g);
        g
    };
    if move_to {
        sp.move_object(root, x, y);
    }
    root
}
