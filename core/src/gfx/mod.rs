//! Графическое пространство: объекты с дескрипторами, инструменты, окна.
//!
//! Сцена не знает о рендере — она только хранит объекты и отвечает на
//! 2D-функции языка ([`api`]). Рисование — отдельно ([`svg`]).

pub mod api;
pub mod svg;

use crate::formats::vdr::{self, ObjectKind, Picture};
use std::collections::HashMap;

pub type Handle = u32;

#[derive(Debug, Clone)]
pub struct Pen {
    pub color: u32,
    pub style: u16,
    pub width: u16,
    pub rop: u16,
}

#[derive(Debug, Clone)]
pub struct Brush {
    pub color: u32,
    pub style: u16,
    pub hatch: u16,
    pub rop: u16,
    pub dib: Handle,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub height: i32,
    pub weight: i32,
    pub italic: bool,
    pub underline: bool,
    pub face: String,
}

#[derive(Debug, Clone)]
pub struct TextPart {
    pub fg: u32,
    pub bg: u32,
    pub font: Handle,
    pub string: Handle,
}

#[derive(Debug, Clone)]
pub struct Dib {
    pub bmp: Vec<u8>,
    pub mask: Vec<u8>,
    pub file: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Polyline { pen: Handle, brush: Handle, points: Vec<(f64, f64)> },
    Bitmap { dib: Handle, src: (f64, f64, f64, f64), masked: bool },
    Text { text: Handle },
    Control { class: String, caption: String, style: u32, text: String },
    Group { children: Vec<Handle> },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub handle: Handle,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub angle: f64,
    pub visible: bool,
    pub layer: u32,
    pub parent: Option<Handle>,
    pub flags: u16,
    /// Иконка имиджа или линия связи: показывается только в редакторе схемы.
    pub scheme_element: bool,
    pub shape: Shape,
}

impl Object {
    fn new(handle: Handle, x: f64, y: f64, w: f64, h: f64, shape: Shape) -> Self {
        Object {
            handle,
            name: String::new(),
            x,
            y,
            w,
            h,
            angle: 0.0,
            visible: true,
            layer: 0,
            parent: None,
            flags: 0,
            scheme_element: false,
            shape,
        }
    }
}

/// Двумерное графическое пространство — содержимое одного окна.
#[derive(Debug, Clone, Default)]
pub struct Space {
    pub handle: Handle,
    pub window: String,
    pub objects: HashMap<Handle, Object>,
    /// Порядок отрисовки объектов верхнего уровня, снизу вверх.
    pub zorder: Vec<Handle>,
    pub pens: HashMap<Handle, Pen>,
    pub brushes: HashMap<Handle, Brush>,
    pub fonts: HashMap<Handle, Font>,
    pub strings: HashMap<Handle, String>,
    pub texts: HashMap<Handle, Vec<TextPart>>,
    pub dibs: HashMap<Handle, Dib>,
    pub origin: (f64, f64),
    pub scale: (f64, f64),
    pub client: (f64, f64),
    pub visible: bool,
    /// Наибольший выданный дескриптор: объекты и инструменты нумеруются
    /// одним счётчиком, как в оригинале.
    next: Handle,
}

impl Space {
    pub fn new(handle: Handle, window: &str) -> Self {
        Space {
            handle,
            window: window.to_string(),
            scale: (1.0, 1.0),
            client: (640.0, 480.0),
            visible: true,
            next: 1,
            ..Default::default()
        }
    }

    pub(crate) fn alloc(&mut self) -> Handle {
        let h = self.next;
        self.next += 1;
        h
    }

    pub(crate) fn reserve(&mut self, h: Handle) {
        if h >= self.next {
            self.next = h + 1;
        }
    }

    /// Наполняет пространство из рисунка `.vdr`.
    pub fn load(&mut self, pic: &Picture) {
        self.origin = pic.origin;
        // в файле масштаб хранится в процентах, в API — множителем
        if pic.scale.0 > 0.0 {
            self.scale = (pic.scale.0 / 100.0, pic.scale.1 / 100.0);
        }
        if pic.window.0 > 0.0 && pic.window.1 > 0.0 {
            self.client = pic.window;
        }
        for p in &pic.pens {
            self.reserve(p.handle as Handle);
            self.pens.insert(p.handle as Handle, Pen { color: p.color, style: p.style, width: p.width, rop: p.rop });
        }
        for b in &pic.brushes {
            self.reserve(b.handle as Handle);
            self.brushes.insert(b.handle as Handle, Brush { color: b.color, style: b.style, hatch: b.hatch, rop: b.rop, dib: b.dib as Handle });
        }
        for f in &pic.fonts {
            self.reserve(f.handle as Handle);
            self.fonts.insert(f.handle as Handle, Font { height: f.height, weight: f.weight, italic: f.italic, underline: f.underline, face: f.face.clone() });
        }
        for s in &pic.strings {
            self.reserve(s.handle as Handle);
            self.strings.insert(s.handle as Handle, s.text.clone());
        }
        for t in &pic.texts {
            self.reserve(t.handle as Handle);
            let parts = t
                .parts
                .iter()
                .map(|p| TextPart { fg: p.fg, bg: p.bg, font: p.font as Handle, string: p.string as Handle })
                .collect();
            self.texts.insert(t.handle as Handle, parts);
        }
        for d in &pic.dibs {
            self.reserve(d.handle as Handle);
            let (width, height) = bmp_size(&d.bmp);
            self.dibs.insert(d.handle as Handle, Dib { bmp: d.bmp.clone(), mask: d.mask.clone(), file: d.file.clone(), width, height });
        }
        for o in &pic.objects {
            let h = o.handle as Handle;
            self.reserve(h);
            let mut obj = match &o.kind {
                ObjectKind::Polyline { x, y, w, h: hh, pen, brush, points } => Object::new(
                    h, *x, *y, *w, *hh,
                    Shape::Polyline { pen: *pen as Handle, brush: *brush as Handle, points: points.clone() },
                ),
                ObjectKind::Bitmap { x, y, w, h: hh, src, dib, masked } => Object::new(
                    h, *x, *y, *w, *hh,
                    Shape::Bitmap { dib: *dib as Handle, src: *src, masked: *masked },
                ),
                ObjectKind::Text { x, y, w, h: hh, text } => {
                    Object::new(h, *x, *y, *w, *hh, Shape::Text { text: *text as Handle })
                }
                ObjectKind::Control { x, y, w, h: hh, class, caption, style } => Object::new(
                    h, *x, *y, *w, *hh,
                    Shape::Control { class: class.clone(), caption: caption.clone(), style: *style, text: caption.clone() },
                ),
                ObjectKind::Group { children } => Object::new(
                    h, 0.0, 0.0, 0.0, 0.0,
                    Shape::Group { children: children.iter().map(|c| *c as Handle).collect() },
                ),
                ObjectKind::Unknown { .. } => Object::new(h, 0.0, 0.0, 0.0, 0.0, Shape::Unknown),
            };
            obj.name = o.name.clone();
            obj.flags = o.flags;
            // 0x1000 — элемент схемы (иконка имиджа, линия связи 0x1800):
            // виден в редакторе, но не в окне модели
            obj.scheme_element = o.flags & 0x1000 != 0;
            self.objects.insert(h, obj);
        }
        // родители по спискам групп, габариты групп по детям
        let groups: Vec<(Handle, Vec<Handle>)> = self
            .objects
            .values()
            .filter_map(|o| match &o.shape {
                Shape::Group { children } => Some((o.handle, children.clone())),
                _ => None,
            })
            .collect();
        for (g, children) in &groups {
            for c in children {
                if let Some(o) = self.objects.get_mut(c) {
                    o.parent = Some(*g);
                }
            }
        }
        for (g, _) in &groups {
            self.update_group_bounds(*g);
        }
        self.zorder = pic.zorder.iter().map(|z| *z as Handle).collect();
        // объекты, не попавшие в Z-порядок (и не в группах) — в конец
        for o in &pic.objects {
            let h = o.handle as Handle;
            if !self.zorder.contains(&h) && self.objects.get(&h).is_some_and(|o| o.parent.is_none()) {
                self.zorder.push(h);
            }
        }
    }

    pub fn update_group_bounds(&mut self, group: Handle) {
        let Some(Shape::Group { children }) = self.objects.get(&group).map(|o| o.shape.clone()) else { return };
        let mut bounds: Option<(f64, f64, f64, f64)> = None;
        for c in &children {
            if let Some(o) = self.objects.get(c) {
                let (x0, y0, x1, y1) = (o.x, o.y, o.x + o.w, o.y + o.h);
                bounds = Some(match bounds {
                    None => (x0, y0, x1, y1),
                    Some(b) => (b.0.min(x0), b.1.min(y0), b.2.max(x1), b.3.max(y1)),
                });
            }
        }
        if let (Some(b), Some(g)) = (bounds, self.objects.get_mut(&group)) {
            g.x = b.0;
            g.y = b.1;
            g.w = b.2 - b.0;
            g.h = b.3 - b.1;
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<Handle> {
        // сначала точное совпадение, затем без учёта регистра
        self.objects
            .values()
            .find(|o| o.name == name)
            .or_else(|| self.objects.values().find(|o| o.name.eq_ignore_ascii_case(name)))
            .map(|o| o.handle)
    }

    pub fn add_object(&mut self, mut obj: Object) -> Handle {
        let h = self.alloc();
        obj.handle = h;
        self.objects.insert(h, obj);
        self.zorder.push(h);
        h
    }

    pub fn add_pen(&mut self, pen: Pen) -> Handle {
        let h = self.alloc();
        self.pens.insert(h, pen);
        h
    }

    pub fn add_brush(&mut self, brush: Brush) -> Handle {
        let h = self.alloc();
        self.brushes.insert(h, brush);
        h
    }

    pub fn add_font(&mut self, font: Font) -> Handle {
        let h = self.alloc();
        self.fonts.insert(h, font);
        h
    }

    pub fn add_string(&mut self, s: String) -> Handle {
        let h = self.alloc();
        self.strings.insert(h, s);
        h
    }

    pub fn add_text(&mut self, parts: Vec<TextPart>) -> Handle {
        let h = self.alloc();
        self.texts.insert(h, parts);
        h
    }

    pub fn add_dib(&mut self, dib: Dib) -> Handle {
        let h = self.alloc();
        self.dibs.insert(h, dib);
        h
    }

    /// Удаляет объект вместе с детьми, если это группа.
    pub fn delete_object(&mut self, h: Handle) -> bool {
        let Some(obj) = self.objects.remove(&h) else { return false };
        self.zorder.retain(|z| *z != h);
        if let Shape::Group { children } = obj.shape {
            for c in children {
                self.delete_object(c);
            }
        }
        if let Some(parent) = obj.parent {
            if let Some(Shape::Group { children }) = self.objects.get_mut(&parent).map(|o| &mut o.shape) {
                children.retain(|c| *c != h);
            }
        }
        true
    }

    /// Сдвигает объект (и детей группы) так, чтобы его начало стало (x, y).
    pub fn move_object(&mut self, h: Handle, x: f64, y: f64) -> bool {
        let Some(obj) = self.objects.get(&h) else { return false };
        let (dx, dy) = (x - obj.x, y - obj.y);
        self.shift(h, dx, dy);
        if let Some(parent) = self.objects.get(&h).and_then(|o| o.parent) {
            self.update_group_bounds(parent);
        }
        true
    }

    fn shift(&mut self, h: Handle, dx: f64, dy: f64) {
        let Some(obj) = self.objects.get_mut(&h) else { return };
        obj.x += dx;
        obj.y += dy;
        let children = match &mut obj.shape {
            Shape::Polyline { points, .. } => {
                for p in points.iter_mut() {
                    p.0 += dx;
                    p.1 += dy;
                }
                Vec::new()
            }
            Shape::Group { children } => children.clone(),
            _ => Vec::new(),
        };
        for c in children {
            self.shift(c, dx, dy);
        }
    }

    /// Масштабирует объект к новому размеру относительно его начала.
    pub fn resize_object(&mut self, h: Handle, w: f64, h_new: f64) -> bool {
        let Some(obj) = self.objects.get(&h) else { return false };
        let (sx, sy) = (
            if obj.w != 0.0 { w / obj.w } else { 1.0 },
            if obj.h != 0.0 { h_new / obj.h } else { 1.0 },
        );
        let (ox, oy) = (obj.x, obj.y);
        self.scale_about(h, ox, oy, sx, sy);
        if let Some(o) = self.objects.get_mut(&h) {
            o.w = w;
            o.h = h_new;
        }
        true
    }

    fn scale_about(&mut self, h: Handle, ox: f64, oy: f64, sx: f64, sy: f64) {
        let Some(obj) = self.objects.get_mut(&h) else { return };
        obj.x = ox + (obj.x - ox) * sx;
        obj.y = oy + (obj.y - oy) * sy;
        obj.w *= sx;
        obj.h *= sy;
        let children = match &mut obj.shape {
            Shape::Polyline { points, .. } => {
                for p in points.iter_mut() {
                    p.0 = ox + (p.0 - ox) * sx;
                    p.1 = oy + (p.1 - oy) * sy;
                }
                Vec::new()
            }
            Shape::Group { children } => children.clone(),
            _ => Vec::new(),
        };
        for c in children {
            self.scale_about(c, ox, oy, sx, sy);
        }
    }

    /// Пересчитывает габариты полилинии по точкам.
    pub fn refit_polyline(&mut self, h: Handle) {
        if let Some(obj) = self.objects.get_mut(&h) {
            if let Shape::Polyline { points, .. } = &obj.shape {
                if points.is_empty() {
                    return;
                }
                let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
                for p in points {
                    x0 = x0.min(p.0);
                    y0 = y0.min(p.1);
                    x1 = x1.max(p.0);
                    y1 = y1.max(p.1);
                }
                obj.x = x0;
                obj.y = y0;
                obj.w = x1 - x0;
                obj.h = y1 - y0;
            }
        }
    }

    /// Верхний видимый объект под точкой (в координатах пространства).
    pub fn object_at(&self, x: f64, y: f64) -> Option<Handle> {
        for &h in self.zorder.iter().rev() {
            if let Some(found) = self.hit(h, x, y) {
                return Some(found);
            }
        }
        None
    }

    fn hit(&self, h: Handle, x: f64, y: f64) -> Option<Handle> {
        let obj = self.objects.get(&h)?;
        if !obj.visible || obj.scheme_element || obj.flags & 0x8000 != 0 {
            return None;
        }
        match &obj.shape {
            Shape::Group { children } => {
                for c in children.iter().rev() {
                    if self.hit(*c, x, y).is_some() {
                        return Some(h);
                    }
                }
                None
            }
            _ => {
                let pad = 2.0;
                (x >= obj.x - pad && x <= obj.x + obj.w + pad && y >= obj.y - pad && y <= obj.y + obj.h + pad)
                    .then_some(h)
            }
        }
    }

    pub fn to_top(&mut self, h: Handle) {
        self.zorder.retain(|z| *z != h);
        self.zorder.push(h);
    }

    pub fn to_bottom(&mut self, h: Handle) {
        self.zorder.retain(|z| *z != h);
        self.zorder.insert(0, h);
    }
}

/// Размеры растра из заголовка BMP.
pub fn bmp_size(bmp: &[u8]) -> (u32, u32) {
    if bmp.len() >= 26 && &bmp[0..2] == b"BM" {
        let w = i32::from_le_bytes([bmp[18], bmp[19], bmp[20], bmp[21]]);
        let h = i32::from_le_bytes([bmp[22], bmp[23], bmp[24], bmp[25]]);
        (w.unsigned_abs(), h.unsigned_abs())
    } else {
        (0, 0)
    }
}

/// Все окна и пространства модели.
#[derive(Debug, Default)]
pub struct Gfx {
    pub spaces: HashMap<Handle, Space>,
    /// Имя окна (в нижнем регистре) → пространство.
    pub windows: HashMap<String, Handle>,
    /// Рисунки имиджей по имени класса (в нижнем регистре) — для
    /// `OpenSchemeWindow`.
    pub pictures: HashMap<String, Picture>,
    /// Папка проекта — здесь ищутся `.vdr`, `.bmp` и наборы иконок.
    pub project_dir: std::path::PathBuf,
    pub library_dirs: Vec<std::path::PathBuf>,
    next_space: Handle,
    /// Порядок открытия окон — для стабильного вывода.
    pub window_order: Vec<String>,
}

impl Gfx {
    pub fn new() -> Self {
        Gfx { next_space: 1, ..Default::default() }
    }

    pub fn space(&self, h: Handle) -> Option<&Space> {
        self.spaces.get(&h)
    }

    pub fn space_mut(&mut self, h: Handle) -> Option<&mut Space> {
        self.spaces.get_mut(&h)
    }

    pub fn window_space(&self, name: &str) -> Option<Handle> {
        self.windows.get(&name.to_lowercase()).copied()
    }

    /// Создаёт окно с пустым пространством или возвращает существующее.
    pub fn open_window(&mut self, name: &str) -> Handle {
        if let Some(h) = self.window_space(name) {
            return h;
        }
        if self.next_space == 0 {
            self.next_space = 1;
        }
        let h = self.next_space;
        self.next_space += 1;
        self.spaces.insert(h, Space::new(h, name));
        self.windows.insert(name.to_lowercase(), h);
        self.window_order.push(name.to_string());
        h
    }

    pub fn close_window(&mut self, name: &str) -> bool {
        let Some(h) = self.windows.remove(&name.to_lowercase()) else { return false };
        self.spaces.remove(&h);
        self.window_order.retain(|w| !w.eq_ignore_ascii_case(name));
        true
    }

    /// Ищет файл рядом с проектом, затем в библиотеках; регистр не важен.
    pub fn find_file(&self, name: &str) -> Option<std::path::PathBuf> {
        let name = name.replace('\\', "/");
        let name = name.rsplit('/').next().unwrap_or(&name);
        let mut dirs = vec![self.project_dir.clone()];
        dirs.extend(self.library_dirs.iter().cloned());
        // наборы иконок лежат рядом с библиотекой: fixtures/ICONS, data/ICONS
        for lib in &self.library_dirs {
            if let Some(parent) = lib.parent() {
                dirs.push(parent.join("ICONS"));
                dirs.push(parent.join("data").join("ICONS"));
            }
        }
        for dir in dirs {
            if let Some(p) = find_case_insensitive(&dir, name, 3) {
                return Some(p);
            }
        }
        None
    }

    /// Подгружает растры, заданные ссылкой на файл (`default.dbm` и т. п.).
    pub fn resolve_dibs(&mut self, space: Handle) {
        let Some(sp) = self.spaces.get(&space) else { return };
        let pending: Vec<(Handle, String)> = sp
            .dibs
            .iter()
            .filter(|(_, d)| d.bmp.is_empty())
            .filter_map(|(h, d)| d.file.clone().map(|f| (*h, f)))
            .collect();
        for (h, file) in pending {
            let Some(path) = self.find_file(&file) else { continue };
            let Ok(data) = std::fs::read(&path) else { continue };
            if !data.starts_with(b"BM") {
                continue;
            }
            let (width, height) = bmp_size(&data);
            if let Some(d) = self.spaces.get_mut(&space).and_then(|sp| sp.dibs.get_mut(&h)) {
                d.bmp = data;
                d.width = width;
                d.height = height;
            }
        }
    }

    /// Окно без явного размера клиентской области подгоняется под объекты.
    pub fn fit_client(&mut self, space: Handle) {
        let Some(sp) = self.spaces.get_mut(&space) else { return };
        if sp.client != (100.0, 100.0) && sp.client != (0.0, 0.0) {
            return;
        }
        let (mut x1, mut y1) = (0.0f64, 0.0f64);
        for o in sp.objects.values().filter(|o| !o.scheme_element) {
            x1 = x1.max(o.x + o.w);
            y1 = y1.max(o.y + o.h);
        }
        sp.client = ((x1 + 8.0).max(200.0), (y1 + 8.0).max(150.0));
    }

    pub fn load_picture_file(&self, name: &str) -> Option<Picture> {
        let path = self.find_file(name)?;
        let data = std::fs::read(&path).ok()?;
        vdr::parse(&data, &path.display().to_string()).ok()
    }
}

fn find_case_insensitive(dir: &std::path::Path, name: &str, depth: u32) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.file_name().is_some_and(|f| f.to_string_lossy().eq_ignore_ascii_case(name)) {
            return Some(path);
        }
    }
    if depth > 0 {
        for sub in subdirs {
            if let Some(p) = find_case_insensitive(&sub, name, depth - 1) {
                return Some(p);
            }
        }
    }
    None
}
