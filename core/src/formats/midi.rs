//! Стандартный MIDI-файл (.mid): разбор в ноты со временем в секундах
//! (с картой темпа) и простой синтезатор: группы тембров GM по 8 программ,
//! ударные на канале 10. Выход - стерео i16 (канал один, дублируется).

use super::sequencer::{Pcm, MAX_SECONDS, RATE};

const MAX_EVENTS: usize = 400_000;
const MAX_NOTES: usize = 100_000;
const MAX_OPEN: usize = 128;
/// Предел работы синтезатора: отсчётов, суммарно по всем нотам.
const MAX_VOICE_SAMPLES: usize = 120_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub start: f64,
    pub end: f64,
    pub ch: u8,
    pub key: u8,
    pub vel: u8,
    pub program: u8,
    /// Громкость канала (CC7 * CC11) на момент ноты, 0..1.
    pub level: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Song {
    pub notes: Vec<Note>,
    pub length: f64,
}

enum Kind {
    Tempo(u32),
    On(u8, u8, u8),
    Off(u8, u8),
    Prog(u8, u8),
    Ctl(u8, u8, u8),
}

struct Ev {
    tick: u64,
    ord: usize,
    kind: Kind,
}

fn vlq(d: &[u8], at: &mut usize) -> Option<u32> {
    let mut v = 0u32;
    for _ in 0..4 {
        let b = *d.get(*at)?;
        *at += 1;
        v = (v << 7) | (b & 0x7f) as u32;
        if b & 0x80 == 0 {
            return Some(v);
        }
    }
    None
}

fn be16(d: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*d.get(at)?, *d.get(at + 1)?]))
}

pub fn parse(d: &[u8]) -> Option<Song> {
    if d.len() < 14 || &d[0..4] != b"MThd" {
        return None;
    }
    let hl = u32::from_be_bytes([d[4], d[5], d[6], d[7]]) as usize;
    if hl < 6 || hl > 1024 {
        return None;
    }
    let ntrk = be16(d, 10)? as usize;
    let div = be16(d, 12)?;
    // секунд на тик по умолчанию (SMPTE) или тиков на четверть
    let smpte = if div & 0x8000 != 0 {
        let fps = 256 - (div >> 8) as i32;
        let tpf = (div & 0xff) as f64;
        if fps <= 0 || tpf == 0.0 { return None; }
        Some(1.0 / (fps as f64 * tpf))
    } else if div == 0 {
        return None;
    } else {
        None
    };
    let mut evs: Vec<Ev> = Vec::new();
    let mut at = 8 + hl;
    let mut ord = 0usize;
    for _ in 0..ntrk.min(256) {
        if at + 8 > d.len() {
            break;
        }
        let len = u32::from_be_bytes([d[at + 4], d[at + 5], d[at + 6], d[at + 7]]) as usize;
        let id = &d[at..at + 4];
        let body = at + 8;
        let end = body.saturating_add(len).min(d.len());
        at = body.saturating_add(len);
        if id != b"MTrk" {
            continue;
        }
        let mut p = body;
        let mut tick = 0u64;
        let mut status = 0u8;
        while p < end && evs.len() < MAX_EVENTS {
            let Some(dt) = vlq(&d[..end], &mut p) else { break };
            tick += dt as u64;
            let Some(&b) = d[..end].get(p) else { break };
            if b & 0x80 != 0 {
                p += 1;
                if b < 0xf0 {
                    status = b;
                }
                if b >= 0xf0 {
                    status = 0;
                }
            } else if status == 0 {
                break;
            }
            let st = if b & 0x80 != 0 { b } else { status };
            match st {
                0xff => {
                    let Some(&ty) = d[..end].get(p) else { break };
                    p += 1;
                    let Some(l) = vlq(&d[..end], &mut p) else { break };
                    let l = l as usize;
                    if p.saturating_add(l) > end {
                        break;
                    }
                    if ty == 0x51 && l == 3 {
                        let us = ((d[p] as u32) << 16) | ((d[p + 1] as u32) << 8) | d[p + 2] as u32;
                        if us > 0 {
                            evs.push(Ev { tick, ord, kind: Kind::Tempo(us) });
                        }
                    }
                    p += l;
                    if ty == 0x2f {
                        break;
                    }
                }
                0xf0 | 0xf7 => {
                    let Some(l) = vlq(&d[..end], &mut p) else { break };
                    p = p.saturating_add(l as usize);
                }
                0x80..=0xef => {
                    let ch = st & 15;
                    let two = matches!(st & 0xf0, 0xc0 | 0xd0);
                    let Some(&a) = d[..end].get(p) else { break };
                    p += 1;
                    let mut bb = 0;
                    if !two {
                        let Some(&x) = d[..end].get(p) else { break };
                        bb = x;
                        p += 1;
                    }
                    let (a, bb) = (a & 0x7f, bb & 0x7f);
                    let kind = match st & 0xf0 {
                        0x90 if bb > 0 => Some(Kind::On(ch, a, bb)),
                        0x90 | 0x80 => Some(Kind::Off(ch, a)),
                        0xb0 => Some(Kind::Ctl(ch, a, bb)),
                        0xc0 => Some(Kind::Prog(ch, a)),
                        _ => None,
                    };
                    if let Some(kind) = kind {
                        evs.push(Ev { tick, ord, kind });
                    }
                }
                _ => break,
            }
            ord += 1;
        }
    }
    evs.sort_by_key(|e| (e.tick, e.ord));
    let mut tempo = 500_000u32;
    let (mut sec, mut last) = (0.0f64, 0u64);
    let mut prog = [0u8; 16];
    let mut vol = [100u8; 16];
    let mut expr = [127u8; 16];
    let mut open: Vec<Vec<(u8, usize)>> = vec![Vec::new(); 16];
    let mut song = Song::default();
    for e in &evs {
        let dt = (e.tick - last) as f64;
        sec += match smpte {
            Some(s) => dt * s,
            None => dt * tempo as f64 / 1e6 / div as f64,
        };
        last = e.tick;
        match e.kind {
            Kind::Tempo(t) => tempo = t,
            Kind::Prog(c, p) => prog[c as usize] = p,
            Kind::Ctl(c, 7, v) => vol[c as usize] = v,
            Kind::Ctl(c, 11, v) => expr[c as usize] = v,
            Kind::Ctl(..) => {}
            Kind::On(c, k, v) => {
                if song.notes.len() < MAX_NOTES {
                    let level = vol[c as usize] as f32 / 127.0 * expr[c as usize] as f32 / 127.0;
                    song.notes.push(Note { start: sec, end: -1.0, ch: c, key: k, vel: v, program: prog[c as usize], level });
                    let o = &mut open[c as usize];
                    // незакрытых нот на канал не больше MAX_OPEN: поиск конца остаётся быстрым
                    if o.len() >= MAX_OPEN {
                        let (_, old) = o.remove(0);
                        song.notes[old].end = sec;
                    }
                    o.push((k, song.notes.len() - 1));
                }
            }
            Kind::Off(c, k) => {
                let o = &mut open[c as usize];
                if let Some(i) = o.iter().position(|x| x.0 == k) {
                    let (_, n) = o.remove(i);
                    song.notes[n].end = sec;
                }
            }
        }
    }
    for n in &mut song.notes {
        if n.end < 0.0 {
            n.end = sec.max(n.start);
        }
    }
    song.length = sec;
    if song.notes.is_empty() && evs.is_empty() {
        return None;
    }
    Some(song)
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Волна фазы 0..1: 0 синус, 1 треугольник, 2 пила, 3 меандр.
fn wave(kind: u8, ph: f32) -> f32 {
    match kind {
        0 => (ph * std::f32::consts::TAU).sin(),
        1 => 1.0 - 4.0 * (ph - 0.5).abs(),
        2 => 2.0 * ph - 1.0,
        _ => {
            if ph < 0.5 { 0.6 } else { -0.6 }
        }
    }
}

/// (волна, атака с, спад 1/с, 0 - без спада) по группе из 8 программ GM.
const GROUPS: [(u8, f32, f32); 16] = [
    (1, 0.003, 3.0),  // рояли
    (0, 0.002, 5.0),  // хроматические ударные
    (3, 0.01, 0.0),   // органы
    (2, 0.003, 2.5),  // гитары
    (1, 0.005, 1.5),  // басы
    (2, 0.12, 0.0),   // струнные
    (2, 0.1, 0.0),    // ансамбль
    (2, 0.03, 0.0),   // медь
    (3, 0.04, 0.0),   // язычковые
    (0, 0.05, 0.0),   // духовые
    (3, 0.01, 0.0),   // соло-синтезатор
    (1, 0.2, 0.0),    // пэды
    (2, 0.05, 0.0),   // эффекты синтезатора
    (1, 0.003, 3.0),  // этнические
    (0, 0.002, 6.0),  // ударные мелодические
    (0, 0.01, 2.0),   // звуковые эффекты
];

pub fn render(song: &Song) -> Pcm {
    let rate = RATE as f64;
    let total = (song.length + 3.0).min(MAX_SECONDS);
    let n = (total * rate) as usize;
    let mut buf = vec![0f32; n];
    let mut rng = Rng(0x1234_5678);
    let mut budget = MAX_VOICE_SAMPLES;
    for note in &song.notes {
        if budget == 0 {
            break;
        }
        if note.start >= total {
            continue;
        }
        let first = (note.start * rate) as usize;
        let dur = (note.end - note.start).max(0.02);
        let amp = note.vel as f32 / 127.0 * note.level * 0.18;
        if note.ch == 9 {
            budget = budget.saturating_sub(drum(&mut buf, first, note.key, amp * 1.6, &mut rng));
            continue;
        }
        let (w, atk, dec) = GROUPS[(note.program >> 3) as usize & 15];
        let f = 440.0 * 2f32.powf((note.key as f32 - 69.0) / 12.0);
        let rel = 0.08f32;
        let mut tail = dur as f32 + rel;
        if dec > 0.0 {
            tail = tail.min(8.0 / dec);
        }
        let cnt = ((tail * RATE as f32) as usize).min(n.saturating_sub(first)).min(budget);
        budget -= cnt;
        let mut ph = 0f32;
        for i in 0..cnt {
            let t = i as f32 / RATE as f32;
            let mut env = (t / atk).min(1.0);
            if dec > 0.0 {
                env *= (-dec * t).exp();
            }
            let d = dur as f32;
            if t > d {
                env *= (1.0 - (t - d) / rel).max(0.0);
            }
            buf[first + i] += wave(w, ph) * env * amp;
            ph += f / RATE as f32;
            ph -= ph.floor();
        }
    }
    let mut out = Vec::with_capacity(n * 2);
    for s in buf {
        let v = (s.tanh() * 32000.0) as i16;
        out.push(v);
        out.push(v);
    }
    Pcm { channels: 2, samples: out }
}

fn drum(buf: &mut [f32], first: usize, key: u8, amp: f32, rng: &mut Rng) -> usize {
    // (тон Гц начало, Гц конец, доля шума, спад 1/с)
    let (f0, f1, noise, dec) = match key {
        35 | 36 => (150.0, 45.0, 0.0, 14.0),
        38 | 40 => (190.0, 150.0, 0.8, 18.0),
        37 | 39 => (300.0, 250.0, 0.9, 30.0),
        41 | 43 | 45 | 47 | 48 | 50 => (60.0 + (key as f32 - 41.0) * 12.0 + 90.0, 60.0, 0.0, 10.0),
        42 | 44 => (0.0, 0.0, 1.0, 70.0),
        46 => (0.0, 0.0, 1.0, 12.0),
        49 | 51 | 52 | 55 | 57 | 59 => (0.0, 0.0, 1.0, 4.0),
        _ => (400.0, 300.0, 0.5, 25.0),
    };
    let len = ((7.0 / dec) * RATE as f32) as usize;
    let mut ph = 0f32;
    let mut prev = 0f32;
    let cnt = len.min(buf.len().saturating_sub(first));
    for i in 0..cnt {
        let t = i as f32 / RATE as f32;
        let env = (-dec * t).exp();
        let f = f1 + (f0 - f1) * (-30.0 * t).exp();
        ph += f / RATE as f32;
        ph -= ph.floor();
        let r = rng.next();
        let hp = r - prev; // грубый фильтр высоких частот
        prev = r;
        let tone = if f0 > 0.0 { (ph * std::f32::consts::TAU).sin() } else { 0.0 };
        buf[first + i] += (tone * (1.0 - noise) + hp * 0.5 * noise) * env * amp;
    }
    cnt
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Минимальный файл: темп 120 (500000 мкс), 480 тиков на четверть, нота ля одну четверть.
    fn tiny() -> Vec<u8> {
        let mut v = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0".to_vec();
        let trk: Vec<u8> = vec![
            0, 0xff, 0x51, 3, 0x07, 0xa1, 0x20, // темп
            0, 0xc0, 0, // программа 0
            0, 0x90, 69, 100, // нота on
            0x83, 0x60, 0x80, 69, 0, // 480 тиков, off
            0, 0xff, 0x2f, 0,
        ];
        v.extend_from_slice(b"MTrk");
        v.extend_from_slice(&(trk.len() as u32).to_be_bytes());
        v.extend_from_slice(&trk);
        v
    }

    #[test]
    fn parses_notes_with_tempo() {
        let s = parse(&tiny()).unwrap();
        assert_eq!(s.notes.len(), 1);
        assert_eq!(s.notes[0].key, 69);
        assert!((s.notes[0].end - 0.5).abs() < 1e-9);
        assert!((s.length - 0.5).abs() < 1e-9);
    }

    #[test]
    fn tempo_change_stretches_time() {
        let mut d = tiny();
        // темп 60: 1_000_000 мкс = 0x0f4240
        d[22 + 4..22 + 7].copy_from_slice(&[0x0f, 0x42, 0x40]);
        let s = parse(&d).unwrap();
        assert!((s.length - 1.0).abs() < 1e-9);
    }

    #[test]
    fn render_makes_sound_and_survives_garbage() {
        let p = render(&parse(&tiny()).unwrap());
        assert!(p.samples.iter().any(|&x| x.abs() > 500));
        let d = tiny();
        for n in 0..d.len() {
            if let Some(s) = parse(&d[..n]) {
                let _ = render(&s);
            }
        }
        assert!(parse(b"nonsense nonsense").is_none());
    }

    #[test]
    fn corpus_file_if_present() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples/ROBOT2/DAS_BOOT.MID");
        if let Ok(d) = std::fs::read(p) {
            let s = parse(&d).unwrap();
            assert!(s.notes.len() > 10 && s.length > 1.0);
            assert!(render(&s).samples.len() > 1000);
        }
    }
}
