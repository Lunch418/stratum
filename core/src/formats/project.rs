//! Чтение `project.spj` и `_preload.stt`. Форматы описаны в `docs/formats/spj.md`.

use super::cp1251;
use super::reader::{Reader, Result};

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Int(u32),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone)]
pub struct ProjectVariable {
    pub kind: u16,
    pub flags: u16,
    pub handle: Option<u16>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct Project {
    pub root: String,
    pub properties: Vec<Property>,
    pub variables: Vec<ProjectVariable>,
}

impl Project {
    pub fn property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties
            .iter()
            .find(|p| p.key.eq_ignore_ascii_case(key))
            .map(|p| &p.value)
    }
}

pub fn parse_project(data: &[u8], path: &str) -> Result<Project> {
    let mut r = Reader::new(data, path);
    if data.len() < 4 || &data[0..2] != b"Ih" {
        return r.err("не файл проекта: нет сигнатуры Ih");
    }
    let kind = data[2];
    r.pos = 4;
    let mut project = Project::default();
    match kind {
        b'f' => {
            let n = r.u16()?;
            read_properties(&mut r, n, &mut project)?;
            r.u16()?;
            project.root = r.string()?;
        }
        b'd' => project.root = r.string()?,
        other => return r.err(format!("неизвестный вид проекта {:?}", other as char)),
    }
    while !r.eof() {
        match r.u8()? {
            0 => break,
            b'f' => {
                r.u8()?;
                let n = r.u16()?;
                read_properties(&mut r, n, &mut project)?;
            }
            b'g' => {
                r.u8()?;
                let n = r.u16()?;
                for _ in 0..n {
                    let kind = r.u16()?;
                    let flags = r.u16()?;
                    let handle = if kind == 2 { Some(r.u16()?) } else { None };
                    let name = r.string()?;
                    let description = r.string()?;
                    project.variables.push(ProjectVariable {
                        kind,
                        flags,
                        handle,
                        name,
                        description,
                    });
                }
            }
            other => return r.err(format!("неизвестный маркер блока {:?}", other as char)),
        }
    }
    Ok(project)
}

fn read_properties(r: &mut Reader, count: u16, project: &mut Project) -> Result<()> {
    for _ in 0..count {
        let size = r.u16()? as usize;
        let end = r.pos + size;
        let ptype = r.u8()?;
        let key_len = r.u8()? as usize;
        let key_bytes = r.bytes(key_len)?;
        let key = cp1251::decode(split_nul(&key_bytes));
        if end < r.pos || end > r.data.len() {
            return r.err("длина записи-свойства выходит за конец файла");
        }
        let raw = r.bytes(end - r.pos)?;
        let value = if ptype == 0 && raw.len() >= 4 {
            PropertyValue::Int(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
        } else {
            PropertyValue::Text(cp1251::decode(split_nul(&raw)))
        };
        project.properties.push(Property { key, value });
    }
    Ok(())
}

/// Собирает `project.spj` вида `d` — так пишет сама Stratum 2000 v3.
pub fn write_project(project: &Project) -> Vec<u8> {
    use super::writer::Writer;
    let mut w = Writer::new();
    w.bytes(b"Ihd\0");
    w.string(&project.root);
    if !project.properties.is_empty() {
        w.u8(b'f');
        w.u8(0);
        w.u16(project.properties.len() as u16);
        for p in &project.properties {
            let mut key = cp1251::encode(&p.key);
            key.push(0);
            let (ptype, value): (u8, Vec<u8>) = match &p.value {
                PropertyValue::Int(i) => (0, i.to_le_bytes().to_vec()),
                PropertyValue::Text(t) => {
                    let mut v = cp1251::encode(t);
                    v.push(0);
                    (2, v)
                }
            };
            w.u16((2 + key.len() + value.len()) as u16);
            w.u8(ptype);
            w.u8(key.len() as u8);
            w.bytes(&key);
            w.bytes(&value);
        }
    }
    if !project.variables.is_empty() {
        w.u8(b'g');
        w.u8(0);
        w.u16(project.variables.len() as u16);
        for v in &project.variables {
            w.u16(v.kind);
            w.u16(v.flags);
            if v.kind == 2 {
                w.u16(v.handle.unwrap_or(0));
            }
            w.string(&v.name);
            w.string(&v.description);
        }
    }
    w.u8(0);
    w.data
}

/// Собирает `_preload.stt` в новой редакции (переменные по именам).
pub fn write_state(state: &State) -> Vec<u8> {
    use super::writer::Writer;
    let mut w = Writer::new();
    w.string("SC Scheme Variables");
    w.string(&state.root);
    w.u16(0x28);
    w.u16(2);
    w.u16(0x03ea);
    for im in &state.images {
        w.u32(im.reference);
        w.u16(im.handle);
        w.string(&im.class_name);
        w.u32(0);
        w.u16(im.vars.len() as u16);
        for (name, value) in &im.vars {
            w.string(name);
            w.string(value);
        }
    }
    w.u16(0);
    w.data
}

fn split_nul(bytes: &[u8]) -> &[u8] {
    match bytes.iter().position(|&b| b == 0) {
        Some(i) => &bytes[..i],
        None => bytes,
    }
}

/// Снимок значений переменных всех имиджей (`_preload.stt`).
#[derive(Debug, Clone, Default)]
pub struct State {
    pub root: String,
    pub images: Vec<StateImage>,
}

#[derive(Debug, Clone)]
pub struct StateImage {
    pub class_name: String,
    pub reference: u32,
    /// handle экземпляра на схеме (как в секции детей `.cls`).
    pub handle: u16,
    pub vars: Vec<(String, String)>,
}

/// Редакция 0x3E8 — её пишет `SaveObjectState` оригинала. Сначала таблица
/// «классов», куда входят и типы (`HANDLE`, `STRING`, `FLOAT`, `COLORREF` —
/// у них ненулевой размер): имя, отметка времени, число переменных, число
/// детей, размер; пары (handle ребёнка, номер класса); переменные (имя, номер
/// типа в той же таблице). Затем маркер 0x3E9 и значения экземпляров в
/// обходе дерева от корня: FLOAT — f64, STRING — строка, остальные — u32.
fn parse_state_typed(r: &mut Reader, root: String) -> Result<State> {
    struct Entry {
        name: String,
        kids: Vec<(u16, u16)>,
        vars: Vec<(String, u16)>,
    }
    let count = r.u16()?;
    let mut table = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = r.string()?;
        let _stamp = r.u32()?;
        let nvars = r.u16()?;
        let nkids = r.u16()?;
        let _size = r.u16()?;
        let kids = (0..nkids).map(|_| Ok((r.u16()?, r.u16()?))).collect::<Result<Vec<_>>>()?;
        let vars = (0..nvars).map(|_| Ok((r.string()?, r.u16()?))).collect::<Result<Vec<_>>>()?;
        table.push(Entry { name, kids, vars });
    }
    let marker = r.u16()?;
    if marker != 0x3e9 {
        return r.err(format!("старая редакция .stt: ожидался маркер данных 0x3e9, а не {marker:#x}"));
    }
    let mut state = State { root, images: Vec::new() };
    // обход в глубину от корня (запись 0), как пишет оригинал
    let mut stack: Vec<(usize, u16)> = vec![(0, 0)];
    while let Some((ci, handle)) = stack.pop() {
        let Some(e) = table.get(ci) else { return r.err(format!("нет записи класса {ci}")) };
        let mut vars = Vec::with_capacity(e.vars.len());
        for (name, ty) in &e.vars {
            let value = match table.get(*ty as usize).map(|t| t.name.as_str()) {
                Some("STRING") => r.string()?,
                Some("FLOAT") => super::super::runtime::value::format_number(r.f64()?),
                _ => r.u32()?.to_string(),
            };
            vars.push((name.clone(), value));
        }
        let reference = state.images.len() as u32;
        state.images.push(StateImage { class_name: e.name.clone(), reference, handle, vars });
        for (h, k) in e.kids.iter().rev() {
            stack.push((*k as usize, *h));
        }
    }
    Ok(state)
}

pub fn parse_state(data: &[u8], path: &str) -> Result<State> {
    let mut r = Reader::new(data, path);
    let magic = r.string()?;
    if !magic.starts_with("SC ") {
        return r.err(format!("не файл состояния: {magic:?}"));
    }
    let root = r.string()?;
    let _record_size = r.u16()?;
    let _format = r.u16()?;
    // в старой редакции здесь снова лежит имя корневого имиджа, а переменные
    // хранятся по индексам; такие файлы (DEFAULT.STT и снимки) не читаем
    if r.peek_u16(r.pos) == Some(cp1251::encode(&root).len() as u16) {
        return r.err("старая редакция .stt: переменные хранятся по индексам");
    }
    let id = r.u16()?;
    if id == 0x3e8 {
        return parse_state_typed(&mut r, root);
    }
    let mut state = State { root, images: Vec::new() };
    // список заканчивается двухбайтовым терминатором
    while r.pos + 6 <= r.data.len() {
        let reference = r.u32()?;
        let handle = r.u16()?;
        let class_name = r.string()?;
        let _stamp = r.u32()?;
        let count = r.u16()?;
        let mut vars = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vars.push((r.string()?, r.string()?));
        }
        state.images.push(StateImage { class_name, reference, handle, vars });
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(rel: &str) -> Option<Vec<u8>> {
        std::fs::read(format!("{}/../fixtures/{rel}", env!("CARGO_MANIFEST_DIR"))).ok()
    }

    #[test]
    fn reads_project_with_properties() {
        let Some(data) = fixture("user/solar_system/project.spj") else { return };
        let p = parse_project(&data, "project.spj").unwrap();
        assert_eq!(p.root, "Root_e2c115d_b1a");
        assert!(matches!(p.property("MathMode"), Some(PropertyValue::Int(3))));
    }

    #[test]
    fn reads_project_variables() {
        let Some(data) = fixture("PROJECTS/samples/Osc3d/project.spj") else { return };
        let p = parse_project(&data, "project.spj").unwrap();
        let names: Vec<_> = p.variables.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["ScaleZ", "ScaleY", "OffsetY"]);
    }

    #[test]
    fn reads_preload_state() {
        let Some(data) = fixture("user/solar_system_2/_preload.stt") else { return };
        let s = parse_state(&data, "_preload.stt").unwrap();
        assert_eq!(s.root, "Root_d23262c_b10");
        assert!(s.images.iter().any(|i| i.vars.iter().any(|(k, _)| k == "ObjectName")));
    }
}
