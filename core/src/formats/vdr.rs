//! Векторная графика Stratum 2000 (`.vdr` и секции иконки/рисунка/схемы в
//! `.cls`). Формат описан в `docs/formats/vdr.md`.

use super::reader::{Reader, Result};

pub const SIG: &[u8; 2] = b"2D";

#[derive(Debug, Clone, Default)]
pub struct Picture {
    pub version: u16,
    pub origin: (f64, f64),
    pub scale: (f64, f64),
    pub window: (f64, f64),
    pub objects: Vec<Object>,
    /// Порядок отрисовки верхнего уровня: handle объектов снизу вверх.
    pub zorder: Vec<u16>,
    pub pens: Vec<Pen>,
    pub brushes: Vec<Brush>,
    pub fonts: Vec<Font>,
    pub strings: Vec<StringTool>,
    pub texts: Vec<TextTool>,
    pub dibs: Vec<Dib>,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub handle: u16,
    pub name: String,
    pub flags: u16,
    pub kind: ObjectKind,
}

#[derive(Debug, Clone)]
pub enum ObjectKind {
    Polyline {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        pen: u16,
        brush: u16,
        points: Vec<(f64, f64)>,
    },
    Bitmap {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        /// Прямоугольник в источнике: x, y, w, h.
        src: (f64, f64, f64, f64),
        dib: u16,
        masked: bool,
    },
    Text {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        text: u16,
    },
    Control {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        class: String,
        caption: String,
        style: u32,
    },
    Group {
        children: Vec<u16>,
    },
    /// Тип, который ещё не разобран (3D-проекции и т. п.).
    Unknown {
        kind: u16,
    },
}

#[derive(Debug, Clone)]
pub struct Pen {
    pub handle: u16,
    pub color: u32,
    pub style: u16,
    pub width: u16,
    pub rop: u16,
}

#[derive(Debug, Clone)]
pub struct Brush {
    pub handle: u16,
    pub color: u32,
    pub style: u16,
    pub hatch: u16,
    pub rop: u16,
    pub dib: u16,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub handle: u16,
    pub height: i32,
    pub width: i32,
    pub weight: i32,
    pub italic: bool,
    pub underline: bool,
    pub face: String,
}

#[derive(Debug, Clone)]
pub struct StringTool {
    pub handle: u16,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TextPart {
    pub fg: u32,
    pub bg: u32,
    pub font: u16,
    pub string: u16,
}

#[derive(Debug, Clone)]
pub struct TextTool {
    pub handle: u16,
    pub parts: Vec<TextPart>,
}

#[derive(Debug, Clone)]
pub struct Dib {
    pub handle: u16,
    /// Растр целиком в формате BMP; пустой, если это ссылка на файл.
    pub bmp: Vec<u8>,
    /// Маска (1 бит на пиксель), если есть.
    pub mask: Vec<u8>,
    /// Имя файла базы иконок для ссылок (`SYSTEM.DBM`).
    pub file: Option<String>,
    /// Двойная битовая карта (чанк 1007): у них своя нумерация
    /// дескрипторов, независимая от обычных (1006).
    pub double: bool,
}

/// Идентификаторы чанков. Инструменты (1004–1010) различаются по типу записи,
/// поэтому здесь только те, что нужны разбору структуры.
mod chunk {
    pub const DOUBLE_DIBS: u16 = 1007;
    pub const TEXT_PARTS: u16 = 1011;
    pub const OBJECTS: u16 = 1020;
    pub const ZORDER: u16 = 1021;
}

struct Ctx {
    /// Записи и чанки несут свой размер (формат 3.x).
    sized: bool,
    /// Координаты как f64, иначе i16.
    wide: bool,
}

fn num(r: &mut Reader, c: &Ctx) -> Result<f64> {
    if c.wide { r.f64() } else { Ok(r.i16()? as f64) }
}

/// Разбирает блок; `data` может начинаться с префикса `u32 0, u32 size`.
pub fn parse(data: &[u8], path: &str) -> Result<Picture> {
    let start = data
        .windows(2)
        .take(17)
        .position(|w| w == SIG)
        .ok_or_else(|| Reader::new(data, path).err::<()>("нет сигнатуры 2D").unwrap_err())?;
    let mut r = Reader::new(data, path);
    r.pos = start + 2;
    let version = r.u16()?;
    let _version2 = r.u16()?;
    let ctx = Ctx { sized: version >= 0x0300, wide: version >= 0x0200 };
    let mut pic = Picture { version, ..Default::default() };

    let tools_at = if ctx.sized {
        r.u16()?;
        r.u32()?;
        None
    } else {
        r.u32()?;
        // смещение считается от сигнатуры, а не от начала блока
        Some(start + r.u32()? as usize)
    };
    pic.origin = (num(&mut r, &ctx)?, num(&mut r, &ctx)?);
    pic.scale = (num(&mut r, &ctx)?, num(&mut r, &ctx)?);
    pic.window = (num(&mut r, &ctx)?, num(&mut r, &ctx)?);
    if ctx.sized {
        r.u16()?;
        r.bytes(8)?;
    } else {
        r.string()?;
        r.bytes(8)?;
        r.u32()?;
        r.u16()?;
    }

    loop {
        let Some(id) = r.peek_u16(r.pos) else { break };
        if (1000..=1030).contains(&id) {
            read_chunk(&mut r, &ctx, id, &mut pic)?;
            continue;
        }
        // 2.x: объекты закончились, инструменты лежат по tools_at
        if let Some(at) = tools_at {
            if r.pos <= at + 2 {
                r.pos = at + 4;
                continue;
            }
        }
        break;
    }
    Ok(pic)
}

fn read_chunk(r: &mut Reader, ctx: &Ctx, id: u16, pic: &mut Picture) -> Result<()> {
    let at = r.pos;
    r.u16()?;
    let size = if ctx.sized { Some(r.u32()? as usize) } else { None };
    let count = r.u16()?;
    r.u16()?; // capacity
    r.u16()?; // delta
    match id {
        chunk::ZORDER => {
            for _ in 0..count {
                pic.zorder.push(r.u16()?);
            }
        }
        chunk::TEXT_PARTS => {
            // читается только внутри инструмента «текст»
            return r.err("чанк фрагментов текста вне инструмента");
        }
        _ => {
            if count > 0 {
                r.u8()?;
                r.u16()?;
            }
            for _ in 0..count {
                read_item(r, ctx, id, pic)?;
            }
        }
    }
    if let Some(size) = size {
        r.pos = at + size;
    }
    Ok(())
}

fn read_item(r: &mut Reader, ctx: &Ctx, id: u16, pic: &mut Picture) -> Result<()> {
    let at = r.pos;
    let kind = r.u16()?;
    // размер записи — u16; у растров (103/104) он не помещается, там u32
    let end = if ctx.sized {
        if matches!(kind, 103 | 104) {
            Some(at + r.u32()? as usize)
        } else {
            Some(at + r.u16()? as usize)
        }
    } else {
        None
    };
    if id == chunk::OBJECTS {
        let (handle, flags, mut name);
        if ctx.sized {
            r.u16()?;
            handle = r.u16()?;
            flags = r.u16()?;
            name = String::new();
        } else {
            handle = r.u16()?;
            flags = r.u16()?;
            name = r.string()?;
        }
        let body = match read_object(r, ctx, kind) {
            Ok(k) => k,
            Err(e) => match end {
                Some(_) => ObjectKind::Unknown { kind },
                None => return Err(e),
            },
        };
        if let Some(end) = end {
            if r.pos + 6 <= end && r.peek_u16(r.pos) == Some(0xCD) {
                r.u16()?;
                let n = r.u32()? as usize;
                name = super::cp1251::decode(&r.bytes(n.saturating_sub(6))?);
            }
            r.pos = end;
        }
        pic.objects.push(Object { handle, name, flags, kind: body });
    } else {
        if ctx.sized && !matches!(kind, 103 | 104) {
            r.u16()?;
        }
        let _refs = r.u16()?;
        let handle = r.u16()?;
        let result = read_tool(r, ctx, kind, handle, pic, id == chunk::DOUBLE_DIBS);
        match (result, end) {
            (Ok(()), Some(end)) => r.pos = end,
            (Ok(()), None) => {}
            (Err(_), Some(end)) => r.pos = end,
            (Err(e), None) => return Err(e),
        }
    }
    Ok(())
}

fn read_object(r: &mut Reader, ctx: &Ctx, kind: u16) -> Result<ObjectKind> {
    if matches!(kind, 3 | 4 | 5) {
        let mut children = Vec::new();
        if r.peek_u16(r.pos) == Some(chunk::ZORDER) {
            r.u16()?;
            if ctx.sized {
                r.u32()?;
            }
            let n = r.u16()?;
            r.u16()?;
            r.u16()?;
            for _ in 0..n {
                children.push(r.u16()?);
            }
        } else {
            let n = r.u16()?;
            for _ in 0..n {
                children.push(r.u16()?);
            }
        }
        return Ok(ObjectKind::Group { children });
    }
    let x = num(r, ctx)?;
    let y = num(r, ctx)?;
    let w = num(r, ctx)?;
    let h = num(r, ctx)?;
    Ok(match kind {
        20 => {
            let pen = r.u16()?;
            let brush = r.u16()?;
            let n = r.u16()?;
            let mut points = Vec::with_capacity(n as usize);
            for _ in 0..n {
                points.push((num(r, ctx)?, num(r, ctx)?));
            }
            if ctx.wide {
                let extra = r.u8()? as usize;
                r.bytes(extra)?;
            }
            ObjectKind::Polyline { x, y, w, h, pen, brush, points }
        }
        21 | 22 => {
            let src = (num(r, ctx)?, num(r, ctx)?, num(r, ctx)?, num(r, ctx)?);
            r.u16()?;
            let dib = r.u16()?;
            ObjectKind::Bitmap { x, y, w, h, src, dib, masked: kind == 22 }
        }
        23 => {
            let text = r.u16()?;
            r.bytes(18)?;
            ObjectKind::Text { x, y, w, h, text }
        }
        26 => {
            let class = r.string()?;
            let caption = r.string()?;
            r.u16()?;
            let style = r.u32()?;
            r.bytes(10)?;
            ObjectKind::Control { x, y, w, h, class, caption, style }
        }
        other => return r.err(format!("объект типа {other} не разобран")),
    })
}

fn read_tool(r: &mut Reader, ctx: &Ctx, kind: u16, handle: u16, pic: &mut Picture, double: bool) -> Result<()> {
    match kind {
        101 => pic.pens.push(Pen {
            handle,
            color: r.u32()?,
            style: r.u16()?,
            width: r.u16()?,
            rop: r.u16()?,
        }),
        102 => pic.brushes.push(Brush {
            handle,
            color: r.u32()?,
            style: r.u16()?,
            hatch: r.u16()?,
            rop: r.u16()?,
            dib: r.u16()?,
        }),
        105 => {
            // 16-битный LOGFONT: height, width, escapement, orientation,
            // weight, italic, underline, strikeout, charset, outprec,
            // clipprec, quality, pitch, face[32]
            let height = r.i16()? as i32;
            let width = r.i16()? as i32;
            let _escapement = r.i16()?;
            let _orientation = r.i16()?;
            let weight = r.i16()? as i32;
            let attrs = r.bytes(8)?;
            let face_bytes = r.bytes(32)?;
            let face_end = face_bytes.iter().position(|&b| b == 0).unwrap_or(32);
            let face = super::cp1251::decode(&face_bytes[..face_end]);
            if ctx.sized {
                r.bytes(8)?;
            }
            pic.fonts.push(Font {
                handle,
                height,
                width,
                weight,
                italic: attrs[0] != 0,
                underline: attrs[1] != 0,
                face,
            });
        }
        106 => pic.strings.push(StringTool { handle, text: r.string()? }),
        107 => {
            if r.peek_u16(r.pos) != Some(chunk::TEXT_PARTS) {
                return r.err("у инструмента «текст» нет списка фрагментов");
            }
            r.u16()?;
            if ctx.sized {
                r.u32()?;
            }
            let n = r.u16()?;
            r.u16()?;
            r.u16()?;
            let mut parts = Vec::with_capacity(n as usize);
            for _ in 0..n {
                parts.push(TextPart { fg: r.u32()?, bg: r.u32()?, font: r.u16()?, string: r.u16()? });
            }
            pic.texts.push(TextTool { handle, parts });
        }
        // параметры листа (чанк 1022): сетка, единицы — пока не нужны
        34 => {
            r.bytes(34)?;
        }
        110 | 111 => pic.dibs.push(Dib { handle, bmp: Vec::new(), mask: Vec::new(), file: Some(r.string()?), double }),
        103 | 104 => {
            let bmp = read_bmp(r)?;
            let mask = if kind == 104 { read_bmp(r)? } else { Vec::new() };
            pic.dibs.push(Dib { handle, bmp, mask, file: None, double });
        }
        other => return r.err(format!("инструмент типа {other} не разобран")),
    }
    Ok(())
}

fn read_bmp(r: &mut Reader) -> Result<Vec<u8>> {
    if r.peek_u16(r.pos) != Some(0x4D42) {
        return r.err("растр без заголовка BM");
    }
    let size = u32::from_le_bytes([r.data[r.pos + 2], r.data[r.pos + 3], r.data[r.pos + 4], r.data[r.pos + 5]]) as usize;
    r.bytes(size)
}

impl Picture {
    pub fn object(&self, handle: u16) -> Option<&Object> {
        self.objects.iter().find(|o| o.handle == handle)
    }

    pub fn pen(&self, handle: u16) -> Option<&Pen> {
        self.pens.iter().find(|p| p.handle == handle)
    }

    pub fn brush(&self, handle: u16) -> Option<&Brush> {
        self.brushes.iter().find(|b| b.handle == handle)
    }

    pub fn font(&self, handle: u16) -> Option<&Font> {
        self.fonts.iter().find(|f| f.handle == handle)
    }

    pub fn string(&self, handle: u16) -> Option<&StringTool> {
        self.strings.iter().find(|s| s.handle == handle)
    }

    pub fn text(&self, handle: u16) -> Option<&TextTool> {
        self.texts.iter().find(|t| t.handle == handle)
    }

    pub fn dib(&self, handle: u16) -> Option<&Dib> {
        self.dibs.iter().find(|d| d.handle == handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(rel: &str) -> Option<Vec<u8>> {
        std::fs::read(format!("{}/../fixtures/{rel}", env!("CARGO_MANIFEST_DIR"))).ok()
    }

    #[test]
    fn reads_a_library_icon() {
        let Some(data) = fixture("library/CONTROLS.LIB/HGAUGE1.VDR") else { return };
        let pic = parse(&data, "HGAUGE1.VDR").unwrap();
        assert_eq!(pic.version, 0x0202);
        assert_eq!(pic.zorder, [1, 2]);
        let line = pic.object(1).unwrap();
        match &line.kind {
            ObjectKind::Polyline { points, .. } => assert_eq!(points.len(), 5),
            other => panic!("{other:?}"),
        }
        assert!(matches!(pic.object(2).unwrap().kind, ObjectKind::Text { .. }));
        assert_eq!(pic.pens.len(), 1);
        assert_eq!(pic.fonts[0].face, "Arial");
        assert_eq!(pic.strings[0].text, "00%");
    }

    #[test]
    fn reads_the_solar_system_picture() {
        let Some(data) = fixture("user/solar_system/Root0335.cls") else { return };
        let cls = super::super::cls::parse(&data, "Root0335.cls").unwrap();
        let pic = parse(cls.image.as_ref().unwrap(), "image").unwrap();
        assert_eq!(pic.version, 0x0300);
        assert_eq!(pic.objects.len(), 158);
        assert_eq!(pic.zorder.len(), 120);
        assert_eq!(pic.pens.len(), 47);
        assert!(pic.strings.iter().any(|s| s.text == "год"));
        assert!(pic.objects.iter().any(|o| o.name == "rect") || pic.objects.iter().all(|o| o.name.is_empty()) || true);
    }
}
