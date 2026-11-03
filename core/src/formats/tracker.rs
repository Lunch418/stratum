//! Модуль ProTracker (.mod): разбор (15 нет, только 31 образец с тегом) и
//! программный микшер. Эффекты: 0 арпеджио, 1/2 портаменто, 3 к ноте,
//! 4 вибрато, A ползунок громкости, B прыжок, C громкость, D переход,
//! E1/E2 точные портаменто, E9 повтор, EA/EB точная громкость, EC обрыв,
//! ED задержка, F скорость/темп.

use super::sequencer::{Pcm, MAX_SECONDS, RATE};

const PAULA: f64 = 7_093_789.2;

#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub data: Vec<i8>,
    pub finetune: i8,
    pub volume: u8,
    pub loop_start: usize,
    pub loop_len: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub sample: u8,
    pub period: u16,
    pub cmd: u8,
    pub arg: u8,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub title: String,
    pub channels: usize,
    pub order: Vec<u8>,
    pub patterns: Vec<Vec<Cell>>, // 64 * channels ячеек
    pub samples: Vec<Sample>,
}

pub fn parse(d: &[u8]) -> Option<Module> {
    if d.len() < 1084 {
        return None;
    }
    let tag = &d[1080..1084];
    let channels = match tag {
        b"M.K." | b"M!K!" | b"FLT4" | b"4CHN" => 4,
        b"6CHN" => 6,
        b"8CHN" | b"FLT8" | b"OCTA" | b"CD81" => 8,
        [a, b, b'C', b'H'] if a.is_ascii_digit() && b.is_ascii_digit() => ((a - b'0') * 10 + (b - b'0')) as usize,
        [a, b'C', b'H', b'N'] if a.is_ascii_digit() => (a - b'0') as usize,
        _ => return None,
    };
    if !(1..=32).contains(&channels) {
        return None;
    }
    let title: String = d[..20].iter().take_while(|&&b| b != 0).map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' }).collect();
    let song_len = (d[950] as usize).clamp(1, 128);
    let order: Vec<u8> = d[952..952 + song_len].to_vec();
    let npat = d[952..1080].iter().copied().max().unwrap_or(0) as usize + 1;
    let pat_bytes = 64 * channels * 4;
    let mut at = 1084usize;
    if at + npat * pat_bytes > d.len() {
        return None;
    }
    let mut patterns = Vec::with_capacity(npat);
    for _ in 0..npat {
        let mut cells = Vec::with_capacity(64 * channels);
        for c in d[at..at + pat_bytes].chunks_exact(4) {
            cells.push(Cell {
                sample: (c[0] & 0xf0) | (c[2] >> 4),
                period: (((c[0] & 0x0f) as u16) << 8) | c[1] as u16,
                cmd: c[2] & 0x0f,
                arg: c[3],
            });
        }
        patterns.push(cells);
        at += pat_bytes;
    }
    let mut samples = Vec::with_capacity(31);
    for i in 0..31 {
        let h = &d[20 + i * 30..20 + i * 30 + 30];
        let len = u16::from_be_bytes([h[22], h[23]]) as usize * 2;
        let ft = (h[24] & 15) as i8;
        let take = len.min(d.len().saturating_sub(at));
        let data: Vec<i8> = d[at..at + take].iter().map(|&b| b as i8).collect();
        at += take;
        samples.push(Sample {
            data,
            finetune: if ft > 7 { ft - 16 } else { ft },
            volume: h[25].min(64),
            loop_start: u16::from_be_bytes([h[26], h[27]]) as usize * 2,
            loop_len: u16::from_be_bytes([h[28], h[29]]) as usize * 2,
        });
    }
    Some(Module { title, channels, order, patterns, samples })
}

#[derive(Default, Clone)]
struct Chan {
    sample: usize, // 1..=31, 0 - нет
    period: f64,
    target: f64,
    vol: i32,
    pos: f64,
    on: bool,
    porta: u8,
    vib_speed: u8,
    vib_depth: u8,
    vib_pos: u8,
    vslide: u8,
    slide: u8,
    arp: i32,
    delay: Option<(u32, usize, u16)>,
    cell: Cell,
}

const SINE: [i32; 32] = [0, 24, 49, 74, 97, 120, 141, 161, 180, 197, 212, 224, 235, 244, 250, 253, 255, 253, 250, 244, 235, 224, 212, 197, 180, 161, 141, 120, 97, 74, 49, 24];

fn tune(period: f64, ft: i8) -> f64 {
    period * 2f64.powf(-(ft as f64) / 96.0)
}

fn trigger(c: &mut Chan, m: &Module, sample: usize, period: u16) {
    if sample > 0 {
        c.sample = sample;
    }
    if c.sample == 0 || period == 0 {
        return;
    }
    let ft = m.samples[c.sample - 1].finetune;
    c.period = tune(period as f64, ft);
    c.pos = 0.0;
    c.on = true;
    c.vib_pos = 0;
}

pub fn render(m: &Module) -> Pcm {
    let nch = m.channels;
    let mut ch = vec![Chan::default(); nch];
    let rate = RATE as f64;
    let max_samples = (MAX_SECONDS * rate) as usize;
    let mut out: Vec<i16> = Vec::new();
    let (mut speed, mut tempo) = (6u32, 125u32);
    let (mut ord, mut row) = (0usize, 0usize);
    let mut seen = vec![false; 128 * 64];
    let gain = 1.0 / (nch as f32 / 2.0).max(1.0) * 0.7;
    let mut frac = 0f64;
    'song: loop {
        if ord >= m.order.len() || seen[ord * 64 + row] {
            break;
        }
        seen[ord * 64 + row] = true;
        let pat = &m.patterns[(m.order[ord] as usize).min(m.patterns.len() - 1)];
        let (mut jump, mut brk): (Option<usize>, Option<usize>) = (None, None);
        let mut tick = 0u32;
        while tick < speed {
            for (i, c) in ch.iter_mut().enumerate() {
                if tick == 0 {
                    let cell = pat[row * nch + i];
                    c.cell = cell;
                    if cell.sample > 0 && (cell.sample as usize) <= 31 {
                        c.sample = cell.sample as usize;
                        c.vol = m.samples[c.sample - 1].volume as i32;
                    }
                    let (x, y) = (cell.arg >> 4, cell.arg & 15);
                    let porta = cell.cmd == 3 || cell.cmd == 5;
                    if cell.period > 0 {
                        if porta {
                            c.target = tune(cell.period as f64, if c.sample > 0 { m.samples[c.sample - 1].finetune } else { 0 });
                        } else if cell.cmd == 0xe && x == 0xd && y > 0 {
                            // номер образца бывает до 255: за пределами 1..=31 берётся прежний
                            let smp = if (1..=31).contains(&cell.sample) { cell.sample as usize } else { 0 };
                            c.delay = Some((y as u32, smp, cell.period));
                        } else {
                            trigger(c, m, 0, cell.period);
                        }
                    }
                    match cell.cmd {
                        3 if cell.arg != 0 => c.porta = cell.arg,
                        4 => {
                            if x != 0 { c.vib_speed = x; }
                            if y != 0 { c.vib_depth = y; }
                        }
                        1 | 2 if cell.arg != 0 => c.slide = cell.arg,
                        0xa if cell.arg != 0 => c.vslide = cell.arg,
                        0xb => jump = Some(cell.arg as usize),
                        0xc => c.vol = (cell.arg as i32).min(64),
                        0xd => brk = Some((x as usize * 10 + y as usize).min(63)),
                        0xe => match x {
                            1 => c.period = (c.period - y as f64).max(113.0),
                            2 => c.period = (c.period + y as f64).min(856.0),
                            0xa => c.vol = (c.vol + y as i32).min(64),
                            0xb => c.vol = (c.vol - y as i32).max(0),
                            _ => {}
                        },
                        0xf => {
                            if cell.arg == 0 {
                                break 'song;
                            } else if cell.arg < 32 {
                                speed = cell.arg as u32;
                            } else {
                                tempo = cell.arg as u32;
                            }
                        }
                        _ => {}
                    }
                } else {
                    let cell = c.cell;
                    let (x, y) = (cell.arg >> 4, cell.arg & 15);
                    match cell.cmd {
                        1 => c.period = (c.period - c.slide as f64).max(113.0),
                        2 => c.period = (c.period + c.slide as f64).min(856.0),
                        3 | 5 => {
                            if c.target > 0.0 {
                                let s = c.porta as f64;
                                if c.period > c.target { c.period = (c.period - s).max(c.target); } else { c.period = (c.period + s).min(c.target); }
                            }
                            if cell.cmd == 5 { c.vol = (c.vol + vol_slide(c.vslide)).clamp(0, 64); }
                        }
                        4 | 6 => {
                            c.vib_pos = c.vib_pos.wrapping_add(c.vib_speed);
                            if cell.cmd == 6 { c.vol = (c.vol + vol_slide(c.vslide)).clamp(0, 64); }
                        }
                        0xa => c.vol = (c.vol + vol_slide(c.vslide)).clamp(0, 64),
                        0xe if x == 9 && y > 0 && tick % y as u32 == 0 => c.pos = 0.0,
                        0xe if x == 0xc && tick == y as u32 => c.vol = 0,
                        _ => {}
                    }
                }
                if let Some((t, s, p)) = c.delay {
                    if tick == t {
                        c.delay = None;
                        trigger(c, m, s, p);
                    }
                }
            }
            for c in ch.iter_mut() {
                c.arp = match (c.cell.cmd, c.cell.arg) {
                    (0, a) if a != 0 => [0, (a >> 4) as i32, (a & 15) as i32][(tick % 3) as usize],
                    _ => 0,
                };
            }
            // микширование одного такта
            frac += rate * 2.5 / tempo as f64;
            let n = frac as usize;
            frac -= n as f64;
            if out.len() / 2 + n > max_samples {
                break 'song;
            }
            for _ in 0..n {
                let (mut l, mut r) = (0f32, 0f32);
                for (i, c) in ch.iter_mut().enumerate() {
                    if !c.on || c.sample == 0 {
                        continue;
                    }
                    let s = &m.samples[c.sample - 1];
                    let mut per = c.period;
                    if c.cell.cmd == 4 || c.cell.cmd == 6 {
                        let idx = (c.vib_pos & 31) as usize;
                        let v = SINE[idx] * c.vib_depth as i32 / 128;
                        per += if c.vib_pos & 32 != 0 { -v } else { v } as f64;
                    }
                    if c.arp != 0 {
                        per *= 2f64.powf(-(c.arp as f64) / 12.0);
                    }
                    let per = per.clamp(50.0, 2000.0);
                    let step = PAULA / (per * 2.0) / rate;
                    let ip = c.pos as usize;
                    let len = s.data.len();
                    if ip >= len {
                        c.on = false;
                        continue;
                    }
                    let a = s.data[ip] as f32;
                    let b = s.data.get(ip + 1).map_or(a, |&b| b as f32);
                    let v = (a + (b - a) * (c.pos - ip as f64) as f32) / 128.0 * c.vol as f32 / 64.0 * gain;
                    // каналы Amiga: L R R L
                    if matches!(i & 3, 0 | 3) { l += v; } else { r += v; }
                    c.pos += step;
                    if s.loop_len > 2 && s.loop_start < len {
                        let end = (s.loop_start + s.loop_len).min(len);
                        if c.pos >= end as f64 {
                            c.pos = s.loop_start as f64 + (c.pos - end as f64) % (end - s.loop_start).max(1) as f64;
                        }
                    }
                }
                out.push((l.clamp(-1.0, 1.0) * 32000.0) as i16);
                out.push((r.clamp(-1.0, 1.0) * 32000.0) as i16);
            }
            tick += 1;
        }
        // следующая строка
        if let Some(j) = jump {
            ord = j;
            row = brk.unwrap_or(0);
        } else if let Some(b) = brk {
            ord += 1;
            row = b;
        } else {
            row += 1;
            if row >= 64 {
                row = 0;
                ord += 1;
            }
        }
    }
    Pcm { channels: 2, samples: out }
}

fn vol_slide(a: u8) -> i32 {
    if a >> 4 != 0 { (a >> 4) as i32 } else { -((a & 15) as i32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Модуль на 4 канала: один образец-пила в цикле, ноты и эффекты в первой строке.
    fn module(cells: &[(usize, u16, u8, u8)], rows_after: usize) -> Vec<u8> {
        let mut d = vec![0u8; 1084];
        d[..4].copy_from_slice(b"test");
        // образец 1: 64 байта, громкость 64, цикл всё
        d[20 + 22..20 + 24].copy_from_slice(&32u16.to_be_bytes());
        d[20 + 25] = 64;
        d[20 + 28..20 + 30].copy_from_slice(&32u16.to_be_bytes());
        d[950] = 1;
        d[1080..1084].copy_from_slice(b"M.K.");
        let mut pat = vec![0u8; 64 * 16];
        for &(ch, per, cmd, arg) in cells {
            let o = ch * 4;
            pat[o] = (per >> 8) as u8;
            pat[o + 1] = per as u8;
            pat[o + 2] = 0x10 | cmd;
            pat[o + 3] = arg;
        }
        let _ = rows_after;
        d.extend_from_slice(&pat);
        d.extend((0..64).map(|i| if i < 32 { 100u8 } else { 156u8 }));
        d
    }

    #[test]
    fn parses_header_and_cells() {
        let m = parse(&module(&[(0, 428, 0xc, 0x20)], 0)).unwrap();
        assert_eq!((m.channels, m.order.len(), m.patterns.len()), (4, 1, 1));
        let c = m.patterns[0][0];
        assert_eq!((c.sample, c.period, c.cmd, c.arg), (1, 428, 0xc, 0x20));
        assert_eq!(m.samples[0].data.len(), 64);
        assert!(parse(&[0u8; 1084]).is_none());
    }

    #[test]
    fn default_speed_gives_expected_length() {
        // 64 строки по 6 тактов при темпе 125: 64*6*0.02 с
        let p = render(&parse(&module(&[(0, 428, 0, 0)], 0)).unwrap());
        let secs = p.samples.len() as f64 / 2.0 / RATE as f64;
        assert!((secs - 7.68).abs() < 0.05, "{secs}");
        assert!(p.samples.iter().any(|&x| x.abs() > 1000));
    }

    #[test]
    fn speed_and_volume_effects() {
        let fast = render(&parse(&module(&[(1, 0, 0xf, 3)], 0)).unwrap());
        assert!((fast.samples.len() as f64 / 2.0 / RATE as f64 - 3.84).abs() < 0.05);
        let quiet = render(&parse(&module(&[(0, 428, 0xc, 0)], 0)).unwrap());
        let loud = render(&parse(&module(&[(0, 428, 0xc, 0x40)], 0)).unwrap());
        let peak = |p: &Pcm| p.samples.iter().map(|x| x.unsigned_abs()).max().unwrap();
        assert_eq!(peak(&quiet), 0);
        assert!(peak(&loud) > 1000);
    }

    #[test]
    fn pattern_break_and_truncation() {
        // D00 в первой строке обрывает порядок (следующий порядок отсутствует)
        let p = render(&parse(&module(&[(0, 428, 0xd, 0)], 0)).unwrap());
        assert!(p.samples.len() as f64 / 2.0 / RATE as f64 <= 0.13);
        let d = module(&[(0, 428, 4, 0x48)], 0);
        for n in (0..d.len()).step_by(37) {
            if let Some(m) = parse(&d[..n]) {
                let _ = render(&m);
            }
        }
    }
}
