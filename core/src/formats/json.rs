//! Маленький JSON — читатель и писатель без внешних крейтов. Нужен для
//! родного формата проекта и API; полноты стандарта хватает.

use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn str_or(&self, key: &str, default: &str) -> String {
        self.get(key).and_then(Json::as_str).unwrap_or(default).to_string()
    }

    pub fn num_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(Json::as_f64).unwrap_or(default)
    }

    /// Печать с отступами: файлы формата читают и правят люди, и они
    /// должны хорошо смотреться в git diff.
    pub fn pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    pub fn compact(&self) -> String {
        let mut out = String::new();
        self.write_compact(&mut out);
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        let pad = "  ".repeat(depth);
        match self {
            Json::Array(items) if items.is_empty() => out.push_str("[]"),
            Json::Array(items) if items.iter().all(|i| matches!(i, Json::Number(_) | Json::Bool(_) | Json::Null)) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    item.write_compact(out);
                }
                out.push(']');
            }
            Json::Array(items) => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    out.push_str(&pad);
                    out.push_str("  ");
                    item.write(out, depth + 1);
                    out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
                }
                out.push_str(&pad);
                out.push(']');
            }
            Json::Object(map) if map.is_empty() => out.push_str("{}"),
            Json::Object(map) => {
                out.push_str("{\n");
                for (i, (k, v)) in map.iter().enumerate() {
                    out.push_str(&pad);
                    out.push_str("  ");
                    write_string(out, k);
                    out.push_str(": ");
                    v.write(out, depth + 1);
                    out.push_str(if i + 1 < map.len() { ",\n" } else { "\n" });
                }
                out.push_str(&pad);
                out.push('}');
            }
            other => other.write_compact(out),
        }
    }

    fn write_compact(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Number(n) => {
                if n.is_finite() {
                    if *n == n.trunc() && n.abs() < 1e15 {
                        let _ = write!(out, "{}", *n as i64);
                    } else {
                        let _ = write!(out, "{n}");
                    }
                } else {
                    out.push_str("null");
                }
            }
            Json::Str(s) => write_string(out, s),
            Json::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write_compact(out);
                }
                out.push(']');
            }
            Json::Object(map) => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(out, k);
                    out.push(':');
                    v.write_compact(out);
                }
                out.push('}');
            }
        }
    }
}

pub fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Удобный конструктор объекта в порядке добавления ключей не сохраняется
/// (BTreeMap сортирует), поэтому ключи в файлах всегда упорядочены.
pub fn object(pairs: Vec<(&str, Json)>) -> Json {
    Json::Object(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

pub fn parse(text: &str) -> Result<Json, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut p = Parser { c: &chars, i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != chars.len() {
        return Err(p.err("лишние символы после значения"));
    }
    Ok(v)
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
}

impl Parser<'_> {
    fn err(&self, msg: &str) -> String {
        format!("JSON, символ {}: {msg}", self.i)
    }

    fn ws(&mut self) {
        while self.i < self.c.len() && self.c[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.c.get(self.i).copied()
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('n') => self.literal("null", Json::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err("ожидалось значение")),
        }
    }

    fn literal(&mut self, word: &str, v: Json) -> Result<Json, String> {
        let end = self.i + word.chars().count();
        if end <= self.c.len() && self.c[self.i..end].iter().collect::<String>() == word {
            self.i = end;
            Ok(v)
        } else {
            Err(self.err("неизвестное слово"))
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while self.i < self.c.len() && matches!(self.c[self.i], '-' | '+' | '.' | 'e' | 'E' | '0'..='9') {
            self.i += 1;
        }
        let s: String = self.c[start..self.i].iter().collect();
        s.parse::<f64>().map(Json::Number).map_err(|_| self.err("неверное число"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.i += 1; // "
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else { return Err(self.err("строка не закрыта")) };
            self.i += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let Some(e) = self.peek() else { return Err(self.err("обрыв после \\")) };
                    self.i += 1;
                    match e {
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'u' => {
                            let hex: String = self.c.get(self.i..self.i + 4).ok_or_else(|| self.err("обрыв в \\u"))?.iter().collect();
                            self.i += 4;
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| self.err("плохой \\u"))?;
                            // суррогатные пары
                            if (0xD800..0xDC00).contains(&code) && self.c.get(self.i) == Some(&'\\') && self.c.get(self.i + 1) == Some(&'u') {
                                let hex2: String = self.c.get(self.i + 2..self.i + 6).ok_or_else(|| self.err("обрыв в \\u"))?.iter().collect();
                                if let Ok(low) = u32::from_str_radix(&hex2, 16) {
                                    self.i += 6;
                                    let full = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                    out.push(char::from_u32(full).unwrap_or('\u{fffd}'));
                                    continue;
                                }
                            }
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        other => out.push(other),
                    }
                }
                c => out.push(c),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut items = Vec::new();
        loop {
            self.ws();
            if self.peek() == Some(']') {
                self.i += 1;
                return Ok(Json::Array(items));
            }
            items.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {}
                _ => return Err(self.err("ожидалась , или ]")),
            }
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.i += 1;
        let mut map = BTreeMap::new();
        loop {
            self.ws();
            if self.peek() == Some('}') {
                self.i += 1;
                return Ok(Json::Object(map));
            }
            if self.peek() != Some('"') {
                return Err(self.err("ожидался ключ"));
            }
            let key = self.string()?;
            self.ws();
            if self.peek() != Some(':') {
                return Err(self.err("ожидалось :"));
            }
            self.i += 1;
            self.ws();
            let v = self.value()?;
            map.insert(key, v);
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {}
                _ => return Err(self.err("ожидалась , или }")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let v = object(vec![
            ("name", Json::Str("планета \"x\"\n".into())),
            ("n", Json::Number(2.5)),
            ("list", Json::Array(vec![Json::Number(1.0), Json::Bool(true), Json::Null])),
        ]);
        let text = v.pretty();
        assert_eq!(parse(&text).unwrap(), v);
        assert_eq!(parse(&v.compact()).unwrap(), v);
    }

    #[test]
    fn unicode_escapes() {
        assert_eq!(parse(r#""Ж😀""#).unwrap(), Json::Str("Ж😀".into()));
    }
}
