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
                // цвет по умолчанию в .cls пишется как rgb(r,g,b); значение —
                // COLORREF Windows: r + g·256 + b·65536 (сверено по снимку
                // оригинала: rgb(255,0,0) = 255)
                if let Some(inner) = t.to_ascii_lowercase().strip_prefix("rgb(").and_then(|x| x.strip_suffix(')').map(str::to_string)) {
                    let c: Vec<f64> = inner.split(',').map(|x| x.trim().parse::<f64>().unwrap_or(0.0).clamp(0.0, 255.0)).collect();
                    let v = c.first().copied().unwrap_or(0.0) + c.get(1).copied().unwrap_or(0.0) * 256.0 + c.get(2).copied().unwrap_or(0.0) * 65536.0;
                    return match ty {
                        ValueType::Color => Value::Color(v),
                        ValueType::Handle => Value::Handle(v),
                        _ => Value::Float(v),
                    };
                }
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
/// Число в строку так, как это делает сама модель в оригинале
/// (`String(x)`, «строка + число»): формат `%g`, 6 значащих цифр —
/// `1.41421`, `1e+20`, `1.23457e+11`, `1e-07`. Сверено с Stratum 2000
/// через Wine (tools/verify/math.txt). Для инспектора и `.stt` остаётся
/// полная точность [`format_number`].
pub fn format_g(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    if !v.is_finite() {
        return if v.is_nan() { "nan".into() } else if v > 0.0 { "inf".into() } else { "-inf".into() };
    }
    // показатель после округления до 6 значащих цифр
    let sci = format!("{:.5e}", v);
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let trim = |s: String| -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };
    if !(-4..6).contains(&exp) {
        format!("{}e{}{:02}", trim(mantissa.to_string()), if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        trim(format!("{:.*}", (5 - exp).max(0) as usize, v))
    }
}

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
    fn string_conversion_matches_the_original() {
        // эталоны получены от Stratum 2000 (tools/verify/math.txt)
        for (v, s) in [(2f64.sqrt(), "1.41421"), (1e20, "1e+20"), (123456789012.0, "1.23457e+11"), (1e-7, "1e-07"), (1234.56789, "1234.57"),
            (-1.020682e-11, "-1.02068e-11"), (0.1, "0.1"), (-0.5, "-0.5"), (1024.0, "1024"), (3.5, "3.5"), (1.0 / 3.0, "0.333333")] {
            assert_eq!(format_g(v), s, "{v}");
        }
    }

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
