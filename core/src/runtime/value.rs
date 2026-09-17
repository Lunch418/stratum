//! Значения языка Stratum.
//!
//! Типов немного: FLOAT (double), STRING, HANDLE и COLORREF. Истина — любое
//! ненулевое число; сравнения всегда дают 0 или 1.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Float,
    Str,
    Handle,
    Color,
}

impl ValueType {
    /// Тип по имени из `.cls`; неизвестные считаются FLOAT, как в оригинале.
    pub fn from_name(name: &str) -> ValueType {
        match name.to_ascii_uppercase().as_str() {
            "STRING" => ValueType::Str,
            "HANDLE" | "POINTER" => ValueType::Handle,
            "COLORREF" => ValueType::Color,
            _ => ValueType::Float,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ValueType::Float => "FLOAT",
            ValueType::Str => "STRING",
            ValueType::Handle => "HANDLE",
            ValueType::Color => "COLORREF",
        }
    }

    pub fn default_value(self) -> Value {
        match self {
            ValueType::Str => Value::Str(String::new()),
            ValueType::Handle => Value::Handle(0.0),
            ValueType::Color => Value::Color(0.0),
            ValueType::Float => Value::Float(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    Str(String),
    Handle(f64),
    Color(f64),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Float(_) => ValueType::Float,
            Value::Str(_) => ValueType::Str,
            Value::Handle(_) => ValueType::Handle,
            Value::Color(_) => ValueType::Color,
        }
    }

    /// Числовое значение. Строка разбирается как число, иначе 0 — так ведёт
    /// себя оригинал при смешивании типов.
    pub fn as_float(&self) -> f64 {
        match self {
            Value::Float(v) | Value::Handle(v) | Value::Color(v) => *v,
            Value::Str(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            other => format_number(other.as_float()),
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::Str(s) => !s.is_empty(),
            other => other.as_float() != 0.0,
        }
    }

    /// Приводит значение к типу переменной при записи.
    pub fn cast_to(self, ty: ValueType) -> Value {
        match ty {
            ValueType::Str => Value::Str(self.as_string()),
            ValueType::Float => Value::Float(self.as_float()),
            ValueType::Handle => Value::Handle(self.as_float()),
            ValueType::Color => Value::Color(self.as_float()),
        }
    }

    /// Разбор значения по умолчанию из `.cls`, где всё хранится текстом.
    pub fn parse_default(text: &str, ty: ValueType) -> Value {
        if text.is_empty() {
            return ty.default_value();
        }
        match ty {
            ValueType::Str => Value::Str(text.to_string()),
            _ => {
                let t = text.trim();
                let n = t
                    .strip_prefix('#')
                    .unwrap_or(t)
                    .parse::<f64>()
                    .unwrap_or(0.0);
                match ty {
                    ValueType::Handle => Value::Handle(n),
                    ValueType::Color => Value::Color(n),
                    _ => Value::Float(n),
                }
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{s}"),
            Value::Handle(h) => write!(f, "#{}", format_number(*h)),
            other => write!(f, "{}", format_number(other.as_float())),
        }
    }
}

/// Печать числа так, как это делает оригинал: без хвостовых нулей.
pub fn format_number(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v}");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_without_trailing_zeros() {
        assert_eq!(Value::Float(3.0).to_string(), "3");
        assert_eq!(Value::Float(0.5).to_string(), "0.5");
    }

    #[test]
    fn strings_convert_to_numbers() {
        assert_eq!(Value::Str("12.5".into()).as_float(), 12.5);
        assert_eq!(Value::Str("нет".into()).as_float(), 0.0);
    }

    #[test]
    fn defaults_follow_the_declared_type() {
        assert_eq!(Value::parse_default("", ValueType::Str), Value::Str(String::new()));
        assert_eq!(Value::parse_default("1", ValueType::Float), Value::Float(1.0));
        assert_eq!(Value::parse_default("#5", ValueType::Handle), Value::Handle(5.0));
    }
}
