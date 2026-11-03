//! MIDI и MOD из чужих файлов: разбор и отрисовка не паникуют, не зависают
//! и укладываются в пределы по длине и работе. Мутации детерминированы.

use std::time::{Duration, Instant};
use stratum_core::formats::{midi, sequencer, tracker};

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
    for _ in 0..1 + rng.below(8) {
        let n = v.len();
        if n == 0 {
            break;
        }
        match rng.below(4) {
            0 => v.truncate(1 + rng.below(n)),
            1 => {
                let i = rng.below(n);
                v[i] = rng.next() as u8;
            }
            2 => {
                let i = rng.below(n);
                v[i] = [0, 0xff, 0x7f, 0x80][rng.below(4)];
            }
            _ => {
                let i = rng.below(n);
                v[i] ^= 1 << rng.below(8);
            }
        }
    }
    v
}

fn samples() -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    fn walk(d: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| matches!(x.to_ascii_lowercase().as_str(), "mid" | "mod")) {
                if let Ok(b) = std::fs::read(&p) {
                    if b.len() < 600_000 {
                        out.push((p.to_string_lossy().to_string(), b));
                    }
                }
            }
        }
    }
    walk(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures"), &mut out);
    out.truncate(6);
    out
}

fn header(ntrk: u16, div: u16) -> Vec<u8> {
    let mut v = b"MThd\0\0\0\x06\0\x01".to_vec();
    v.extend_from_slice(&ntrk.to_be_bytes());
    v.extend_from_slice(&div.to_be_bytes());
    v
}

fn track(body: &[u8]) -> Vec<u8> {
    let mut v = b"MTrk".to_vec();
    v.extend_from_slice(&(body.len() as u32).to_be_bytes());
    v.extend_from_slice(body);
    v
}

fn timed<R>(what: &str, secs: u64, f: impl FnOnce() -> R) -> R {
    let t = Instant::now();
    let r = f();
    assert!(t.elapsed() < Duration::from_secs(secs), "{what}: {:?}", t.elapsed());
    r
}

#[test]
fn mutated_midi_and_mod_do_not_panic() {
    let mut rng = Rng(0xFEED_FACE_0123_4567);
    for (name, data) in samples() {
        for _ in 0..6 {
            let m = mutate(&data, &mut rng);
            timed(&name, 30, || {
                if let Some(s) = midi::parse(&m) {
                    let _ = midi::render(&s);
                }
                if let Some(t) = tracker::parse(&m) {
                    let _ = tracker::render(&t);
                }
            });
        }
    }
}

#[test]
fn midi_with_endless_held_notes_renders_in_bounded_time() {
    // 100000 нот органа по 360 секунд каждая: без предела работы это сотни миллиардов отсчётов
    let mut body = vec![0, 0xc0, 16]; // орган
    for i in 0..100_000u32 {
        body.extend_from_slice(&[0, 0x90, (i % 100) as u8 + 10, 100]);
    }
    body.extend_from_slice(&[0x87, 0xff, 0x7f, 0x80, 60, 0]); // закрыть что-то далеко
    let mut d = header(1, 480);
    d.extend(track(&body));
    let song = midi::parse(&d).expect("parse");
    assert!(song.notes.len() >= 1000);
    let pcm = timed("render", 40, || midi::render(&song));
    assert!(pcm.samples.len() as f64 / 2.0 <= (sequencer::MAX_SECONDS + 1.0) * sequencer::RATE as f64);
}

#[test]
fn midi_note_off_storm_is_linear() {
    // 100000 включений одной ноты и 300000 выключений другой
    let mut body = Vec::new();
    for _ in 0..100_000 {
        body.extend_from_slice(&[0, 0x90, 60, 90]);
    }
    for _ in 0..90_000 {
        body.extend_from_slice(&[0, 0x80, 61, 0]);
    }
    let mut d = header(1, 480);
    d.extend(track(&body));
    timed("parse", 5, || {
        let _ = midi::parse(&d);
    });
}

#[test]
fn midi_hostile_headers() {
    for (ntrk, div) in [(0u16, 0u16), (65535, 1), (1, 0x8000), (1, 0xff00), (1, 0xe728), (1, 0x7fff)] {
        let mut d = header(ntrk, div);
        d.extend(track(&[0, 0xff, 0x51, 3, 0xff, 0xff, 0xff, 0x83, 0xff, 0xff, 0x7f, 0x90, 60, 100, 0, 0x80, 60, 0]));
        if let Some(s) = midi::parse(&d) {
            let _ = timed("render", 30, || midi::render(&s));
        }
    }
    // длина дорожки за пределами файла, вложенные заголовки
    let mut d = header(4, 96);
    d.extend_from_slice(b"MTrk\xff\xff\xff\xff\0\x90\x3c\x40");
    let _ = midi::parse(&d).map(|s| midi::render(&s));
}

/// MOD на 4 канала с одной строкой: ячейка (канал, образец, период, команда, аргумент).
fn module(cells: &[(usize, u8, u16, u8, u8)], sample_hdr: impl Fn(&mut [u8])) -> Vec<u8> {
    let mut d = vec![0u8; 1084];
    for i in 0..31 {
        sample_hdr(&mut d[20 + i * 30..20 + i * 30 + 30]);
    }
    d[950] = 1;
    d[1080..1084].copy_from_slice(b"M.K.");
    let mut pat = vec![0u8; 64 * 16];
    for &(ch, smp, per, cmd, arg) in cells {
        let o = ch * 4;
        pat[o] = (smp & 0xf0) | (per >> 8) as u8;
        pat[o + 1] = per as u8;
        pat[o + 2] = (smp << 4) | cmd;
        pat[o + 3] = arg;
    }
    d.extend_from_slice(&pat);
    d.extend((0..2000).map(|i| (i * 7) as u8));
    d
}

#[test]
fn mod_sample_numbers_above_31_with_note_delay_do_not_panic() {
    // ED + номер образца 0xFF (старшие биты в первом байте ячейки)
    let d = module(&[(0, 0xff, 428, 0xe, 0xd2), (1, 0xf1, 300, 0xe, 0xd1)], |h| {
        h[22..24].copy_from_slice(&500u16.to_be_bytes());
        h[25] = 64;
    });
    let m = tracker::parse(&d).unwrap();
    let _ = tracker::render(&m);
}

#[test]
fn mod_loop_start_beyond_sample_end_does_not_underflow() {
    let d = module(&[(0, 1, 428, 0, 0)], |h| {
        h[22..24].copy_from_slice(&50u16.to_be_bytes()); // 100 байт
        h[25] = 64;
        h[26..28].copy_from_slice(&30000u16.to_be_bytes()); // начало цикла далеко за концом
        h[28..30].copy_from_slice(&1000u16.to_be_bytes());
    });
    let m = tracker::parse(&d).unwrap();
    let _ = tracker::render(&m);
}

#[test]
fn mod_zero_tempo_speed_and_jumps_terminate() {
    for (cmd, arg) in [(0xf, 0u8), (0xf, 1), (0xf, 31), (0xf, 32), (0xf, 255), (0xb, 0), (0xb, 255), (0xd, 0x99), (0xe, 0x9f), (0xe, 0xdf)] {
        let d = module(&[(0, 1, 428, cmd, arg), (1, 1, 200, 3, 255), (2, 1, 100, 4, 0xff), (3, 1, 1, 1, 255)], |h| {
            h[22..24].copy_from_slice(&500u16.to_be_bytes());
            h[25] = 64;
            h[28..30].copy_from_slice(&500u16.to_be_bytes());
        });
        let m = tracker::parse(&d).unwrap();
        let p = timed("mod", 30, || tracker::render(&m));
        assert!(p.samples.len() as f64 / 2.0 <= (sequencer::MAX_SECONDS + 1.0) * sequencer::RATE as f64);
    }
}

#[test]
fn mod_32_channels_full_song_is_bounded() {
    // 32 канала, 128 порядков, самый медленный темп: упирается в предел длины
    let mut d = vec![0u8; 1084];
    for i in 0..31 {
        let h = &mut d[20 + i * 30..20 + i * 30 + 30];
        h[22..24].copy_from_slice(&2000u16.to_be_bytes());
        h[25] = 64;
        h[28..30].copy_from_slice(&2000u16.to_be_bytes());
    }
    d[950] = 128;
    d[1080..1084].copy_from_slice(b"32CH");
    let mut pat = vec![0u8; 64 * 32 * 4];
    for ch in 0..32 {
        pat[ch * 4] = 0x01;
        pat[ch * 4 + 1] = 0xac;
        pat[ch * 4 + 2] = 0x10;
    }
    pat[2] |= 0xf;
    pat[3] = 32; // тянуть темп 32
    d.extend_from_slice(&pat);
    d.extend(vec![50u8; 4000 * 31]);
    let m = tracker::parse(&d).unwrap();
    let p = timed("32ch", 40, || tracker::render(&m));
    assert!(p.samples.len() as f64 / 2.0 <= (sequencer::MAX_SECONDS + 1.0) * sequencer::RATE as f64);
}
