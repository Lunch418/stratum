//! Функции `Db*` языка моделирования поверх `.dbf` (dBASE). Пути - только
//! через песочницу файловых функций. Изменения пишутся на диск сразу.

use super::builtins::Effects;
use super::Value;
use crate::formats::dbf::{Codepage, Table};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Наибольшее число одновременно открытых таблиц.
const MAX_TABLES: usize = 64;

#[derive(Debug, Clone)]
pub struct Open {
    pub table: Table,
    pub path: PathBuf,
    pub readonly: bool,
    /// Текущая запись, с нуля.
    pub pos: usize,
    /// Режим удаления: 1 - запись только помечается и восстановима.
    pub soft: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Dbs {
    bases: BTreeMap<u32, String>,
    pub tables: BTreeMap<u32, Open>,
    next: u32,
    pub error: u32,
}

fn num(v: f64) -> Value {
    Value::Float(v)
}

fn ok(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}

fn arg_f(a: &[Value], i: usize) -> f64 {
    a.get(i).map_or(0.0, |v| v.as_float())
}

fn arg_s(a: &[Value], i: usize) -> String {
    a.get(i).map(|v| v.as_string()).unwrap_or_default()
}

fn fail(fx: &mut Effects, code: u32) -> Option<()> {
    fx.dbs.error = code;
    None
}

fn flush(o: &Open) -> bool {
    !o.readonly && write_atomic(&o.path, &o.table.to_bytes())
}

/// Поле по номеру (с 1, число) или по имени (строка).
fn field_of(o: &Open, v: Option<&Value>) -> Option<usize> {
    match v? {
        Value::Str(s) => o.table.field_index(s),
        other => {
            let i = other.as_float();
            (i >= 1.0 && (i as usize) <= o.table.fields.len()).then(|| i as usize - 1)
        }
    }
}

fn type_code(kind: u8) -> f64 {
    match kind {
        b'N' => 514.0,
        b'M' => 515.0,
        b'L' => 516.0,
        b'D' => 517.0,
        b'F' => 518.0,
        _ => 513.0,
    }
}

/// `None` - такой функции базы нет (остаётся заглушкой).
pub fn call(name: &str, args: &[Value], fx: &mut Effects) -> Option<Value> {
    let h = arg_f(args, 0) as u32;
    let v = match name {
        "dbopenbase" => {
            fx.dbs.next += 1;
            let id = fx.dbs.next;
            fx.dbs.bases.insert(id, arg_s(args, 0));
            Value::Handle(id as f64)
        }
        "dbclosebase" => ok(fx.dbs.bases.remove(&h).is_some()),
        "dbcloseall" => {
            fx.dbs.tables.clear();
            fx.dbs.bases.clear();
            num(1.0)
        }
        "dbsetdir" => ok(fx.dbs.bases.contains_key(&h)),
        "dbgeterror" => num(fx.dbs.error as f64),
        "dbgeterrorstr" => Value::Str(
            match arg_f(args, 0) as u32 {
                0 => "",
                1 => "файл таблицы не найден или недоступен",
                2 => "повреждённая таблица",
                3 => "неверный дескриптор",
                4 => "нет такого поля",
                5 => "таблица открыта только для чтения",
                _ => "ошибка базы данных",
            }
            .into(),
        ),
        "dbopentable" => {
            if fx.dbs.tables.len() >= MAX_TABLES {
                fail(fx, 6);
                return Some(Value::Handle(0.0));
            }
            let Some(path) = super::extra::sandboxed(fx, &arg_s(args, 1), false) else {
                fail(fx, 1);
                return Some(Value::Handle(0.0));
            };
            let flags = arg_s(args, 6).to_ascii_uppercase();
            if std::fs::metadata(&path).map_or(true, |m| m.len() > 256 << 20) {
                fail(fx, 1);
                return Some(Value::Handle(0.0));
            }
            let Ok(data) = std::fs::read(&path) else {
                fail(fx, 1);
                return Some(Value::Handle(0.0));
            };
            let Ok(table) = Table::parse(&data) else {
                fail(fx, 2);
                return Some(Value::Handle(0.0));
            };
            // для записи путь проверяется отдельно, иначе таблица только читается
            let readonly = flags.contains("READONLY") || super::extra::sandboxed(fx, &arg_s(args, 1), true).is_none();
            fx.dbs.next += 1;
            let id = fx.dbs.next;
            fx.dbs.tables.insert(id, Open { table, path, readonly, pos: 0, soft: true });
            fx.dbs.error = 0;
            Value::Handle(id as f64)
        }
        "dbclosetable" => ok(fx.dbs.tables.remove(&h).is_some()),
        _ => {
            if !name.starts_with("db") || !fx.dbs.tables.contains_key(&h) {
                // функции нет или дескриптор не таблица
                return if is_known(name) {
                    fail(fx, 3);
                    Some(num(0.0))
                } else {
                    None
                };
            }
            table_call(name, args, fx, h)?
        }
    };
    Some(v)
}

fn is_known(name: &str) -> bool {
    matches!(
        name,
        "dbgotop" | "dbgobottom" | "dbskip" | "dbgetpos" | "dbgetcount" | "dbgetfieldcount" | "dbfieldid" | "dbgetfield" | "dbgetfieldn" | "dbgetfieldname"
            | "dbgetfieldtype" | "dbsetfield" | "dbappendrecord" | "dbinsertrecord" | "dbdeleterecord" | "dbundeleterecord" | "dbzap" | "dbpackfile"
            | "dbgetdelmode" | "dbsetdelmode" | "dblock" | "dbunlock" | "dbgetcodepage" | "dbsetcodepage"
    )
}

fn table_call(name: &str, args: &[Value], fx: &mut Effects, h: u32) -> Option<Value> {
    let o = fx.dbs.tables.get_mut(&h)?;
    let n = o.table.rows.len();
    let mut error = 0;
    let v = match name {
        "dbgotop" => {
            o.pos = 0;
            ok(n > 0)
        }
        "dbgobottom" => {
            o.pos = n.saturating_sub(1);
            ok(n > 0)
        }
        "dbskip" => {
            let to = o.pos as i64 + arg_f(args, 1) as i64;
            if to >= 0 && (to as usize) < n {
                o.pos = to as usize;
                num(1.0)
            } else {
                num(0.0)
            }
        }
        "dbgetpos" => num((o.pos + 1) as f64),
        "dbgetcount" => num(n as f64),
        "dbgetfieldcount" => num(o.table.fields.len() as f64),
        "dbfieldid" => match o.table.field_index(&arg_s(args, 1)) {
            Some(i) => num((i + 1) as f64),
            None => num(0.0),
        },
        "dbgetfieldname" | "dbgetfieldtype" => match field_of(o, args.get(1)) {
            Some(i) if name == "dbgetfieldname" => Value::Str(o.table.fields[i].name.clone()),
            Some(i) => num(type_code(o.table.fields[i].kind)),
            None => {
                error = 4;
                if name == "dbgetfieldname" { Value::Str(String::new()) } else { num(0.0) }
            }
        },
        "dbgetfield" | "dbgetfieldn" => {
            let cell = field_of(o, args.get(1)).and_then(|i| o.table.rows.get(o.pos).map(|r| r.cells[i].clone()));
            match cell {
                Some(c) if name == "dbgetfield" => Value::Str(c),
                Some(c) => num(c.trim().parse::<f64>().unwrap_or(0.0)),
                None => {
                    error = 4;
                    if name == "dbgetfield" { Value::Str(String::new()) } else { num(0.0) }
                }
            }
        }
        "dbsetfield" if o.readonly => {
            error = 5;
            num(0.0)
        }
        "dbsetfield" => match (field_of(o, args.get(1)), o.table.rows.get(o.pos).is_some()) {
            (Some(i), true) => {
                let text = match args.get(2) {
                    Some(Value::Str(s)) => s.clone(),
                    Some(v) => crate::runtime::value::format_g(v.as_float()),
                    None => String::new(),
                };
                o.table.rows[o.pos].cells[i] = text;
                ok(flush(o))
            }
            _ => {
                error = 4;
                num(0.0)
            }
        },
        "dbappendrecord" | "dbinsertrecord" | "dbdeleterecord" | "dbundeleterecord" | "dbzap" | "dbpackfile" | "dbsetdelmode" if o.readonly => {
            error = 5;
            num(0.0)
        }
        "dbappendrecord" => {
            let r = o.table.blank_row();
            o.table.rows.push(r);
            o.pos = o.table.rows.len() - 1;
            ok(flush(o))
        }
        "dbinsertrecord" => {
            let r = o.table.blank_row();
            let at = o.pos.min(n);
            o.table.rows.insert(at, r);
            o.pos = at;
            ok(flush(o))
        }
        "dbdeleterecord" => {
            if o.pos >= n {
                num(0.0)
            } else {
                if o.soft {
                    o.table.rows[o.pos].deleted = true;
                } else {
                    o.table.rows.remove(o.pos);
                    o.pos = o.pos.min(o.table.rows.len().saturating_sub(1));
                }
                ok(flush(o))
            }
        }
        "dbundeleterecord" => match o.table.rows.get_mut(o.pos) {
            Some(r) => {
                r.deleted = false;
                ok(flush(o))
            }
            None => num(0.0),
        },
        "dbzap" => {
            o.table.rows.clear();
            o.pos = 0;
            ok(flush(o))
        }
        "dbpackfile" => {
            o.table.rows.retain(|r| !r.deleted);
            o.pos = o.pos.min(o.table.rows.len().saturating_sub(1));
            ok(flush(o))
        }
        "dbgetdelmode" => num(o.soft as u8 as f64),
        "dbsetdelmode" => {
            o.soft = arg_f(args, 1) != 0.0;
            num(1.0)
        }
        "dblock" | "dbunlock" => num(1.0),
        "dbgetcodepage" => num(o.table.codepage.number() as f64),
        "dbsetcodepage" => match Codepage::from_number(arg_f(args, 1) as u32) {
            Some(cp) => {
                o.table.codepage = cp;
                ok(!o.readonly && flush(o))
            }
            None => num(0.0),
        },
        _ => return None,
    };
    if error != 0 {
        fx.dbs.error = error;
    }
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::dbf::{Field, Row};

    fn s(t: &str) -> Value {
        Value::Str(t.into())
    }

    #[test]
    fn model_functions_round_trip() {
        let dir = std::env::temp_dir().join(format!("stratum-db-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut t = Table::new(vec![Field { name: "NAME".into(), kind: b'C', len: 10, dec: 0 }, Field { name: "AGE".into(), kind: b'N', len: 4, dec: 0 }], Codepage::Cp866);
        t.rows.push(Row { deleted: false, cells: vec!["Аня".into(), "20".into()] });
        std::fs::write(dir.join("a.dbf"), t.to_bytes()).unwrap();
        let mut fx = Effects::default();
        fx.gfx.project_dir = dir.clone();
        let base = call("dbopenbase", &[s(""), s(""), s(""), s("")], &mut fx).unwrap();
        let args = [base, s("a.dbf"), s(""), s(""), s(""), Value::Float(0.0), s("")];
        let h = call("dbopentable", &args, &mut fx).unwrap();
        assert!(h.as_float() > 0.0);
        assert_eq!(call("dbgetcount", &[h.clone()], &mut fx).unwrap().as_float(), 1.0);
        assert_eq!(call("dbgetfield", &[h.clone(), s("name")], &mut fx).unwrap().as_string(), "Аня");
        assert_eq!(call("dbappendrecord", &[h.clone()], &mut fx).unwrap().as_float(), 1.0);
        call("dbsetfield", &[h.clone(), s("NAME"), s("Боря")], &mut fx).unwrap();
        call("dbsetfield", &[h.clone(), Value::Float(2.0), Value::Float(31.0)], &mut fx).unwrap();
        call("dbclosetable", &[h.clone()], &mut fx).unwrap();
        let back = Table::parse(&std::fs::read(dir.join("a.dbf")).unwrap()).unwrap();
        assert_eq!(back.rows.len(), 2);
        assert_eq!(back.rows[1].cells, vec!["Боря".to_string(), "31".to_string()]);
        let bad = [Value::Handle(1.0), s("../../etc/passwd"), s(""), s(""), s(""), Value::Float(0.0), s("")];
        assert_eq!(call("dbopentable", &bad, &mut fx).unwrap().as_float(), 0.0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Запись через временный файл и rename: сбой не обрезает исходную таблицу.
fn write_atomic(path: &std::path::Path, data: &[u8]) -> bool {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp~");
    let tmp = std::path::PathBuf::from(tmp);
    if std::fs::write(&tmp, data).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    true
}
