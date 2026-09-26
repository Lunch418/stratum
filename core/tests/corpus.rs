//! Проверки на корпусе из `fixtures/` (собирается локально, см. README).
//!
//! Без корпуса тесты **падают** с объяснением, а не проходят молча: иначе
//! «зелёный» прогон в чистом клоне ничего не значит. Пропустить их явно —
//! `STRATUM_SKIP_CORPUS=1 cargo test`; так делает CI, где корпуса нет, и в
//! выводе остаётся строка «корпус пропущен».
//!
//! Важно: эталонные числа здесь получены этим же ядром. Они ловят
//! регрессии, но не доказывают совпадения с Stratum 2000 — для этого нужна
//! сверка с оригиналом (см. docs/verification.md).

use std::path::{Path, PathBuf};
use stratum_core::formats::{self, cls};
use stratum_core::lang;
use stratum_core::sim::Simulation;

fn fixtures() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    if p.is_dir() {
        return Some(p);
    }
    if std::env::var_os("STRATUM_SKIP_CORPUS").is_some() {
        eprintln!("корпус пропущен: нет fixtures/ и задан STRATUM_SKIP_CORPUS");
        return None;
    }
    panic!("нет корпуса fixtures/: соберите его `make corpus` (см. README) или пропустите явно: STRATUM_SKIP_CORPUS=1 cargo test");
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
/// на такте 100 воспроизводимы. Числа получены ядром и затем сверены с
/// Stratum 2000 через Wine (stratum instrument + sttdiff): все 612
/// нечисловых и числовых значений, кроме номеров графических объектов,
/// совпали с оригиналом — см. docs/verification.md.
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
    let mut round_trip_failed: Vec<String> = Vec::new();
    let mut try_blob = |data: &[u8], what: String| match vdr::parse(data, &what) {
        Ok(pic) => {
            ok += 1;
            // запись в формате 3.0 и повторное чтение должны дать ту же картинку
            let again = vdr::write(&pic);
            match vdr::parse(&again, "again") {
                Ok(back) => {
                    if let Err(e) = same_picture(&pic, &back) {
                        round_trip_failed.push(format!("{what}: {e}"));
                    }
                }
                Err(e) => round_trip_failed.push(format!("{what}: повторное чтение: {e}")),
            }
        }
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
    assert!(round_trip_failed.is_empty(), "{}", round_trip_failed.join("\n"));
}

fn same_picture(a: &stratum_core::formats::vdr::Picture, b: &stratum_core::formats::vdr::Picture) -> Result<(), String> {
    use stratum_core::formats::vdr::ObjectKind;
    let check = |ok: bool, what: &str| if ok { Ok(()) } else { Err(what.to_string()) };
    check(a.origin == b.origin && a.scale == b.scale && a.window == b.window, "заголовок")?;
    check(a.zorder == b.zorder, "zorder")?;
    check(a.objects.len() == b.objects.len(), "число объектов")?;
    for (x, y) in a.objects.iter().zip(&b.objects) {
        check(x.handle == y.handle && x.name == y.name && x.flags == y.flags, "объект: handle/имя/флаги")?;
        let same = match (&x.kind, &y.kind) {
            (ObjectKind::Polyline { x: a1, y: a2, w: a3, h: a4, pen: p1, brush: b1, points: pt1 }, ObjectKind::Polyline { x: c1, y: c2, w: c3, h: c4, pen: p2, brush: b2, points: pt2 }) =>
                (a1, a2, a3, a4, p1, b1, pt1) == (c1, c2, c3, c4, p2, b2, pt2),
            (ObjectKind::Bitmap { x: a1, y: a2, w: a3, h: a4, src: s1, dib: d1, masked: m1 }, ObjectKind::Bitmap { x: c1, y: c2, w: c3, h: c4, src: s2, dib: d2, masked: m2 }) =>
                (a1, a2, a3, a4, s1, d1, m1) == (c1, c2, c3, c4, s2, d2, m2),
            (ObjectKind::Text { x: a1, y: a2, w: a3, h: a4, text: t1 }, ObjectKind::Text { x: c1, y: c2, w: c3, h: c4, text: t2 }) => (a1, a2, a3, a4, t1) == (c1, c2, c3, c4, t2),
            (ObjectKind::Control { class: k1, caption: c1, style: s1, .. }, ObjectKind::Control { class: k2, caption: c2, style: s2, .. }) => (k1, c1, s1) == (k2, c2, s2),
            (ObjectKind::Group { children: g1 }, ObjectKind::Group { children: g2 }) => g1 == g2,
            (ObjectKind::View3d { x: a1, y: a2, w: a3, h: a4, space: s1, camera: c1 }, ObjectKind::View3d { x: b1, y: b2, w: b3, h: b4, space: s2, camera: c2 }) =>
                (a1, a2, a3, a4, s1, c1) == (b1, b2, b3, b4, s2, c2),
            (ObjectKind::Unknown { .. }, ObjectKind::Unknown { .. }) => true,
            _ => false,
        };
        check(same, &format!("объект #{}: тело", x.handle))?;
    }
    check(a.pens.len() == b.pens.len() && a.pens.iter().zip(&b.pens).all(|(p, q)| (p.handle, p.color, p.style, p.width, p.rop) == (q.handle, q.color, q.style, q.width, q.rop)), "перья")?;
    check(a.brushes.len() == b.brushes.len() && a.brushes.iter().zip(&b.brushes).all(|(p, q)| (p.handle, p.color, p.style, p.hatch, p.rop, p.dib) == (q.handle, q.color, q.style, q.hatch, q.rop, q.dib)), "кисти")?;
    check(a.fonts.len() == b.fonts.len() && a.fonts.iter().zip(&b.fonts).all(|(p, q)| (p.handle, p.height, p.weight, p.italic, p.underline, &p.face) == (q.handle, q.height, q.weight, q.italic, q.underline, &q.face)), "шрифты")?;
    check(a.strings.len() == b.strings.len() && a.strings.iter().zip(&b.strings).all(|(p, q)| (p.handle, &p.text) == (q.handle, &q.text)), "строки")?;
    check(a.texts.len() == b.texts.len() && a.texts.iter().zip(&b.texts).all(|(p, q)| p.handle == q.handle && p.parts.len() == q.parts.len() && p.parts.iter().zip(&q.parts).all(|(u, v)| (u.fg, u.bg, u.font, u.string) == (v.fg, v.bg, v.font, v.string))), "тексты")?;
    check(a.dibs.len() == b.dibs.len() && a.dibs.iter().zip(&b.dibs).all(|(p, q)| (p.handle, &p.bmp, &p.mask, &p.file, p.double) == (q.handle, &q.bmp, &q.mask, &q.file, q.double)), "растры")?;
    Ok(())
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

fn same_class(a: &cls::Class, b: &cls::Class, what: &str) -> Result<(), String> {
    let check = |ok: bool, field: &str| if ok { Ok(()) } else { Err(format!("{what} [{}]: расходится {field}", a.name)) };
    check(a.name == b.name, "имя")?;
    check(a.description == b.description, "описание")?;
    check(a.text.replace("\r\n", "\n") == b.text.replace("\r\n", "\n"), "текст")?;
    check(a.vars.len() == b.vars.len(), "число переменных")?;
    for (x, y) in a.vars.iter().zip(&b.vars) {
        check(x.name == y.name && x.var_type == y.var_type && x.default == y.default && x.description == y.description && x.flags == y.flags, "переменная")?;
    }
    check(a.children.len() == b.children.len(), "число детей")?;
    for (x, y) in a.children.iter().zip(&b.children) {
        check(x.class_name == y.class_name && x.handle == y.handle && x.name == y.name && x.x == y.x && x.y == y.y && x.flags == y.flags, "ребёнок")?;
    }
    check(a.links.len() == b.links.len(), "число связей")?;
    for (x, y) in a.links.iter().zip(&b.links) {
        check(x.source == y.source && x.target == y.target && x.handle == y.handle && x.flags == y.flags && x.vars == y.vars, "связь")?;
    }
    check(a.icon == b.icon, "иконка")?;
    check(a.image == b.image, "рисунок")?;
    check(a.scheme == b.scheme, "схема")?;
    check(a.equations == b.equations, "уравнения")?;
    check(a.icon_file == b.icon_file && a.icon_index == b.icon_index, "ссылка на иконку")?;
    check(a.flags == b.flags && a.timestamp == b.timestamp, "флаги/время")
}

/// Критерий приёмки этапа 3: каждый `.cls` корпуса переживает запись и
/// повторное чтение без потерь (кроме байт-кода, который мы не храним).
#[test]
fn every_class_survives_cls_round_trip() {
    let Some(root) = fixtures() else { return };
    let mut files = Vec::new();
    collect(&root, &mut files);
    let mut failures = Vec::new();
    let mut count = 0;
    for file in &files {
        let data = std::fs::read(file).unwrap();
        let Ok(a) = cls::parse(&data, &file.display().to_string()) else { continue };
        count += 1;
        let again = cls::write(&a);
        match cls::parse(&again, "again") {
            Ok(b) => {
                if let Err(e) = same_class(&a, &b, &file.display().to_string()) {
                    failures.push(e);
                }
            }
            Err(e) => failures.push(format!("{}: {e}", file.display())),
        }
    }
    assert!(count > 600, "разобрано только {count} имиджей");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Импорт всех примеров в родной формат и экспорт обратно в Stratum 2000:
/// после двух преобразований проект читается и совпадает с исходным.
#[test]
fn every_sample_project_survives_import_export() {
    let Some(root) = fixtures() else { return };
    let libs = libraries(&root);
    let work = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/roundtrip");
    let _ = std::fs::remove_dir_all(&work);
    let mut failures = Vec::new();
    let mut count = 0;
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("PROJECTS/samples")).unwrap().map(|e| e.unwrap().path()).collect();
    dirs.push(root.join("user/solar_system"));
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let Some(spj) = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).find(|p| {
            p.extension().is_some_and(|e| e.eq_ignore_ascii_case("spj"))
                && std::fs::read(p).map(|d| d.starts_with(b"Ih")).unwrap_or(false)
        }) else {
            continue;
        };
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let Ok(original) = formats::load_project(&spj, &libs).unwrap() else { continue };
        count += 1;
        let native_dir = work.join(&name).join("native");
        let back_dir = work.join(&name).join("stratum2000");
        std::fs::create_dir_all(&native_dir).unwrap();
        formats::native::save(&native_dir, &original).unwrap();
        let imported = match formats::load_project(&native_dir, &libs).unwrap() {
            Ok(p) => p,
            Err(e) => {
                failures.push(format!("{name}: импорт: {e}"));
                continue;
            }
        };
        formats::native::export_stratum2000(&back_dir, &imported).unwrap();
        let exported = match formats::load_project(&back_dir, &libs).unwrap() {
            Ok(p) => p,
            Err(e) => {
                failures.push(format!("{name}: экспорт: {e}"));
                continue;
            }
        };
        for (which, p) in [("родной", &imported), ("экспорт", &exported)] {
            if p.project.root != original.project.root || p.own_classes != original.own_classes {
                failures.push(format!("{name}: {which}: корень или число имиджей"));
                continue;
            }
            if p.project.properties.len() != original.project.properties.len() || p.project.variables.len() != original.project.variables.len() {
                failures.push(format!("{name}: {which}: свойства проекта"));
            }
            if which == "родной" && p.state.as_ref().map(|s| s.images.len()) != original.state.as_ref().map(|s| s.images.len()) {
                failures.push(format!("{name}: {which}: снимок состояния"));
            }
            for a in &original.classes[..original.own_classes] {
                match p.class(&a.name) {
                    Some(b) => {
                        if let Err(e) = same_class(a, b, &format!("{name}: {which}")) {
                            failures.push(e);
                        }
                    }
                    None => failures.push(format!("{name}: {which}: нет имиджа {}", a.name)),
                }
            }
        }
    }
    assert!(count >= 40, "проверено только {count} проектов");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Трёхмерное пространство в рисунке: проекция (тип 24) и инструмент `3D`
/// с объектами, камерой и группой (EDS_IND, номера сверены в Wine,
/// tools/verify/load3d.txt).
#[test]
fn space3d_in_a_picture_is_read() {
    use stratum_core::formats::vdr::{self, Object3dKind, ObjectKind};
    let Some(root) = fixtures() else { return };
    let data = std::fs::read(root.join("PROJECTS/samples/EDS_IND/3D_PICT.VDR")).unwrap();
    let pic = vdr::parse(&data, "3D_PICT.VDR").unwrap();
    let view = pic.objects.iter().find(|o| o.handle == 1).unwrap();
    assert!(matches!(view.kind, ObjectKind::View3d { camera: 10, .. }), "{:?}", view.kind);
    assert_eq!(pic.spaces3d.len(), 1);
    let sp = &pic.spaces3d[0];
    let named = |n: &str| sp.objects.iter().find(|o| o.name == n).map(|o| o.handle);
    assert_eq!((named("pol"), named("ramka"), named("ramka_tok"), named("Sc Default Camera")), (Some(1), Some(2), Some(11), Some(10)));
    assert!(sp.objects.iter().any(|o| matches!(&o.kind, Object3dKind::Group { children, .. } if children == &vec![3, 2])));
    // запись в 3.0 и повторное чтение сохраняют пространство
    let back = vdr::parse(&vdr::write(&pic), "again").unwrap();
    assert_eq!(back.spaces3d[0].objects.len(), sp.objects.len());
}
