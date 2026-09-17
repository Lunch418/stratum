//! Встроенные функции языка.
//!
//! Сигнатуры и имена сверены с таблицами компилятора (`docs/lang/builtins.json`)
//! и справкой. Здесь реализованы группы, нужные для счёта: математика,
//! преобразование типов, строки, системные. Графика, окна и сообщения на этом
//! этапе — заглушки: они возвращают 0, а обращения к ним считаются, чтобы было
//! видно, чего модели не хватает (см. [`Effects::missing`]).

use super::value::{format_number, Value};
use crate::gfx::Gfx;
use std::collections::BTreeMap;

/// Побочные эффекты текста модели, которые ядро пока только записывает.
#[derive(Debug, Default)]
pub struct Effects {
    /// Сообщения `LogMessage`.
    pub log: Vec<String>,
    /// Вызовы функций, которые ещё не реализованы, и сколько раз.
    pub missing: BTreeMap<String, u32>,
    /// `exit()` — прервать текст имиджа в этом такте.
    pub exit_requested: bool,
    /// `Stop`/`Quit` — остановить всю модель.
    pub stop_requested: bool,
    /// Окна и графические пространства модели.
    pub gfx: Gfx,
    /// Выходные аргументы последнего вызова (`&FLOAT` в таблице компилятора):
    /// номер аргумента и новое значение; интерпретатор записывает их в
    /// переменные, переданные на этих позициях.
    pub outputs: Vec<(usize, Value)>,
}

impl Effects {
    pub fn clear_exit(&mut self) {
        self.exit_requested = false;
    }
}

fn f(args: &[Value], i: usize) -> f64 {
    args.get(i).map(|v| v.as_float()).unwrap_or(0.0)
}

fn s(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.as_string()).unwrap_or_default()
}

fn num(v: f64) -> Value {
    Value::Float(v)
}

fn boolean(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}

/// Вызывает встроенную функцию. `None` — функции с таким именем нет.
pub fn call(name: &str, args: &[Value], fx: &mut Effects) -> Option<Value> {
    let lower = name.to_ascii_lowercase();
    let v = match lower.as_str() {
        // ── математика ────────────────────────────────────────────────────
        "sin" => num(f(args, 0).sin()),
        "cos" => num(f(args, 0).cos()),
        "tg" | "tan" => num(f(args, 0).tan()),
        "arcsin" => num(f(args, 0).clamp(-1.0, 1.0).asin()),
        "arccos" => num(f(args, 0).clamp(-1.0, 1.0).acos()),
        "arctan" | "atan" => num(f(args, 0).atan()),
        "exp" => num(f(args, 0).exp()),
        "ln" => num(safe_ln(f(args, 0))),
        "lg" => num(safe_log10(f(args, 0))),
        "log" => {
            let (base, x) = (f(args, 0), f(args, 1));
            if base > 0.0 && base != 1.0 && x > 0.0 { num(x.log(base)) } else { num(0.0) }
        }
        "sqrt" => num(if f(args, 0) >= 0.0 { f(args, 0).sqrt() } else { 0.0 }),
        "sqr" => num(f(args, 0) * f(args, 0)),
        "abs" => num(f(args, 0).abs()),
        "sgn" => num(match f(args, 0) {
            v if v > 0.0 => 1.0,
            v if v < 0.0 => -1.0,
            _ => 0.0,
        }),
        "max" => num(f(args, 0).max(f(args, 1))),
        "min" => num(f(args, 0).min(f(args, 1))),
        "average" => num((f(args, 0) + f(args, 1)) / 2.0),
        "trunc" => num(f(args, 0).trunc()),
        // round(x, n) — округление до n знаков; без второго аргумента до целого
        "round" => {
            let x = f(args, 0);
            let digits = if args.len() > 1 { f(args, 1) } else { 0.0 };
            let k = 10f64.powf(digits);
            num((x * k).round() / k)
        }
        "roundt" => num(f(args, 0).round()),
        "rad" => num(f(args, 0).to_radians()),
        "deg" => num(f(args, 0).to_degrees()),
        // единичная функция и дельта-функция
        "ed" => boolean(f(args, 0) != 0.0),
        "delta" => boolean(f(args, 0) == 0.0),
        "limit" => {
            let (x, lo, hi) = (f(args, 0), f(args, 1), f(args, 2));
            num(x.clamp(lo.min(hi), hi.max(lo)))
        }
        "not" => boolean(f(args, 0) == 0.0),
        "and" => boolean(f(args, 0) != 0.0 && f(args, 1) != 0.0),
        "or" => boolean(f(args, 0) != 0.0 || f(args, 1) != 0.0),
        "xor" => boolean((f(args, 0) != 0.0) ^ (f(args, 1) != 0.0)),
        "notbin" => num(!(f(args, 0) as i64) as f64),
        "xorbin" => num(((f(args, 0) as i64) ^ (f(args, 1) as i64)) as f64),
        "rnd" => num(0.0), // без источника случайности детерминизм важнее
        "randomize" => num(0.0),

        // ── преобразование типов ──────────────────────────────────────────
        "float" => num(f(args, 0)),
        "integer" => num(f(args, 0).trunc()),
        "handle" => Value::Handle(f(args, 0)),
        "string" => Value::Str(s(args, 0)),
        "chr" => Value::Str(
            char::from_u32(f(args, 0) as u32).map(String::from).unwrap_or_default(),
        ),
        "ascii" => num(s(args, 0).chars().next().map(|c| c as u32 as f64).unwrap_or(0.0)),
        "rgb" => Value::Color(
            ((f(args, 0) as u32 & 0xFF)
                | ((f(args, 1) as u32 & 0xFF) << 8)
                | ((f(args, 2) as u32 & 0xFF) << 16)) as f64,
        ),
        "getrvalue" => num((f(args, 0) as u32 & 0xFF) as f64),
        "getgvalue" => num(((f(args, 0) as u32 >> 8) & 0xFF) as f64),
        "getbvalue" => num(((f(args, 0) as u32 >> 16) & 0xFF) as f64),

        // ── строки ────────────────────────────────────────────────────────
        "length" => num(s(args, 0).chars().count() as f64),
        "upper" => Value::Str(s(args, 0).to_uppercase()),
        "lower" => Value::Str(s(args, 0).to_lowercase()),
        "substr" => {
            let text: Vec<char> = s(args, 0).chars().collect();
            let from = f(args, 1).max(0.0) as usize;
            let count = f(args, 2).max(0.0) as usize;
            Value::Str(text.iter().skip(from).take(count).collect())
        }
        "pos" => {
            let (haystack, needle) = (s(args, 0), s(args, 1));
            num(match haystack.find(&needle) {
                Some(byte) => haystack[..byte].chars().count() as f64,
                None => -1.0,
            })
        }
        "addslash" => {
            let mut p = s(args, 0);
            if !p.is_empty() && !p.ends_with('\\') && !p.ends_with('/') {
                p.push('\\');
            }
            Value::Str(p)
        }
        "getpathfromfile" => {
            let p = s(args, 0);
            let cut = p.rfind(['\\', '/']).map(|i| i + 1).unwrap_or(0);
            Value::Str(p[..cut].to_string())
        }

        // ── системные ─────────────────────────────────────────────────────
        "exit" => {
            fx.exit_requested = true;
            num(0.0)
        }
        "stop" | "quit" | "closeall" => {
            fx.stop_requested = true;
            num(0.0)
        }
        "logmessage" => {
            fx.log.push(s(args, 0));
            num(0.0)
        }
        "gettickcount" => num(0.0),
        // inc(x [, step]) / dec(x [, step]) меняют переменную-аргумент
        "inc" | "dec" => {
            let step = if args.len() > 1 { f(args, 1) } else { 1.0 };
            let v = f(args, 0) + if lower == "inc" { step } else { -step };
            fx.outputs.push((0, num(v)));
            num(v)
        }

        // функции 2D с выходными аргументами
        "getactualsize2d" => {
            let (w, h) = crate::gfx::api::object_size(&fx.gfx, args).unwrap_or((0.0, 0.0));
            fx.outputs.push((2, num(w)));
            fx.outputs.push((3, num(h)));
            boolean(true)
        }
        "getbitmapsrcrect2d" => {
            let (x, y, w, h) = crate::gfx::api::bitmap_src(&fx.gfx, args).unwrap_or((0.0, 0.0, 0.0, 0.0));
            fx.outputs.push((2, num(x)));
            fx.outputs.push((3, num(y)));
            fx.outputs.push((4, num(w)));
            fx.outputs.push((5, num(h)));
            boolean(true)
        }
        _ => return crate::gfx::api::call(name, args, &mut fx.gfx),
    };
    let _ = format_number; // используется в Display значения
    Some(v)
}

/// Вызов функции, которой ещё нет: считаем обращение и отдаём 0.
pub fn call_stub(name: &str, fx: &mut Effects) -> Value {
    *fx.missing.entry(name.to_ascii_lowercase()).or_insert(0) += 1;
    Value::Float(0.0)
}

fn safe_ln(x: f64) -> f64 {
    if x > 0.0 { x.ln() } else { 0.0 }
}

fn safe_log10(x: f64) -> f64 {
    if x > 0.0 { x.log10() } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(name: &str, args: &[Value]) -> Value {
        let mut fx = Effects::default();
        call(name, args, &mut fx).expect("функция должна быть известна")
    }

    #[test]
    fn math_matches_the_help() {
        assert_eq!(eval("sqr", &[Value::Float(3.0)]), Value::Float(9.0));
        assert_eq!(eval("abs", &[Value::Float(-2.5)]), Value::Float(2.5));
        assert_eq!(eval("sgn", &[Value::Float(-7.0)]), Value::Float(-1.0));
        assert_eq!(eval("round", &[Value::Float(1.2345), Value::Float(2.0)]), Value::Float(1.23));
    }

    #[test]
    fn names_are_case_insensitive() {
        assert_eq!(eval("SIN", &[Value::Float(0.0)]), Value::Float(0.0));
    }

    #[test]
    fn strings() {
        assert_eq!(eval("length", &[Value::Str("планета".into())]), Value::Float(7.0));
        assert_eq!(
            eval("substr", &[Value::Str("Stratum".into()), Value::Float(0.0), Value::Float(4.0)]),
            Value::Str("Stra".into())
        );
        assert_eq!(
            eval("addslash", &[Value::Str("C:\\SC3".into())]),
            Value::Str("C:\\SC3\\".into())
        );
    }

    #[test]
    fn exit_is_recorded() {
        let mut fx = Effects::default();
        call("exit", &[], &mut fx);
        assert!(fx.exit_requested);
    }

    #[test]
    fn unknown_function_is_counted() {
        let mut fx = Effects::default();
        assert!(call("RegisterObject", &[], &mut fx).is_none());
        call_stub("RegisterObject", &mut fx);
        assert_eq!(fx.missing["registerobject"], 1);
    }
}
