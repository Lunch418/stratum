//! Функции `Db*` языка моделирования поверх `.dbf` (dBASE). Пути - только
//! через песочницу файловых функций. Изменения пишутся на диск сразу.

use super::builtins::Effects;
use super::Value;
use crate::formats::dbf::{Codepage, Table};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Наибольшее число запросов `DbBrowse`, ожидающих показа.
const MAX_BROWSE: usize = 64;
/// Наибольший размер MEMO, который читает `DbPutBlob`.
const MAX_BLOB: usize = crate::formats::dbf::MAX_MEMO;

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
    /// Подключённые индексы по имени (без регистра).
    pub defs: BTreeMap<String, IndexDef>,
    /// Активный индекс: порядок обхода записей.
    pub active: Option<Active>,
}

/// Ключ индекса: (номер поля, по убыванию, без учёта регистра).
type Key = (usize, bool, bool);

#[derive(Debug, Clone)]
pub struct IndexDef {
    pub spec: String,
    pub keys: Vec<Key>,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct Active {
    pub name: String,
    pub def: IndexDef,
    /// Номера записей в порядке индекса.
    pub order: Vec<usize>,
}

/// Столбец просмотра `DbBrowse`.
#[derive(Debug, Clone)]
pub struct BrowseCol {
    pub field: String,
    pub width: u32,
    pub header: String,
    pub readonly: bool,
}

/// Запрос на показ таблицы в окне; IDE открывает его в просмотрщике DBF.
#[derive(Debug, Clone)]
pub struct Browse {
    pub window: String,
    /// Путь файла `.dbf`; пусто у курсора запроса.
    pub file: PathBuf,
    pub columns: Vec<BrowseCol>,
}

#[derive(Debug, Clone, Default)]
pub struct Dbs {
    bases: BTreeMap<u32, String>,
    pub tables: BTreeMap<u32, Open>,
    next: u32,
    pub error: u32,
    pub browse: Vec<Browse>,
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

fn flush(o: &mut Open) -> bool {
    if o.readonly {
        return false;
    }
    reindex(o);
    o.path.as_os_str().is_empty() || save(&o.path, &o.table)
}

/// Запись `.dbf` и, если есть MEMO-поля, `.dbt` (каждый атомарно).
pub fn save(path: &std::path::Path, t: &Table) -> bool {
    let (dbf, dbt) = t.to_files();
    if let Some(m) = dbt {
        if !write_atomic(&path.with_extension("dbt"), &m) {
            return false;
        }
    }
    write_atomic(path, &dbf)
}

/// Путь таблицы или индекса из модели; префикс `($project)` - папка проекта.
pub fn sandbox_path(fx: &mut Effects, name: &str, write: bool) -> Option<PathBuf> {
    let t = name.trim();
    let t = match t.get(..10) {
        Some(h) if h.eq_ignore_ascii_case("($project)") => &t[10..],
        _ => t,
    };
    super::extra::sandboxed(fx, t, write)
}

/// Таблица с диска вместе с MEMO; `write` - ещё и проверка права записи.
pub fn load(fx: &mut Effects, name: &str, write: bool) -> Result<(Table, PathBuf), u32> {
    let path = sandbox_path(fx, name, write).ok_or(1u32)?;
    let read = |p: &std::path::Path| match std::fs::metadata(p) {
        Ok(m) if m.len() <= 256 << 20 => std::fs::read(p).ok(),
        _ => None,
    };
    let data = read(&path).ok_or(1u32)?;
    // соседний .dbt тоже проходит песочницу: ссылка наружу не читается
    // (от исходного имени, а не от готового абсолютного пути: на Windows он не проходит повторно)
    let dbt = std::path::Path::new(name.trim()).with_extension("dbt");
    let dbt = sandbox_path(fx, &dbt.to_string_lossy(), false).and_then(|p| read(&p));
    let table = Table::parse_with_memo(&data, dbt.as_deref()).map_err(|_| 2u32)?;
    Ok((table, path))
}

/// Порядок записей по ключам; `unique` оставляет первую из равных.
fn build_order(t: &Table, def: &IndexDef) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..t.rows.len()).collect();
    let cmp = |a: usize, b: usize| {
        for &(f, desc, ic) in &def.keys {
            let o = crate::formats::dbf::compare_cells(t.fields[f].kind, &t.rows[a].cells[f], &t.rows[b].cells[f], ic);
            if o != std::cmp::Ordering::Equal {
                return if desc { o.reverse() } else { o };
            }
        }
        std::cmp::Ordering::Equal
    };
    idx.sort_by(|&a, &b| cmp(a, b));
    if def.unique {
        idx.dedup_by(|b, a| cmp(*a, *b) == std::cmp::Ordering::Equal);
    }
    idx
}

/// После изменения набора записей индекс строится заново.
fn reindex(o: &mut Open) {
    if let Some(a) = &mut o.active {
        a.order = build_order(&o.table, &a.def);
    }
}

/// Ключи вида `name/i/d, other` (`/c` с регистром, `/i` без, `/a` вверх,
/// `/d` вниз); `desc_all` меняет направление у всех.
pub fn parse_keys(t: &Table, spec: &str, desc_all: bool) -> Option<Vec<Key>> {
    let mut keys = Vec::new();
    for part in spec.split([',', ';']).map(str::trim).filter(|p| !p.is_empty()) {
        let mut it = part.split('/');
        let f = t.field_index(it.next()?.trim())?;
        let (mut desc, mut ic) = (desc_all, false);
        for fl in it {
            match fl.trim().to_ascii_lowercase().as_str() {
                "d" => desc = true,
                "a" => desc = false,
                "i" => ic = true,
                "c" | "" => ic = false,
                _ => return None,
            }
        }
        keys.push((f, desc, ic));
    }
    (!keys.is_empty() && keys.len() <= 16).then_some(keys)
}

/// Номер записи (в `table.rows`), на которую указывает текущая позиция.
fn phys(o: &Open) -> Option<usize> {
    match &o.active {
        Some(a) => a.order.get(o.pos).copied(),
        None => (o.pos < o.table.rows.len()).then_some(o.pos),
    }
}

fn count(o: &Open) -> usize {
    o.active.as_ref().map_or(o.table.rows.len(), |a| a.order.len())
}

fn new_open(table: Table, path: PathBuf, readonly: bool) -> Open {
    Open { table, path, readonly, pos: 0, soft: true, defs: BTreeMap::new(), active: None }
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
                6 => "превышен предел (таблицы, записи или MEMO)",
                7 => "индекс не найден",
                8 => "ошибка в SQL-запросе",
                _ => "ошибка базы данных",
            }
            .into(),
        ),
        "dbopentable" => {
            if fx.dbs.tables.len() >= MAX_TABLES {
                fail(fx, 6);
                return Some(Value::Handle(0.0));
            }
            let flags = arg_s(args, 6).to_ascii_uppercase();
            let (table, path) = match load(fx, &arg_s(args, 1), false) {
                Ok(x) => x,
                Err(code) => {
                    fail(fx, code);
                    return Some(Value::Handle(0.0));
                }
            };
            // для записи путь проверяется отдельно, иначе таблица только читается
            let readonly = flags.contains("READONLY") || sandbox_path(fx, &arg_s(args, 1), true).is_none();
            fx.dbs.next += 1;
            let id = fx.dbs.next;
            fx.dbs.tables.insert(id, new_open(table, path, readonly));
            fx.dbs.error = 0;
            Value::Handle(id as f64)
        }
        "dbclosetable" => ok(fx.dbs.tables.remove(&h).is_some()),
        "dbcreatetable" => create_table(args, fx),
        "dbsql" => sql_call(args, fx),
        "dbbrowse" | "dbsettable" => browse(args, fx),
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
            | "dbgetdelmode" | "dbsetdelmode" | "dblock" | "dbunlock" | "dbgetcodepage" | "dbsetcodepage" | "dbputblob" | "dbgetblob" | "dbfreeblob" | "dbsorttable"
            | "dbcopyto" | "dbaddindex" | "dbopenindex" | "dbcloseindex" | "dbdeleteindex" | "dbregenindex" | "dbswitchtoindex" | "dbsettokey"
    )
}

fn table_call(name: &str, args: &[Value], fx: &mut Effects, h: u32) -> Option<Value> {
    let o = fx.dbs.tables.get_mut(&h)?;
    let n = count(o);
    let cur = phys(o);
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
            let cell = field_of(o, args.get(1)).and_then(|i| cur.map(|p| o.table.rows[p].cells[i].clone()));
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
        "dbsetfield" => match (field_of(o, args.get(1)), cur.is_some()) {
            (Some(i), true) => {
                let text = match args.get(2) {
                    Some(Value::Str(s)) => s.clone(),
                    Some(v) => crate::runtime::value::format_g(v.as_float()),
                    None => String::new(),
                };
                o.table.rows[cur.unwrap_or(0)].cells[i] = text;
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
            if o.table.rows.len() >= crate::formats::dbf::MAX_CELLS / o.table.fields.len().max(1) {
                error = 6;
                return finish(fx, error, num(0.0));
            }
            o.table.rows.push(r);
            let last = o.table.rows.len() - 1;
            let r = ok(flush(o));
            o.pos = o.active.as_ref().map_or(last, |a| a.order.iter().position(|&x| x == last).unwrap_or(0));
            r
        }
        "dbinsertrecord" => {
            let r = o.table.blank_row();
            let at = if o.active.is_some() { o.table.rows.len() } else { o.pos.min(o.table.rows.len()) };
            o.table.rows.insert(at, r);
            let r = ok(flush(o));
            o.pos = o.active.as_ref().map_or(at, |a| a.order.iter().position(|&x| x == at).unwrap_or(0));
            r
        }
        "dbdeleterecord" => {
            match cur {
                None => num(0.0),
                Some(p) => {
                    if o.soft {
                        o.table.rows[p].deleted = true;
                    } else {
                        o.table.rows.remove(p);
                    }
                    let r = ok(flush(o));
                    o.pos = o.pos.min(count(o).saturating_sub(1));
                    r
                }
            }
        }
        "dbundeleterecord" => match cur {
            Some(p) => {
                o.table.rows[p].deleted = false;
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
            let r = ok(flush(o));
            o.pos = o.pos.min(count(o).saturating_sub(1));
            r
        }
        "dbgetdelmode" => num(o.soft as u8 as f64),
        "dbputblob" | "dbgetblob" | "dbfreeblob" | "dbsorttable" | "dbcopyto" | "dbaddindex" | "dbopenindex" | "dbcloseindex" | "dbdeleteindex" | "dbregenindex" | "dbswitchtoindex"
        | "dbsettokey" => return extended(name, args, fx, h),
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

fn finish(fx: &mut Effects, error: u32, v: Value) -> Option<Value> {
    if error != 0 {
        fx.dbs.error = error;
    }
    Some(v)
}

/// `DbCreateTable(hBase, name, fields, codepage, flags)`: flags - число,
/// бит 1 - перезаписать существующий файл. Поля - строка
/// `ИМЯ:тип:длина:точность` через запятую (типы C N F D L M).
fn create_table(args: &[Value], fx: &mut Effects) -> Value {
    use crate::formats::dbf::{Field, MAX_FIELDS};
    let mut fields = Vec::new();
    for part in arg_s(args, 2).split([',', ';']).map(str::trim).filter(|p| !p.is_empty()) {
        let mut it = part.split(':').map(str::trim);
        let name = it.next().unwrap_or("");
        let kind = it.next().and_then(|k| k.bytes().next()).map_or(b'C', |k| k.to_ascii_uppercase());
        let len = it.next().and_then(|v| v.parse::<u32>().ok());
        let dec = it.next().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let len = match (kind, len) {
            (b'D', _) => 8,
            (b'L', _) => 1,
            (b'M', _) => 10,
            (_, Some(l)) => l,
            (b'C', None) => 10,
            _ => 10,
        };
        let valid = !name.is_empty() && name.len() <= 10 && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
            && matches!(kind, b'C' | b'N' | b'F' | b'D' | b'L' | b'M') && (1..=255).contains(&len) && dec < len.max(1);
        if !valid || fields.len() >= MAX_FIELDS {
            fx.dbs.error = 4;
            return Value::Handle(0.0);
        }
        fields.push(Field { name: name.to_ascii_uppercase(), kind, len: len as u8, dec: dec as u8 });
    }
    let t = Table::new(fields, match arg_s(args, 3).trim() {
        "1251" | "cp1251" | "CP1251" => Codepage::Cp1251,
        _ => Codepage::Cp866,
    });
    if t.fields.is_empty() || fx.dbs.tables.len() >= MAX_TABLES {
        fx.dbs.error = if t.fields.is_empty() { 4 } else { 6 };
        return Value::Handle(0.0);
    }
    let Some(path) = sandbox_path(fx, &arg_s(args, 1), true) else {
        fx.dbs.error = 1;
        return Value::Handle(0.0);
    };
    // не затираем существующую таблицу без флага перезаписи (бит 1 числа flags)
    if path.exists() && arg_f(args, 4) as i64 & 1 == 0 {
        fx.dbs.error = 1;
        return Value::Handle(0.0);
    }
    if !save(&path, &t) {
        fx.dbs.error = 1;
        return Value::Handle(0.0);
    }
    open_new(fx, t, path, false)
}

fn open_new(fx: &mut Effects, t: Table, path: PathBuf, readonly: bool) -> Value {
    fx.dbs.next += 1;
    let id = fx.dbs.next;
    fx.dbs.tables.insert(id, new_open(t, path, readonly));
    fx.dbs.error = 0;
    Value::Handle(id as f64)
}

/// `DbSQL(hBase, запрос | поток, язык)`: результат - дескриптор таблицы.
fn sql_call(args: &[Value], fx: &mut Effects) -> Value {
    if fx.dbs.tables.len() >= MAX_TABLES {
        fx.dbs.error = 6;
        return Value::Handle(0.0);
    }
    let text = match args.get(1) {
        Some(Value::Handle(h)) => match fx.streams.items.get(&(*h as u32)) {
            Some(st) => String::from_utf8_lossy(&st.data).into_owned(),
            None => {
                fx.dbs.error = 3;
                return Value::Handle(0.0);
            }
        },
        _ => arg_s(args, 1),
    };
    if arg_f(args, 2) == 1.0 {
        // QBE не поддерживается
        fx.dbs.error = 8;
        return Value::Handle(0.0);
    }
    match super::sql::run(fx, &text) {
        Ok(d) => {
            let ro = d.path.as_os_str().is_empty();
            open_new(fx, d.table, d.path, ro)
        }
        Err(code) => {
            fx.dbs.error = code;
            Value::Handle(0.0)
        }
    }
}

/// `DbBrowse(окно, hTable, flags)`: запрос на показ в окне. Поля:
/// `имя[:n][|/h="Заголовок"][|/r]`, через запятую.
fn browse(args: &[Value], fx: &mut Effects) -> Value {
    let h = arg_f(args, 1) as u32;
    let Some(o) = fx.dbs.tables.get(&h) else {
        fx.dbs.error = 3;
        return num(0.0);
    };
    let mut columns = Vec::new();
    let spec = arg_s(args, 2);
    if spec.trim().is_empty() {
        for f in &o.table.fields {
            columns.push(BrowseCol { field: f.name.clone(), width: f.len as u32, header: f.name.clone(), readonly: false });
        }
    } else {
        for part in spec.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let mut seg = part.split('|').map(str::trim);
            let head = seg.next().unwrap_or("");
            let (name, width) = head.split_once(':').map_or((head, None), |(a, b)| (a.trim(), b.trim().parse::<u32>().ok()));
            let Some(i) = o.table.field_index(name) else {
                fx.dbs.error = 4;
                return num(0.0);
            };
            let f = &o.table.fields[i];
            let mut col = BrowseCol { field: f.name.clone(), width: width.unwrap_or(f.len as u32).clamp(1, 255), header: f.name.clone(), readonly: false };
            for flag in seg {
                if flag.eq_ignore_ascii_case("/r") {
                    col.readonly = true;
                } else if let Some(h) = flag.strip_prefix("/h").map(|x| x.trim_start_matches([' ', '=']).trim().trim_matches('"')) {
                    col.header = h.chars().take(64).collect();
                }
            }
            columns.push(col);
            if columns.len() > 255 {
                break;
            }
        }
    }
    let file = o.path.clone();
    let window: String = arg_s(args, 0).chars().take(128).collect();
    fx.dbs.browse.retain(|b| b.window != window);
    if fx.dbs.browse.len() >= MAX_BROWSE {
        fx.dbs.error = 6;
        return num(0.0);
    }
    fx.dbs.browse.push(Browse { window, file, columns });
    fx.dbs.error = 0;
    num(1.0)
}

fn index_name(args: &[Value]) -> String {
    let n = arg_s(args, 1);
    let n = n.trim();
    let n = match n.get(..10) {
        Some(h) if h.eq_ignore_ascii_case("($project)") => &n[10..],
        _ => n,
    };
    n.to_lowercase()
}

/// Файл индекса не должен быть таблицей или MEMO: их он бы затёр.
fn index_file_ok(p: &std::path::Path) -> bool {
    !p.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "dbf" | "dbt" | "fpt"))
}

/// Собственный текстовый формат индекса: заголовок, флаги, выражение ключа.
/// Порядок записей при открытии строится заново по таблице.
fn encode_index(spec: &str, flags: i64) -> String {
    format!("STRATUM-IDX 1\n{}\n{}\n", flags, spec.replace(['\r', '\n'], " "))
}

fn read_index(p: &std::path::Path) -> Option<(String, i64)> {
    if std::fs::metadata(p).ok()?.len() > 64 << 10 {
        return None;
    }
    let text = String::from_utf8(std::fs::read(p).ok()?).ok()?;
    let mut l = text.lines();
    (l.next()? == "STRATUM-IDX 1").then_some(())?;
    let flags = l.next()?.trim().parse::<i64>().ok()?;
    Some((l.next()?.to_string(), flags))
}

/// Индексы, MEMO, сортировка и копирование таблицы.
fn extended(name: &str, args: &[Value], fx: &mut Effects, h: u32) -> Option<Value> {
    let mut error = 0;
    let mut copy: Option<(Table, String, String)> = None;
    // файл индекса: `($project)CUST1.IDX` в песочнице; пустое имя - индекс только в памяти
    let ipath = match name {
        "dbaddindex" | "dbdeleteindex" | "dbopenindex" | "dbregenindex" if !arg_s(args, 1).trim().is_empty() => {
            sandbox_path(fx, &arg_s(args, 1), name != "dbopenindex" && name != "dbregenindex").filter(|p| index_file_ok(p)).map(|p| if p.extension().is_none() { p.with_extension("idx") } else { p })
        }
        _ => None,
    };
    let o = fx.dbs.tables.get_mut(&h)?;
    let v = match name {
        "dbgetblob" | "dbputblob" | "dbfreeblob" => {
            let fi = field_of(o, args.get(1)).filter(|&i| o.table.fields[i].kind == b'M');
            let cur = phys(o);
            match (fi, cur) {
                (Some(i), Some(p)) => match name {
                    "dbgetblob" => {
                        let r = &o.table.rows[p];
                        let data = r.blobs.get(&i).cloned().unwrap_or_else(|| o.table.codepage.encode(&r.cells[i]));
                        match fx.streams.items.get_mut(&(arg_f(args, 2) as u32)) {
                            Some(st) if st.writable && st.data.len() + data.len() <= MAX_BLOB * 4 => {
                                st.data.extend_from_slice(&data);
                                num(1.0)
                            }
                            _ => {
                                error = 3;
                                num(0.0)
                            }
                        }
                    }
                    _ if o.readonly => {
                        error = 5;
                        num(0.0)
                    }
                    "dbfreeblob" => {
                        o.table.rows[p].cells[i].clear();
                        o.table.rows[p].blobs.remove(&i);
                        ok(flush(o))
                    }
                    _ => match fx.streams.items.get(&(arg_f(args, 2) as u32)) {
                        Some(st) if st.data.len() <= MAX_BLOB => {
                            let data = st.data.clone();
                            let r = &mut o.table.rows[p];
                            r.cells[i] = o.table.codepage.decode(&data);
                            r.blobs.insert(i, data);
                            ok(flush(o))
                        }
                        _ => {
                            error = 3;
                            num(0.0)
                        }
                    },
                },
                _ => {
                    error = 4;
                    num(0.0)
                }
            }
        }
        "dbsorttable" => {
            if o.readonly {
                error = 5;
                num(0.0)
            } else {
                match parse_keys(&o.table, &arg_s(args, 2), false) {
                    Some(keys) => {
                        let def = IndexDef { spec: String::new(), keys, unique: arg_f(args, 1) as i64 & 1 == 1 };
                        let order = build_order(&o.table, &def);
                        let rows: Vec<_> = order.iter().map(|&i| o.table.rows[i].clone()).collect();
                        o.table.rows = rows;
                        o.pos = 0;
                        reindex(o);
                        ok(flush(o))
                    }
                    None => {
                        error = 4;
                        num(0.0)
                    }
                }
            }
        }
        "dbcopyto" => {
            copy = Some((o.table.clone(), arg_s(args, 1), arg_s(args, 2)));
            num(0.0)
        }
        // DbAddIndex(h, имя, номер, тег, флаги, тип, выражение, условие, поля)
        "dbaddindex" => {
            let spec = if args.len() > 8 { arg_s(args, 8) } else { arg_s(args, 6) };
            let flags = arg_f(args, 4) as i64;
            match parse_keys(&o.table, &spec, flags & 4 != 0) {
                Some(keys) if o.defs.len() < 32 => {
                    let def = IndexDef { spec, keys, unique: flags & 2 != 0 };
                    // с именем файла индекс сохраняется на диск, чтобы DbOpenIndex открыл его снова
                    let saved = match &ipath {
                        Some(p) => write_atomic(p, encode_index(&def.spec, flags).as_bytes()),
                        None => arg_s(args, 1).trim().is_empty(),
                    };
                    if saved {
                        o.defs.insert(index_name(args), def);
                        num(1.0)
                    } else {
                        error = 1;
                        num(0.0)
                    }
                }
                _ => {
                    error = 4;
                    num(0.0)
                }
            }
        }
        // индексы живут вместе с открытой таблицей: открыть можно только добавленный
        "dbopenindex" | "dbregenindex" => {
            let loaded = match (o.defs.contains_key(&index_name(args)), &ipath) {
                (true, _) => true,
                (false, Some(p)) => match read_index(p).and_then(|(spec, flags)| parse_keys(&o.table, &spec, flags & 4 != 0).map(|k| (spec, flags, k))) {
                    Some((spec, flags, keys)) if o.defs.len() < 32 => {
                        o.defs.insert(index_name(args), IndexDef { spec, keys, unique: flags & 2 != 0 });
                        true
                    }
                    _ => false,
                },
                _ => false,
            };
            if loaded {
                reindex(o);
                num(1.0)
            } else {
                error = 7;
                num(0.0)
            }
        }
        "dbcloseindex" | "dbdeleteindex" => {
            let n = index_name(args);
            if o.active.as_ref().is_some_and(|a| a.name == n) {
                o.active = None;
            }
            let had = o.defs.remove(&n).is_some();
            // удаление убирает и файл индекса; закрытие оставляет его
            let gone = name == "dbdeleteindex" && ipath.as_ref().is_some_and(|p| std::fs::remove_file(p).is_ok());
            ok(had || gone)
        }
        "dbswitchtoindex" => {
            let n = index_name(args);
            if n.is_empty() {
                o.active = None;
                o.pos = 0;
                num(1.0)
            } else if let Some(def) = o.defs.get(&n).cloned() {
                o.active = Some(Active { name: n, order: build_order(&o.table, &def), def });
                o.pos = 0;
                num(1.0)
            } else {
                error = 7;
                num(0.0)
            }
        }
        // поиск по ключу первого поля активного индекса (точное совпадение,
        // без регистра, для чисел - числовое)
        "dbsettokey" => match &o.active {
            Some(a) => {
                let (f, _, _) = a.def.keys[0];
                let key = arg_s(args, 1);
                let kind = o.table.fields[f].kind;
                let hit = a.order.iter().position(|&i| {
                    crate::formats::dbf::compare_cells(kind, &o.table.rows[i].cells[f], &key, true) == std::cmp::Ordering::Equal
                });
                match hit {
                    Some(p) => {
                        o.pos = p;
                        num(1.0)
                    }
                    None => num(0.0),
                }
            }
            None => {
                error = 7;
                num(0.0)
            }
        },
        _ => return None,
    };
    if let Some((t, target, kind)) = copy {
        let ok_copy = copy_to(fx, t, &target, &kind);
        return finish(fx, if ok_copy { 0 } else { 1 }, ok(ok_copy));
    }
    finish(fx, error, v)
}

/// Копия в `.dbf` песочницы; другие форматы (Paradox и т.п.) не поддержаны.
fn copy_to(fx: &mut Effects, t: Table, target: &str, kind: &str) -> bool {
    if !matches!(kind.trim().to_ascii_lowercase().as_str(), "" | "dbase" | "dbf") {
        return false;
    }
    let Some(path) = sandbox_path(fx, target, true) else { return false };
    if path.extension().is_some_and(|e| !e.eq_ignore_ascii_case("dbf")) {
        return false;
    }
    let path = if path.extension().is_none() { path.with_extension("dbf") } else { path };
    save(&path, &t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::dbf::{Field, Row};

    fn s(t: &str) -> Value {
        Value::Str(t.into())
    }

    fn f(v: f64) -> Value {
        Value::Float(v)
    }

    fn setup(tag: &str) -> (Effects, PathBuf) {
        let dir = std::env::temp_dir().join(format!("stratum-db2-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut fx = Effects::default();
        fx.gfx.project_dir = dir.clone();
        (fx, dir)
    }

    #[test]
    fn create_copy_sort_index_memo_browse() {
        let (mut fx, dir) = setup("full");
        let h = call("dbcreatetable", &[f(1.0), s("t.dbf"), s("NAME:C:8,AGE:N:3:0,NOTE:M"), s("1251"), s("")], &mut fx).unwrap();
        assert!(h.as_float() > 0.0);
        for (n, a) in [("bob", "30"), ("Ann", "25"), ("cat", "7")] {
            call("dbappendrecord", &[h.clone()], &mut fx).unwrap();
            call("dbsetfield", &[h.clone(), s("NAME"), s(n)], &mut fx).unwrap();
            call("dbsetfield", &[h.clone(), s("AGE"), s(a)], &mut fx).unwrap();
        }
        // индекс по возрасту: числовой порядок
        let r = call("dbaddindex", &[h.clone(), s("ia"), f(1.0), s(""), f(0.0), f(0.0), s(""), s(""), s("AGE")], &mut fx).unwrap();
        assert_eq!(r.as_float(), 1.0);
        call("dbswitchtoindex", &[h.clone(), s("ia"), f(1.0), s(""), f(0.0)], &mut fx).unwrap();
        assert_eq!(call("dbgetfield", &[h.clone(), s("NAME")], &mut fx).unwrap().as_string(), "cat");
        assert_eq!(call("dbsettokey", &[h.clone(), s("25")], &mut fx).unwrap().as_float(), 1.0);
        assert_eq!(call("dbgetfield", &[h.clone(), s("NAME")], &mut fx).unwrap().as_string(), "Ann");
        call("dbswitchtoindex", &[h.clone(), s(""), f(0.0), s(""), f(0.0)], &mut fx).unwrap();
        // сортировка без регистра
        call("dbsorttable", &[h.clone(), f(0.0), s("NAME/i")], &mut fx).unwrap();
        call("dbgotop", &[h.clone()], &mut fx).unwrap();
        assert_eq!(call("dbgetfield", &[h.clone(), s("NAME")], &mut fx).unwrap().as_string(), "Ann");
        // MEMO через потоки
        let st = fx.streams.open(super::super::extra::Stream { data: "заметка".as_bytes().to_vec(), writable: true, ..Default::default() });
        assert_eq!(call("dbputblob", &[h.clone(), f(3.0), Value::Handle(st as f64)], &mut fx).unwrap().as_float(), 1.0);
        call("dbclosetable", &[h.clone()], &mut fx).unwrap();
        assert!(dir.join("t.dbt").exists());
        let (t, _) = load(&mut fx, "t.dbf", false).unwrap();
        assert_eq!(t.rows[0].blobs[&2], "заметка".as_bytes());
        // копирование и просмотр
        let h = call("dbopentable", &[f(1.0), s("t.dbf"), s(""), s(""), s(""), f(0.0), s("")], &mut fx).unwrap();
        assert_eq!(call("dbcopyto", &[h.clone(), s("c.dbf"), s("dBASE")], &mut fx).unwrap().as_float(), 1.0);
        assert_eq!(call("dbcopyto", &[h.clone(), s("/proc/x.dbf"), s("dBASE")], &mut fx).unwrap().as_float(), 0.0);
        assert_eq!(call("dbcopyto", &[h.clone(), s("c.db"), s("PARADOX")], &mut fx).unwrap().as_float(), 0.0);
        assert!(dir.join("c.dbf").exists());
        assert_eq!(call("dbbrowse", &[s("W"), h.clone(), s("name:12|/h=\"Имя\", age|/r")], &mut fx).unwrap().as_float(), 1.0);
        assert_eq!(fx.dbs.browse[0].columns[0].header, "Имя");
        assert!(fx.dbs.browse[0].columns[1].readonly);
        assert_eq!(call("dbbrowse", &[s("W"), h.clone(), s("nope")], &mut fx).unwrap().as_float(), 0.0);
        // SQL
        let b = call("dbopenbase", &[s(""), s(""), s(""), s("")], &mut fx).unwrap();
        let q = call("dbsql", &[b.clone(), s("SELECT NAME FROM t WHERE AGE > 10 ORDER BY NAME"), f(2.0)], &mut fx).unwrap();
        assert_eq!(call("dbgetcount", &[q.clone()], &mut fx).unwrap().as_float(), 2.0);
        assert_eq!(call("dbsql", &[b, s("garbage"), f(2.0)], &mut fx).unwrap().as_float(), 0.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_table_guards() {
        let (mut fx, dir) = setup("guards");
        for bad in ["", "A:Z:3", "A:C:0", "A:C:999", "TOOLONGNAME1:C:3", "A:N:3:3", "A B:C:3"] {
            let r = call("dbcreatetable", &[f(1.0), s("g.dbf"), s(bad), s(""), s("")], &mut fx).unwrap();
            assert_eq!(r.as_float(), 0.0, "{bad}");
        }
        let many = (0..300).map(|i| format!("F{i}:C:1")).collect::<Vec<_>>().join(",");
        assert_eq!(call("dbcreatetable", &[f(1.0), s("g.dbf"), s(&many), s(""), s("")], &mut fx).unwrap().as_float(), 0.0);
        for p in ["/proc/g.dbf", "/etc/g.dbf"] {
            assert_eq!(call("dbcreatetable", &[f(1.0), s(p), s("A:C:3"), s(""), s("")], &mut fx).unwrap().as_float(), 0.0);
        }
        assert!(call("dbcreatetable", &[f(1.0), s("g.dbf"), s("A:C:3"), s(""), s("")], &mut fx).unwrap().as_float() > 0.0);
        // существующую таблицу не затираем
        assert_eq!(call("dbcreatetable", &[f(1.0), s("g.dbf"), s("A:C:3"), s(""), s("")], &mut fx).unwrap().as_float(), 0.0);
        assert!(!dir.join("g.dbf.tmp~").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn real_sample_with_memo_if_present() {
        let p = "/snap/libreoffice/379/lib/libreoffice/presets/database/biblio/biblio.dbf";
        let Ok(d) = std::fs::read(p) else { return };
        let m = std::fs::read(p.replace(".dbf", ".dbt")).ok();
        let t = Table::parse_with_memo(&d, m.as_deref()).unwrap();
        assert!(!t.rows.is_empty());
        let (a, b) = t.to_files();
        let back = Table::parse_with_memo(&a, b.as_deref()).unwrap();
        assert_eq!(back.rows.len(), t.rows.len());
        assert_eq!(back.rows[0].cells, t.rows[0].cells);
    }

    #[test]
    fn model_functions_round_trip() {
        let dir = std::env::temp_dir().join(format!("stratum-db-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut t = Table::new(vec![Field { name: "NAME".into(), kind: b'C', len: 10, dec: 0 }, Field { name: "AGE".into(), kind: b'N', len: 4, dec: 0 }], Codepage::Cp866);
        t.rows.push(Row { deleted: false, cells: vec!["Аня".into(), "20".into()], ..Default::default() });
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

    #[test]
    fn index_file_persists_and_reopens() {
        let (mut fx, dir) = setup("idxfile");
        let h = call("dbcreatetable", &[f(1.0), s("t.dbf"), s("NAME:C:8,AGE:N:3:0"), s("1251"), s("")], &mut fx).unwrap();
        for (n, a) in [("bob", "30"), ("Ann", "25"), ("cat", "7")] {
            call("dbappendrecord", &[h.clone()], &mut fx).unwrap();
            call("dbsetfield", &[h.clone(), s("NAME"), s(n)], &mut fx).unwrap();
            call("dbsetfield", &[h.clone(), s("AGE"), s(a)], &mut fx).unwrap();
        }
        let add = |fx: &mut Effects, h: &Value, name: &str| call("dbaddindex", &[h.clone(), s(name), f(1.0), s(""), f(0.0), f(0.0), s(""), s(""), s("AGE")], fx).unwrap().as_float();
        assert_eq!(add(&mut fx, &h, "($project)ages.idx"), 1.0);
        assert!(dir.join("ages.idx").is_file());
        // индекс не может затереть таблицу
        assert_eq!(add(&mut fx, &h, "t.dbf"), 0.0);
        // выход из песочницы отвергается
        assert_eq!(add(&mut fx, &h, "/etc/evil.idx"), 0.0);
        call("dbclosetable", &[h.clone()], &mut fx).unwrap();
        let h = call("dbopentable", &[f(1.0), s("t.dbf"), s(""), s(""), s(""), f(0.0), s("")], &mut fx).unwrap();
        assert_eq!(call("dbswitchtoindex", &[h.clone(), s("ages.idx"), f(1.0), s(""), f(0.0)], &mut fx).unwrap().as_float(), 0.0);
        assert_eq!(call("dbopenindex", &[h.clone(), s("($project)AGES.IDX"), f(1.0)], &mut fx).unwrap().as_float(), 1.0);
        call("dbswitchtoindex", &[h.clone(), s("ages.idx"), f(1.0), s(""), f(0.0)], &mut fx).unwrap();
        assert_eq!(call("dbgetfield", &[h.clone(), s("NAME")], &mut fx).unwrap().as_string(), "cat");
        // повреждённый файл не открывается
        std::fs::write(dir.join("bad.idx"), "junk").unwrap();
        assert_eq!(call("dbopenindex", &[h.clone(), s("bad.idx"), f(1.0)], &mut fx).unwrap().as_float(), 0.0);
        assert_eq!(call("dbdeleteindex", &[h.clone(), s("ages.idx")], &mut fx).unwrap().as_float(), 1.0);
        assert!(!dir.join("ages.idx").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// Запись через временный файл и rename: сбой не обрезает исходную таблицу.
fn write_atomic(path: &std::path::Path, data: &[u8]) -> bool {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp~");
    let tmp = std::path::PathBuf::from(tmp);
    // заранее подложенная ссылка на месте временного файла не выбирается
    let _ = std::fs::remove_file(&tmp);
    let wrote = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp).and_then(|mut f| std::io::Write::write_all(&mut f, data));
    if wrote.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    true
}
