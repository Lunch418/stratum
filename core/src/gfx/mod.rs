//! Графическое пространство: объекты с дескрипторами, инструменты, окна.
//!
//! Сцена не знает о рендере — она только хранит объекты и отвечает на
//! 2D-функции языка ([`api`]). Рисование — отдельно ([`svg`]).

pub mod api;
pub mod api3d;
pub mod space3d;
pub mod svg;

use crate::formats::vdr::{self, ObjectKind, Picture};
use std::collections::BTreeMap;

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
    /// Пиксели RGB после первого обращения по пикселям; `bmp` перекодируется
    /// из них раз в такт (`Gfx::flush_dibs`), а не на каждый `SetDibPixel2d`.
    pub pixels: Option<Vec<u8>>,
    pub dirty: bool,
}

impl Dib {
    pub fn new(bmp: Vec<u8>, mask: Vec<u8>, file: Option<String>) -> Self {
        let (width, height) = bmp_size(&bmp);
        Dib { bmp, mask, file, width, height, pixels: None, dirty: false }
    }

    fn ensure_pixels(&mut self) -> Option<()> {
        if self.pixels.is_none() {
            let (w, h, rgb) = decode_bmp(&self.bmp)?;
            self.width = w;
            self.height = h;
            self.pixels = Some(rgb);
        }
        Some(())
    }

    pub fn pixel(&mut self, x: i64, y: i64) -> Option<u32> {
        self.ensure_pixels()?;
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return None;
        }
        let o = ((y as u32 * self.width + x as u32) * 3) as usize;
        let p = self.pixels.as_ref()?;
        Some(p[o] as u32 | (p[o + 1] as u32) << 8 | (p[o + 2] as u32) << 16)
    }

    pub fn set_pixel(&mut self, x: i64, y: i64, color: u32) -> Option<()> {
        self.ensure_pixels()?;
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return None;
        }
        let o = ((y as u32 * self.width + x as u32) * 3) as usize;
        let p = self.pixels.as_mut()?;
        p[o] = (color & 0xff) as u8;
        p[o + 1] = ((color >> 8) & 0xff) as u8;
        p[o + 2] = ((color >> 16) & 0xff) as u8;
        self.dirty = true;
        Some(())
    }

    /// Перекодирует изменённые пиксели в `bmp`.
    pub fn flush(&mut self) {
        if self.dirty {
            if let Some(p) = &self.pixels {
                self.bmp = encode_bmp24(self.width, self.height, p);
            }
            self.dirty = false;
        }
    }
}

#[derive(Debug, Clone)]
pub enum Shape {
    Polyline { pen: Handle, brush: Handle, points: Vec<(f64, f64)> },
    Bitmap { dib: Handle, src: (f64, f64, f64, f64), masked: bool },
    Text { text: Handle },
    Control { class: String, caption: String, style: u32, text: String, checked: bool, enabled: bool },
    /// Проекция трёхмерного пространства через камеру.
    View3d { space: Handle, camera: Handle },
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
    /// Гиперссылка (`SetHyperJump2d`): режим, цель (файл .vdr или имидж), окно.
    pub hyper: Option<(i32, String, String)>,
    /// Прозрачность 0–255 (`SetObjectAlpha2d`); 255 — непрозрачный.
    pub alpha: u8,
    pub shape: Shape,
}

impl Object {
    pub fn is_group(&self) -> bool {
        matches!(self.shape, Shape::Group { .. })
    }

    pub fn new(handle: Handle, x: f64, y: f64, w: f64, h: f64, shape: Shape) -> Self {
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
            hyper: None,
            alpha: 255,
            shape,
        }
    }
}

/// Двумерное графическое пространство — содержимое одного окна.
#[derive(Debug, Clone, Default)]
pub struct Space {
    pub handle: Handle,
    pub window: String,
    pub objects: BTreeMap<Handle, Object>,
    /// Z-порядок, снизу вверх: только простые объекты, в том числе вложенные
    /// в группы, — как в файле рисунка и в оригинале. Группа своего места в
    /// нём не имеет (`GetZOrder2d` группы — 0), она лишь объединяет объекты.
    pub zorder: Vec<Handle>,
    pub pens: BTreeMap<Handle, Pen>,
    pub brushes: BTreeMap<Handle, Brush>,
    pub fonts: BTreeMap<Handle, Font>,
    pub strings: BTreeMap<Handle, String>,
    pub texts: BTreeMap<Handle, Vec<TextPart>>,
    pub dibs: BTreeMap<Handle, Dib>,
    pub origin: (f64, f64),
    pub scale: (f64, f64),
    pub client: (f64, f64),
    /// Левый верхний угол окна в рабочей области главного окна.
    pub org: (f64, f64),
    pub visible: bool,
    /// Из чего открыто окно: имидж (`OpenSchemeWindow`) или файл `.vdr`
    /// (`LoadSpaceWindow`) — для `GetWindowProp`.
    pub source_class: String,
    pub source_file: String,
    /// Маска видимых слоёв 0–31 (`SetSpaceLayers2d`, «Параметры листа → Слои»).
    pub layers: u32,
    /// Размер окна из «Параметров листа»: "max" — на всю вкладку.
    pub window_size: String,
    /// Стиль окна: "dialog" — с рамкой диалога, "popup" — без заголовка.
    pub window_style: String,
    /// Наибольший выданный дескриптор: объекты и инструменты нумеруются
    /// одним счётчиком, как в оригинале.
    next: Handle,
}

/// Окна модели в оригинале — дочерние MDI-окна. Без заданного размера
/// окно получает размер MDI по умолчанию и встаёт каскадом; числа сняты с
/// оригинала в эталонном окружении (Wine, экран 1920×1080), см.
/// tools/verify/windows.txt.
pub const DEFAULT_CLIENT: (f64, f64) = (625.0, 611.0);
/// Рамка и заголовок: окно больше клиентской области на столько.
pub const WINDOW_FRAME: (f64, f64) = (8.0, 34.0);
/// Шаг каскада (заголовок + рамка) и число ступеней до возврата в угол.
pub const CASCADE_STEP: f64 = 29.0;
pub const CASCADE_STEPS: usize = 11;
/// Где рабочая область главного окна лежит на экране: GetWindowOrgX/Y
/// отвечают в экранных координатах, SetWindowOrg принимает координаты
/// рабочей области.
pub const WORKSPACE_ON_SCREEN: (f64, f64) = (85.0, 121.0);

impl Space {
    pub fn new(handle: Handle, window: &str) -> Self {
        Space {
            handle,
            window: window.to_string(),
            scale: (1.0, 1.0),
            client: DEFAULT_CLIENT,
            visible: true,
            layers: u32::MAX,
            next: 1,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
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
    /// Обратное к `load`: содержимое пространства как рисунок `.vdr` —
    /// для сохранения отредактированной графики в имидж.
    pub fn to_picture(&self) -> Picture {
        use crate::formats::vdr;
        let mut pic = Picture {
            version: 0x0300,
            origin: self.origin,
            scale: (self.scale.0 * 100.0, self.scale.1 * 100.0),
            window: self.client,
            ..Default::default()
        };
        pic.zorder = self.zorder.iter().map(|h| *h as u16).collect();
        for (h, p) in &self.pens {
            pic.pens.push(vdr::Pen { handle: *h as u16, color: p.color, style: p.style, width: p.width, rop: p.rop });
        }
        for (h, b) in &self.brushes {
            pic.brushes.push(vdr::Brush { handle: *h as u16, color: b.color, style: b.style, hatch: b.hatch, rop: b.rop, dib: b.dib as u16 });
        }
        for (h, f) in &self.fonts {
            pic.fonts.push(vdr::Font { handle: *h as u16, height: f.height, width: 0, weight: f.weight, italic: f.italic, underline: f.underline, face: f.face.clone() });
        }
        for (h, s) in &self.strings {
            pic.strings.push(vdr::StringTool { handle: *h as u16, text: s.clone() });
        }
        for (h, parts) in &self.texts {
            pic.texts.push(vdr::TextTool {
                handle: *h as u16,
                parts: parts.iter().map(|p| vdr::TextPart { fg: p.fg, bg: p.bg, font: p.font as u16, string: p.string as u16 }).collect(),
            });
        }
        // растры двойных битовых карт получают свою нумерацию (чанк 1007)
        let masked_dibs: std::collections::BTreeSet<Handle> = self
            .objects
            .values()
            .filter_map(|o| match &o.shape { Shape::Bitmap { dib, masked: true, .. } => Some(*dib), _ => None })
            .collect();
        let mut double_map: BTreeMap<Handle, u16> = BTreeMap::new();
        for (h, d) in &self.dibs {
            let double = masked_dibs.contains(h);
            let handle = if double {
                let n = double_map.len() as u16 + 1;
                double_map.insert(*h, n);
                n
            } else {
                *h as u16
            };
            pic.dibs.push(vdr::Dib { handle, bmp: d.bmp.clone(), mask: d.mask.clone(), file: d.file.clone(), double });
        }
        for (h, o) in &self.objects {
            let kind = match &o.shape {
                Shape::Polyline { pen, brush, points } => vdr::ObjectKind::Polyline { x: o.x, y: o.y, w: o.w, h: o.h, pen: *pen as u16, brush: *brush as u16, points: points.clone() },
                Shape::Bitmap { dib, src, masked } => vdr::ObjectKind::Bitmap {
                    x: o.x, y: o.y, w: o.w, h: o.h, src: *src,
                    dib: if *masked { double_map.get(dib).copied().unwrap_or(*dib as u16) } else { *dib as u16 },
                    masked: *masked,
                },
                Shape::Text { text } => vdr::ObjectKind::Text { x: o.x, y: o.y, w: o.w, h: o.h, text: *text as u16 },
                Shape::Control { class, caption, style, .. } => vdr::ObjectKind::Control { x: o.x, y: o.y, w: o.w, h: o.h, class: class.clone(), caption: caption.clone(), style: *style },
                Shape::Group { children } => vdr::ObjectKind::Group { children: children.iter().map(|c| *c as u16).collect() },
                Shape::View3d { .. } | Shape::Unknown => continue,
            };
            pic.objects.push(vdr::Object { handle: *h as u16, name: o.name.clone(), flags: o.flags, kind });
        }
        pic
    }

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
        // обычные растры сохраняют номера, двойные (своя нумерация в файле)
        // получают новые, чтобы не перекрывать обычные
        for d in pic.dibs.iter().filter(|d| !d.double) {
            self.reserve(d.handle as Handle);
        }
        for o in &pic.objects {
            self.reserve(o.handle as Handle);
        }
        let mut double_map: BTreeMap<Handle, Handle> = BTreeMap::new();
        for d in &pic.dibs {
            let dib = Dib::new(d.bmp.clone(), d.mask.clone(), d.file.clone());
            if d.double {
                let h = lowest_free(&self.dibs).max(pic.dibs.iter().filter(|x| !x.double).map(|x| x.handle as Handle + 1).max().unwrap_or(1));
                double_map.insert(d.handle as Handle, h);
                self.dibs.insert(h, dib);
            } else {
                self.dibs.insert(d.handle as Handle, dib);
            }
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
                    Shape::Bitmap {
                        dib: if *masked { double_map.get(&(*dib as Handle)).copied().unwrap_or(*dib as Handle) } else { *dib as Handle },
                        src: *src,
                        masked: *masked,
                    },
                ),
                ObjectKind::Text { x, y, w, h: hh, text } => {
                    Object::new(h, *x, *y, *w, *hh, Shape::Text { text: *text as Handle })
                }
                ObjectKind::Control { x, y, w, h: hh, class, caption, style } => Object::new(
                    h, *x, *y, *w, *hh,
                    Shape::Control { class: class.clone(), caption: caption.clone(), style: *style, text: caption.clone(), checked: false, enabled: true },
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
        self.zorder = pic.zorder.iter().map(|z| *z as Handle).filter(|h| self.objects.get(h).is_some_and(|o| !o.is_group())).collect();
        // простые объекты, не попавшие в Z-порядок файла, — в конец
        for o in &pic.objects {
            let h = o.handle as Handle;
            if !self.zorder.contains(&h) && self.objects.get(&h).is_some_and(|o| !o.is_group()) {
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

    /// Объект с именем внутри группы (рекурсивно, в порядке детей).
    pub fn find_in_group(&self, group: Handle, name: &str) -> Option<Handle> {
        let Some(Object { shape: Shape::Group { children }, .. }) = self.objects.get(&group) else { return None };
        for c in children {
            if let Some(o) = self.objects.get(c) {
                if crate::lang::same_name(&o.name, name) {
                    return Some(*c);
                }
            }
            if let Some(found) = self.find_in_group(*c, name) {
                return Some(found);
            }
        }
        None
    }

    pub fn add_object(&mut self, mut obj: Object) -> Handle {
        let h = lowest_free(&self.objects);
        obj.handle = h;
        let group = obj.is_group();
        self.objects.insert(h, obj);
        if !group {
            self.zorder.push(h);
        }
        h
    }

    /// Простые объекты группы (или сам объект) в Z-порядке.
    pub fn primitives(&self, h: Handle) -> Vec<Handle> {
        match self.objects.get(&h).map(|o| &o.shape) {
            Some(Shape::Group { .. }) => self.zorder.iter().copied().filter(|&z| self.descends(z, h)).collect(),
            Some(_) => vec![h],
            None => Vec::new(),
        }
    }

    /// Лежит ли объект `h` внутри группы `ancestor` (на любой глубине).
    pub fn descends(&self, h: Handle, ancestor: Handle) -> bool {
        let mut cur = self.objects.get(&h).and_then(|o| o.parent);
        let mut guard = 0;
        while let Some(p) = cur {
            if p == ancestor {
                return true;
            }
            guard += 1;
            if guard > 64 {
                break;
            }
            cur = self.objects.get(&p).and_then(|o| o.parent);
        }
        false
    }

    /// Самая внешняя группа, в которую входит объект (или он сам).
    pub fn top_ancestor(&self, h: Handle) -> Handle {
        let mut top = h;
        let mut guard = 0;
        while let Some(p) = self.objects.get(&top).and_then(|o| o.parent) {
            top = p;
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        top
    }

    /// Виден ли объект с учётом групп, в которые он вложен.
    pub fn shown(&self, h: Handle) -> bool {
        let mut cur = Some(h);
        let mut guard = 0;
        while let Some(c) = cur {
            let Some(o) = self.objects.get(&c) else { return false };
            if !o.visible || o.scheme_element || (self.layers >> (o.layer & 31)) & 1 == 0 {
                return false;
            }
            cur = o.parent;
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        true
    }

    /// Объекты верхнего уровня (группы и одиночные объекты) снизу вверх —
    /// по первому их простому объекту в Z-порядке.
    pub fn top_order(&self) -> Vec<Handle> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for &z in &self.zorder {
            let t = self.top_ancestor(z);
            if seen.insert(t) {
                out.push(t);
            }
        }
        out
    }

    /// Ставит объект верхнего уровня на место `index` среди объектов
    /// верхнего уровня (0 — в самый низ).
    pub fn move_among_tops(&mut self, h: Handle, index: usize) {
        let others: Vec<Handle> = self.top_order().into_iter().filter(|&t| t != h).collect();
        let block = self.primitives(h);
        self.zorder.retain(|z| !block.contains(z));
        // встаём перед первым объектом того, кто займёт место следом
        let at = others
            .get(index)
            .and_then(|&next| self.zorder.iter().position(|&z| self.top_ancestor(z) == next))
            .unwrap_or(self.zorder.len());
        for (i, b) in block.into_iter().enumerate() {
            self.zorder.insert(at + i, b);
        }
    }

    /// Ставит объекты группы (или объект) блоком на место `at` в Z-порядке,
    /// сохраняя их взаимный порядок.
    pub fn z_move(&mut self, h: Handle, at: usize) {
        let block = self.primitives(h);
        if block.is_empty() {
            return;
        }
        self.zorder.retain(|z| !block.contains(z));
        let at = at.min(self.zorder.len());
        for (i, b) in block.into_iter().enumerate() {
            self.zorder.insert(at + i, b);
        }
    }

    pub fn add_pen(&mut self, pen: Pen) -> Handle {
        let h = lowest_free(&self.pens);
        self.pens.insert(h, pen);
        h
    }

    pub fn add_brush(&mut self, brush: Brush) -> Handle {
        let h = lowest_free(&self.brushes);
        self.brushes.insert(h, brush);
        h
    }

    pub fn add_font(&mut self, font: Font) -> Handle {
        let h = lowest_free(&self.fonts);
        self.fonts.insert(h, font);
        h
    }

    pub fn add_string(&mut self, s: String) -> Handle {
        let h = lowest_free(&self.strings);
        self.strings.insert(h, s);
        h
    }

    pub fn add_text(&mut self, parts: Vec<TextPart>) -> Handle {
        let h = lowest_free(&self.texts);
        self.texts.insert(h, parts);
        h
    }

    pub fn add_dib(&mut self, dib: Dib) -> Handle {
        let h = lowest_free(&self.dibs);
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

    /// Верхний видимый объект под точкой (в координатах пространства);
    /// для объекта внутри группы — самая внешняя группа.
    pub fn object_at(&self, x: f64, y: f64) -> Option<Handle> {
        for &h in self.zorder.iter().rev() {
            if self.objects.get(&h).is_some_and(|o| o.flags & 0x8000 != 0) || !self.shown(h) {
                continue;
            }
            if self.hit(h, x, y).is_some() {
                return Some(self.top_ancestor(h));
            }
        }
        None
    }

    fn hit(&self, h: Handle, x: f64, y: f64) -> Option<Handle> {
        let obj = self.objects.get(&h)?;
        if !obj.visible || obj.scheme_element || obj.flags & 0x8000 != 0 || (self.layers >> (obj.layer & 31)) & 1 == 0 {
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
        self.z_move(h, usize::MAX);
    }

    pub fn to_bottom(&mut self, h: Handle) {
        self.z_move(h, 0);
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

/// Растровое изображение BMP → пиксели RGB построчно сверху вниз.
/// Понимает 1/4/8 бит с палитрой, 24 и 32 бита без сжатия.
pub fn decode_bmp(bmp: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if bmp.len() < 54 || &bmp[0..2] != b"BM" {
        return None;
    }
    let u32_at = |i: usize| u32::from_le_bytes([bmp[i], bmp[i + 1], bmp[i + 2], bmp[i + 3]]);
    let offset = u32_at(10) as usize;
    let header = u32_at(14) as usize;
    let w = u32_at(18) as i32;
    let h_signed = u32_at(22) as i32;
    let bpp = u16::from_le_bytes([bmp[28], bmp[29]]) as usize;
    let compression = u32_at(30);
    if compression != 0 && !(compression == 3 && bpp == 32) {
        return None;
    }
    let (w, h) = (w.unsigned_abs(), h_signed.unsigned_abs());
    let top_down = h_signed < 0;
    let colors = if bpp <= 8 {
        let n = u32_at(46) as usize;
        if n == 0 { 1 << bpp } else { n }
    } else {
        0
    };
    let palette = &bmp[14 + header..];
    let palette = &palette[..(colors * 4).min(palette.len())];
    let stride = (w as usize * bpp).div_ceil(32) * 4;
    let mut out = vec![0u8; (w * h) as usize * 3];
    for row in 0..h as usize {
        let src_row = if top_down { row } else { h as usize - 1 - row };
        let line = bmp.get(offset + src_row * stride..offset + (src_row + 1) * stride)?;
        for x in 0..w as usize {
            let (r, g, b) = match bpp {
                24 => (line[x * 3 + 2], line[x * 3 + 1], line[x * 3]),
                32 => (line[x * 4 + 2], line[x * 4 + 1], line[x * 4]),
                8 | 4 | 1 => {
                    let idx = match bpp {
                        8 => line[x] as usize,
                        4 => ((line[x / 2] >> (if x % 2 == 0 { 4 } else { 0 })) & 15) as usize,
                        _ => ((line[x / 8] >> (7 - x % 8)) & 1) as usize,
                    };
                    match palette.get(idx * 4..idx * 4 + 3) {
                        Some(p) => (p[2], p[1], p[0]),
                        None => (0, 0, 0),
                    }
                }
                _ => return None,
            };
            let o = (row * w as usize + x) * 3;
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
        }
    }
    Some((w, h, out))
}

/// Пиксели RGB (сверху вниз) → 24-битный BMP.
pub fn encode_bmp24(w: u32, h: u32, rgb: &[u8]) -> Vec<u8> {
    let stride = (w as usize * 3).div_ceil(4) * 4;
    let size = 54 + stride * h as usize;
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&((stride * h as usize) as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    for row in (0..h as usize).rev() {
        let start = out.len();
        for x in 0..w as usize {
            let o = (row * w as usize + x) * 3;
            out.push(rgb[o + 2]);
            out.push(rgb[o + 1]);
            out.push(rgb[o]);
        }
        out.resize(start + stride, 0);
    }
    out
}

/// Все окна и пространства модели.
#[derive(Debug, Default, Clone)]
pub struct Gfx {
    pub spaces: BTreeMap<Handle, Space>,
    /// Имя окна (в нижнем регистре) → пространство.
    pub windows: BTreeMap<String, Handle>,
    /// Рисунки имиджей по имени класса (в нижнем регистре) — для
    /// `OpenSchemeWindow`.
    pub pictures: BTreeMap<String, Picture>,
    /// Параметры окна из «Параметров листа» имиджа: размер окна
    /// ("max" | "min" | "fixed" | …), ширина и высота при "fixed", запрет
    /// изменения размера, видимые слои.
    pub sheets: BTreeMap<String, crate::formats::SheetOptions>,
    /// Дети схемы каждого имиджа (handle, класс) — чтобы окно схемы
    /// показывало рисунки дочерних имиджей на месте значков.
    pub class_children: BTreeMap<String, Vec<(u16, String)>>,
    /// Вид имиджа на схеме родителя (секция 0x0c): в окне схемы он
    /// разворачивается на месте значка экземпляра.
    pub scheme_pictures: BTreeMap<String, Picture>,
    /// История гиперпереходов: (окно, что было открыто) — для кнопки «назад».
    pub hyper_history: Vec<(String, String)>,
    /// Что открыто в каждом окне гипербазы (окно → цель).
    pub hyper_current: BTreeMap<String, String>,
    /// Трёхмерные пространства; дескрипторы общие с двумерными.
    pub spaces3d: BTreeMap<Handle, space3d::Space3d>,
    /// Буфер обмена `CopyToClipboard2d` (объект вместе с инструментами).
    pub clipboard: Option<(Object, Option<Pen>, Option<Brush>)>,
    /// Последний объект, попавший под мышь (`GetLastPrimary2d`).
    pub last_primary: Handle,
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

    pub fn create_space3d(&mut self, owner: Handle) -> Handle {
        if self.next_space == 0 {
            self.next_space = 1;
        }
        let h = self.next_space;
        self.next_space += 1;
        self.spaces3d.insert(h, space3d::Space3d::new(h, owner));
        h
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
        let mut space = Space::new(h, name);
        // каскад MDI: номер ступени — число уже открытых окон
        let step = (self.windows.len() % CASCADE_STEPS) as f64 * CASCADE_STEP;
        space.org = (step, step);
        self.spaces.insert(h, space);
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

    /// Переносит попиксельные правки растров в BMP; вызывается раз в такт.
    pub fn flush_dibs(&mut self) {
        for sp in self.spaces.values_mut() {
            for d in sp.dibs.values_mut() {
                d.flush();
            }
        }
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

    /// Окно, для которого рисунок размера не задаёт (в файле заглушка
    /// 100×100), получает размер MDI-окна по умолчанию, как в оригинале.
    pub fn fit_client(&mut self, space: Handle) {
        let Some(sp) = self.spaces.get_mut(&space) else { return };
        if sp.client == (100.0, 100.0) || sp.client == (0.0, 0.0) {
            sp.client = DEFAULT_CLIENT;
        }
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

impl Gfx {
    /// Гиперпереход: показать цель (файл `.vdr` или рисунок имиджа) в окне,
    /// запомнив прежнее содержимое для возврата.
    pub fn hyper_jump(&mut self, window: &str, target: &str) -> bool {
        let window = if window.is_empty() { "MainWindow" } else { window };
        let pic = if target.to_lowercase().ends_with(".vdr") {
            self.load_picture_file(target)
        } else {
            self.pictures.get(&target.to_lowercase()).cloned()
        };
        // имидж без рисунка — пустая страница
        let Some(pic) = pic.or_else(|| (!target.to_lowercase().ends_with(".vdr")).then(crate::formats::Picture::default)) else { return false };
        if let Some(prev) = self.hyper_current.get(window).cloned() {
            self.hyper_history.push((window.to_string(), prev));
        }
        let sp = self.open_window(window);
        let name = self.space(sp).map(|s| s.window.clone()).unwrap_or_default();
        self.spaces.insert(sp, Space::new(sp, &name));
        self.space_mut(sp).unwrap().load(&pic);
        self.resolve_dibs(sp);
        self.fit_client(sp);
        self.hyper_current.insert(window.to_string(), target.to_string());
        true
    }

    /// Возврат на предыдущую страницу гипербазы.
    pub fn hyper_back(&mut self) -> bool {
        let Some((window, target)) = self.hyper_history.pop() else { return false };
        let ok = self.hyper_jump(&window, &target);
        // сам переход назад не должен ложиться в историю
        self.hyper_history.pop();
        ok
    }
}

/// Наименьший свободный дескриптор таблицы. У оригинала у объектов, перьев,
/// кистей, шрифтов, строк, текстов и растров своя нумерация, и новый элемент
/// получает наименьший свободный номер (сверено по снимку оригинала:
/// перья рисунка «Солнечной системы» 19, 21, 41, 43…, строки 1, 2, 3).
pub(crate) fn lowest_free<T>(table: &BTreeMap<Handle, T>) -> Handle {
    let mut h: Handle = 1;
    for &k in table.keys() {
        if k > h {
            break;
        }
        if k == h {
            h += 1;
        }
    }
    h
}
