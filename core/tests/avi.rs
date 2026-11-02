//! Чтение AVI: синтетические файлы и образцы VIDEO/VIDEO2 из корпуса.

use stratum_core::formats::avi::{build_rgb24, Avi, AviError};

fn frames(w: u32, h: u32, n: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|k| (0..w * h).flat_map(|i| [(i % w) as u8 * 20, (i / w) as u8 * 20, k as u8 * 50]).collect())
        .collect()
}

#[test]
fn rgb24_roundtrip() {
    let f = frames(5, 3, 3);
    let mut avi = Avi::parse(build_rgb24(5, 3, &f, 25)).unwrap();
    assert_eq!((avi.width(), avi.height(), avi.frame_count()), (5, 3, 3));
    assert!((avi.fps() - 25.0).abs() < 1e-9);
    for (i, fr) in f.iter().enumerate() {
        assert_eq!(&avi.frame_rgb(i).unwrap(), fr);
    }
    assert_eq!(avi.frame_rgb(3), Err(AviError::NoFrame));
}

/// Меняет в готовом AVI поле сжатия и добавляет кадры другого кодека.
fn with_codec(w: u32, h: u32, bpp: u16, comp: &[u8; 4], payloads: &[Vec<u8>]) -> Vec<u8> {
    let mut v = build_rgb24(w, h, &frames(w, h, 1), 10);
    let strf = v.windows(4).position(|x| x == b"strf").unwrap() + 8;
    v[strf + 14..strf + 16].copy_from_slice(&bpp.to_le_bytes());
    v[strf + 16..strf + 20].copy_from_slice(comp);
    // выкидываем исходный кадр и дописываем свои в конец movi
    let movi = v.windows(4).position(|x| x == b"movi").unwrap();
    v.truncate(movi + 4);
    for p in payloads {
        v.extend_from_slice(b"00dc");
        v.extend_from_slice(&(p.len() as u32).to_le_bytes());
        v.extend_from_slice(p);
        if p.len() & 1 == 1 {
            v.push(0);
        }
    }
    let total = v.len() as u32 - 8;
    v[4..8].copy_from_slice(&total.to_le_bytes());
    let list = v[..movi].windows(4).rposition(|x| x == b"LIST").unwrap();
    let ls = (v.len() - list - 8) as u32;
    v[list + 4..list + 8].copy_from_slice(&ls.to_le_bytes());
    v
}

#[test]
fn rle8_encoded_and_delta() {
    // палитры нет: индекс = серый не гарантируется, проверяем индексы через цвет (0,0,0)
    // кадр 1: строка 0 (нижняя): 4 пикселя 7; строка 1: абсолютный прогон 1,2,3,4
    let f1 = vec![4, 7, 0, 0, 0, 4, 1, 2, 3, 4, 0, 1];
    // кадр 2: сдвиг (1,0) и один пиксель 9 в нижней строке
    let f2 = vec![0, 2, 1, 0, 1, 9, 0, 1];
    let v = with_codec(4, 2, 8, &[1, 0, 0, 0], &[f1, f2]);
    let mut avi = Avi::parse(v).unwrap();
    assert_eq!(avi.frame_count(), 2);
    assert!(avi.frame_rgb(0).is_ok());
    assert!(avi.frame_rgb(1).is_ok());
    assert!(avi.frame_rgb(0).is_ok());
}

#[test]
fn msvc1_one_color_blocks() {
    // 8x4 16-бит: блок 1 - один цвет 0x7fff (белый), блок 2 - пропуск
    let data = vec![0xff, 0xff, 0x01, 0x84];
    let v = with_codec(8, 4, 16, b"CRAM", &[data]);
    let mut avi = Avi::parse(v).unwrap();
    let rgb = avi.frame_rgb(0).unwrap();
    assert_eq!(&rgb[0..3], &[255, 255, 255]);
    assert_eq!(&rgb[4 * 3..4 * 3 + 3], &[0, 0, 0]);
}

#[test]
fn unknown_codec_is_an_error_not_a_panic() {
    let v = with_codec(4, 4, 24, b"IV50", &[vec![1, 2, 3]]);
    assert!(matches!(Avi::parse(v), Err(AviError::Unsupported(_))));
    assert!(Avi::parse(vec![]).is_err());
    assert!(Avi::parse(b"RIFF\0\0\0\0AVI ".to_vec()).is_err());
}

fn sample(rel: &str) -> Option<Vec<u8>> {
    let root = std::env::var_os("STRATUM_FIXTURES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures"));
    std::fs::read(root.join(rel)).ok()
}

#[test]
fn sample_scene_avi_decodes() {
    let Some(d) = sample("PROJECTS/samples/VIDEO2/_SCENE.AVI") else { return };
    let mut avi = Avi::parse(d).unwrap();
    assert_eq!((avi.width(), avi.height()), (320, 240));
    assert!(avi.frame_count() > 50);
    let last = avi.frame_count() - 1;
    let rgb = avi.frame_rgb(last).unwrap();
    assert_eq!(rgb.len(), 320 * 240 * 3);
    assert!(rgb.iter().any(|&b| b != 0));
    if let Some(out) = std::env::var_os("AVI_DUMP") {
        let mut ppm = b"P6\n320 240\n255\n".to_vec();
        ppm.extend_from_slice(&rgb);
        std::fs::write(out, ppm).unwrap();
    }
}

mod model {
    use super::*;
    use stratum_core::gfx::{api, Gfx, Shape, Space};
    use stratum_core::runtime::value::Value;

    fn call(g: &mut Gfx, n: &str, a: &[f64]) -> f64 {
        let args: Vec<Value> = a.iter().map(|x| Value::Float(*x)).collect();
        api::call(n, &args, g).unwrap().as_float()
    }

    fn open(g: &mut Gfx, name: &str) -> f64 {
        api::call("OpenVideo", &[Value::Str(name.into()), Value::Float(16.0)], g).unwrap().as_float()
    }

    #[test]
    fn frames_follow_position() {
        let dir = std::env::temp_dir().join(format!("stratum_avi_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("t.avi"), build_rgb24(4, 2, &frames(4, 2, 3), 10)).unwrap();
        std::fs::write(dir.join("bad.avi"), b"RIFF\0\0\0\0AVI junk").unwrap();
        let mut g = Gfx::new();
        g.project_dir = dir.clone();
        g.spaces.insert(1, Space::new(1, "w"));
        assert_eq!(open(&mut g, "missing.avi"), 0.0);
        assert_eq!(open(&mut g, "bad.avi"), 0.0);
        let v = open(&mut g, "t.avi");
        assert!(v > 0.0);
        let fr = call(&mut g, "CreateVideoFrame2d", &[1.0, v, 5.0, 6.0, -1.0, -1.0, 0.0]);
        assert!(fr > 0.0);
        let o = &g.space(1).unwrap().objects[&(fr as u32)];
        assert!(matches!(o.shape, Shape::Bitmap { .. }) && o.w == 4.0 && o.h == 2.0);
        assert_eq!(call(&mut g, "FrameGetVideo2d", &[1.0, fr]), v);
        assert_eq!(call(&mut g, "VideoSetPos2d", &[v, 2.0]), 2.0);
        assert_eq!(call(&mut g, "VideoSetPos2d", &[v, 99.0]), 2.0);
        let dib = g.space(1).unwrap().dibs.values().next().unwrap().clone();
        let (_, _, rgb) = stratum_core::gfx::decode_bmp(&dib.bmp).unwrap();
        assert_eq!(rgb[2], 100);
        assert_eq!(call(&mut g, "CloseVideo", &[v]), 0.0);
        assert_eq!(call(&mut g, "VideoPlay2d", &[1.0, fr, 0.0, -1.0, 0.0, 0.0]), 1.0);
        g.advance_videos();
        assert_eq!(call(&mut g, "VideoGetPos2d", &[v]), 1.0);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn sample_video2_opens() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples/VIDEO2");
        if !root.is_dir() {
            return;
        }
        let mut g = Gfx::new();
        g.project_dir = root;
        g.spaces.insert(1, Space::new(1, "w"));
        let v = open(&mut g, "..\\_SCENE.AVI");
        assert!(v > 0.0, "{}", g.video_error);
        let fr = call(&mut g, "CreateVideoFrame2d", &[1.0, v, 0.0, 0.0, -1.0, -1.0, 0.0]);
        assert!(fr > 0.0);
        assert!(call(&mut g, "VideoSetPos2d", &[v, 50.0]) == 50.0);
    }
}
