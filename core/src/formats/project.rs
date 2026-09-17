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
    /// handle экземпляра, к которому относится запись.
    pub reference: u32,
    pub flags: u16,
    pub vars: Vec<(String, String)>,
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
    let _id = r.u16()?;
    let mut state = State { root, images: Vec::new() };
    // список заканчивается двухбайтовым терминатором
    while r.pos + 6 <= r.data.len() {
        let reference = r.u32()?;
        let flags = r.u16()?;
        let class_name = r.string()?;
        let _stamp = r.u32()?;
        let count = r.u16()?;
        let mut vars = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vars.push((r.string()?, r.string()?));
        }
        state.images.push(StateImage { class_name, reference, flags, vars });
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
