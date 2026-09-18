//! Встроенные функции языка.
//!
//! Сигнатуры и имена сверены с таблицами компилятора (`docs/lang/builtins.json`)
//! и справкой. Здесь реализованы группы, нужные для счёта: математика,
//! преобразование типов, строки, системные. Графика, окна и сообщения на этом
//! этапе — заглушки: они возвращают 0, а обращения к ним считаются, чтобы было
//! видно, чего модели не хватает (см. [`Effects::missing`]).

use super::data::{self, Arrays, Matrices, Matrix};
use super::value::{format_number, Value};
use crate::gfx::Gfx;
use std::collections::BTreeMap;

/// Побочные эффекты текста модели, которые ядро пока только записывает.
#[derive(Debug, Default, Clone)]
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
    pub matrices: Matrices,
    pub arrays: Arrays,
    /// Состояние генератора `rnd`: детерминированный xorshift, чтобы прогон
    /// повторялся; `randomize` задаёт зерно.
    pub rng: u64,
    /// Выходные аргументы последнего вызова (`&FLOAT` в таблице компилятора):
    /// номер аргумента и новое значение; интерпретатор записывает их в
    /// переменные, переданные на этих позициях.
    pub outputs: Vec<(usize, Value)>,
    /// Команды звука для плеера: (`play`|`stop`, файл, зациклить).
    pub sounds: Vec<(String, String, bool)>,
    /// Открытые через MCI псевдонимы: alias → файл.
    pub mci: BTreeMap<String, String>,
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
        "getanglebyxy" => num(f(args, 1).atan2(f(args, 0))),
        "fileexist" => num(if std::path::Path::new(&s(args, 0)).exists() { 1.0 } else { 0.0 }),
        // диалог ввода строки: без окна возвращаем значение по умолчанию
        "inputbox" => Value::Str(s(args, 2)),
        "change" => Value::Str(s(args, 0).replace(&s(args, 1), &s(args, 2))),
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
        // rnd(x) — случайное число в [0, x); зерно фиксировано, чтобы прогон
        // повторялся от запуска к запуску
        "rnd" => {
            if fx.rng == 0 {
                fx.rng = 0x2545_F491_4F6C_DD1D;
            }
            let mut x = fx.rng;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            fx.rng = x;
            let unit = (x >> 11) as f64 / (1u64 << 53) as f64;
            num(unit * f(args, 0))
        }
        "randomize" => {
            fx.rng = (f(args, 0) as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1;
            num(0.0)
        }

        // ── преобразование типов ──────────────────────────────────────────
        "float" => num(f(args, 0)),
        "integer" => num(f(args, 0).trunc()),
        "handle" => Value::Handle(f(args, 0)),
        "string" => Value::Str(s(args, 0)),
        "chr" => Value::Str(
            char::from_u32(f(args, 0) as u32).map(String::from).unwrap_or_default(),
        ),
        "ascii" => num(s(args, 0).chars().next().map(|c| c as u32 as f64).unwrap_or(0.0)),
        "rgbf" => Value::Color(f(args, 0)),
        "rgb" => Value::Color(
            ((f(args, 0) as u32 & 0xFF)
                | ((f(args, 1) as u32 & 0xFF) << 8)
                | ((f(args, 2) as u32 & 0xFF) << 16)) as f64,
        ),
        // RGBEx(r, g, b, flags): старший байт — прозрачность и прочие флаги
        "rgbex" => Value::Color(
            ((f(args, 0) as u32 & 0xFF)
                | ((f(args, 1) as u32 & 0xFF) << 8)
                | ((f(args, 2) as u32 & 0xFF) << 16)
                | ((f(args, 3) as u32 & 0xFF) << 24)) as f64,
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
        // звук: SndPlaySound(file, flags) — SND_LOOP = 8; пустое имя останавливает
        "sndplaysound" => {
            let file = s(args, 0);
            let flags = f(args, 1) as u32;
            if file.is_empty() {
                fx.sounds.push(("stop".into(), String::new(), false));
            } else {
                fx.sounds.push(("play".into(), file, flags & 8 != 0));
            }
            num(1.0)
        }
        // MCISendString: open <file> alias <x> / play <x> [repeat] / stop|close <x>
        "mcisendstring" => {
            let cmd = s(args, 0);
            let words: Vec<&str> = cmd.split_whitespace().collect();
            match words.first().map(|w| w.to_ascii_lowercase()).as_deref() {
                Some("open") => {
                    let file = words.get(1).unwrap_or(&"").to_string();
                    let alias = words.iter().position(|w| w.eq_ignore_ascii_case("alias")).and_then(|i| words.get(i + 1)).unwrap_or(&file.as_str()).to_string();
                    fx.mci.insert(alias.to_lowercase(), file);
                }
                Some("play") => {
                    let alias = words.get(1).unwrap_or(&"").to_lowercase();
                    if let Some(file) = fx.mci.get(&alias).cloned() {
                        fx.sounds.push(("play".into(), file, words.iter().any(|w| w.eq_ignore_ascii_case("repeat"))));
                    }
                }
                Some("stop") | Some("close") => {
                    let alias = words.get(1).unwrap_or(&"").to_lowercase();
                    if let Some(file) = fx.mci.get(&alias).cloned() {
                        fx.sounds.push(("stop".into(), file, false));
                    }
                    if words[0].eq_ignore_ascii_case("close") {
                        fx.mci.remove(&alias);
                    }
                }
                _ => {}
            }
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

        // ── матрицы ───────────────────────────────────────────────────────
        "mcreate" => {
            if f(args, 5) <= 0.0 { return Some(num(0.0)); }
            match Matrix::new(data::idx(f(args, 1)), data::idx(f(args, 2)), data::idx(f(args, 3)), data::idx(f(args, 4))) {
                Some(m) => num(fx.matrices.create(data::idx(f(args, 0)), m) as f64),
                None => num(0.0),
            }
        }
        "mdelete" => {
            if f(args, 1) <= 0.0 { return Some(num(0.0)); }
            boolean(fx.matrices.items.remove(&data::idx(f(args, 0))).is_some())
        }
        "mput" => {
            if f(args, 4) <= 0.0 { return Some(num(0.0)); }
            boolean(fx.matrices.get_mut(data::idx(f(args, 0)))
                .is_some_and(|m| m.set(data::idx(f(args, 1)), data::idx(f(args, 2)), f(args, 3))))
        }
        "mget" => {
            if f(args, 3) <= 0.0 { return Some(num(0.0)); }
            num(fx.matrices.get(data::idx(f(args, 0))).map(|m| m.get(data::idx(f(args, 1)), data::idx(f(args, 2)))).unwrap_or(0.0))
        }
        "mdim" => match fx.matrices.get(data::idx(f(args, 0))) {
            Some(m) => {
                for (i, v) in [m.min_i, m.max_i, m.min_j, m.max_j].into_iter().enumerate() {
                    fx.outputs.push((i + 1, num(v as f64)));
                }
                boolean(true)
            }
            None => boolean(false),
        },
        "mfill" | "mdiag" | "mmove" | "maddx" | "msubx" | "mmulx" | "mdivx" | "msort" => {
            let flag_at = if lower == "mmove" || lower == "msort" { 3 } else { 2 };
            if f(args, flag_at) <= 0.0 { return Some(num(0.0)); }
            let q = data::idx(f(args, 0));
            let v = f(args, 1);
            let ok = match fx.matrices.get_mut(q) {
                Some(m) => {
                    match lower.as_str() {
                        "mfill" => m.data.iter_mut().for_each(|x| *x = v),
                        "mdiag" => {
                            let n = m.rows().min(m.cols());
                            let cols = m.cols();
                            for k in 0..n { m.data[k * cols + k] = v; }
                        }
                        "mmove" => {
                            let (di, dj) = (data::idx(f(args, 1)), data::idx(f(args, 2)));
                            m.min_i += di; m.max_i += di; m.min_j += dj; m.max_j += dj;
                        }
                        "maddx" => m.data.iter_mut().for_each(|x| *x += v),
                        "msubx" => m.data.iter_mut().for_each(|x| *x -= v),
                        "mmulx" => m.data.iter_mut().for_each(|x| *x *= v),
                        "mdivx" => if v != 0.0 { m.data.iter_mut().for_each(|x| *x /= v) },
                        _ => { data::sort(m, data::idx(f(args, 1)), data::idx(f(args, 2))); }
                    }
                    true
                }
                None => false,
            };
            boolean(ok)
        }
        "maddc" | "msubc" | "mmulc" | "mdivc" | "mmul" => {
            if f(args, 3) <= 0.0 { return Some(num(0.0)); }
            let (q1, q2, q3) = (data::idx(f(args, 0)), data::idx(f(args, 1)), data::idx(f(args, 2)));
            num(match lower.as_str() {
                "maddc" => data::elementwise(&mut fx.matrices, q1, q2, q3, |a, b| a + b),
                "msubc" => data::elementwise(&mut fx.matrices, q1, q2, q3, |a, b| a - b),
                "mmulc" => data::elementwise(&mut fx.matrices, q1, q2, q3, |a, b| a * b),
                "mdivc" => data::elementwise(&mut fx.matrices, q1, q2, q3, |a, b| if b != 0.0 { a / b } else { 0.0 }),
                _ => data::multiply(&mut fx.matrices, q1, q2, q3),
            } as f64)
        }
        "mtransp" | "mobr" | "mnot" | "med" => {
            if f(args, 2) <= 0.0 && lower != "med" && lower != "mnot" { return Some(num(0.0)); }
            let (q1, q2) = (data::idx(f(args, 0)), data::idx(f(args, 1)));
            num(match lower.as_str() {
                "mtransp" => data::transpose(&mut fx.matrices, q1, q2),
                "mobr" => match fx.matrices.get(q1).and_then(data::inverse) {
                    Some(m) => fx.matrices.put_result(q2, m),
                    None => 0,
                },
                "mnot" => match fx.matrices.get(q1).cloned() {
                    Some(mut m) => { m.data.iter_mut().for_each(|x| *x = if *x == 0.0 { 1.0 } else { 0.0 }); fx.matrices.put_result(q2, m) }
                    None => 0,
                },
                _ => match fx.matrices.get(q1).cloned() {
                    Some(mut m) => { m.data.iter_mut().for_each(|x| *x = if *x != 0.0 { 1.0 } else { 0.0 }); fx.matrices.put_result(q2, m) }
                    None => 0,
                },
            } as f64)
        }
        "mdet" => num(fx.matrices.get(data::idx(f(args, 0))).map(data::determinant).unwrap_or(0.0)),
        "msum" => num(fx.matrices.get(data::idx(f(args, 0))).map(|m| m.data.iter().sum()).unwrap_or(0.0)),
        "mdelta" => num(fx.matrices.get(data::idx(f(args, 0))).map(|m| (m.rows() * m.cols()) as f64).unwrap_or(0.0)),
        "mcut" => {
            if f(args, 6) <= 0.0 { return Some(num(0.0)); }
            let (q1, q2) = (data::idx(f(args, 0)), data::idx(f(args, 1)));
            let (i0, j0, di, dj) = (data::idx(f(args, 2)), data::idx(f(args, 3)), data::idx(f(args, 4)), data::idx(f(args, 5)));
            num(match fx.matrices.get(q1).cloned() {
                Some(src) => match Matrix::new(i0, i0 + di - 1, j0, j0 + dj - 1) {
                    Some(mut m) => {
                        for i in i0..i0 + di { for j in j0..j0 + dj { m.set(i, j, src.get(i, j)); } }
                        fx.matrices.put_result(q2, m)
                    }
                    None => 0,
                },
                None => 0,
            } as f64)
        }
        "mglue" => {
            if f(args, 5) <= 0.0 { return Some(num(0.0)); }
            let (q1, q2, q3) = (data::idx(f(args, 0)), data::idx(f(args, 1)), data::idx(f(args, 2)));
            let (di, dj) = (data::idx(f(args, 3)), data::idx(f(args, 4)));
            num(match (fx.matrices.get(q1).cloned(), fx.matrices.get(q2).cloned()) {
                (Some(a), Some(b)) => {
                    let (min_i, max_i) = (a.min_i.min(b.min_i + di), a.max_i.max(b.max_i + di));
                    let (min_j, max_j) = (a.min_j.min(b.min_j + dj), a.max_j.max(b.max_j + dj));
                    match Matrix::new(min_i, max_i, min_j, max_j) {
                        Some(mut m) => {
                            for i in a.min_i..=a.max_i { for j in a.min_j..=a.max_j { m.set(i, j, a.get(i, j)); } }
                            for i in b.min_i..=b.max_i { for j in b.min_j..=b.max_j { m.set(i + di, j + dj, b.get(i, j)); } }
                            fx.matrices.put_result(q3, m)
                        }
                        None => 0,
                    }
                }
                _ => 0,
            } as f64)
        }
        "msaveas" => {
            if f(args, 2) <= 0.0 { return Some(num(0.0)); }
            let path = fx.gfx.project_dir.join(s(args, 1));
            boolean(fx.matrices.get(data::idx(f(args, 0)))
                .is_some_and(|m| std::fs::write(&path, data::to_text(m)).is_ok()))
        }
        "mload" => {
            if f(args, 2) <= 0.0 { return Some(num(0.0)); }
            let file = s(args, 1);
            let found = fx.gfx.find_file(&file).and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| data::from_text(&t));
            match found {
                Some(m) => num(fx.matrices.create(data::idx(f(args, 0)), m) as f64),
                None => num(0.0),
            }
        }

        // ── динамические массивы ──────────────────────────────────────────
        "new" => Value::Handle(fx.arrays.new_array() as f64),
        "delete" => boolean(fx.arrays.delete(f(args, 0) as u32)),
        "vclearall" => { fx.arrays = Arrays::default(); num(0.0) }
        "vgetcount" => num(fx.arrays.get(f(args, 0) as u32).map(|a| a.len() as f64).unwrap_or(0.0)),
        "vinsert" => {
            let ty = s(args, 1);
            let mut el = data::Element { type_name: ty.clone(), ..Default::default() };
            match ty.to_ascii_uppercase().as_str() {
                "FLOAT" => el.set("", Value::Float(0.0)),
                "STRING" => el.set("", Value::Str(String::new())),
                "HANDLE" => el.set("", Value::Handle(0.0)),
                _ => {}
            }
            match fx.arrays.get_mut(f(args, 0) as u32) {
                Some(a) => { a.push(el); num((a.len() - 1) as f64) }
                None => num(-1.0),
            }
        }
        "vdelete" => {
            let i = f(args, 1) as usize;
            boolean(fx.arrays.get_mut(f(args, 0) as u32).is_some_and(|a| if i < a.len() { a.remove(i); true } else { false }))
        }
        "vgettype" => Value::Str(fx.arrays.get(f(args, 0) as u32)
            .and_then(|a| a.get(f(args, 1) as usize)).map(|e| e.type_name.clone()).unwrap_or_default()),
        "vgetf" | "vgets" | "vgeth" => {
            let v = fx.arrays.get(f(args, 0) as u32)
                .and_then(|a| a.get(f(args, 1) as usize))
                .and_then(|e| e.get(&s(args, 2)).cloned());
            match (lower.as_str(), v) {
                ("vgets", Some(v)) => Value::Str(v.as_string()),
                ("vgets", None) => Value::Str(String::new()),
                ("vgeth", v) => Value::Handle(v.map(|v| v.as_float()).unwrap_or(0.0)),
                (_, v) => num(v.map(|v| v.as_float()).unwrap_or(0.0)),
            }
        }
        "vset" => {
            let value = args.get(3).cloned().unwrap_or(Value::Float(0.0));
            let field = s(args, 2);
            boolean(fx.arrays.get_mut(f(args, 0) as u32)
                .and_then(|a| a.get_mut(f(args, 1) as usize))
                .map(|e| e.set(&field, value)).is_some())
        }
        "vsort" => {
            // vSort(HArray, [field]) / vSort(HArray, descending, [field])
            let (desc, field) = match args.get(1) {
                Some(Value::Str(fld)) => (false, fld.clone()),
                Some(v) => (v.as_float() != 0.0, s(args, 2)),
                None => (false, String::new()),
            };
            boolean(fx.arrays.get_mut(f(args, 0) as u32).map(|a| {
                a.sort_by(|x, y| {
                    let (vx, vy) = (x.get(&field), y.get(&field));
                    let ord = match (vx, vy) {
                        (Some(Value::Str(p)), Some(Value::Str(q))) => p.cmp(q),
                        (Some(p), Some(q)) => p.as_float().partial_cmp(&q.as_float()).unwrap_or(std::cmp::Ordering::Equal),
                        _ => std::cmp::Ordering::Equal,
                    };
                    if desc { ord.reverse() } else { ord }
                });
            }).is_some())
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
        _ => {
            if let Some(v) = crate::gfx::api3d::call(name, args, &mut fx.gfx, &mut fx.matrices, &mut fx.outputs) {
                return Some(v);
            }
            return crate::gfx::api::call(name, args, &mut fx.gfx);
        }
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
