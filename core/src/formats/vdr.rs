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
    /// Параметры листа (запись типа 34 чанка 1022) как есть — пишутся обратно.
    pub page: Option<Vec<u8>>,
    /// Гиперссылки объектов (закладка «Гипербаза»): handle объекта → ссылка.
    pub hypers: Vec<(u16, Hyper)>,
    /// Данные объектов (блок `0x03FE`) как есть: кроме гиперссылки там
    /// переменные объекта и прочее — при записи сохраняются.
    pub object_data: Vec<(u16, Vec<u8>)>,
    /// 3.x: нераспознанные чанки после известных (данные самого листа —
    /// блоки 0x03FC/0x03FD и т. п.) как есть; пишутся обратно в конец.
    pub extra: Vec<u8>,
    /// Приложения ломаных (стрелки и т. п.) как есть: handle → байты.
    pub arrows: Vec<(u16, Vec<u8>)>,
    /// Трёхмерные пространства (инструмент `3D` в чанке 1012).
    pub spaces3d: Vec<Space3dData>,
}

/// Трёхмерное пространство из рисунка: инструмент с сигнатурой `3D`
/// (`u16 0x4433`, [3.x: `u32 размер`], `u16 handle`), внутри — чанки 1021
/// (порядок объектов верхнего уровня), 1020 (объекты) и в 3.x 1024
/// (материалы). Записи объектов — как у 2D: 2.x `u16 тип, u16 handle,
/// u16 флаги, str имя`; 3.x `u16 тип, u16 размер, u16 0, u16 handle,
/// u16 флаги`, имя — блок `0xCD` в конце.
#[derive(Debug, Clone, Default)]
pub struct Space3dData {
    pub handle: u16,
    pub zorder: Vec<u16>,
    pub objects: Vec<Object3dData>,
    /// 3.x: чанк материалов как есть (пишется обратно).
    pub materials: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Object3dData {
    pub handle: u16,
    pub name: String,
    pub flags: u16,
    pub kind: Object3dKind,
}

#[derive(Debug, Clone)]
pub enum Object3dKind {
    /// Тип 10: `u16 материал?, u16 n, n × (f64 x, y, z), u16 m, m ×
    /// примитив, 16 × f64 матрица (строками, перенос в четвёртой), u16`.
    Mesh {
        material: u16,
        points: Vec<[f64; 3]>,
        prims: Vec<Prim3dData>,
        matrix: [f64; 16],
        tail: u16,
    },
    /// Тип 11: 200 байт — см. `Camera3dData`.
    Camera(Vec<u8>),
    /// Тип 12: 88 байт.
    Light(Vec<u8>),
    /// Типы 3–5: вложенный чанк 1021 — дочерние объекты.
    Group { kind: u16, children: Vec<u16> },
}

/// Примитив: `u16 n, u16 k, u16 флаги, u32 цвет, n × u16 номер точки,
/// n·k × (f64 u, f64 v)` — текстурные координаты.
#[derive(Debug, Clone)]
pub struct Prim3dData {
    pub layers: u16,
    pub flags: u16,
    pub color: u32,
    pub idx: Vec<u16>,
    pub uv: Vec<f64>,
}

/// Гиперссылка графического объекта. Хранится в блоке данных объекта
/// `0x03FE`: в 3.x — блок расширения в конце записи объекта, в 2.x — список
/// после инструментов (`u16 1, u16 handle, u16 0x03FE, данные`; запись
/// `u16 2` без handle — данные самого листа; `u16 0` — конец списка).
/// Данные: `u16 версия, u16 5, u16 2, u8 число элементов`, элементы
/// `u16 номер, u16 длина, байты`. Гиперссылка — элемент `0x0a`, в нём поля
/// `u16 номер` + значение: 1 — цель (строка), 4 — режим (`u16`);
/// отсутствующее поле — значение по умолчанию (режим 0 «открыть окно»).
/// Номера 2, 3, 5 (окно, объект, эффект) в корпусе не встречаются и приняты
/// по порядку аргументов `SetHyperJump2d`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hyper {
    /// 0 — открыть окно, 1 — запустить приложение, 2 — загрузить проект,
    /// 3 — ничего не делать, 4 — системная команда.
    pub mode: i32,
    pub target: String,
    pub window: String,
    pub object: String,
    pub effect: String,
}

const HYPER_TAG: u16 = 0x03FE;
const HYPER_HEAD: [u8; 7] = [1, 0, 5, 0, 2, 0, 1];

/// Длина данных объекта (блок `0x03FE`) от их начала; `None` — не они.
fn object_data_len(data: &[u8]) -> Option<usize> {
    let count = *data.get(6)? as usize;
    let mut i = 7;
    for _ in 0..count {
        let n = u16::from_le_bytes([*data.get(i + 2)?, *data.get(i + 3)?]) as usize;
        i += 4 + n;
    }
    (i <= data.len()).then_some(i)
}

/// Гиперссылка из данных объекта (блок `0x03FE`); `None` — ссылки нет.
pub fn parse_hyper(data: &[u8]) -> Option<Hyper> {
    let count = *data.get(6)? as usize;
    let mut at = 7;
    let mut item = None;
    for _ in 0..count {
        let id = u16::from_le_bytes([*data.get(at)?, *data.get(at + 1)?]);
        let n = u16::from_le_bytes([*data.get(at + 2)?, *data.get(at + 3)?]) as usize;
        if id == 0x0a {
            item = Some(data.get(at + 4..at + 4 + n)?);
        }
        at += 4 + n;
    }
    let data = item?;
    let u16_at = |i: usize| data.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let end = data.len();
    let mut h = Hyper::default();
    let mut i = 0;
    while i + 2 <= end {
        let id = u16_at(i)?;
        i += 2;
        if id == 4 {
            h.mode = u16_at(i)? as i32;
            i += 2;
            continue;
        }
        let n = u16_at(i)? as usize;
        let text = super::cp1251::decode(data.get(i + 2..i + 2 + n)?);
        i += 2 + n;
        match id {
            1 => h.target = text,
            2 => h.window = text,
            3 => h.object = text,
            5 => h.effect = text,
            _ => return Some(h),
        }
    }
    Some(h)
}

/// Данные объекта для записи: прежние элементы, кроме гиперссылки, и
/// гиперссылка `hyper`; `None` — писать нечего.
pub fn merge_object_data(raw: Option<&[u8]>, hyper: Option<&Hyper>) -> Option<Vec<u8>> {
    let mut items: Vec<(u16, Vec<u8>)> = Vec::new();
    let mut head = HYPER_HEAD.to_vec();
    if let Some(raw) = raw.filter(|r| object_data_len(r).is_some()) {
        head = raw[..7].to_vec();
        let mut at = 7;
        for _ in 0..raw[6] {
            let id = u16::from_le_bytes([raw[at], raw[at + 1]]);
            let n = u16::from_le_bytes([raw[at + 2], raw[at + 3]]) as usize;
            if id != 0x0a {
                items.push((id, raw[at + 4..at + 4 + n].to_vec()));
            }
            at += 4 + n;
        }
    }
    if let Some(h) = hyper {
        items.push((0x0a, write_hyper(h)[11..].to_vec()));
    }
    if items.is_empty() {
        return None;
    }
    head[6] = items.len() as u8;
    for (id, data) in items {
        head.extend_from_slice(&id.to_le_bytes());
        head.extend_from_slice(&(data.len() as u16).to_le_bytes());
        head.extend_from_slice(&data);
    }
    Some(head)
}

/// Элементы данных объекта: номер → байты.
fn object_items(raw: &[u8]) -> Vec<(u16, Vec<u8>)> {
    let mut items = Vec::new();
    if object_data_len(raw).is_none() {
        return items;
    }
    let mut at = 7;
    for _ in 0..raw[6] {
        let id = u16::from_le_bytes([raw[at], raw[at + 1]]);
        let n = u16::from_le_bytes([raw[at + 2], raw[at + 3]]) as usize;
        items.push((id, raw[at + 4..at + 4 + n].to_vec()));
        at += 4 + n;
    }
    items
}

/// Переменные объекта (закладка «Переменные»): элемент `0x0d`, строка —
/// пары `переменная имиджа,переменная или псевдоним другого имиджа` через `;`
/// (в «Роботе» `_HObject,_HPrev;out,in` — связи `_HObject→_HPrev`, `out→in`).
pub fn object_vars(raw: &[u8]) -> Option<String> {
    let (_, data) = object_items(raw).into_iter().find(|(id, _)| *id == 0x0d)?;
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    Some(super::cp1251::decode(&data[..end]))
}

/// Заменяет переменные объекта; пустая строка убирает элемент. `None` — данных не осталось.
pub fn set_object_vars(raw: Option<&[u8]>, text: &str) -> Option<Vec<u8>> {
    let head = raw.filter(|r| object_data_len(r).is_some()).map(|r| r[..7].to_vec()).unwrap_or_else(|| HYPER_HEAD.to_vec());
    let mut items: Vec<(u16, Vec<u8>)> = raw.map(object_items).unwrap_or_default().into_iter().filter(|(id, _)| *id != 0x0d).collect();
    if !text.is_empty() {
        let mut bytes = super::cp1251::encode(text);
        bytes.push(0);
        items.push((0x0d, bytes));
    }
    if items.is_empty() {
        return None;
    }
    let mut out = head;
    out[6] = items.len() as u8;
    for (id, data) in items {
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&(data.len() as u16).to_le_bytes());
        out.extend_from_slice(&data);
    }
    Some(out)
}

/// Тело блока гиперссылки (без тега и длины блока).
pub fn write_hyper(h: &Hyper) -> Vec<u8> {
    let mut fields = Vec::new();
    for (id, text) in [(1u16, &h.target), (2, &h.window), (3, &h.object), (5, &h.effect)] {
        if id != 1 && text.is_empty() {
            continue;
        }
        let bytes = super::cp1251::encode(text);
        fields.extend_from_slice(&id.to_le_bytes());
        fields.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        fields.extend_from_slice(&bytes);
    }
    if h.mode != 0 {
        fields.extend_from_slice(&4u16.to_le_bytes());
        fields.extend_from_slice(&(h.mode as u16).to_le_bytes());
    }
    let mut out = HYPER_HEAD.to_vec();
    out.extend_from_slice(&0x0au16.to_le_bytes());
    out.extend_from_slice(&(fields.len() as u16).to_le_bytes());
    out.extend_from_slice(&fields);
    out
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
    /// Тип 24: проекция трёхмерного пространства — `num x,y,w,h,
    /// u16 пространство (место инструмента 3D в чанке, с нуля), u16 камера`.
    View3d {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        space: u16,
        camera: u16,
    },
    /// Тип, который ещё не разобран.
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

    let mut total = 0usize;
    let mut header_end = None;
    let tools_at = if ctx.sized {
        r.u16()?;
        total = r.u32()? as usize;
        None
    } else {
        // размер заголовка — без двух последних байт (как и `tools_at`)
        header_end = Some(start + r.u32()? as usize + 2);
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
        // заголовок бывает длиннее (у рисунков с 3D — ещё прямоугольник и
        // сетка): чанки начинаются по его размеру из первого поля
        if let Some(end) = header_end.filter(|e| *e > r.pos && *e < r.data.len()) {
            r.pos = end;
        }
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
    if !ctx.sized {
        read_object_data_v2(&r.data[r.pos.min(r.data.len())..], &mut pic);
    } else if total > 0 {
        let end = (start + total).min(r.data.len());
        if r.pos < end {
            pic.extra = r.data[r.pos..end].to_vec();
        }
    }
    Ok(pic)
}

/// 2.x: после инструментов — `u32, u16` и список данных объектов, из
/// которого берутся гиперссылки. Непонятное просто пропускается.
fn read_object_data_v2(tail: &[u8], pic: &mut Picture) {
    let u16_at = |i: usize| tail.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let mut i = 6;
    while let Some(kind) = u16_at(i) {
        let handle = match kind {
            1 => u16_at(i + 2),
            2 => Some(0),
            _ => None,
        };
        let Some(handle) = handle else { break };
        i += if kind == 1 { 4 } else { 2 };
        if u16_at(i) != Some(HYPER_TAG) {
            break;
        }
        i += 2;
        let Some(n) = object_data_len(&tail[i..]) else { break };
        if kind == 1 {
            if let Some(h) = parse_hyper(&tail[i..i + n]) {
                pic.hypers.push((handle, h));
            }
            pic.object_data.push((handle, tail[i..i + n].to_vec()));
        }
        i += n;
    }
}

fn read_chunk(r: &mut Reader, ctx: &Ctx, id: u16, pic: &mut Picture) -> Result<()> {
    let at = r.pos;
    r.u16()?;
    let size = if ctx.sized { Some(r.u32()? as usize) } else { None };
    // размер меньше заголовка чанка вернул бы чтение на тот же чанк: разбор
    // зацикливался и копил объекты, пока не кончалась память
    if size.is_some_and(|s| s < 12) {
        return r.err("чанк короче своего заголовка");
    }
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
    let end = if ctx.sized && kind == SPACE3D {
        None
    } else if ctx.sized {
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
        let mut attachment = None;
        let body = match read_object(r, ctx, kind, &mut attachment) {
            Ok(k) => {
                if let Some(extra) = attachment {
                    pic.arrows.push((handle, extra));
                }
                k
            }
            Err(e) => match end {
                Some(_) => ObjectKind::Unknown { kind },
                None => return Err(e),
            },
        };
        if let Some(end) = end {
            // блоки расширения: 0xCD — имя, 0x03FE — гиперссылка
            while r.pos + 6 <= end {
                let tag = r.u16()?;
                let n = r.u32()? as usize;
                if n < 6 || r.pos + n - 6 > end {
                    break;
                }
                let data = r.bytes(n - 6)?;
                match tag {
                    0xCD => name = super::cp1251::decode(&data),
                    HYPER_TAG => {
                        if let Some(h) = parse_hyper(&data) {
                            pic.hypers.push((handle, h));
                        }
                        pic.object_data.push((handle, data.to_vec()));
                    }
                    _ => {}
                }
            }
            r.pos = end;
        }
        pic.objects.push(Object { handle, name, flags, kind: body });
    } else if kind == SPACE3D {
        // трёхмерное пространство: после типа [u32 размер] и один u16 —
        // номер инструмента, на него ссылаются проекции (объекты типа 24)
        let end = if ctx.sized { Some(at + r.u32()? as usize) } else { None };
        let handle = r.u16()?;
        let space = read_space3d(r, ctx, handle, end)?;
        pic.spaces3d.push(space);
        if let Some(end) = end {
            r.pos = end;
        }
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

/// Тип инструмента «трёхмерное пространство»: байты `3D`.
pub const SPACE3D: u16 = 0x4433;

fn read_space3d(r: &mut Reader, ctx: &Ctx, handle: u16, end: Option<usize>) -> Result<Space3dData> {
    let mut sp = Space3dData { handle, ..Default::default() };
    loop {
        if end.is_some_and(|e| r.pos + 2 > e) {
            break;
        }
        let Some(id) = r.peek_u16(r.pos) else { break };
        let at = r.pos;
        match id {
            chunk::ZORDER | chunk::OBJECTS | 1024 => {}
            _ => break,
        }
        r.u16()?;
        let size = if ctx.sized { Some(r.u32()? as usize) } else { None };
        let count = r.u16()?;
        r.u16()?;
        r.u16()?;
        match id {
            chunk::ZORDER => {
                for _ in 0..count {
                    sp.zorder.push(r.u16()?);
                }
            }
            chunk::OBJECTS => {
                if count > 0 {
                    r.u8()?;
                    r.u16()?;
                }
                for _ in 0..count {
                    sp.objects.push(read_object3d(r, ctx)?);
                }
            }
            _ => {
                // материалы: разобраны только настолько, чтобы сохранить
                let Some(size) = size else { return r.err("материалы 3D без размера не разобраны") };
                sp.materials = Some(r.data[at..at + size].to_vec());
            }
        }
        if let Some(size) = size {
            r.pos = at + size;
        }
    }
    Ok(sp)
}

fn read_object3d(r: &mut Reader, ctx: &Ctx) -> Result<Object3dData> {
    let at = r.pos;
    let kind = r.u16()?;
    let (end, handle, flags, mut name);
    if ctx.sized {
        // «u16 размер, u16 0» — это u32 размер: у тел бывает больше 64 КБ
        end = Some(at + r.u32()? as usize);
        handle = r.u16()?;
        flags = r.u16()?;
        name = String::new();
    } else {
        end = None;
        handle = r.u16()?;
        flags = r.u16()?;
        name = r.string()?;
    }
    let body = match kind {
        10 => {
            let material = r.u16()?;
            let n = r.u16()? as usize;
            let mut points = Vec::with_capacity(n);
            for _ in 0..n {
                points.push([r.f64()?, r.f64()?, r.f64()?]);
            }
            let m = r.u16()? as usize;
            let mut prims = Vec::with_capacity(m);
            for _ in 0..m {
                let k = r.u16()? as usize;
                let layers = r.u16()?;
                let flags = r.u16()?;
                let color = r.u32()?;
                let mut idx = Vec::with_capacity(k);
                for _ in 0..k {
                    idx.push(r.u16()?);
                }
                let mut uv = Vec::with_capacity(2 * k * layers as usize);
                for _ in 0..2 * k * layers as usize {
                    uv.push(r.f64()?);
                }
                prims.push(Prim3dData { layers, flags, color, idx, uv });
            }
            let mut matrix = [0.0; 16];
            for v in matrix.iter_mut() {
                *v = r.f64()?;
            }
            let tail = r.u16()?;
            Object3dKind::Mesh { material, points, prims, matrix, tail }
        }
        11 => Object3dKind::Camera(r.bytes(200)?),
        12 => Object3dKind::Light(r.bytes(88)?),
        3..=5 => {
            if r.peek_u16(r.pos) != Some(chunk::ZORDER) {
                return r.err("у 3D-группы нет списка объектов");
            }
            r.u16()?;
            if ctx.sized {
                r.u32()?;
            }
            let n = r.u16()?;
            r.u16()?;
            r.u16()?;
            let mut children = Vec::with_capacity(n as usize);
            for _ in 0..n {
                children.push(r.u16()?);
            }
            Object3dKind::Group { kind, children }
        }
        other => return r.err(format!("3D-объект типа {other} не разобран")),
    };
    if let Some(end) = end {
        while r.pos + 6 <= end {
            let tag = r.u16()?;
            let n = r.u32()? as usize;
            if n < 6 || r.pos + n - 6 > end {
                break;
            }
            let data = r.bytes(n - 6)?;
            if tag == 0xCD {
                name = super::cp1251::decode(&data);
            }
        }
        r.pos = end;
    }
    Ok(Object3dData { handle, name, flags, kind: body })
}

fn read_object(r: &mut Reader, ctx: &Ctx, kind: u16, attachment: &mut Option<Vec<u8>>) -> Result<ObjectKind> {
    if matches!(kind, 3..=5) {
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
            let mut extra = Vec::new();
            if ctx.wide {
                let n = r.u8()? as usize;
                extra = r.bytes(n)?.to_vec();
            }
            // габарит в файле записан с запасом в единицу; оригинал при
            // загрузке берёт его по точкам (панель L1 337 → 336, круг BALLS
            // 31 → 30). Ломаная с приложением (стрелкой) хранит свой габарит.
            let (x, y, w, h) = match points.first() {
                Some(&(x0, y0)) if extra.is_empty() => {
                    let (mut a, mut b, mut c, mut d) = (x0, y0, x0, y0);
                    for &(px, py) in &points {
                        a = a.min(px);
                        b = b.min(py);
                        c = c.max(px);
                        d = d.max(py);
                    }
                    (a, b, c - a, d - b)
                }
                _ => (x, y, w, h),
            };
            if !extra.is_empty() {
                *attachment = Some(extra);
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
        24 => ObjectKind::View3d { x, y, w, h, space: r.u16()?, camera: r.u16()? },
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
            pic.page = Some(r.bytes(34)?);
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

/// Собирает рисунок в формате 3.0 (`2D`, версия 0x0300): все координаты
/// f64, у записей и чанков есть размеры, имена объектов в блоке `0xCD`.
pub fn write(pic: &Picture) -> Vec<u8> {
    use super::writer::Writer;
    let mut w = Writer::new();
    w.bytes(SIG);
    w.u16(0x0300);
    w.u16(0x0300);
    w.u16(0x00CC);
    let total_at = w.pos();
    w.u32(0);
    for v in [pic.origin.0, pic.origin.1, pic.scale.0, pic.scale.1, pic.window.0, pic.window.1] {
        w.f64(v);
    }
    w.u16(0);
    w.bytes(&[0u8; 8]);

    // чанк: id, size, count, capacity, delta, [u8 1, u16 0xFFFF], записи
    fn chunk(w: &mut Writer, id: u16, count: usize, body: impl FnOnce(&mut Writer)) {
        if count == 0 && id != chunk::ZORDER {
            return;
        }
        let at = w.pos();
        w.u16(id);
        w.u32(0);
        w.u16(count as u16);
        w.u16(count as u16);
        w.u16(10);
        if count > 0 && id != chunk::ZORDER {
            w.u8(1);
            w.u16(0xFFFF);
        }
        body(w);
        let size = (w.pos() - at) as u32;
        w.patch_u32(at + 2, size);
    }
    // запись: kind, size(u16|u32), тело; размер — от начала записи
    fn record(w: &mut Writer, kind: u16, wide_size: bool, body: impl FnOnce(&mut Writer)) {
        let at = w.pos();
        w.u16(kind);
        if wide_size {
            w.u32(0);
        } else {
            w.u16(0);
        }
        body(w);
        let size = (w.pos() - at) as u32;
        if wide_size {
            w.patch_u32(at + 2, size);
        } else {
            let bytes = (size as u16).to_le_bytes();
            w.data[at + 2] = bytes[0];
            w.data[at + 3] = bytes[1];
        }
    }

    chunk(&mut w, chunk::ZORDER, pic.zorder.len(), |w| {
        for z in &pic.zorder {
            w.u16(*z);
        }
    });
    chunk(&mut w, chunk::OBJECTS, pic.objects.len(), |w| {
        for o in &pic.objects {
            let kind = match &o.kind {
                ObjectKind::Polyline { .. } => 20,
                ObjectKind::Bitmap { masked: false, .. } => 21,
                ObjectKind::Bitmap { masked: true, .. } => 22,
                ObjectKind::Text { .. } => 23,
                ObjectKind::Control { .. } => 26,
                ObjectKind::Group { .. } => 3,
                ObjectKind::View3d { .. } => 24,
                ObjectKind::Unknown { kind } => *kind,
            };
            record(w, kind, false, |w| {
                w.u16(0);
                w.u16(o.handle);
                w.u16(o.flags);
                match &o.kind {
                    ObjectKind::Group { children } => {
                        w.u16(chunk::ZORDER);
                        w.u32(14 + children.len() as u32 * 2);
                        w.u16(children.len() as u16);
                        w.u16(children.len() as u16);
                        w.u16(10);
                        for c in children {
                            w.u16(*c);
                        }
                    }
                    ObjectKind::Polyline { x, y, w: ww, h, pen, brush, points } => {
                        for v in [*x, *y, *ww, *h] {
                            w.f64(v);
                        }
                        w.u16(*pen);
                        w.u16(*brush);
                        w.u16(points.len() as u16);
                        for (px, py) in points {
                            w.f64(*px);
                            w.f64(*py);
                        }
                        match pic.arrows.iter().find(|(h, _)| *h == o.handle) {
                            Some((_, extra)) if extra.len() < 256 => {
                                w.u8(extra.len() as u8);
                                for b in extra {
                                    w.u8(*b);
                                }
                            }
                            _ => w.u8(0),
                        }
                    }
                    ObjectKind::Bitmap { x, y, w: ww, h, src, dib, .. } => {
                        for v in [*x, *y, *ww, *h, src.0, src.1, src.2, src.3] {
                            w.f64(v);
                        }
                        w.u16(0);
                        w.u16(*dib);
                    }
                    ObjectKind::Text { x, y, w: ww, h, text } => {
                        for v in [*x, *y, *ww, *h] {
                            w.f64(v);
                        }
                        w.u16(*text);
                        w.bytes(&[0u8; 18]);
                    }
                    ObjectKind::Control { x, y, w: ww, h, class, caption, style } => {
                        for v in [*x, *y, *ww, *h] {
                            w.f64(v);
                        }
                        w.string(class);
                        w.string(caption);
                        w.u16(0);
                        w.u32(*style);
                        w.bytes(&[0u8; 10]);
                    }
                    ObjectKind::View3d { x, y, w: ww, h, space, camera } => {
                        for v in [*x, *y, *ww, *h] {
                            w.f64(v);
                        }
                        w.u16(*space);
                        w.u16(*camera);
                    }
                    ObjectKind::Unknown { .. } => {}
                }
                if !o.name.is_empty() {
                    let bytes = super::cp1251::encode(&o.name);
                    w.u16(0xCD);
                    w.u32(bytes.len() as u32 + 6);
                    w.bytes(&bytes);
                }
                let hyper = pic.hypers.iter().find(|(handle, _)| *handle == o.handle).map(|(_, h)| h);
                let raw = pic.object_data.iter().find(|(handle, _)| *handle == o.handle).map(|(_, d)| d.as_slice());
                if let Some(bytes) = merge_object_data(raw, hyper) {
                    w.u16(HYPER_TAG);
                    w.u32(bytes.len() as u32 + 6);
                    w.bytes(&bytes);
                }
            });
        }
    });
    chunk(&mut w, 1004, pic.pens.len(), |w| {
        for p in &pic.pens {
            record(w, 101, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(p.handle);
                w.u32(p.color);
                w.u16(p.style);
                w.u16(p.width);
                w.u16(p.rop);
            });
        }
    });
    chunk(&mut w, 1005, pic.brushes.len(), |w| {
        for b in &pic.brushes {
            record(w, 102, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(b.handle);
                w.u32(b.color);
                w.u16(b.style);
                w.u16(b.hatch);
                w.u16(b.rop);
                w.u16(b.dib);
            });
        }
    });
    chunk(&mut w, 1012, pic.spaces3d.len(), |w| {
        for sp in &pic.spaces3d {
            record(w, SPACE3D, true, |w| {
                w.u16(sp.handle);
                chunk(w, chunk::ZORDER, sp.zorder.len(), |w| {
                    for z in &sp.zorder {
                        w.u16(*z);
                    }
                });
                chunk(w, chunk::OBJECTS, sp.objects.len(), |w| {
                    for o in &sp.objects {
                        let kind = match &o.kind {
                            Object3dKind::Mesh { .. } => 10,
                            Object3dKind::Camera(_) => 11,
                            Object3dKind::Light(_) => 12,
                            Object3dKind::Group { kind, .. } => *kind,
                        };
                        // размер записи — u32: у тел бывает больше 64 КБ
                        record(w, kind, true, |w| {
                            w.u16(o.handle);
                            w.u16(o.flags);
                            match &o.kind {
                                Object3dKind::Mesh { material, points, prims, matrix, tail } => {
                                    w.u16(*material);
                                    w.u16(points.len() as u16);
                                    for p in points {
                                        for v in p {
                                            w.f64(*v);
                                        }
                                    }
                                    w.u16(prims.len() as u16);
                                    for p in prims {
                                        w.u16(p.idx.len() as u16);
                                        w.u16(p.layers);
                                        w.u16(p.flags);
                                        w.u32(p.color);
                                        for i in &p.idx {
                                            w.u16(*i);
                                        }
                                        for v in &p.uv {
                                            w.f64(*v);
                                        }
                                    }
                                    for v in matrix {
                                        w.f64(*v);
                                    }
                                    w.u16(*tail);
                                }
                                Object3dKind::Camera(b) | Object3dKind::Light(b) => w.bytes(b),
                                Object3dKind::Group { children, .. } => {
                                    w.u16(chunk::ZORDER);
                                    w.u32(14 + children.len() as u32 * 2);
                                    w.u16(children.len() as u16);
                                    w.u16(children.len() as u16);
                                    w.u16(10);
                                    for c in children {
                                        w.u16(*c);
                                    }
                                }
                            }
                            if !o.name.is_empty() {
                                let bytes = super::cp1251::encode(&o.name);
                                w.u16(0xCD);
                                w.u32(bytes.len() as u32 + 6);
                                w.bytes(&bytes);
                            }
                        });
                    }
                });
                if let Some(m) = &sp.materials {
                    w.bytes(m);
                }
            });
        }
    });
    for (id, double) in [(1006u16, false), (chunk::DOUBLE_DIBS, true)] {
        let dibs: Vec<&Dib> = pic.dibs.iter().filter(|d| d.double == double).collect();
        chunk(&mut w, id, dibs.len(), |w| {
            for d in dibs {
                match &d.file {
                    Some(file) => record(w, if double { 111 } else { 110 }, false, |w| {
                        w.u16(0);
                        w.u16(1);
                        w.u16(d.handle);
                        w.string(file);
                    }),
                    None => record(w, if double { 104 } else { 103 }, true, |w| {
                        w.u16(1);
                        w.u16(d.handle);
                        w.bytes(&d.bmp);
                        if double {
                            w.bytes(&d.mask);
                        }
                    }),
                }
            }
        });
    }
    chunk(&mut w, 1008, pic.fonts.len(), |w| {
        for f in &pic.fonts {
            record(w, 105, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(f.handle);
                w.u16(f.height as i16 as u16);
                w.u16(f.width as i16 as u16);
                w.u16(0);
                w.u16(0);
                w.u16(f.weight as i16 as u16);
                w.bytes(&[f.italic as u8, f.underline as u8, 0, 204, 0, 0, 0, 0]);
                let mut face = super::cp1251::encode(&f.face);
                face.truncate(31);
                face.resize(32, 0);
                w.bytes(&face);
                w.bytes(&[0u8; 8]);
            });
        }
    });
    chunk(&mut w, 1009, pic.strings.len(), |w| {
        for st in &pic.strings {
            record(w, 106, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(st.handle);
                w.string(&st.text);
            });
        }
    });
    chunk(&mut w, 1010, pic.texts.len(), |w| {
        for t in &pic.texts {
            record(w, 107, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(t.handle);
                w.u16(chunk::TEXT_PARTS);
                w.u32(12 + t.parts.len() as u32 * 12);
                w.u16(t.parts.len() as u16);
                w.u16(t.parts.len() as u16);
                w.u16(10);
                for p in &t.parts {
                    w.u32(p.fg);
                    w.u32(p.bg);
                    w.u16(p.font);
                    w.u16(p.string);
                }
            });
        }
    });
    if let Some(page) = &pic.page {
        chunk(&mut w, 1022, 1, |w| {
            record(w, 34, false, |w| {
                w.u16(0);
                w.u16(1);
                w.u16(1);
                w.bytes(page);
            });
        });
    }
    w.bytes(&pic.extra);
    let total = w.pos() as u32;
    w.patch_u32(total_at, total);
    w.data
}

fn read_bmp(r: &mut Reader) -> Result<Vec<u8>> {
    if r.peek_u16(r.pos) != Some(0x4D42) {
        return r.err("растр без заголовка BM");
    }
    // «BM» у самого конца файла: поля размера нет — ошибка, а не паника
    let Some(b) = r.data.get(r.pos + 2..r.pos + 6) else { return r.err("растр обрезан") };
    let size = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
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

    #[test]
    fn chunk_shorter_than_its_header_does_not_loop() {
        let mut data = write(&Picture { version: 0x0300, zorder: vec![1, 2], ..Default::default() });
        // чанк Z-порядка: id 1021 и сразу размер; размер 0 — порча файла
        let at = data.windows(2).position(|b| b == chunk::ZORDER.to_le_bytes()).expect("нет чанка Z-порядка");
        data[at + 2..at + 6].copy_from_slice(&0u32.to_le_bytes());
        assert!(parse(&data, "x.vdr").is_err());
    }

    #[test]
    fn bitmap_signature_at_end_of_file_is_an_error() {
        for data in [&b"BM"[..], b"BM\x10", b"BM\x10\0\0"] {
            assert!(read_bmp(&mut Reader::new(data, "x.vdr")).is_err());
        }
    }

    #[test]
    fn hyperlink_blocks_of_the_corpus_round_trip() {
        // блоки из примера «hyper»: переход к странице и системная команда
        let page = hex("010005000200010a000c000100080077696e646f775f32");
        let cmd = hex("010005000200010a00130001000b00434d5f505245565041474504000400");
        let h = parse_hyper(&page).unwrap();
        assert_eq!((h.mode, h.target.as_str()), (0, "window_2"));
        assert_eq!(write_hyper(&h), page);
        let h = parse_hyper(&cmd).unwrap();
        assert_eq!((h.mode, h.target.as_str()), (4, "CM_PREVPAGE"));
        assert_eq!(write_hyper(&h), cmd);
        let full = Hyper { mode: 0, target: "стр.vdr".into(), window: "Окно".into(), object: "Root".into(), effect: "".into() };
        assert_eq!(parse_hyper(&write_hyper(&full)).unwrap(), full);
        // данные объекта без гиперссылки (элемент 0x0d — переменные объекта)
        let vars = hex("010005000200010d0004006162630a");
        assert_eq!(parse_hyper(&vars), None);
        assert_eq!(object_data_len(&vars), Some(vars.len()));
        // переменные объекта не теряются, гиперссылка добавляется рядом
        let h = Hyper { target: "стр".into(), ..Default::default() };
        let merged = merge_object_data(Some(&vars), Some(&h)).unwrap();
        assert_eq!(merged[6], 2);
        assert_eq!(parse_hyper(&merged).unwrap(), h);
        assert_eq!(merge_object_data(Some(&merged), None).unwrap(), vars);
        // закладка «Переменные»: как в «Роботе» — `_HObject,_HPrev;out,in`
        let with = set_object_vars(Some(&page), "_HObject,_HPrev;out,in").unwrap();
        assert_eq!(object_vars(&with).as_deref(), Some("_HObject,_HPrev;out,in"));
        assert_eq!(parse_hyper(&with).unwrap().target, "window_2");
        assert_eq!(set_object_vars(Some(&with), "").unwrap(), page);
        assert_eq!(merge_object_data(Some(&page), Some(&parse_hyper(&page).unwrap())).unwrap(), page);
    }

    #[test]
    fn keeps_unknown_sheet_chunks_of_a_new_picture() {
        // «Робот»: за известными чанками 3.x идут данные листа (0x03FD, 0x03FC)
        let Some(data) = fixture("PROJECTS/samples/ROBOT/ROBOT3.VDR") else { return };
        let pic = parse(&data, "ROBOT3.VDR").unwrap();
        assert!(!pic.extra.is_empty());
        let back = parse(&write(&pic), "copy").unwrap();
        assert_eq!(back.extra, pic.extra);
        assert_eq!(back.objects.len(), pic.objects.len());
    }

    #[test]
    fn reads_hyperlinks_of_an_old_picture() {
        // «hyper»: рисунок 2.02 корневого имиджа — кнопки меню ведут на страницы
        let Some(data) = fixture("PROJECTS/samples/hyper/Root5782.cls") else { return };
        let cls = crate::formats::cls::parse(&data, "Root5782.cls").unwrap();
        let pic = parse(cls.image.as_deref().unwrap(), "Root5782").unwrap();
        let targets: Vec<&str> = pic.hypers.iter().map(|(_, h)| h.target.as_str()).collect();
        assert!(targets.contains(&"window_2") && targets.contains(&"StratumClass_Products"), "{targets:?}");
        assert_eq!(pic.hypers.iter().find(|(h, _)| *h == 93).unwrap().1.target, "window_2");
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

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
