//! Охранные и случайные проверки (фиксированные зерна): повреждённые
//! архивы, картинки, меши, скрипты и сцены не должны паниковать, зависать
//! или раздувать память. Образцы - testdata/ и fixtures/ (если есть).

use super::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn c(fx: &mut Effects, name: &str, args: &[Value]) -> Value {
    super::super::extra::call(name, args, fx).unwrap_or_else(|| panic!("no {name}"))
}
fn hv(v: &Value) -> Value {
    Value::Handle(v.as_float())
}
fn st(s: &str) -> Value {
    Value::Str(s.into())
}
fn f(x: f64) -> Value {
    Value::Float(x)
}

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

fn mutate(data: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut v = data.to_vec();
    if v.is_empty() {
        return v;
    }
    for _ in 0..1 + rng.below(6) {
        let n = v.len();
        if n == 0 {
            break;
        }
        match rng.below(6) {
            0 => v.truncate(rng.below(n)),
            1 => {
                let i = rng.below(n);
                v[i] ^= 1 << rng.below(8);
            }
            2 => {
                let i = rng.below(n);
                v[i] = rng.next() as u8;
            }
            3 => {
                // крайние значения в слове
                let i = rng.below(n.saturating_sub(4).max(1));
                let w = [0u32, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff, 0xffff][rng.below(6)].to_le_bytes();
                for (k, b) in w.iter().enumerate() {
                    if i + k < v.len() {
                        v[i + k] = *b;
                    }
                }
            }
            4 => {
                let (a, b) = (rng.below(n), rng.below(n));
                let (a, b) = (a.min(b), a.max(b));
                let piece: Vec<u8> = v[a..b].to_vec();
                let at = rng.below(v.len());
                v.splice(at..at, piece.into_iter().take(64));
            }
            _ => {
                let i = rng.below(n);
                v[i] = [0, 0xff, 0x7f, 0x80][rng.below(4)];
            }
        }
    }
    v
}

fn collect(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, exts, out);
        } else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| exts.contains(&x.to_ascii_lowercase().as_str())) {
            out.push(p);
        }
    }
}

fn corpus(exts: &[&str], max_len: usize, take: usize) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    collect(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/ogre/testdata"), exts, &mut files);
    collect(&&Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples"), exts, &mut files);
    files.sort();
    let mut out = Vec::new();
    for p in files {
        if let Ok(d) = std::fs::read(&p) {
            if d.len() <= max_len {
                out.push((p.file_name().unwrap().to_string_lossy().to_string(), d));
            }
        }
        if out.len() >= take {
            break;
        }
    }
    out
}

/// Выполняет `f` и требует уложиться во время.
fn timed<R>(what: &str, limit: Duration, f: impl FnOnce() -> R) -> R {
    let t = Instant::now();
    let r = f();
    assert!(t.elapsed() < limit, "{what}: {:?}", t.elapsed());
    r
}

fn deflate_stored(data: &[u8]) -> Vec<u8> {
    // сырой deflate из хранимых блоков
    let mut out = Vec::new();
    let mut chunks = data.chunks(65535).peekable();
    if data.is_empty() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }
    while let Some(c) = chunks.next() {
        out.push(if chunks.peek().is_none() { 1 } else { 0 });
        out.extend_from_slice(&(c.len() as u16).to_le_bytes());
        out.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        out.extend_from_slice(c);
    }
    out
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut v = vec![0x78, 0x01];
    v.extend(deflate_stored(data));
    v
}

fn crc_less_chunk(ty: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut v = (body.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(ty);
    v.extend_from_slice(body);
    v.extend_from_slice(&[0; 4]);
    v
}

fn png_with(w: u32, h: u32, depth: u8, ct: u8, interlace: u8, idat: &[u8]) -> Vec<u8> {
    let mut v = vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[depth, ct, 0, 0, interlace]);
    v.extend(crc_less_chunk(b"IHDR", &ihdr));
    v.extend(crc_less_chunk(b"IDAT", idat));
    v.extend(crc_less_chunk(b"IEND", &[]));
    v
}

/// Zip с одной записью; размеры можно подделать.
fn zip_one(name: &str, method: u16, comp: &[u8], usize_: u32, csize: u32) -> Vec<u8> {
    let mut z = Vec::new();
    z.extend_from_slice(&0x04034B50u32.to_le_bytes());
    z.extend_from_slice(&[20, 0, 0, 0]);
    z.extend_from_slice(&method.to_le_bytes());
    z.extend_from_slice(&[0; 8]);
    z.extend_from_slice(&csize.to_le_bytes());
    z.extend_from_slice(&usize_.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&[0, 0]);
    z.extend_from_slice(name.as_bytes());
    z.extend_from_slice(comp);
    let cd = z.len();
    z.extend_from_slice(&0x02014B50u32.to_le_bytes());
    z.extend_from_slice(&[20, 0, 20, 0, 0, 0]);
    z.extend_from_slice(&method.to_le_bytes());
    z.extend_from_slice(&[0; 8]);
    z.extend_from_slice(&csize.to_le_bytes());
    z.extend_from_slice(&usize_.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&[0; 12]);
    z.extend_from_slice(&0u32.to_le_bytes());
    z.extend_from_slice(name.as_bytes());
    let cdlen = z.len() - cd;
    z.extend_from_slice(&0x06054B50u32.to_le_bytes());
    z.extend_from_slice(&[0; 4]);
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&(cdlen as u32).to_le_bytes());
    z.extend_from_slice(&(cd as u32).to_le_bytes());
    z.extend_from_slice(&[0, 0]);
    z
}

// ---- картинки -----------------------------------------------------------------

#[test]
fn mutated_images_do_not_panic_or_hang() {
    let seeds = corpus(&["png", "jpg", "jpeg", "bmp", "tga"], 200_000, 40);
    let mut rng = Rng(0x1234_ABCD_9876_5432);
    for (name, data) in &seeds {
        for round in 0..30 {
            let m = mutate(data, &mut rng);
            timed(&format!("{name} #{round}"), Duration::from_secs(20), || {
                let _ = image::decode(name, &m);
            });
        }
    }
}

#[test]
fn truncated_images_at_every_cut_do_not_panic() {
    for (name, data) in corpus(&["png", "jpg"], 4000, 12) {
        for cut in 0..data.len() {
            let _ = image::decode(&name, &data[..cut]);
        }
    }
}

#[test]
fn short_jpeg_frame_header_is_rejected_not_a_panic() {
    // SOF0 с сегментом короче 6 байт
    for seg_len in 2..8usize {
        let mut j = vec![0xFF, 0xD8, 0xFF, 0xC0];
        j.extend_from_slice(&(seg_len as u16).to_be_bytes());
        j.extend(std::iter::repeat_n(8u8, seg_len.saturating_sub(2)));
        j.extend_from_slice(&[0xFF, 0xD9]);
        assert!(image::decode("a.jpg", &j).is_none());
    }
}

#[test]
fn jpeg_dc_predictor_overflow_is_not_a_panic() {
    // много блоков с максимальным DC-приращением: сумма выходит за i32
    let base = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/ogre/testdata/base_g.jpg")).unwrap();
    let mut rng = Rng(7);
    for _ in 0..200 {
        let mut m = base.clone();
        for b in m.iter_mut().skip(300) {
            *b = if rng.below(2) == 0 { 0xff } else { 0 };
            if *b == 0xff {
                *b = 0xfe;
            }
        }
        let _ = image::decode("a.jpg", &m);
    }
}

#[test]
fn png_claiming_huge_size_with_tiny_data_fails_fast() {
    let idat = zlib_stored(&[0; 16]);
    for (w, h, d, ct, il) in [(4096, 4096, 16, 6, 0), (4096, 4096, 16, 6, 1), (4096, 4096, 1, 0, 1), (u32::MAX, u32::MAX, 8, 2, 0), (4097, 1, 8, 0, 0)] {
        let p = png_with(w, h, d, ct, il, &idat);
        timed("png", Duration::from_secs(5), || assert!(image::decode("a.png", &p).is_none()));
    }
}

#[test]
fn inflate_bomb_is_capped() {
    // 64 КБ нулей сжимаются в сотню байт; лимит выхода не должен превышаться
    let fixed_zero_run = {
        // фиксированный блок: литерал 0, затем повтор длины 258 с расстоянием 1 много раз
        let mut bits: Vec<u8> = Vec::new();
        let mut put = |v: u32, n: u32, rev: bool| {
            for i in 0..n {
                let bit = if rev { (v >> (n - 1 - i)) & 1 } else { (v >> i) & 1 };
                bits.push(bit as u8);
            }
        };
        put(1, 1, false);
        put(1, 2, false);
        put(0x30, 8, true); // литерал 0
        for _ in 0..2000 {
            put(0xC5, 8, true); // 285: длина 258
            put(0, 5, true); // расстояние 1
        }
        put(0, 7, true); // конец блока
        let mut out = vec![0u8; bits.len().div_ceil(8)];
        for (i, b) in bits.iter().enumerate() {
            out[i / 8] |= b << (i % 8);
        }
        out
    };
    assert!(image::inflate_raw(&fixed_zero_run, 1000).is_none());
    let full = image::inflate_raw(&fixed_zero_run, 1 << 20).unwrap();
    assert_eq!(full.len(), 1 + 2000 * 258);
}

// ---- zip ------------------------------------------------------------------------

#[test]
fn forged_zip_sizes_are_rejected() {
    let body = deflate_stored(b"hello");
    // заявленный размер больше действительного
    let z = zip::Zip::open(zip_one("a.txt", 8, &body, 100, body.len() as u32)).unwrap();
    assert!(z.read("a.txt").is_none());
    // сжатый размер за пределами файла
    let z = zip::Zip::open(zip_one("a.txt", 8, &body, 5, 0x7fff_ffff)).unwrap();
    assert!(z.read("a.txt").is_none());
    let z = zip::Zip::open(zip_one("a.txt", 8, &body, 5, 0xffff_ffff)).unwrap();
    assert!(z.read("a.txt").is_none());
    // заявленный размер больше предела записи
    let z = zip::Zip::open(zip_one("a.txt", 8, &body, 0xffff_ffff, body.len() as u32)).unwrap();
    assert!(z.read("a.txt").is_none());
    // хранение с разными размерами
    let z = zip::Zip::open(zip_one("a.txt", 0, b"abc", 10, 3)).unwrap();
    assert!(z.read("a.txt").is_none());
    // исправный
    let z = zip::Zip::open(zip_one("a.txt", 8, &body, 5, body.len() as u32)).unwrap();
    assert_eq!(z.read("a.txt").unwrap(), b"hello");
    // шифрование/неизвестный метод
    let z = zip::Zip::open(zip_one("a.txt", 99, &body, 5, body.len() as u32)).unwrap();
    assert!(z.read("a.txt").is_none());
}

#[test]
fn zip_names_with_traversal_are_dropped() {
    for n in ["../evil.txt", "a/../../evil.txt", "/abs.txt", "..\\evil.txt", "dir/"] {
        let z = zip::Zip::open(zip_one(n, 0, b"x", 1, 1)).unwrap();
        assert!(z.names().is_empty(), "{n}");
    }
}

#[test]
fn mutated_zip_does_not_panic_or_hang() {
    let body = deflate_stored(b"material test { technique { pass { } } }");
    let mut seeds = vec![zip_one("a/b.material", 8, &body, 40, body.len() as u32)];
    for (_, d) in corpus(&["zip"], 3 << 20, 2) {
        seeds.push(d);
    }
    let mut rng = Rng(0xDEAD_BEEF_1234_5678);
    for data in &seeds {
        let rounds = if data.len() > 100_000 { 25 } else { 400 };
        for _ in 0..rounds {
            let m = mutate(data, &mut rng);
            timed("zip", Duration::from_secs(20), || {
                if let Some(z) = zip::Zip::open(m) {
                    for n in z.names().into_iter().take(40) {
                        let _ = z.read(&n);
                    }
                }
            });
        }
    }
}

#[test]
fn zip_with_many_entries_to_one_blob_is_cheap_to_scan() {
    // 20000 записей `.material`, указывающих на один и тот же 4 МБ вспухающий поток
    let mut inner = Vec::new();
    inner.extend(std::iter::repeat_n(b' ', 3 << 20));
    let comp = deflate_stored(&inner);
    let one = zip_one("x0.material", 8, &comp, inner.len() as u32, comp.len() as u32);
    // собираем каталог вручную: файл один, записей много
    let local_end = one.windows(4).position(|w| w == [0x50, 0x4B, 0x01, 0x02]).unwrap();
    let mut z = one[..local_end].to_vec();
    let cd = z.len();
    let count = 5000usize;
    for i in 0..count {
        let name = format!("n{i}.material");
        z.extend_from_slice(&0x02014B50u32.to_le_bytes());
        z.extend_from_slice(&[20, 0, 20, 0, 0, 0]);
        z.extend_from_slice(&8u16.to_le_bytes());
        z.extend_from_slice(&[0; 8]);
        z.extend_from_slice(&(comp.len() as u32).to_le_bytes());
        z.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        z.extend_from_slice(&(name.len() as u16).to_le_bytes());
        z.extend_from_slice(&[0; 12]);
        z.extend_from_slice(&0u32.to_le_bytes());
        z.extend_from_slice(name.as_bytes());
    }
    let cdlen = z.len() - cd;
    z.extend_from_slice(&0x06054B50u32.to_le_bytes());
    z.extend_from_slice(&[0; 4]);
    z.extend_from_slice(&(count as u16).to_le_bytes());
    z.extend_from_slice(&(count as u16).to_le_bytes());
    z.extend_from_slice(&(cdlen as u32).to_le_bytes());
    z.extend_from_slice(&(cd as u32).to_le_bytes());
    z.extend_from_slice(&[0, 0]);
    let dir = std::env::temp_dir().join(format!("stratum-ogre-zipscan-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("bomb.zip"), &z).unwrap();
    let mut fx = Effects::default();
    fx.gfx.project_dir = dir.clone();
    let _sc = boot_plain(&mut fx);
    c(&mut fx, "Root_AddResourceLocation", &[st("bomb.zip"), st("Zip"), st("General"), f(0.0)]);
    timed("scan", Duration::from_secs(10), || {
        c(&mut fx, "Root_InitialiseAllResourceGroups", &[]);
    });
    let _ = std::fs::remove_dir_all(dir);
}

// ---- меши и скелеты -------------------------------------------------------------

#[test]
fn mutated_meshes_and_skeletons_do_not_panic() {
    let mut seeds = corpus(&["mesh", "skeleton"], 300_000, 12);
    // скелеты и меши внутри Sinbad.zip
    for (_, d) in corpus(&["zip"], 3 << 20, 1) {
        if let Some(z) = zip::Zip::open(d) {
            for n in z.names() {
                if n.ends_with(".mesh") || n.ends_with(".skeleton") {
                    if let Some(b) = z.read(&n) {
                        if b.len() < 400_000 {
                            seeds.push((n, b));
                        }
                    }
                }
            }
        }
    }
    let mut rng = Rng(0x0BAD_CAFE_F00D_1111);
    for (name, data) in &seeds {
        for _ in 0..60 {
            let m = mutate(data, &mut rng);
            timed(name, Duration::from_secs(10), || {
                let _ = parse_mesh(&m);
                let _ = parse_skeleton(&m);
            });
        }
    }
}

#[test]
fn truncated_mesh_at_every_cut_does_not_panic() {
    for (_, d) in corpus(&["mesh", "skeleton"], 20_000, 6) {
        for cut in (0..d.len()).step_by(3) {
            let _ = parse_mesh(&d[..cut]);
            let _ = parse_skeleton(&d[..cut]);
        }
    }
}

// ---- скрипты --------------------------------------------------------------------

#[test]
fn mutated_scripts_do_not_panic_or_hang() {
    let seeds = corpus(&["material", "compositor", "particle", "overlay", "fontdef"], 100_000, 30);
    let mut rng = Rng(0x5151_5151_ABAB_CDCD);
    for (name, data) in &seeds {
        let ext = name.rsplit('.').next().unwrap().to_ascii_lowercase();
        for _ in 0..80 {
            let m = mutate(data, &mut rng);
            let text = String::from_utf8_lossy(&m).to_string();
            timed(name, Duration::from_secs(10), || {
                let mut o = Ogre::default();
                o.ingest_script(&ext, &text);
                let mut fx = Effects::default();
                let _ = &mut fx;
            });
        }
    }
}

#[test]
fn deeply_nested_and_huge_scripts_are_bounded() {
    let mut o = Ogre::default();
    let deep = "{".repeat(200_000);
    for ext in ["material", "compositor", "particle", "overlay"] {
        timed(ext, Duration::from_secs(10), || o.ingest_script(ext, &deep));
        let many = format!("{} {{ {} }}", ext, "x ".repeat(500_000));
        timed(ext, Duration::from_secs(10), || o.ingest_script(ext, &many));
        let rep = format!("{ext} a {{ ").repeat(100_000);
        timed(ext, Duration::from_secs(10), || o.ingest_script(ext, &rep));
    }
    let names: String = (0..30_000).map(|i| format!("material m{i} {{ technique {{ pass {{ }} }} }}\n")).collect();
    timed("many materials", Duration::from_secs(20), || o.ingest_script("material", &names));
    assert!(o.objs.len() <= MAX_OBJECTS + 8);
}

fn boot_plain(fx: &mut Effects) -> Value {
    c(fx, "Root_Create", &[st(""), st("none.cfg"), st("")]);
    c(fx, "Root_Initialise", &[]);
    c(fx, "RenderWindow_Create2", &[Value::Handle(0.0), st("t"), f(64.0), f(48.0), f(0.0), f(0.0), f(0.0), f(0.0)]);
    c(fx, "Scene_Create", &[st("OctreeSceneManager")])
}

#[test]
fn mutated_scene_files_do_not_panic() {
    let seeds = corpus(&["scene"], 200_000, 6);
    let mut rng = Rng(0x7777_1111_2222_3333);
    let dir = std::env::temp_dir().join(format!("stratum-ogre-scenefuzz-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (_, data) in &seeds {
        for round in 0..25 {
            let m = if round == 0 { data.clone() } else { mutate(data, &mut rng) };
            std::fs::write(dir.join("a.scene"), &m).unwrap();
            let mut fx = Effects::default();
            fx.gfx.project_dir = dir.clone();
            let sc = boot_plain(&mut fx);
            timed("scene", Duration::from_secs(20), || {
                c(&mut fx, "Scene_Load", &[hv(&sc), st("a.scene")]);
                let _ = fx.ogre.frame_svg();
            });
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

// ---- рендер с враждебными числами -----------------------------------------------

#[test]
fn render_with_hostile_numbers_is_bounded() {
    let hostile = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e300, -1e300, 1e-300, 0.0, 3.0e38, 4.0e9, -1.0];
    let mut rng = Rng(0x4242_4242_4242_4242);
    for round in 0..40 {
        let mut fx = Effects::default();
        let sc = boot_plain(&mut fx);
        let mo = c(&mut fx, "ManualObject_Create", &[hv(&sc), st("m")]);
        c(&mut fx, "ManualObject_Begin", &[hv(&mo), st(""), f(4.0)]);
        for _ in 0..30 {
            let p = [0, 1, 2].map(|_| if rng.below(3) == 0 { hostile[rng.below(hostile.len())] } else { (rng.below(2000) as f64 - 1000.0) / 10.0 });
            c(&mut fx, "ManualObject_Position", &[hv(&mo), f(p[0]), f(p[1]), f(p[2])]);
        }
        c(&mut fx, "ManualObject_End", &[hv(&mo)]);
        let n = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("n")]);
        c(&mut fx, "SceneNode_AttachObject", &[hv(&n), hv(&mo)]);
        let h = |rng: &mut Rng| if rng.below(2) == 0 { hostile[rng.below(hostile.len())] } else { rng.below(100) as f64 - 50.0 };
        c(&mut fx, "Node_SetPosition", &[hv(&n), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        c(&mut fx, "Node_SetScale", &[hv(&n), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        c(&mut fx, "Node_SetRotationAxis", &[hv(&n), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        let cam = c(&mut fx, "Camera_Create", &[hv(&sc), st("c")]);
        let cn = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("cn")]);
        c(&mut fx, "Node_SetPosition", &[hv(&cn), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        c(&mut fx, "SceneNode_AttachObject", &[hv(&cn), hv(&cam)]);
        c(&mut fx, "Camera_SetNearClipDistance", &[hv(&cam), f(h(&mut rng))]);
        c(&mut fx, "Camera_SetFarClipDistance", &[hv(&cam), f(h(&mut rng))]);
        c(&mut fx, "Camera_SetFOV", &[hv(&cam), f(h(&mut rng))]);
        let win = c(&mut fx, "RenderWindow_GetPrimary", &[]);
        let vp = c(&mut fx, "Viewport_Create", &[hv(&win), hv(&cam), f(0.0)]);
        c(&mut fx, "Viewport_SetDimensions", &[hv(&vp), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        c(&mut fx, "Scene_SetFog", &[hv(&sc), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng)), f(h(&mut rng))]);
        timed(&format!("render {round}"), Duration::from_secs(30), || {
            c(&mut fx, "Root_RenderOneFrame", &[]);
            let _ = fx.ogre.frame_svg();
        });
    }
}

#[test]
fn texture_and_object_tables_are_capped() {
    let mut fx = Effects::default();
    let sc = boot_plain(&mut fx);
    for _ in 0..(MAX_OBJECTS + 100) {
        c(&mut fx, "SceneNode_Create", &[hv(&sc), st("")]);
    }
    assert!(fx.ogre.objs.len() <= MAX_OBJECTS + 16);
    assert!(fx.ogre.errors > 0);
}

#[test]
fn frame_size_is_capped_by_pixels() {
    let mut fx = Effects::default();
    c(&mut fx, "Root_Create", &[st(""), st("none.cfg"), st("")]);
    c(&mut fx, "Root_Initialise", &[]);
    c(&mut fx, "RenderWindow_Create2", &[Value::Handle(0.0), st("t"), f(4096.0), f(4096.0), f(0.0), f(0.0), f(0.0), f(0.0)]);
    let fr = fx.ogre.render(None).unwrap();
    assert!(fr.width * fr.height <= render::MAX_FRAME_PIXELS && fr.width >= 2000);
}

#[test]
fn names_index_follows_create_and_destroy() {
    let mut fx = Effects::default();
    let sc = boot_plain(&mut fx);
    let a = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("Dup")]);
    assert_eq!(c(&mut fx, "SceneNode_Create", &[hv(&sc), st("dup")]).as_float(), 0.0);
    c(&mut fx, "SceneNode_Destroy", &[hv(&a)]);
    assert!(c(&mut fx, "SceneNode_Create", &[hv(&sc), st("DUP")]).as_float() >= 1.0);
}
