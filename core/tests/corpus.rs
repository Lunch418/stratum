//! Проверки на корпусе из `fixtures/` (собирается локально, см. README).
//! Без корпуса тесты молча пропускаются.

use std::path::{Path, PathBuf};
use stratum_core::formats::{self, cls};
use stratum_core::lang;
use stratum_core::sim::Simulation;

fn fixtures() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    p.is_dir().then_some(p)
}

fn libraries(root: &Path) -> Vec<PathBuf> {
    vec![root.join("library"), root.join("add.lib")]
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cls")) {
            out.push(path);
        }
    }
}

/// Критерий приёмки этапа 1: «Солнечная система» считается, и значения
/// на такте 100 воспроизводимы. Числа получены этим же ядром и зафиксированы
/// как ожидание; сверка с оригиналом через Wine — отдельный шаг.
#[test]
fn solar_system_runs_100_ticks() {
    let Some(root) = fixtures() else { return };
    let project = root.join("user/solar_system");
    let loaded = formats::load_project(&project, &libraries(&root)).unwrap().unwrap();
    let mut sim = Simulation::build(&loaded).unwrap();
    sim.run(100).unwrap();
    assert_eq!(sim.tick_number(), 100);

    let planets = sim.find("StratumClass_0e3148_ce").unwrap();
    let t = sim.value(planets, "t").unwrap().as_float();
    let x_e = sim.value(planets, "xE").unwrap().as_float();
    let y_e = sim.value(planets, "yE").unwrap().as_float();
    assert!((t - 1.0).abs() < 1e-9, "t = {t}");
    // xE = x0 + rE1·sin(t), где t читается на начало шага (0.99)
    assert!((x_e - (6.0 + 200.0 * 0.99f64.sin())).abs() < 1e-9, "xE = {x_e}");
    assert!((y_e - 190.0 * 0.99f64.cos()).abs() < 1e-9, "yE = {y_e}");

    // связь «день.day → NumberView.Value»: значение видно на другом конце
    let day = sim.find("день").unwrap();
    let day_value = sim.value(day, "day").unwrap().as_float();
    assert!(day_value > 58.0 && day_value < 59.0, "day = {day_value}");
    let shown = sim
        .instances()
        .iter()
        .enumerate()
        .filter(|(_, i)| i.class_name == "NumberView")
        .any(|(i, _)| sim.value(i, "Value").unwrap().as_float() == day_value);
    assert!(shown, "значение дня не дошло по связи до NumberView");
}

/// Все имиджи корпуса читаются, тексты разбираются — кроме трёх заведомо
/// битых в самом корпусе.
#[test]
fn whole_corpus_parses() {
    let Some(root) = fixtures() else { return };
    let mut files = Vec::new();
    collect(&root, &mut files);
    let mut unread = Vec::new();
    let mut unparsed = Vec::new();
    for file in &files {
        let data = std::fs::read(file).unwrap();
        if !data.starts_with(b"SB") {
            continue;
        }
        match cls::parse(&data, &file.display().to_string()) {
            Ok(c) => {
                if lang::parse(&c.text).is_err() {
                    unparsed.push(c.name);
                }
            }
            Err(e) => unread.push(e.to_string()),
        }
    }
    assert!(unread.is_empty(), "{unread:?}");
    unparsed.sort();
    assert_eq!(unparsed, ["INTEGER", "Ogre_OverlayElement", "main"]);
}

/// Каждый пример открывается и считает 20 тактов без ошибок времени выполнения.
#[test]
fn every_sample_project_runs() {
    let Some(root) = fixtures() else { return };
    let libs = libraries(&root);
    let mut failures = Vec::new();
    let mut count = 0;
    for entry in std::fs::read_dir(root.join("PROJECTS/samples")).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let Some(spj) = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).find(|p| {
            p.extension().is_some_and(|e| e.eq_ignore_ascii_case("spj"))
                && std::fs::read(p).map(|d| d.starts_with(b"Ih")).unwrap_or(false)
        }) else {
            continue;
        };
        count += 1;
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let loaded = match formats::load_project(&spj, &libs).unwrap() {
            Ok(l) => l,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        match Simulation::build(&loaded) {
            Ok(mut sim) => {
                if let Err(e) = sim.run(20) {
                    failures.push(format!("{name}: {}", e.message));
                }
            }
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    assert!(count >= 40, "примеров найдено {count}");
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Векторная графика: все иконки, рисунки и схемы имиджей корпуса плюс
/// отдельные `.vdr`. Не разбираются только блоки с вложенным 3D.
#[test]
fn vector_graphics_of_the_corpus_parse() {
    use stratum_core::formats::vdr;
    let Some(root) = fixtures() else { return };
    let mut files = Vec::new();
    collect(&root, &mut files);
    let (mut ok, mut failed) = (0, Vec::new());
    let mut try_blob = |data: &[u8], what: String| match vdr::parse(data, &what) {
        Ok(_) => ok += 1,
        Err(e) => failed.push(e.to_string()),
    };
    for file in &files {
        let data = std::fs::read(file).unwrap();
        let Ok(c) = cls::parse(&data, &file.display().to_string()) else { continue };
        for (tag, blob) in [("icon", &c.icon), ("image", &c.image), ("scheme", &c.scheme)] {
            if let Some(b) = blob {
                if b.windows(2).take(17).any(|w| w == b"2D") {
                    try_blob(b, format!("{}:{tag}", c.name));
                }
            }
        }
    }
    for entry in walk_ext(&root, "vdr") {
        try_blob(&std::fs::read(&entry).unwrap(), entry.display().to_string());
    }
    assert!(ok >= 350, "разобрано {ok}, ошибки: {failed:#?}");
    assert!(failed.len() <= 16, "{failed:#?}");
}

fn walk_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn go(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                go(&path, ext, out);
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
                out.push(path);
            }
        }
    }
    go(dir, ext, &mut out);
    out
}
