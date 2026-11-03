//! dBASE III/IV/FoxPro (`.dbf`): заголовок, описания полей C/N/F/D/L/M,
//! записи с признаком удаления, текст в cp866 или cp1251.
//!
//! Читатель не доверяет заголовку: число записей, длины и смещения
//! сверяются с настоящим размером данных, память под записи выделяется
//! только по прочитанному, а не по заявленному числу.

use super::cp1251;

/// Наибольшее число ячеек в таблице: память под строки растёт с числом
/// записей x полей, а не с размером файла.
pub const MAX_CELLS: usize = 16 << 20;

/// Наибольшее число полей (предел формата dBASE).
pub const MAX_FIELDS: usize = 255;

/// Наибольший размер одного MEMO (байт) и всего файла `.dbt`.
pub const MAX_MEMO: usize = 16 << 20;
pub const MAX_DBT: usize = 128 << 20;
const BLOCK: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codepage {
    Cp866,
    Cp1251,
}

impl Codepage {
    fn language_byte(self) -> u8 {
        match self {
            Codepage::Cp866 => 0x65,
            Codepage::Cp1251 => 0xC9,
        }
    }
    pub fn number(self) -> u32 {
        match self {
            Codepage::Cp866 => 866,
            Codepage::Cp1251 => 1251,
        }
    }
    pub fn from_number(n: u32) -> Option<Codepage> {
        match n {
            866 => Some(Codepage::Cp866),
            1251 => Some(Codepage::Cp1251),
            _ => None,
        }
    }
    pub fn decode(self, b: &[u8]) -> String {
        match self {
            Codepage::Cp1251 => cp1251::decode(b),
            Codepage::Cp866 => b
                .iter()
                .map(|&c| match c {
                    0..=0x7f => c as char,
                    0x80..=0xaf => char::from_u32(0x410 + (c - 0x80) as u32).unwrap_or('?'),
                    0xe0..=0xef => char::from_u32(0x440 + (c - 0xe0) as u32).unwrap_or('?'),
                    0xf0 => '\u{401}',
                    0xf1 => '\u{451}',
                    _ => '?',
                })
                .collect(),
        }
    }
    pub fn encode(self, t: &str) -> Vec<u8> {
        match self {
            Codepage::Cp1251 => cp1251::encode(t),
            Codepage::Cp866 => t
                .chars()
                .map(|c| match c as u32 {
                    0..=0x7f => c as u8,
                    0x410..=0x43f => (c as u32 - 0x410) as u8 + 0x80,
                    0x440..=0x44f => (c as u32 - 0x440) as u8 + 0xe0,
                    0x401 => 0xf0,
                    0x451 => 0xf1,
                    _ => b'?',
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    /// `C`, `N`, `F`, `D`, `L`, `M`.
    pub kind: u8,
    pub len: u8,
    pub dec: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Row {
    pub deleted: bool,
    /// Значения полей текстом, без хвостовых пробелов.
    pub cells: Vec<String>,
    /// Сырое содержимое MEMO-полей (номер поля -> байты); для текстовых
    /// MEMO совпадает с `cells[i]` в кодовой странице таблицы.
    pub blobs: std::collections::BTreeMap<usize, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub version: u8,
    pub codepage: Codepage,
    pub fields: Vec<Field>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbfError(pub String);

impl std::fmt::Display for DbfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn err<T>(m: &str) -> Result<T, DbfError> {
    Err(DbfError(m.to_string()))
}

fn u16le(d: &[u8], at: usize) -> usize {
    d.get(at..at + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as usize)
}

fn width(kind: u8, len: u8) -> usize {
    match kind {
        b'D' => 8,
        b'L' => 1,
        b'M' => 10,
        _ => len as usize,
    }
}

impl Table {
    pub fn new(fields: Vec<Field>, codepage: Codepage) -> Table {
        Table { version: 3, codepage, fields, rows: Vec::new() }
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name.eq_ignore_ascii_case(name))
    }

    pub fn blank_row(&self) -> Row {
        Row { deleted: false, cells: self.fields.iter().map(|_| String::new()).collect(), blobs: Default::default() }
    }

    pub fn has_memo(&self) -> bool {
        self.fields.iter().any(|f| f.kind == b'M')
    }

    pub fn parse(d: &[u8]) -> Result<Table, DbfError> {
        Self::parse_with_memo(d, None)
    }

    /// Разбор вместе с файлом `.dbt` (dBASE III/IV). Без него MEMO пустые.
    pub fn parse_with_memo(d: &[u8], dbt: Option<&[u8]>) -> Result<Table, DbfError> {
        let mut t = Self::parse_raw(d)?;
        t.load_memos(dbt);
        Ok(t)
    }

    fn load_memos(&mut self, dbt: Option<&[u8]>) {
        let cp = self.codepage;
        let mem: Vec<usize> = self.fields.iter().enumerate().filter(|(_, f)| f.kind == b'M').map(|(i, _)| i).collect();
        // общий предел: тысячи строк с одним указателем на большой блок не
        // должны раздуть память в тысячи раз
        let mut budget = MAX_DBT;
        for r in &mut self.rows {
            for &i in &mem {
                let block = r.cells[i].trim().parse::<usize>().unwrap_or(0);
                r.cells[i].clear();
                let Some(bytes) = dbt.and_then(|d| read_block(d, block)) else { continue };
                if bytes.len().saturating_mul(2) > budget {
                    continue;
                }
                budget -= bytes.len() * 2;
                r.cells[i] = cp.decode(&bytes);
                r.blobs.insert(i, bytes);
            }
        }
    }

    fn parse_raw(d: &[u8]) -> Result<Table, DbfError> {
        if d.len() < 33 {
            return err("файл короче заголовка dBASE");
        }
        let version = d[0];
        if !matches!(version & 0x07, 2..=5) && version != 0xf5 && version != 0x30 {
            return err("не похоже на файл dBASE");
        }
        let header_len = u16le(d, 8);
        let rec_len = u16le(d, 10);
        if header_len < 33 || header_len > d.len() || rec_len == 0 {
            return err("повреждён заголовок dBASE");
        }
        let codepage = match d[29] {
            0x65 | 0x26 | 0x64 => Codepage::Cp866,
            _ => Codepage::Cp1251,
        };
        let mut fields = Vec::new();
        let mut pos = 32;
        let mut total = 1usize;
        while pos + 32 <= header_len && d[pos] != 0x0d {
            if fields.len() >= MAX_FIELDS {
                return err("слишком много полей");
            }
            let raw = &d[pos..pos + 32];
            let nul = raw[..11].iter().position(|&b| b == 0).unwrap_or(11);
            let kind = raw[11].to_ascii_uppercase();
            if !matches!(kind, b'C' | b'N' | b'F' | b'D' | b'L' | b'M') {
                return err("неизвестный тип поля");
            }
            let f = Field { name: codepage.decode(&raw[..nul]), kind, len: raw[16], dec: raw[17] };
            total += width(kind, f.len);
            fields.push(f);
            pos += 32;
        }
        if fields.is_empty() {
            return err("в таблице нет полей");
        }
        if total > rec_len {
            return err("поля не умещаются в запись");
        }
        if fields.iter().any(|f| f.len == 0 && matches!(f.kind, b'C' | b'N' | b'F')) {
            return err("поле нулевой длины");
        }
        let declared = d.get(4..8).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])) as usize;
        let room = (d.len() - header_len) / rec_len;
        let count = declared.min(room);
        if count.saturating_mul(fields.len()) > MAX_CELLS {
            return err("таблица слишком велика");
        }
        let mut rows = Vec::new();
        for i in 0..count {
            let rec = &d[header_len + i * rec_len..header_len + (i + 1) * rec_len];
            let mut at = 1;
            let mut cells = Vec::with_capacity(fields.len());
            for f in &fields {
                let w = width(f.kind, f.len);
                let raw = &rec[at..at + w];
                at += w;
                let t = codepage.decode(raw);
                cells.push(t.trim_matches(|c| c == ' ' || c == '\0').to_string());
            }
            rows.push(Row { deleted: rec[0] == b'*', cells, blobs: Default::default() });
        }
        Ok(Table { version, codepage, fields, rows })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_files().0
    }

    /// Содержимое `.dbf` и, если есть MEMO-поля, `.dbt`.
    pub fn to_files(&self) -> (Vec<u8>, Option<Vec<u8>>) {
        let mut dbt: Option<Vec<u8>> = self.has_memo().then(|| vec![0u8; BLOCK]);
        let mut next = 1usize;
        let rec_len: usize = 1 + self.fields.iter().map(|f| width(f.kind, f.len)).sum::<usize>();
        let header_len = 32 + 32 * self.fields.len() + 1;
        let mut out = vec![0u8; 32];
        out[0] = if dbt.is_some() { 0x83 } else { 3 };
        out[1] = 126; // 2026
        out[2] = 1;
        out[3] = 1;
        out[4..8].copy_from_slice(&(self.rows.len() as u32).to_le_bytes());
        out[8..10].copy_from_slice(&(header_len.min(0xffff) as u16).to_le_bytes());
        out[10..12].copy_from_slice(&(rec_len.min(0xffff) as u16).to_le_bytes());
        out[29] = self.codepage.language_byte();
        for f in &self.fields {
            let mut d = [0u8; 32];
            let name = self.codepage.encode(&f.name);
            let n = name.len().min(10);
            d[..n].copy_from_slice(&name[..n]);
            d[11] = f.kind;
            d[16] = width(f.kind, f.len).min(255) as u8;
            d[17] = f.dec;
            out.extend_from_slice(&d);
        }
        out.push(0x0d);
        for r in &self.rows {
            out.push(if r.deleted { b'*' } else { b' ' });
            for (i, f) in self.fields.iter().enumerate() {
                let w = width(f.kind, f.len);
                let mut b = if f.kind == b'M' {
                    let data = match r.blobs.get(&i) {
                        Some(b) => b.clone(),
                        None => self.codepage.encode(r.cells.get(i).map_or("", |s| s.as_str())),
                    };
                    let data = &data[..data.len().min(MAX_MEMO)];
                    match (&mut dbt, data.is_empty()) {
                        (Some(file), false) => {
                            let at = next;
                            file.extend_from_slice(data);
                            file.extend_from_slice(&[0x1a, 0x1a]);
                            let blocks = (data.len() + 2).div_ceil(BLOCK);
                            file.resize((at + blocks) * BLOCK, 0);
                            next = at + blocks;
                            format!("{at:>10}").into_bytes()
                        }
                        _ => Vec::new(),
                    }
                } else {
                    self.codepage.encode(r.cells.get(i).map_or("", |s| s.as_str()))
                };
                b.truncate(w);
                let pad = w - b.len();
                if matches!(f.kind, b'N' | b'F') {
                    out.extend(std::iter::repeat_n(b' ', pad));
                    out.extend_from_slice(&b);
                } else {
                    out.extend_from_slice(&b);
                    out.extend(std::iter::repeat_n(b' ', pad));
                }
            }
        }
        out.push(0x1a);
        if let Some(file) = &mut dbt {
            file[..4].copy_from_slice(&(next as u32).to_le_bytes());
        }
        (out, dbt)
    }
}

/// Сравнение значений ячеек одного поля: числа - как числа (пустое меньше
/// всех), остальное - как текст, с учётом регистра или без.
pub fn compare_cells(kind: u8, a: &str, b: &str, icase: bool) -> std::cmp::Ordering {
    if matches!(kind, b'N' | b'F') {
        let (x, y) = (a.trim().parse::<f64>().ok(), b.trim().parse::<f64>().ok());
        return match (x, y) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            (x, y) => x.is_some().cmp(&y.is_some()),
        };
    }
    if icase { a.to_lowercase().cmp(&b.to_lowercase()) } else { a.cmp(b) }
}

/// Блок `n` файла `.dbt`: dBASE IV (заголовок FF FF 08 00 + длина) или III
/// (текст до 0x1A). Любая неувязка - `None`, а не паника.
fn read_block(d: &[u8], n: usize) -> Option<Vec<u8>> {
    if n == 0 || d.len() > MAX_DBT {
        return None;
    }
    let start = n.checked_mul(BLOCK)?;
    let tail = d.get(start..)?;
    if tail.len() >= 8 && tail[..4] == [0xff, 0xff, 0x08, 0x00] {
        let len = u32::from_le_bytes([tail[4], tail[5], tail[6], tail[7]]) as usize;
        let body = tail.get(8..)?;
        let len = len.saturating_sub(8).min(body.len()).min(MAX_MEMO);
        return Some(body[..len].to_vec());
    }
    let end = tail.iter().take(MAX_MEMO).position(|&b| b == 0x1a).unwrap_or(tail.len().min(MAX_MEMO));
    Some(tail[..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(cp: Codepage) -> Table {
        let f = |n: &str, k: u8, l: u8, d: u8| Field { name: n.into(), kind: k, len: l, dec: d };
        let mut t = Table::new(vec![f("NAME", b'C', 12, 0), f("AGE", b'N', 5, 0), f("SUM", b'F', 8, 2), f("BORN", b'D', 8, 0), f("OK", b'L', 1, 0)], cp);
        t.rows.push(Row { deleted: false, cells: vec!["Иванов".into(), "26".into(), "12.50".into(), "20000101".into(), "T".into()], ..Default::default() });
        t.rows.push(Row { deleted: true, cells: vec!["Пётр".into(), "7".into(), "".into(), "".into(), "F".into()], ..Default::default() });
        t
    }

    #[test]
    fn round_trip_both_codepages() {
        for cp in [Codepage::Cp866, Codepage::Cp1251] {
            let t = sample(cp);
            let back = Table::parse(&t.to_bytes()).unwrap();
            assert_eq!(back.codepage, cp);
            assert_eq!(back.fields, t.fields);
            assert_eq!(back.rows.len(), 2);
            assert_eq!(back.rows[0].cells[0], "Иванов");
            assert_eq!(back.rows[0].cells[2], "12.50");
            assert!(back.rows[1].deleted);
            assert_eq!(back.rows[1].cells[0], "Пётр");
        }
    }

    #[test]
    fn hostile_counts_do_not_allocate_or_panic() {
        let mut b = sample(Codepage::Cp1251).to_bytes();
        b[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Table::parse(&b).unwrap().rows.len(), 2);
        for n in 0..b.len() {
            let _ = Table::parse(&b[..n]);
        }
        b[10] = 0;
        b[11] = 0;
        assert!(Table::parse(&b).is_err());
    }

    #[test]
    fn many_tiny_fields_do_not_amplify_memory() {
        // 255 полей, запись из одного байта, 10 МБ записей: по ячейке на поле
        let mut b = vec![0u8; 32 + 255 * 32 + 1];
        b[0] = 3;
        b[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        let hl = b.len() as u16;
        b[8..10].copy_from_slice(&hl.to_le_bytes());
        b[10] = 256u16.to_le_bytes()[0];
        b[11] = 1;
        for i in 0..255 {
            let o = 32 + i * 32;
            b[o] = b'A';
            b[o + 11] = b'C';
            b[o + 16] = 1;
        }
        b[32 + 255 * 32] = 0x0d;
        b.resize(b.len() + 256 * 100_000, b' ');
        assert!(Table::parse(&b).is_err());
        // нулевая длина текстового поля
        b[32 + 16] = 0;
        assert!(Table::parse(&b[..hl as usize + 256]).is_err());
    }

    #[test]
    fn memo_round_trip_and_hostile_dbt() {
        let f = |n: &str, k: u8, l: u8| Field { name: n.into(), kind: k, len: l, dec: 0 };
        let mut t = Table::new(vec![f("ID", b'N', 3), f("NOTE", b'M', 10)], Codepage::Cp1251);
        t.rows.push(Row { deleted: false, cells: vec!["1".into(), "Привет".into()], ..Default::default() });
        t.rows.push(Row { deleted: false, cells: vec!["2".into(), "".into()], ..Default::default() });
        let mut big = Row { deleted: false, cells: vec!["3".into(), "".into()], ..Default::default() };
        big.blobs.insert(1, vec![0u8, 1, 2, 255, 7].repeat(300));
        t.rows.push(big);
        let (a, b) = t.to_files();
        let b = b.unwrap();
        let back = Table::parse_with_memo(&a, Some(&b)).unwrap();
        assert_eq!(back.rows[0].cells[1], "Привет");
        assert_eq!(back.rows[1].cells[1], "");
        assert_eq!(back.rows[2].blobs[&1].len(), 1500);
        // без .dbt память пуста, с мусорным - не паникует
        assert_eq!(Table::parse(&a).unwrap().rows[0].cells[1], "");
        for n in 0..b.len().min(2000) {
            let _ = Table::parse_with_memo(&a, Some(&b[..n]));
        }
        let junk = vec![0xffu8; 4096];
        let _ = Table::parse_with_memo(&a, Some(&junk));
    }

    #[test]
    fn memo_pointers_to_one_big_block_are_budgeted() {
        let f = |n: &str, k: u8, l: u8| Field { name: n.into(), kind: k, len: l, dec: 0 };
        let mut t = Table::new(vec![f("M", b'M', 10)], Codepage::Cp1251);
        for _ in 0..40 {
            t.rows.push(Row { deleted: false, cells: vec!["1".into()], ..Default::default() });
        }
        let (a, _) = t.to_files();
        let mut dbt = vec![0u8; 512];
        dbt.extend([0xff, 0xff, 0x08, 0x00]);
        dbt.extend((MAX_MEMO as u32).to_le_bytes());
        dbt.resize(512 + MAX_MEMO, b'x');
        let back = Table::parse_with_memo(&a, Some(&dbt)).unwrap();
        let total: usize = back.rows.iter().map(|r| r.blobs.values().map(|b| b.len()).sum::<usize>()).sum();
        assert!(total <= MAX_DBT);
    }
}
