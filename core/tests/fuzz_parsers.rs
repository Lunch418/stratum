//! Разбор чужих файлов не должен падать: обрезанные и испорченные `.cls`,
//! `.vdr`, `.stt`, `.spj`, `.dbm`/`.bmp` из корпуса подаются в разборщики, и
//! ни один не должен паниковать (паника в потоке симуляции отравляет общее
//! состояние сервера IDE). Корпус — `fixtures/` рядом с `core/` или папка из
//! `STRATUM_FIXTURES`; без него проверка пропускается.
//!
//! Мутации детерминированы (свой генератор), чтобы падение повторялось.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use stratum_core::formats::{cls, project, vdr};
use stratum_core::gfx::{self, svg, Space};

fn fixtures() -> Option<PathBuf> {
    let p = std::env::var_os("STRATUM_FIXTURES")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures"));
    if p.is_dir() {
        Some(p)
    } else {
        eprintln!("корпус не найден ({}) — проверка разборщиков пропущена", p.display());
        None
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// xorshift64: воспроизводимые мутации без внешних крейтов.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// Обрезки и порченые копии файла.
fn mutants(data: &[u8], seed: u64, count: usize) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let n = data.len();
    for cut in [0, 1, 2, 3, 7, 16, n / 3, n / 2, n.saturating_sub(4), n.saturating_sub(1)] {
        if cut <= n {
            out.push(data[..cut].to_vec());
        }
    }
    if n == 0 {
        return out;
    }
    let mut rng = Rng(seed | 1);
    for i in 0..count {
        let mut m = data.to_vec();
        match i % 4 {
            // случайные байты
            0 => {
                for _ in 0..1 + rng.below(8) {
                    let at = rng.below(n);
                    m[at] = rng.next() as u8;
                }
            }
            // «огромное» 32-битное поле: длины и счётчики
            1 => {
                let at = rng.below(n);
                for (k, b) in [0xff, 0xff, 0xff, 0x7f].iter().enumerate() {
                    if let Some(x) = m.get_mut(at + k) {
                        *x = *b;
                    }
                }
            }
            // отрицательное / нулевое 16-битное поле
            2 => {
                let at = rng.below(n);
                let v: [u8; 2] = if rng.next() & 1 == 0 { [0xff, 0xff] } else { [0, 0] };
                for (k, b) in v.iter().enumerate() {
                    if let Some(x) = m.get_mut(at + k) {
                        *x = *b;
                    }
                }
            }
            // порча и обрезка вместе
            _ => {
                let at = rng.below(n);
                m[at] ^= 0x80;
                m.truncate(at + rng.below(n - at) + 1);
            }
        }
        out.push(m);
    }
    out
}

fn picture(data: &[u8], name: &str) {
    if let Ok(pic) = vdr::parse(data, name) {
        let mut sp = Space::new(1, name);
        sp.load(&pic);
        let _ = svg::render(&sp);
        let _ = vdr::write(&pic);
    }
}

fn bitmap(data: &[u8]) {
    let _ = gfx::bmp_size(data);
    let _ = gfx::decode_bmp(data);
    let mut dib = gfx::Dib::new(data.to_vec(), Vec::new(), None);
    let _ = dib.pixel(0, 0);
    let _ = dib.set_pixel(1, 1, 0xffffff);
    dib.flush();
}

fn check(path: &Path, data: &[u8]) {
    let name = path.display().to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "cls" => {
            if let Ok(c) = cls::parse(data, &name) {
                for blob in [&c.icon, &c.image, &c.scheme].into_iter().flatten() {
                    picture(blob, &name);
                }
                let _ = stratum_core::lang::parse(&c.text);
                let _ = cls::write(&c);
            }
        }
        "vdr" => picture(data, &name),
        "stt" => {
            if let Ok(st) = project::parse_state(data, &name) {
                let _ = project::write_state(&st);
            }
        }
        "spj" | "prj" => {
            if let Ok(p) = project::parse_project(data, &name) {
                let _ = project::write_project(&p);
            }
        }
        "dbm" | "bmp" => bitmap(data),
        _ => {}
    }
}

#[test]
fn damaged_files_do_not_panic() {
    let Some(root) = fixtures() else { return };
    let mut files = Vec::new();
    collect(&root, &mut files);
    files.retain(|p| {
        let e = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        matches!(e.as_str(), "cls" | "vdr" | "stt" | "spj" | "prj" | "dbm" | "bmp")
    });
    files.sort();
    // STRATUM_FUZZ_SCALE=N — в N раз больше мутаций и все .cls (долгий прогон);
    // по умолчанию корпус большой, поэтому берётся каждый седьмой .cls
    let scale: usize = std::env::var("STRATUM_FUZZ_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
    let mut i = 0;
    files.retain(|p| {
        i += 1;
        scale > 1 || !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("cls")) || i % 7 == 0
    });
    let mut failures = Vec::new();
    for (k, path) in files.iter().enumerate() {
        let Ok(data) = std::fs::read(path) else { continue };
        // большие файлы мутируем реже: проверка должна идти секунды, а не минуты
        let count = if data.len() > 200_000 { 4 } else { 24 } * scale;
        for (j, m) in mutants(&data, 0x9e37_79b9_7f4a_7c15 ^ k as u64, count).into_iter().enumerate() {
            if catch_unwind(AssertUnwindSafe(|| check(path, &m))).is_err() {
                failures.push(format!("{} (мутация {j}, {} байт)", path.display(), m.len()));
            }
        }
    }
    assert!(failures.is_empty(), "разбор паникует на {} файлах:\n{}", failures.len(), failures.join("\n"));
}

/// Без корпуса — хотя бы вручную собранные испорченные BMP.
#[test]
fn crafted_bitmaps_do_not_panic() {
    let mut bmp = vec![0u8; 54];
    bmp[0..2].copy_from_slice(b"BM");
    // заголовок DIB длиннее файла
    bmp[14..18].copy_from_slice(&0x7fff_0000u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&4i32.to_le_bytes());
    bmp[22..26].copy_from_slice(&4i32.to_le_bytes());
    bmp[28..30].copy_from_slice(&8u16.to_le_bytes());
    assert!(catch_unwind(|| bitmap(&bmp)).is_ok(), "заголовок за концом файла");
    // огромные размеры без данных: не выделять гигабайты и не переполняться
    let mut big = vec![0u8; 54];
    big[0..2].copy_from_slice(b"BM");
    big[10..14].copy_from_slice(&54u32.to_le_bytes());
    big[14..18].copy_from_slice(&40u32.to_le_bytes());
    big[18..22].copy_from_slice(&100_000i32.to_le_bytes());
    big[22..26].copy_from_slice(&100_000i32.to_le_bytes());
    big[28..30].copy_from_slice(&24u16.to_le_bytes());
    assert!(catch_unwind(|| bitmap(&big)).is_ok(), "огромный растр без данных");
    assert!(gfx::decode_bmp(&big).is_none());
}

/// Текст имиджа из чужого файла с огромной вложенностью скобок: разбор
/// должен вернуть ошибку, а не переполнить стек (это роняет процесс).
#[test]
fn deeply_nested_text_does_not_overflow_stack() {
    for depth in [150usize, 1_000, 100_000] {
        for (open, close) in [("(", ")"), ("-", ""), ("if (1) ", " endif")] {
            let text = format!("x := {}1{}", open.repeat(depth), close.repeat(depth));
            let _ = std::thread::Builder::new()
                // как у потоков сервера IDE (по умолчанию 2 МБ)
                .stack_size(2 << 20)
                .spawn(move || {
                    let _ = stratum_core::lang::parse(&text);
                })
                .unwrap()
                .join();
        }
    }
}

/// Группа, входящая сама в себя (прямо или через другую группу), — порча
/// файла; рендер не должен уходить в бесконечную рекурсию (переполнение
/// стека не ловится и роняет весь процесс IDE).
#[test]
fn cyclic_groups_do_not_recurse_forever() {
    use stratum_core::formats::vdr::{Object, ObjectKind, Picture};
    let group = |handle: u16, children: Vec<u16>| Object { handle, name: String::new(), flags: 0, kind: ObjectKind::Group { children } };
    let line = Object { handle: 3, name: String::new(), flags: 0, kind: ObjectKind::Polyline { x: 0.0, y: 0.0, w: 10.0, h: 10.0, pen: 0, brush: 0, points: vec![(0.0, 0.0), (10.0, 10.0)] } };
    let pic = Picture { version: 0x0300, objects: vec![group(1, vec![2, 3]), group(2, vec![1]), group(4, vec![4]), line], zorder: vec![1, 2, 3, 4], ..Default::default() };
    let mut sp = Space::new(1, "x");
    sp.load(&pic);
    let _ = svg::render(&sp);
    let _ = vdr::write(&pic);
}

/// Наибольшая допустимая вложенность проходит и дальше разбора: сборка
/// модели и такт на стеке потока сервера.
#[test]
fn deepest_allowed_text_builds_and_runs() {
    use stratum_core::formats::{Class, LoadedProject, Project};
    let text = format!("x := {}1{}\ny := {}2", "(".repeat(195), ")".repeat(195), "-".repeat(195));
    std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(move || {
            assert!(stratum_core::lang::parse(&text).is_ok());
            let root = Class { name: "Main".into(), version: 0x3003, text, ..Default::default() };
            let project = LoadedProject {
                dir: PathBuf::new(),
                project: Project { root: "Main".into(), ..Default::default() },
                classes: vec![root],
                own_classes: 1,
                state: None,
                library_dirs: Vec::new(),
            };
            if let Ok(mut sim) = stratum_core::sim::Simulation::build(&project) {
                let _ = sim.step();
                let _ = sim.clone();
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

