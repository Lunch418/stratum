//! Встроенные функции с крайними аргументами из модели: каждая функция,
//! которую знает ядро, вызывается с NaN, бесконечностями, огромными и
//! отрицательными числами, пустыми и не-ASCII строками. Ни одна не должна
//! паниковать, зависать или просить гигабайты памяти: модель из чужого
//! файла иначе роняет IDE (выделение памяти и переполнение стека не
//! перехватываются).
//!
//! Имена функций берутся из исходников диспетчеров (ветки `"имя" =>`).
//! Файловые функции работают в своей временной папке проекта.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};
use stratum_core::runtime::builtins::{call, Effects};
use stratum_core::runtime::value::Value;

const SOURCES: [&str; 4] = [
    include_str!("../src/runtime/builtins.rs"),
    include_str!("../src/runtime/extra.rs"),
    include_str!("../src/gfx/api.rs"),
    include_str!("../src/gfx/api3d.rs"),
];

/// Имена из веток `"a" | "b" => ...` диспетчеров.
fn names() -> Vec<String> {
    let mut out = Vec::new();
    for src in SOURCES {
        for line in src.lines() {
            let t = line.trim_start();
            if !t.starts_with('"') || !t.contains("=>") {
                continue;
            }
            let head = t.split("=>").next().unwrap_or("");
            for part in head.split('|') {
                let p = part.trim();
                if p.len() > 2 && p.starts_with('"') && p.ends_with('"') {
                    let n = &p[1..p.len() - 1];
                    if n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        out.push(n.to_string());
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn arg_sets() -> Vec<Vec<Value>> {
    let f = Value::Float;
    let s = |t: &str| Value::Str(t.to_string());
    let mut sets = vec![Vec::new()];
    for v in [f(f64::NAN), f(f64::INFINITY), f(f64::NEG_INFINITY), f(-1.0), f(0.0), f(1e18), f(-1e18), f(4e9), f(65536.0)] {
        sets.push(vec![v.clone(); 8]);
    }
    sets.push(vec![s(""); 8]);
    sets.push(vec![s("Ж%z"); 8]);
    sets.push(vec![s("C:Ж"), f(1e12), f(1e12), f(-5.0), s(""), f(f64::NAN), f(1e9), f(1e9)]);
    sets.push(vec![f(1.0), f(1e9), f(1e9), f(1e9), f(1e9), f(1e9), f(1e9), f(1e9)]);
    sets.push(vec![f(1.0), f(1.0), f(-1e9), f(1e9), f(-1e9), f(1e9), f(1.0), f(1.0)]);
    sets.push(vec![s("x"), f(1e12)]);
    // настоящие дескрипторы и крайние значения вперемешку
    for tail in [f(f64::NAN), f(1e18), f(-1e18), f(4e9), f(-1.0), s("Ж")] {
        for lead in [vec![f(1.0)], vec![f(1.0), f(1.0)], vec![f(1.0), f(2.0)], vec![f(1.0), f(3.0)], vec![f(1.0), f(1.0), f(1.0)]] {
            let mut v = lead;
            v.resize(8, tail.clone());
            sets.push(v);
        }
    }
    sets
}

/// Состояние, на котором малые номера (1, 2, 3) - настоящие дескрипторы:
/// окно с линией, текстом и растром, 3D-пространство, матрица, массив,
/// поток.
fn populated(dir: &std::path::Path) -> Effects {
    use stratum_core::gfx::{encode_bmp24, Brush, Dib, Font, Object, Pen, Shape, TextPart};
    let mut fx = Effects::default();
    fx.gfx.project_dir = dir.to_path_buf();
    let w = fx.gfx.open_window("w");
    let _ = fx.gfx.create_space3d(w);
    if let Some(sp) = fx.gfx.space_mut(w) {
        let pen = sp.add_pen(Pen { color: 0, style: 0, width: 1, rop: 0 });
        let brush = sp.add_brush(Brush { color: 0xffffff, style: 0, hatch: 0, rop: 0, dib: 0 });
        let font = sp.add_font(Font { height: 12, weight: 400, italic: false, underline: false, face: "Arial".into() });
        let string = sp.add_string("Ж".into());
        let text = sp.add_text(vec![TextPart { fg: 0, bg: 0, font, string }]);
        let dib = sp.add_dib(Dib::new(encode_bmp24(2, 2, &[0; 12]), Vec::new(), None));
        sp.add_object(Object::new(0, 1.0, 1.0, 10.0, 10.0, Shape::Polyline { pen, brush, points: vec![(1.0, 1.0), (10.0, 10.0), (1.0, 10.0)] }));
        sp.add_object(Object::new(0, 2.0, 2.0, 10.0, 10.0, Shape::Text { text }));
        sp.add_object(Object::new(0, 3.0, 3.0, 2.0, 2.0, Shape::Bitmap { dib, src: (0.0, 0.0, 2.0, 2.0), masked: false }));
    }
    let _ = call("MCreate", &[Value::Float(1.0), Value::Float(1.0), Value::Float(3.0), Value::Float(1.0), Value::Float(3.0)], &mut fx);
    let _ = call("CreateStream", &[Value::Str("MEMORY".into()), Value::Str(String::new()), Value::Str(String::new())], &mut fx);
    let _ = call("vCreate", &[], &mut fx);
    fx
}

#[test]
fn builtins_survive_extreme_arguments() {
    let names = names();
    assert!(names.len() > 300, "диспетчеры не разобраны: {} имён", names.len());
    let dir = std::env::temp_dir().join(format!("stratum-fuzz-builtins-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let only = std::env::var("STRATUM_FUZZ_ONLY").ok();
    let mut failures = Vec::new();
    for name in &names {
        if only.as_deref().is_some_and(|o| !o.eq_ignore_ascii_case(name)) {
            continue;
        }
        for (k, args) in arg_sets().into_iter().enumerate() {
            let mut fx = populated(&dir);
            let started = Instant::now();
            if std::env::var_os("STRATUM_FUZZ_TRACE").is_some() {
                eprintln!("{name} {k}");
            }
            if catch_unwind(AssertUnwindSafe(|| {
                let _ = call(name, &args, &mut fx);
                // второй вызов - после первого: созданные им дескрипторы
                let _ = call(name, &args, &mut fx);
            }))
            .is_err()
            {
                failures.push(format!("{name} (набор {k}): паника"));
            }
            if started.elapsed() > Duration::from_secs(2) {
                failures.push(format!("{name} (набор {k}): {:?}", started.elapsed()));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(failures.is_empty(), "{} сбоев:\n{}", failures.len(), failures.join("\n"));
}
