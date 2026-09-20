//! Чтение имиджа (`.cls`). Формат описан в `docs/formats/cls.md`.

use super::reader::{Reader, Result};

/// Смещения в оглавлении отсчитываются от байта 6 — сразу после "SB" и версии.
const INDEX_BASE: usize = 6;

pub mod section {
    pub const FLAGS: u16 = 0x04;
    pub const LINKS: u16 = 0x07;
    pub const ICON: u16 = 0x08;
    pub const IMAGE: u16 = 0x09;
    pub const TEXT: u16 = 0x0A;
    pub const UNKNOWN_0B: u16 = 0x0B;
    pub const SCHEME: u16 = 0x0C;
    pub const BYTECODE: u16 = 0x0D;
    pub const DESCRIPTION: u16 = 0x0E;
    pub const VARS: u16 = 0x0F;
    pub const CHILDREN: u16 = 0x11;
    pub const ICON_INDEX: u16 = 0x14;
    pub const TIMESTAMP: u16 = 0x15;
    pub const ICON_FILE: u16 = 0x16;
    pub const EQUATIONS: u16 = 0x1E;
    pub const END: u16 = 0x00;
}

pub mod var_flags {
    /// Локальная: наружу через контактную площадку не выводится.
    pub const LOCAL: u32 = 0x0000_0002;
    /// Вычисляется самим имиджем.
    pub const COMPUTED: u32 = 0x0000_0080;
    pub const IN: u32 = 0x0000_0100;
    pub const OUT: u32 = 0x0000_0200;
    pub const INOUT: u32 = 0x0000_0400;
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub description: String,
    pub default: String,
    pub var_type: String,
    pub flags: u32,
}

impl Variable {
    pub fn is_local(&self) -> bool {
        self.flags & var_flags::LOCAL != 0
    }
}

/// Экземпляр дочернего имиджа на схеме.
#[derive(Debug, Clone)]
pub struct Child {
    pub class_name: String,
    pub handle: u16,
    /// Имя экземпляра; пустое, если совпадает с именем класса.
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub flags: u8,
}

/// Оформление связи (диалог «Параметры связи» оригинала): цвет, толщина,
/// выключение, стрелки, слой. В `.cls` не хранится — только в родном формате.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LinkStyle {
    /// Цвет `#rrggbb`; пусто — цвет по умолчанию.
    pub color: String,
    /// Толщина линии в пикселях; 0 — по умолчанию.
    pub width: u8,
    /// «Связь не работает»: переменные не объединяются.
    pub disabled: bool,
    pub arrows: bool,
    pub layer: u8,
}

impl LinkStyle {
    pub fn is_default(&self) -> bool {
        *self == LinkStyle::default()
    }
}

/// Связь: одна или несколько пар переменных между двумя экземплярами.
#[derive(Debug, Clone, Default)]
pub struct Link {
    pub source: u16,
    pub target: u16,
    pub handle: u16,
    pub flags: u32,
    pub vars: Vec<(String, String)>,
    pub style: LinkStyle,
}

/// Параметры листа (диалог «Параметры листа»: сетка, окно, слои).
#[derive(Debug, Clone, PartialEq)]
pub struct SheetOptions {
    pub grid_origin: (f64, f64),
    pub grid_step: (f64, f64),
    pub grid_visible: bool,
    pub grid_snap: bool,
    /// Стиль окна модели: "mdi" | "dialog" | "popup" | "default".
    pub window_style: String,
    /// Размер окна: "max" | "min" | "default" | "space" | "fixed".
    pub window_size: String,
    pub window_wh: (f64, f64),
    pub window_fixed: bool,
    pub hscroll: bool,
    pub vscroll: bool,
    pub auto_origin: bool,
    /// Битовая маска видимых слоёв 0–31.
    pub layers: u32,
    pub no_subwindows: bool,
}

impl Default for SheetOptions {
    fn default() -> Self {
        SheetOptions {
            grid_origin: (0.0, 0.0),
            grid_step: (10.0, 10.0),
            grid_visible: false,
            grid_snap: false,
            window_style: "default".into(),
            window_size: "space".into(),
            window_wh: (0.0, 0.0),
            window_fixed: false,
            hscroll: false,
            vscroll: false,
            auto_origin: true,
            layers: u32::MAX,
            no_subwindows: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Class {
    pub name: String,
    /// Откуда прочитан файл; пусто для имиджей, созданных в памяти.
    pub source: String,
    pub version: u32,
    pub description: String,
    pub vars: Vec<Variable>,
    pub text: String,
    pub children: Vec<Child>,
    pub links: Vec<Link>,
    pub icon_file: Option<String>,
    pub icon_index: Option<u16>,
    /// Иконка, рисунок имиджа и графика схемы — блоки в формате `.vdr`.
    pub icon: Option<Vec<u8>>,
    pub image: Option<Vec<u8>>,
    pub scheme: Option<Vec<u8>>,
    /// Скомпилированный оригиналом байт-код; мы компилируем текст заново.
    pub bytecode: Option<Vec<u8>>,
    /// Уравнения библиотек электрических цепей; структура пока не разобрана.
    pub equations: Option<Vec<u8>>,
    pub timestamp: Option<u32>,
    pub flags: Option<u32>,
    /// Параметры листа; `None` — по умолчанию (в `.cls` не хранятся).
    pub sheet: Option<SheetOptions>,
}

impl Class {
    /// Экземпляр по handle — связи ссылаются именно на них.
    pub fn child(&self, handle: u16) -> Option<&Child> {
        self.children.iter().find(|c| c.handle == handle)
    }

    /// Имя экземпляра на схеме: собственное, иначе имя класса.
    pub fn instance_name(child: &Child) -> &str {
        if child.name.is_empty() { &child.class_name } else { &child.name }
    }
}

pub fn parse(data: &[u8], path: &str) -> Result<Class> {
    let mut r = Reader::new(data, path);
    if data.len() < 16 || &data[0..2] != b"SB" {
        return r.err("не файл имиджа: нет сигнатуры SB");
    }
    r.pos = 2;
    let version = r.u32()?;
    let body_end = r.u32()? as usize;
    let _index_size = r.u32()?;
    let mut cls = Class { version, name: r.string()?, source: path.to_string(), ..Default::default() };

    let index_at = body_end + INDEX_BASE;
    while r.pos < index_at {
        let id = r.u16()?;
        if id == section::END {
            break;
        }
        read_section(&mut r, id, version, index_at, &mut cls)?;
    }
    Ok(cls)
}

fn read_section(
    r: &mut Reader,
    id: u16,
    version: u32,
    index_at: usize,
    cls: &mut Class,
) -> Result<()> {
    match id {
        section::VARS => {
            let n = r.u16()?;
            cls.vars.reserve(n as usize);
            for _ in 0..n {
                let name = r.string()?;
                let description = r.string()?;
                let default = r.string()?;
                let var_type = r.string()?;
                let flags = r.u32()?;
                cls.vars.push(Variable { name, description, default, var_type, flags });
            }
        }
        section::TEXT => cls.text = r.string()?,
        section::DESCRIPTION => cls.description = r.string()?,
        section::ICON_FILE => cls.icon_file = Some(r.string()?),
        section::ICON_INDEX => cls.icon_index = Some(r.u16()?),
        section::TIMESTAMP => cls.timestamp = Some(r.u32()?),
        section::FLAGS => cls.flags = Some(r.u32()?),
        section::UNKNOWN_0B => {
            r.u16()?;
        }
        section::LINKS => {
            let n = r.u16()?;
            for _ in 0..n {
                let source = r.u16()?;
                let target = r.u16()?;
                let handle = r.u16()?;
                let flags = r.u32()?;
                let pairs = r.u16()?;
                let _reserved = r.u16()?;
                let mut vars = Vec::with_capacity(pairs as usize);
                for _ in 0..pairs {
                    vars.push((r.string()?, r.string()?));
                }
                cls.links.push(Link { source, target, handle, flags, vars, style: LinkStyle::default() });
            }
        }
        section::CHILDREN => {
            let n = r.u16()?;
            for _ in 0..n {
                let class_name = r.string()?;
                let handle = r.u16()?;
                let name = r.string()?;
                // до версии 3.002 координаты хранились 16-битными пикселями
                let (x, y) = if version >= 0x3002 {
                    (r.f64()?, r.f64()?)
                } else {
                    (r.i16()? as f64, r.i16()? as f64)
                };
                let flags = r.u8()?;
                cls.children.push(Child { class_name, handle, name, x, y, flags });
            }
        }
        section::ICON | section::IMAGE | section::SCHEME => {
            // размер включает само поле размера
            let size = r.u32()? as usize;
            let body = r.bytes(size.saturating_sub(4))?;
            match id {
                section::ICON => cls.icon = Some(body),
                section::IMAGE => cls.image = Some(body),
                _ => cls.scheme = Some(body),
            }
        }
        section::BYTECODE => {
            let words = r.u32()? as usize;
            cls.bytecode = Some(r.bytes(words * 2)?);
        }
        section::EQUATIONS => {
            // секция всегда последняя в теле; за ней может идти отметка времени
            // варианты хвоста: `15 00 <u32>` либо `15 00 <u32> 00 00`
            let mut end = index_at;
            if index_at >= 10 && r.peek_u16(index_at - 10) == Some(section::TIMESTAMP) && r.peek_u16(index_at - 2) == Some(section::END) {
                end = index_at - 10;
            } else if index_at >= 8 && r.peek_u16(index_at - 8) == Some(section::TIMESTAMP) {
                end = index_at - 8;
            }
            let n = end.saturating_sub(r.pos);
            cls.equations = Some(r.bytes(n)?);
        }
        other => return r.err(format!("неизвестная секция 0x{:02x}", other)),
    }
    Ok(())
}

/// Собирает файл `.cls` из структуры. Версии ниже 3.002 повышаются до 3.003
/// (координаты детей — f64). Байт-код не пишется: оригинальная среда
/// перекомпилирует текст при открытии.
pub fn write(cls: &Class) -> Vec<u8> {
    use super::writer::Writer;
    let version = if cls.version >= 0x3002 { cls.version } else { 0x3003 };
    let mut w = Writer::new();
    w.bytes(b"SB");
    w.u32(version);
    let body_end_at = w.pos();
    w.u32(0);
    let index_size_at = w.pos();
    w.u32(0);
    w.string(&cls.name);

    // (id, смещение секции от байта 6) — для оглавления
    let mut index: Vec<(u16, u32)> = Vec::new();
    let mut begin = |w: &mut Writer, id: u16, indexed: bool| {
        if indexed {
            index.push((id, (w.pos() - INDEX_BASE) as u32));
        }
        w.u16(id);
    };

    if let Some(flags) = cls.flags {
        begin(&mut w, section::FLAGS, false);
        w.u32(flags);
    }
    if !cls.description.is_empty() {
        begin(&mut w, section::DESCRIPTION, false);
        w.string(&cls.description);
    }
    if !cls.vars.is_empty() {
        begin(&mut w, section::VARS, false);
        w.u16(cls.vars.len() as u16);
        for v in &cls.vars {
            w.string(&v.name);
            w.string(&v.description);
            w.string(&v.default);
            w.string(&v.var_type);
            w.u32(v.flags);
        }
    }
    if !cls.text.is_empty() {
        begin(&mut w, section::TEXT, true);
        w.string(&cls.text);
    }
    if !cls.children.is_empty() {
        begin(&mut w, section::CHILDREN, false);
        w.u16(cls.children.len() as u16);
        for c in &cls.children {
            w.string(&c.class_name);
            w.u16(c.handle);
            w.string(&c.name);
            w.f64(c.x);
            w.f64(c.y);
            w.u8(c.flags);
        }
    }
    if !cls.links.is_empty() {
        begin(&mut w, section::LINKS, false);
        w.u16(cls.links.len() as u16);
        for l in &cls.links {
            w.u16(l.source);
            w.u16(l.target);
            w.u16(l.handle);
            w.u32(l.flags);
            w.u16(l.vars.len() as u16);
            w.u16(0);
            for (a, b) in &l.vars {
                w.string(a);
                w.string(b);
            }
        }
    }
    for (id, blob) in [(section::ICON, &cls.icon), (section::IMAGE, &cls.image), (section::SCHEME, &cls.scheme)] {
        if let Some(body) = blob {
            begin(&mut w, id, true);
            w.u32(body.len() as u32 + 4);
            w.bytes(body);
        }
    }
    if let Some(file) = &cls.icon_file {
        begin(&mut w, section::ICON_FILE, false);
        w.string(file);
    }
    if let Some(i) = cls.icon_index {
        begin(&mut w, section::ICON_INDEX, false);
        w.u16(i);
    }
    if let Some(eq) = &cls.equations {
        begin(&mut w, section::EQUATIONS, false);
        w.bytes(eq);
    }
    if let Some(t) = cls.timestamp {
        begin(&mut w, section::TIMESTAMP, false);
        w.u32(t);
    }
    // блок уравнений читается «до конца тела» и уже содержит свой хвост;
    // маркер конца после него нужен только чтобы отделить отметку времени
    if cls.equations.is_none() || cls.timestamp.is_some() {
        w.u16(section::END);
    }
    let body_end = w.pos() - INDEX_BASE;
    w.patch_u32(body_end_at, body_end as u32);
    w.patch_u32(index_size_at, (index.len() * 6) as u32);
    for (id, off) in index {
        w.u32(off);
        w.u16(id);
    }
    w.data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_keeps_everything() {
        let cls = Class {
            name: "Планета".into(),
            version: 0x3003,
            description: "описание".into(),
            vars: vec![Variable { name: "x".into(), description: String::new(), default: "1.5".into(), var_type: "FLOAT".into(), flags: 0x100 }],
            text: "x := ~x + 1".into(),
            children: vec![Child { class_name: "Луна".into(), handle: 3, name: String::new(), x: -12.0, y: 48.5, flags: 0 }],
            links: vec![Link { source: 0, target: 3, handle: 1, flags: 0, vars: vec![("x".into(), "y".into())], style: LinkStyle::default() }],
            icon: Some(vec![1, 2, 3]),
            timestamp: Some(7),
            flags: Some(0x200),
            ..Default::default()
        };
        let back = parse(&write(&cls), "mem").unwrap();
        assert_eq!(back.name, cls.name);
        assert_eq!(back.text, cls.text);
        assert_eq!(back.vars.len(), 1);
        assert_eq!(back.children[0].y, 48.5);
        assert_eq!(back.links[0].vars, cls.links[0].vars);
        assert_eq!(back.icon, cls.icon);
        assert_eq!(back.timestamp, Some(7));
        assert_eq!(back.flags, Some(0x200));
    }

    fn fixture(rel: &str) -> Option<Vec<u8>> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../fixtures/");
        std::fs::read(format!("{path}{rel}")).ok()
    }

    #[test]
    fn reads_a_library_class() {
        let Some(data) = fixture("library/CONTROLS.LIB/CHECKBOX.cls") else {
            return; // корпус собирается локально через `make corpus`
        };
        let cls = parse(&data, "CHECKBOX.cls").unwrap();
        assert_eq!(cls.name, "CheckBox");
        assert_eq!(cls.version, 0x3003);
        assert!(cls.vars.iter().any(|v| v.name == "WindowName" && v.var_type == "STRING"));
        assert!(cls.text.contains("SetControlText2d"));
    }

    #[test]
    fn reads_scheme_of_a_root_class() {
        let Some(data) = fixture("user/solar_system_2/Root2787.cls") else { return };
        let cls = parse(&data, "Root2787.cls").unwrap();
        assert_eq!(cls.children.len(), 15);
        assert_eq!(cls.links.len(), 12);
        let planet = cls.child(3).unwrap();
        assert_eq!(planet.class_name, "планета");
        assert_eq!((planet.x, planet.y), (-544.0, 48.0));
    }
}
