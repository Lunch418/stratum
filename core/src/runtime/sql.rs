//! Малый локальный SQL поверх `.dbf` для `DbSQL`: CREATE/ALTER/DROP TABLE,
//! INSERT, UPDATE, DELETE и SELECT (WHERE, ORDER BY, DISTINCT, агрегаты
//! COUNT/SUM/AVG/MIN/MAX без GROUP BY). Запрос ограничен по длине, числу
//! лексем и глубине выражения; ничего, кроме таблиц песочницы, не трогает.

use super::builtins::Effects;
use crate::formats::dbf::{Codepage, Field, Row, Table, MAX_CELLS, MAX_FIELDS};
use std::path::PathBuf;

const MAX_SQL: usize = 64 * 1024;
const MAX_TOKENS: usize = 20_000;
const MAX_DEPTH: usize = 48;

/// Результат запроса: таблица и её путь на диске (пустой - курсор в памяти).
pub struct Done {
    pub table: Table,
    pub path: PathBuf,
}

/// Ошибка разбора или выполнения - код для `DbGetError`.
pub const E_SYNTAX: u32 = 8;
const E_FILE: u32 = 1;
const E_FIELD: u32 = 4;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    /// Идентификатор в двойных кавычках.
    Quoted(String),
    Str(String),
    Num(f64),
    Sym(&'static str),
}

fn lex(s: &str) -> Option<Vec<Tok>> {
    if s.len() > MAX_SQL {
        return None;
    }
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < c.len() {
        let ch = c[i];
        if out.len() > MAX_TOKENS {
            return None;
        }
        if ch.is_whitespace() || ch == ';' {
            i += 1;
        } else if ch == '-' && c.get(i + 1) == Some(&'-') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '\'' || ch == '"' {
            let mut j = i + 1;
            let mut t = String::new();
            loop {
                match c.get(j) {
                    None => return None,
                    Some(&q) if q == ch => {
                        if c.get(j + 1) == Some(&ch) {
                            t.push(ch);
                            j += 2;
                        } else {
                            break;
                        }
                    }
                    Some(&x) => {
                        t.push(x);
                        j += 1;
                    }
                }
            }
            out.push(if ch == '\'' { Tok::Str(t) } else { Tok::Quoted(t) });
            i = j + 1;
        } else if ch.is_ascii_digit() || (ch == '.' && c.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let mut j = i;
            while j < c.len() && (c[j].is_ascii_digit() || c[j] == '.') {
                j += 1;
            }
            out.push(Tok::Num(c[i..j].iter().collect::<String>().parse().ok()?));
            i = j;
        } else if ch.is_alphabetic() || ch == '_' {
            let mut j = i;
            while j < c.len() && (c[j].is_alphanumeric() || c[j] == '_' || c[j] == '.') {
                j += 1;
            }
            out.push(Tok::Word(c[i..j].iter().collect()));
            i = j;
        } else {
            let two: String = c[i..(i + 2).min(c.len())].iter().collect();
            let sym = match two.as_str() {
                "<>" => Some("<>"),
                "!=" => Some("<>"),
                "<=" => Some("<="),
                ">=" => Some(">="),
                _ => None,
            };
            if let Some(s) = sym {
                out.push(Tok::Sym(s));
                i += 2;
                continue;
            }
            out.push(Tok::Sym(match ch {
                '(' => "(",
                ')' => ")",
                ',' => ",",
                '=' => "=",
                '<' => "<",
                '>' => ">",
                '+' => "+",
                '-' => "-",
                '*' => "*",
                '/' => "/",
                '%' => "%",
                _ => return None,
            }));
            i += 1;
        }
    }
    Some(out)
}

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Str(String),
    Null,
    Col(String),
    Neg(Box<Expr>),
    Bin(Box<Expr>, &'static str, Box<Expr>),
    Not(Box<Expr>),
    IsNull(Box<Expr>, bool),
    Like(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Val {
    Num(f64),
    Str(String),
    Null,
}

struct P {
    t: Vec<Tok>,
    i: usize,
    depth: usize,
}

type R<T> = Result<T, u32>;

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn kw(&self, w: &str) -> bool {
        matches!(self.peek(), Some(Tok::Word(x)) if x.eq_ignore_ascii_case(w))
    }
    fn eat_kw(&mut self, w: &str) -> bool {
        let h = self.kw(w);
        if h {
            self.i += 1;
        }
        h
    }
    fn need_kw(&mut self, w: &str) -> R<()> {
        if self.eat_kw(w) { Ok(()) } else { Err(E_SYNTAX) }
    }
    fn sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }
    fn eat_sym(&mut self, s: &str) -> bool {
        let h = self.sym(s);
        if h {
            self.i += 1;
        }
        h
    }
    fn need_sym(&mut self, s: &str) -> R<()> {
        if self.eat_sym(s) { Ok(()) } else { Err(E_SYNTAX) }
    }
    fn ident(&mut self) -> R<String> {
        match self.peek().cloned() {
            Some(Tok::Word(w)) | Some(Tok::Quoted(w)) | Some(Tok::Str(w)) => {
                self.i += 1;
                Ok(w)
            }
            _ => Err(E_SYNTAX),
        }
    }
    fn end(&self) -> R<()> {
        if self.i >= self.t.len() { Ok(()) } else { Err(E_SYNTAX) }
    }

    fn expr(&mut self) -> R<Expr> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(E_SYNTAX);
        }
        let mut l = self.and()?;
        while self.eat_kw("or") {
            l = Expr::Bin(Box::new(l), "or", Box::new(self.and()?));
        }
        self.depth -= 1;
        Ok(l)
    }
    fn and(&mut self) -> R<Expr> {
        let mut l = self.not()?;
        while self.eat_kw("and") {
            l = Expr::Bin(Box::new(l), "and", Box::new(self.not()?));
        }
        Ok(l)
    }
    fn not(&mut self) -> R<Expr> {
        if self.eat_kw("not") {
            self.depth += 1;
            if self.depth > MAX_DEPTH {
                return Err(E_SYNTAX);
            }
            let e = self.not()?;
            self.depth -= 1;
            return Ok(Expr::Not(Box::new(e)));
        }
        self.cmp()
    }
    fn cmp(&mut self) -> R<Expr> {
        let l = self.add()?;
        for op in ["=", "<>", "<=", ">=", "<", ">"] {
            if self.eat_sym(op) {
                return Ok(Expr::Bin(Box::new(l), op, Box::new(self.add()?)));
            }
        }
        if self.eat_kw("is") {
            let neg = self.eat_kw("not");
            self.need_kw("null")?;
            return Ok(Expr::IsNull(Box::new(l), neg));
        }
        let neg = self.kw("not") && matches!(self.t.get(self.i + 1), Some(Tok::Word(w)) if w.eq_ignore_ascii_case("like") || w.eq_ignore_ascii_case("in"));
        if neg {
            self.i += 1;
        }
        let e = if self.eat_kw("like") {
            Expr::Like(Box::new(l), Box::new(self.add()?))
        } else if self.eat_kw("in") {
            self.need_sym("(")?;
            let mut v = vec![self.add()?];
            while self.eat_sym(",") {
                v.push(self.add()?);
            }
            self.need_sym(")")?;
            Expr::In(Box::new(l), v)
        } else {
            return Ok(l);
        };
        Ok(if neg { Expr::Not(Box::new(e)) } else { e })
    }
    fn add(&mut self) -> R<Expr> {
        let mut l = self.mul()?;
        loop {
            let op = if self.eat_sym("+") {
                "+"
            } else if self.eat_sym("-") {
                "-"
            } else {
                return Ok(l);
            };
            l = Expr::Bin(Box::new(l), op, Box::new(self.mul()?));
        }
    }
    fn mul(&mut self) -> R<Expr> {
        let mut l = self.atom()?;
        loop {
            let op = if self.eat_sym("*") {
                "*"
            } else if self.eat_sym("/") {
                "/"
            } else {
                return Ok(l);
            };
            l = Expr::Bin(Box::new(l), op, Box::new(self.atom()?));
        }
    }
    fn atom(&mut self) -> R<Expr> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(E_SYNTAX);
        }
        let r = match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.i += 1;
                Expr::Num(n)
            }
            Some(Tok::Str(s)) => {
                self.i += 1;
                Expr::Str(s)
            }
            Some(Tok::Quoted(s)) => {
                self.i += 1;
                Expr::Col(s)
            }
            Some(Tok::Sym("-")) => {
                self.i += 1;
                Expr::Neg(Box::new(self.atom()?))
            }
            Some(Tok::Sym("(")) => {
                self.i += 1;
                let e = self.expr()?;
                self.need_sym(")")?;
                e
            }
            Some(Tok::Word(w)) => {
                self.i += 1;
                match w.to_ascii_lowercase().as_str() {
                    "null" => Expr::Null,
                    "true" => Expr::Num(1.0),
                    "false" => Expr::Num(0.0),
                    _ => Expr::Col(w),
                }
            }
            _ => return Err(E_SYNTAX),
        };
        self.depth -= 1;
        Ok(r)
    }
}

fn strip_qual(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn cell_val(f: &Field, c: &str) -> Val {
    match f.kind {
        b'N' | b'F' => c.trim().parse().map(Val::Num).unwrap_or(Val::Null),
        b'L' => match c.trim().to_ascii_uppercase().as_str() {
            "T" | "Y" => Val::Num(1.0),
            "F" | "N" => Val::Num(0.0),
            _ => Val::Null,
        },
        b'D' if c.trim().is_empty() => Val::Null,
        _ => Val::Str(c.to_string()),
    }
}

fn like(t: &[char], p: &[char]) -> bool {
    match p.first() {
        None => t.is_empty(),
        Some('%') => (0..=t.len()).any(|k| like(&t[k..], &p[1..])),
        Some('_') => !t.is_empty() && like(&t[1..], &p[1..]),
        Some(&c) => t.first().is_some_and(|x| x.to_lowercase().eq(c.to_lowercase())) && like(&t[1..], &p[1..]),
    }
}

fn truthy(v: &Val) -> bool {
    matches!(v, Val::Num(n) if *n != 0.0)
}

fn eval(e: &Expr, t: &Table, r: &Row) -> R<Val> {
    Ok(match e {
        Expr::Num(n) => Val::Num(*n),
        Expr::Str(s) => Val::Str(s.clone()),
        Expr::Null => Val::Null,
        Expr::Col(n) => {
            let i = t.field_index(strip_qual(n)).ok_or(E_FIELD)?;
            cell_val(&t.fields[i], &r.cells[i])
        }
        Expr::Neg(x) => match eval(x, t, r)? {
            Val::Num(n) => Val::Num(-n),
            _ => Val::Null,
        },
        Expr::Not(x) => Val::Num(!truthy(&eval(x, t, r)?) as u8 as f64),
        Expr::IsNull(x, neg) => Val::Num(((eval(x, t, r)? == Val::Null) != *neg) as u8 as f64),
        Expr::Like(a, b) => match (eval(a, t, r)?, eval(b, t, r)?) {
            (Val::Str(a), Val::Str(b)) => Val::Num(like(&a.trim_end().chars().collect::<Vec<_>>(), &b.chars().collect::<Vec<_>>()) as u8 as f64),
            _ => Val::Num(0.0),
        },
        Expr::In(a, list) => {
            let a = eval(a, t, r)?;
            let mut hit = false;
            for x in list {
                hit |= a != Val::Null && cmp(&a, &eval(x, t, r)?) == Some(std::cmp::Ordering::Equal);
            }
            Val::Num(hit as u8 as f64)
        }
        Expr::Bin(a, op, b) => {
            let (x, y) = (eval(a, t, r)?, eval(b, t, r)?);
            match *op {
                "and" => Val::Num((truthy(&x) && truthy(&y)) as u8 as f64),
                "or" => Val::Num((truthy(&x) || truthy(&y)) as u8 as f64),
                "+" | "-" | "*" | "/" => match (&x, &y) {
                    (Val::Num(p), Val::Num(q)) => Val::Num(match *op {
                        "+" => p + q,
                        "-" => p - q,
                        "*" => p * q,
                        _ if *q == 0.0 => return Ok(Val::Null),
                        _ => p / q,
                    }),
                    (Val::Str(p), Val::Str(q)) if *op == "+" => Val::Str(format!("{p}{q}")),
                    _ => Val::Null,
                },
                _ => {
                    let Some(o) = cmp(&x, &y) else { return Ok(Val::Num(0.0)) };
                    use std::cmp::Ordering::*;
                    Val::Num(match *op {
                        "=" => o == Equal,
                        "<>" => o != Equal,
                        "<" => o == Less,
                        "<=" => o != Greater,
                        ">" => o == Greater,
                        _ => o != Less,
                    } as u8 as f64)
                }
            }
        }
    })
}

fn cmp(a: &Val, b: &Val) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Val::Num(x), Val::Num(y)) => x.partial_cmp(y),
        (Val::Str(x), Val::Str(y)) => Some(x.trim_end().cmp(y.trim_end())),
        // число против текста (поле C и литерал-число, дата 20240101 и т.п.)
        (Val::Num(x), Val::Str(y)) => y.trim().parse::<f64>().ok().and_then(|y| x.partial_cmp(&y)),
        (Val::Str(x), Val::Num(y)) => x.trim().parse::<f64>().ok().and_then(|x| x.partial_cmp(y)),
        _ => None,
    }
}

fn to_text(v: &Val, f: &Field) -> String {
    match v {
        Val::Null => String::new(),
        Val::Str(s) => s.clone(),
        Val::Num(n) if f.kind == b'L' => (if *n != 0.0 { "T" } else { "F" }).into(),
        Val::Num(n) if f.dec > 0 && matches!(f.kind, b'N' | b'F') => format!("{n:.*}", f.dec as usize),
        Val::Num(n) if matches!(f.kind, b'N' | b'F') => format!("{}", n.round()),
        Val::Num(n) => super::value::format_g(*n),
    }
}

/// Тип столбца из `CHAR(20)`, `NUMERIC(10,4)`, `BOOLEAN`...
fn col_def(p: &mut P) -> R<Field> {
    let name = p.ident()?;
    if name.len() > 10 || name.is_empty() {
        return Err(E_SYNTAX);
    }
    let ty = p.ident()?.to_ascii_lowercase();
    let mut args = Vec::new();
    if p.eat_sym("(") {
        loop {
            match p.peek() {
                Some(Tok::Num(n)) if *n >= 0.0 && *n <= 255.0 => args.push(*n as u8),
                _ => return Err(E_SYNTAX),
            }
            p.i += 1;
            if !p.eat_sym(",") {
                break;
            }
        }
        p.need_sym(")")?;
    }
    let a0 = args.first().copied();
    let a1 = args.get(1).copied().unwrap_or(0);
    let (kind, len, dec) = match ty.as_str() {
        "char" | "character" | "varchar" | "string" | "c" => (b'C', a0.unwrap_or(1), 0),
        "numeric" | "decimal" | "number" | "n" => (b'N', a0.unwrap_or(10), a1),
        "integer" | "int" | "smallint" | "n_int" => (b'N', 10, 0),
        "float" | "real" | "double" | "money" | "f" => (b'F', a0.unwrap_or(20), if args.len() > 1 { a1 } else { 4 }),
        "date" | "d" => (b'D', 8, 0),
        "boolean" | "logical" | "bool" | "l" => (b'L', 1, 0),
        "blob" | "memo" | "m" => (b'M', 10, 0),
        _ => return Err(E_SYNTAX),
    };
    if len == 0 || dec > len {
        return Err(E_SYNTAX);
    }
    Ok(Field { name, kind, len, dec })
}

fn table_name(p: &mut P) -> R<String> {
    let mut n = p.ident()?;
    if !n.contains('.') {
        n.push_str(".dbf");
    }
    Ok(n)
}

pub fn check_fields(t: &Table) -> R<()> {
    if t.fields.len() > MAX_FIELDS || t.fields.iter().enumerate().any(|(i, f)| t.fields[..i].iter().any(|g| g.name.eq_ignore_ascii_case(&f.name))) {
        return Err(E_SYNTAX);
    }
    Ok(())
}

/// Строки по условию WHERE (индексы в `t.rows`).
fn matching(p: &mut P, t: &Table) -> R<Vec<usize>> {
    let cond = if p.eat_kw("where") { Some(p.expr()?) } else { None };
    p.end()?;
    let mut out = Vec::new();
    for (i, r) in t.rows.iter().enumerate() {
        if r.deleted {
            continue;
        }
        if match &cond {
            Some(c) => truthy(&eval(c, t, r)?),
            None => true,
        } {
            out.push(i);
        }
    }
    Ok(out)
}

pub fn run(fx: &mut Effects, sql: &str) -> R<Done> {
    let mut p = P { t: lex(sql).ok_or(E_SYNTAX)?, i: 0, depth: 0 };
    if p.eat_kw("create") {
        p.need_kw("table")?;
        let name = table_name(&mut p)?;
        p.need_sym("(")?;
        let mut fields = vec![col_def(&mut p)?];
        while p.eat_sym(",") {
            fields.push(col_def(&mut p)?);
        }
        p.need_sym(")")?;
        p.end()?;
        let t = Table::new(fields, Codepage::Cp866);
        check_fields(&t)?;
        let path = super::db::sandbox_path(fx, &name, true).ok_or(E_FILE)?;
        if path.exists() {
            return Err(E_FILE);
        }
        super::db::save(&path, &t).then_some(()).ok_or(E_FILE)?;
        return Ok(Done { table: t, path });
    }
    if p.eat_kw("drop") {
        p.need_kw("table")?;
        let name = table_name(&mut p)?;
        p.end()?;
        let path = super::db::sandbox_path(fx, &name, true).ok_or(E_FILE)?;
        std::fs::remove_file(&path).map_err(|_| E_FILE)?;
        let _ = std::fs::remove_file(path.with_extension("dbt"));
        return Ok(Done { table: Table::new(vec![], Codepage::Cp866), path: PathBuf::new() });
    }
    if p.eat_kw("alter") {
        p.need_kw("table")?;
        let name = table_name(&mut p)?;
        let (mut t, path) = super::db::load(fx, &name, true)?;
        loop {
            if p.eat_kw("add") {
                p.eat_kw("column");
                let f = col_def(&mut p)?;
                for r in &mut t.rows {
                    r.cells.push(String::new());
                }
                t.fields.push(f);
            } else if p.eat_kw("drop") {
                p.eat_kw("column");
                let n = p.ident()?;
                let i = t.field_index(strip_qual(&n)).ok_or(E_FIELD)?;
                t.fields.remove(i);
                for r in &mut t.rows {
                    r.cells.remove(i);
                    r.blobs = std::mem::take(&mut r.blobs).into_iter().filter(|(k, _)| *k != i).map(|(k, v)| (if k > i { k - 1 } else { k }, v)).collect();
                }
            } else {
                return Err(E_SYNTAX);
            }
            if !p.eat_sym(",") {
                break;
            }
        }
        p.end()?;
        if t.fields.is_empty() {
            return Err(E_SYNTAX);
        }
        check_fields(&t)?;
        if t.rows.len().saturating_mul(t.fields.len()) > MAX_CELLS {
            return Err(E_SYNTAX);
        }
        super::db::save(&path, &t).then_some(()).ok_or(E_FILE)?;
        return Ok(Done { table: t, path });
    }
    if p.eat_kw("insert") {
        p.need_kw("into")?;
        let name = table_name(&mut p)?;
        let (mut t, path) = super::db::load(fx, &name, true)?;
        let mut cols = Vec::new();
        if p.eat_sym("(") {
            loop {
                let c = p.ident()?;
                cols.push(t.field_index(strip_qual(&c)).ok_or(E_FIELD)?);
                if !p.eat_sym(",") {
                    break;
                }
            }
            p.need_sym(")")?;
        } else {
            cols = (0..t.fields.len()).collect();
        }
        p.need_kw("values")?;
        p.need_sym("(")?;
        let mut vals = Vec::new();
        loop {
            vals.push(p.expr()?);
            if !p.eat_sym(",") {
                break;
            }
        }
        p.need_sym(")")?;
        p.end()?;
        if vals.len() != cols.len() || (t.rows.len() + 1).saturating_mul(t.fields.len()) > MAX_CELLS {
            return Err(E_SYNTAX);
        }
        let mut row = t.blank_row();
        for (c, v) in cols.iter().zip(&vals) {
            row.cells[*c] = to_text(&eval(v, &t, &row)?, &t.fields[*c]);
        }
        t.rows.push(row);
        super::db::save(&path, &t).then_some(()).ok_or(E_FILE)?;
        return Ok(Done { table: t, path });
    }
    if p.eat_kw("update") {
        let name = table_name(&mut p)?;
        let (mut t, path) = super::db::load(fx, &name, true)?;
        p.need_kw("set")?;
        let mut sets = Vec::new();
        loop {
            let c = p.ident()?;
            p.need_sym("=")?;
            sets.push((t.field_index(strip_qual(&c)).ok_or(E_FIELD)?, p.expr()?));
            if !p.eat_sym(",") {
                break;
            }
        }
        let hits = matching(&mut p, &t)?;
        for i in hits {
            // значения считаются по записи до изменения
            let vals: Vec<String> = sets.iter().map(|(c, e)| eval(e, &t, &t.rows[i]).map(|v| to_text(&v, &t.fields[*c]))).collect::<R<_>>()?;
            for ((c, _), v) in sets.iter().zip(vals) {
                t.rows[i].cells[*c] = v;
                t.rows[i].blobs.remove(c);
            }
        }
        super::db::save(&path, &t).then_some(()).ok_or(E_FILE)?;
        return Ok(Done { table: t, path });
    }
    if p.eat_kw("delete") {
        p.need_kw("from")?;
        let name = table_name(&mut p)?;
        let (mut t, path) = super::db::load(fx, &name, true)?;
        for i in matching(&mut p, &t)? {
            t.rows[i].deleted = true;
        }
        super::db::save(&path, &t).then_some(()).ok_or(E_FILE)?;
        return Ok(Done { table: t, path });
    }
    p.need_kw("select")?;
    select(fx, &mut p)
}

enum Item {
    Star,
    Col(String, Option<String>),
    Agg(String, Option<String>),
}

fn select(fx: &mut Effects, p: &mut P) -> R<Done> {
    let distinct = p.eat_kw("distinct");
    let mut items = Vec::new();
    loop {
        if p.eat_sym("*") {
            items.push(Item::Star);
        } else {
            let w = p.ident()?;
            if p.sym("(") && ["count", "sum", "avg", "min", "max"].contains(&w.to_ascii_lowercase().as_str()) {
                p.i += 1;
                let arg = if p.eat_sym("*") { None } else { Some(p.ident()?) };
                p.need_sym(")")?;
                items.push(Item::Agg(w.to_ascii_lowercase(), arg));
            } else {
                let alias = if p.eat_kw("as") { Some(p.ident()?) } else { None };
                items.push(Item::Col(w, alias));
            }
        }
        if !p.eat_sym(",") {
            break;
        }
    }
    p.need_kw("from")?;
    let name = table_name(p)?;
    // псевдоним таблицы: игнорируем, имена полей берутся без префикса
    if !p.kw("where") && !p.kw("order") && p.peek().is_some() {
        p.ident()?;
    }
    let (t, _) = super::db::load(fx, &name, false)?;
    let cond = if p.eat_kw("where") { Some(p.expr()?) } else { None };
    let mut order: Vec<(usize, bool)> = Vec::new();
    if p.eat_kw("order") {
        p.need_kw("by")?;
        loop {
            let c = p.ident()?;
            let i = match c.parse::<usize>() {
                Ok(n) => n.checked_sub(1).filter(|n| *n < t.fields.len()).ok_or(E_FIELD)?,
                Err(_) => t.field_index(strip_qual(&c)).ok_or(E_FIELD)?,
            };
            let desc = if p.eat_kw("desc") {
                true
            } else {
                p.eat_kw("asc");
                false
            };
            order.push((i, desc));
            if !p.eat_sym(",") {
                break;
            }
        }
    }
    p.end()?;
    let mut rows: Vec<&Row> = Vec::new();
    for r in &t.rows {
        if r.deleted {
            continue;
        }
        if match &cond {
            Some(c) => truthy(&eval(c, &t, r)?),
            None => true,
        } {
            rows.push(r);
        }
    }
    if !order.is_empty() {
        rows.sort_by(|a, b| {
            for &(i, desc) in &order {
                let o = crate::formats::dbf::compare_cells(t.fields[i].kind, &a.cells[i], &b.cells[i], false);
                let o = if desc { o.reverse() } else { o };
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
            }
            std::cmp::Ordering::Equal
        });
    }
    let cp = t.codepage;
    // агрегаты: одна строка итога
    if items.iter().any(|i| matches!(i, Item::Agg(..))) {
        if items.iter().any(|i| !matches!(i, Item::Agg(..))) {
            return Err(E_SYNTAX);
        }
        let mut fields = Vec::new();
        let mut cells = Vec::new();
        for it in &items {
            let Item::Agg(f, arg) = it else { unreachable!() };
            let col = match arg {
                Some(a) => Some(t.field_index(strip_qual(a)).ok_or(E_FIELD)?),
                None => None,
            };
            let nums: Vec<f64> = rows.iter().filter_map(|r| col.and_then(|c| r.cells[c].trim().parse().ok())).collect();
            let present = rows.iter().filter(|r| col.is_none_or(|c| !r.cells[c].trim().is_empty())).count();
            let v = match f.as_str() {
                "count" => present as f64,
                "sum" => nums.iter().sum(),
                "avg" if nums.is_empty() => 0.0,
                "avg" => nums.iter().sum::<f64>() / nums.len() as f64,
                "min" => nums.iter().cloned().fold(f64::INFINITY, f64::min),
                _ => nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            };
            let v = if v.is_finite() { v } else { 0.0 };
            fields.push(Field { name: f.to_uppercase(), kind: b'N', len: 20, dec: if f == "count" { 0 } else { 4 } });
            cells.push(if f == "count" { format!("{v}") } else { super::value::format_g(v) });
        }
        let mut out = Table::new(fields, cp);
        out.rows.push(Row { deleted: false, cells, ..Default::default() });
        return Ok(Done { table: out, path: PathBuf::new() });
    }
    let mut cols: Vec<(usize, String)> = Vec::new();
    for it in &items {
        match it {
            Item::Star => cols.extend(t.fields.iter().enumerate().map(|(i, f)| (i, f.name.clone()))),
            Item::Col(n, alias) => {
                let i = t.field_index(strip_qual(n)).ok_or(E_FIELD)?;
                cols.push((i, alias.clone().unwrap_or_else(|| t.fields[i].name.clone())));
            }
            Item::Agg(..) => unreachable!(),
        }
    }
    if cols.len() > MAX_FIELDS || rows.len().saturating_mul(cols.len().max(1)) > MAX_CELLS {
        return Err(E_SYNTAX);
    }
    let fields = cols.iter().map(|(i, n)| Field { name: n.chars().take(10).collect(), ..t.fields[*i].clone() }).collect();
    let mut out = Table::new(fields, cp);
    for r in rows {
        let cells: Vec<String> = cols.iter().map(|(i, _)| r.cells[*i].clone()).collect();
        if distinct && out.rows.iter().any(|o| o.cells == cells) {
            continue;
        }
        let blobs = cols.iter().enumerate().filter_map(|(k, (i, _))| r.blobs.get(i).map(|b| (k, b.clone()))).collect();
        out.rows.push(Row { deleted: false, cells, blobs });
    }
    Ok(Done { table: out, path: PathBuf::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx() -> (Effects, PathBuf) {
        let dir = std::env::temp_dir().join(format!("stratum-sql-{}-{}", std::process::id(), std::thread::current().name().unwrap_or("t").replace("::", "_")));
        std::fs::create_dir_all(&dir).unwrap();
        let mut fx = Effects::default();
        fx.gfx.project_dir = dir.clone();
        (fx, dir)
    }

    #[test]
    fn ddl_dml_and_select() {
        let (mut fx, dir) = fx();
        run(&mut fx, "CREATE TABLE \"tea.dbf\" (Number NUMERIC(3,0), Title CHAR(20), Price NUMERIC(10,2), Ok BOOLEAN)").unwrap();
        for (n, t, pr) in [(1, "Чёрный", 10.5), (2, "Зелёный", 7.25), (3, "Улун", 30.0), (4, "Чёрный", 1.0)] {
            run(&mut fx, &format!("INSERT INTO tea (Number, Title, Price, Ok) VALUES ({n}, '{t}', {pr}, TRUE)")).unwrap();
        }
        run(&mut fx, "ALTER TABLE tea ADD Note CHAR(5)").unwrap();
        let r = run(&mut fx, "SELECT Title, Price FROM tea WHERE Price > 5 AND Title <> 'Улун' ORDER BY Price DESC").unwrap().table;
        let got: Vec<_> = r.rows.iter().map(|x| x.cells.clone()).collect();
        assert_eq!(got, vec![vec!["Чёрный".to_string(), "10.50".into()], vec!["Зелёный".into(), "7.25".into()]]);
        let d = run(&mut fx, "SELECT DISTINCT Title FROM tea").unwrap().table;
        assert_eq!(d.rows.len(), 3);
        let c = run(&mut fx, "SELECT COUNT(*), SUM(Price) FROM tea WHERE Title LIKE 'ч%'").unwrap().table;
        assert_eq!(c.rows[0].cells, vec!["2".to_string(), "11.5".into()]);
        run(&mut fx, "UPDATE tea SET Price = Price * 2, Note = 'x' WHERE Number = 3").unwrap();
        run(&mut fx, "DELETE FROM tea WHERE Number IN (1, 2)").unwrap();
        let a = run(&mut fx, "SELECT * FROM tea ORDER BY Number").unwrap().table;
        assert_eq!(a.rows.len(), 2);
        assert_eq!(a.rows[0].cells[2], "60.00");
        assert_eq!(a.rows[0].cells[4], "x");
        run(&mut fx, "DROP TABLE tea").unwrap();
        assert!(!dir.join("tea.dbf").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_garbage_and_escapes() {
        let (mut fx, dir) = fx();
        for bad in ["", "SELECT", "SELECT * FROM", "SELECT * FROM t WHERE (((", "DROP TABLE \"../../etc/passwd\"", "CREATE TABLE \"/etc/x.dbf\" (a CHAR(1))", "CREATE TABLE t (a CHAR(0))", "CREATE TABLE t (a CHAR(1), A CHAR(1))", "SELECT * FROM nope"] {
            assert!(run(&mut fx, bad).is_err(), "{bad}");
        }
        let deep = format!("SELECT * FROM t WHERE {}1{}", "(".repeat(5000), ")".repeat(5000));
        assert!(run(&mut fx, &deep).is_err());
        assert!(run(&mut fx, &"x ".repeat(100_000)).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
