//! Лексер языка моделирования Stratum.
//!
//! Язык регистронезависим, операторы разделяются переводом строки или `;`,
//! поэтому перевод строки — значимый токен.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// Идентификатор в исходном написании; сравнивать без учёта регистра.
    Ident(String),
    Number(f64),
    Str(String),
    /// Литерал дескриптора: `#0`, `#175`.
    Handle(f64),
    Op(&'static str),
    /// Конец оператора: перевод строки или `;`.
    Eol,
    Eof,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Ident(s) => write!(f, "{s}"),
            Tok::Number(n) => write!(f, "{n}"),
            Tok::Str(s) => write!(f, "\"{s}\""),
            Tok::Handle(h) => write!(f, "#{h}"),
            Tok::Op(o) => write!(f, "{o}"),
            Tok::Eol => write!(f, "конец строки"),
            Tok::Eof => write!(f, "конец текста"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "строка {}, позиция {}: {}", self.line, self.column, self.message)
    }
}

/// Операторы, отсортированы так, чтобы длинные проверялись первыми.
const OPERATORS: &[&str] = &[
    "::=", ":=", "==", "!=", ">=", "<=", "&&", "||", "<<", ">>", "++",
    "=", ">", "<", "+", "-", "*", "/", "%", "^", "&", "|", "!", "~", "(", ")", ",", "?", "[", "]",
];

pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1u32;
    let mut col = 1u32;

    macro_rules! bump {
        ($n:expr) => {{
            for _ in 0..$n {
                if chars[i] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
        }};
    }

    while i < chars.len() {
        let c = chars[i];

        if c == '\r' {
            bump!(1);
            continue;
        }
        if c == '\n' || c == ';' {
            out.push(Token { tok: Tok::Eol, line, column: col });
            bump!(1);
            continue;
        }
        if c == ' ' || c == '\t' {
            bump!(1);
            continue;
        }
        // комментарии
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                bump!(1);
            }
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let (sl, sc) = (line, col);
            bump!(2);
            loop {
                if i + 1 >= chars.len() {
                    return Err(LexError {
                        message: "комментарий /* не закрыт".into(),
                        line: sl,
                        column: sc,
                    });
                }
                if chars[i] == '*' && chars[i + 1] == '/' {
                    bump!(2);
                    break;
                }
                bump!(1);
            }
            continue;
        }
        // строка; одинарные кавычки тоже принимаются — так написана
        // часть стандартной библиотеки (`case (~s=='r')`)
        if c == '"' || c == '\'' {
            let quote = c;
            let (sl, sc) = (line, col);
            bump!(1);
            let mut s = String::new();
            loop {
                if i >= chars.len() || chars[i] == '\n' {
                    return Err(LexError {
                        message: "строковый литерал не закрыт".into(),
                        line: sl,
                        column: sc,
                    });
                }
                if chars[i] == quote {
                    // удвоенная кавычка внутри строки
                    if i + 1 < chars.len() && chars[i + 1] == quote {
                        s.push(quote);
                        bump!(2);
                        continue;
                    }
                    bump!(1);
                    break;
                }
                s.push(chars[i]);
                bump!(1);
            }
            out.push(Token { tok: Tok::Str(s), line: sl, column: sc });
            continue;
        }
        // дескриптор #N
        if c == '#' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            let (sl, sc) = (line, col);
            bump!(1);
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                bump!(1);
            }
            let text: String = chars[start..i].iter().collect();
            let value = text.parse::<f64>().map_err(|_| LexError {
                message: format!("не число в дескрипторе: #{text}"),
                line: sl,
                column: sc,
            })?;
            out.push(Token { tok: Tok::Handle(value), line: sl, column: sc });
            continue;
        }
        // число
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let (sl, sc) = (line, col);
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                bump!(1);
            }
            if i < chars.len() && chars[i] == '.' {
                bump!(1);
                while i < chars.len() && chars[i].is_ascii_digit() {
                    bump!(1);
                }
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let save = i;
                bump!(1);
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    bump!(1);
                }
                if i < chars.len() && chars[i].is_ascii_digit() {
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        bump!(1);
                    }
                } else {
                    // не экспонента, а что-то вроде 1end — откатываемся
                    while i > save {
                        i -= 1;
                        col -= 1;
                    }
                }
            }
            let text: String = chars[start..i].iter().collect();
            let value = text.parse::<f64>().map_err(|_| LexError {
                message: format!("не разобрано число {text}"),
                line: sl,
                column: sc,
            })?;
            out.push(Token { tok: Tok::Number(value), line: sl, column: sc });
            continue;
        }
        // идентификатор: латиница, кириллица, цифры и подчёркивание
        if is_ident_start(c) {
            let (sl, sc) = (line, col);
            let start = i;
            while i < chars.len() && is_ident_char(chars[i]) {
                bump!(1);
            }
            let text: String = chars[start..i].iter().collect();
            out.push(Token { tok: Tok::Ident(text), line: sl, column: sc });
            continue;
        }
        // операторы
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        if let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            let (sl, sc) = (line, col);
            bump!(op.chars().count());
            out.push(Token { tok: Tok::Op(op), line: sl, column: sc });
            continue;
        }
        return Err(LexError {
            message: format!("неизвестный символ {c:?}"),
            line,
            column: col,
        });
    }
    out.push(Token { tok: Tok::Eof, line, column: col });
    Ok(out)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Tok> {
        tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn reads_assignment() {
        assert_eq!(
            kinds("x := ~y + 1.5e3"),
            vec![
                Tok::Ident("x".into()),
                Tok::Op(":="),
                Tok::Op("~"),
                Tok::Ident("y".into()),
                Tok::Op("+"),
                Tok::Number(1500.0),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn semicolon_ends_a_statement() {
        assert_eq!(kinds("a:=1; b:=2").len(), 8);
    }

    #[test]
    fn reads_handles_strings_and_comments() {
        assert_eq!(
            kinds("HSpace := #0 // комментарий\ns := \"Январь\""),
            vec![
                Tok::Ident("HSpace".into()),
                Tok::Op(":="),
                Tok::Handle(0.0),
                Tok::Eol,
                Tok::Ident("s".into()),
                Tok::Op(":="),
                Tok::Str("Январь".into()),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn cyrillic_identifiers() {
        assert_eq!(kinds("день := 1")[0], Tok::Ident("день".into()));
    }
}
