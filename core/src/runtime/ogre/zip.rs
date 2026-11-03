//! Чтение zip-архивов ресурсов (stored и deflate) с защитой от zip-бомб.

use super::image::inflate_raw;
use std::collections::HashMap;

const MAX_ENTRIES: usize = 20_000;
pub const MAX_ENTRY: usize = 64 << 20;
pub const MAX_ARCHIVE: u64 = 256 << 20;

#[derive(Debug, Clone)]
struct Entry {
    method: u16,
    csize: usize,
    usize_: usize,
    offset: usize,
}

#[derive(Debug, Default)]
pub struct Zip {
    data: Vec<u8>,
    /// Ключ - путь внутри архива в нижнем регистре с "/".
    entries: HashMap<String, Entry>,
    /// Имя файла без папок -> ключ записи.
    by_name: HashMap<String, String>,
}

fn le16(d: &[u8], i: usize) -> Option<usize> {
    Some(u16::from_le_bytes([*d.get(i)?, *d.get(i + 1)?]) as usize)
}
fn le32(d: &[u8], i: usize) -> Option<usize> {
    Some(u32::from_le_bytes([*d.get(i)?, *d.get(i + 1)?, *d.get(i + 2)?, *d.get(i + 3)?]) as usize)
}

impl Zip {
    pub fn open(data: Vec<u8>) -> Option<Zip> {
        let n = data.len();
        if n < 22 {
            return None;
        }
        // конец центрального каталога ищем с хвоста (комментарий до 64 КБ)
        let lo = n.saturating_sub(22 + 65_535);
        let eocd = (lo..=n - 22).rev().find(|&i| data[i..i + 4] == [0x50, 0x4B, 0x05, 0x06])?;
        let count = le16(&data, eocd + 10)?;
        let cd_off = le32(&data, eocd + 16)?;
        if count > MAX_ENTRIES || cd_off >= n {
            return None;
        }
        let mut z = Zip { data, ..Default::default() };
        let mut p = cd_off;
        for _ in 0..count {
            if z.data.get(p..p + 4)? != [0x50, 0x4B, 0x01, 0x02] {
                return None;
            }
            let method = le16(&z.data, p + 10)? as u16;
            let csize = le32(&z.data, p + 20)?;
            let usize_ = le32(&z.data, p + 24)?;
            let (nl, el, cl) = (le16(&z.data, p + 28)?, le16(&z.data, p + 30)?, le16(&z.data, p + 32)?);
            let offset = le32(&z.data, p + 42)?;
            let name = String::from_utf8_lossy(z.data.get(p + 46..p + 46 + nl)?).replace('\\', "/").to_ascii_lowercase();
            p += 46 + nl + el + cl;
            if name.ends_with('/') || name.is_empty() || name.split('/').any(|s| s == "..") || name.starts_with('/') {
                continue;
            }
            if let Some(base) = name.rsplit('/').next() {
                z.by_name.entry(base.to_string()).or_insert_with(|| name.clone());
            }
            z.entries.insert(name, Entry { method, csize, usize_, offset });
        }
        Some(z)
    }

    pub fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.entries.keys().cloned().collect();
        v.sort();
        v
    }

    /// Заявленный размер распакованной записи (по каталогу).
    pub fn size(&self, name: &str) -> Option<usize> {
        self.entries.get(name).map(|e| e.usize_)
    }

    /// Файл по пути внутри архива либо по одному имени.
    pub fn read(&self, name: &str) -> Option<Vec<u8>> {
        let key = name.replace('\\', "/").to_ascii_lowercase();
        let key = key.trim_start_matches("./");
        let e = self.entries.get(key).or_else(|| self.by_name.get(key.rsplit('/').next()?).and_then(|k| self.entries.get(k)))?;
        if e.usize_ > MAX_ENTRY || e.csize > self.data.len() {
            return None;
        }
        if le32(&self.data, e.offset)? != 0x04034B50 {
            return None;
        }
        let (nl, el) = (le16(&self.data, e.offset + 26)?, le16(&self.data, e.offset + 28)?);
        let start = e.offset + 30 + nl + el;
        let raw = self.data.get(start..start.checked_add(e.csize)?)?;
        match e.method {
            0 if e.csize == e.usize_ => Some(raw.to_vec()),
            8 => {
                let out = inflate_raw(raw, e.usize_)?;
                (out.len() == e.usize_).then_some(out)
            }
            _ => None,
        }
    }
}
