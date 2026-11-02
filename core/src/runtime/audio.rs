//! Состояние аудио-файлов модели (`AudioOpenSound`, `AudioPlay`…): на
//! каждый дескриптор - громкость, баланс, тональность, повтор, позиция.
//! Звук играет страница IDE; ядро держит состояние для ответов функций
//! (`AudioIsPlaying`, `AudioGetPosition`…) и отправляет страницe команды.
//! Позиция и длина - в отсчётах (частота * секунды), как в справке.

use std::collections::BTreeMap;
use std::time::Instant;

/// Сколько аудио-файлов можно держать открытыми одновременно.
pub const MAX_SOUNDS: usize = 256;

#[derive(Debug, Clone)]
pub struct Sound {
    pub file: String,
    /// Частота дискретизации; для WAV из заголовка, иначе 44100.
    pub rate: f64,
    /// Длина в секундах; 0 - неизвестна.
    pub length: f64,
    pub volume: f64,
    pub balance: f64,
    pub tone: f64,
    pub repeat: bool,
    playing: bool,
    base: f64,
    since: Option<Instant>,
}

/// Команда странице: полный снимок состояния дескриптора.
#[derive(Debug, Clone)]
pub struct AudioCmd {
    /// `open`, `play`, `stop`, `update`, `seek`.
    pub cmd: &'static str,
    pub id: u32,
    pub file: String,
    pub volume: f64,
    pub balance: f64,
    pub tone: f64,
    pub repeat: bool,
    /// Позиция в секундах.
    pub pos: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Audios {
    pub items: BTreeMap<u32, Sound>,
    next: u32,
    pub cmds: Vec<AudioCmd>,
}

/// Частота и длина WAV по заголовку (первые байты файла).
pub fn wav_info(head: &[u8]) -> Option<(f64, f64)> {
    if head.len() < 12 || &head[0..4] != b"RIFF" || &head[8..12] != b"WAVE" {
        return None;
    }
    let (mut rate, mut byte_rate) = (0u32, 0u32);
    let mut at = 12usize;
    while at + 8 <= head.len() {
        let id = &head[at..at + 4];
        let size = u32::from_le_bytes([head[at + 4], head[at + 5], head[at + 6], head[at + 7]]) as u64;
        let body = at + 8;
        if id == b"fmt " && body + 12 <= head.len() {
            rate = u32::from_le_bytes([head[body + 4], head[body + 5], head[body + 6], head[body + 7]]);
            byte_rate = u32::from_le_bytes([head[body + 8], head[body + 9], head[body + 10], head[body + 11]]);
        } else if id == b"data" {
            if rate == 0 || byte_rate == 0 {
                return None;
            }
            return Some((rate as f64, size as f64 / byte_rate as f64));
        }
        at = body.checked_add(size as usize + (size as usize & 1))?;
    }
    None
}

impl Sound {
    fn position_sec(&self) -> f64 {
        let mut p = self.base;
        if let (true, Some(t)) = (self.playing, self.since) {
            p += t.elapsed().as_secs_f64() * self.tone;
        }
        if self.length > 0.0 {
            if self.repeat {
                p %= self.length;
            } else {
                p = p.min(self.length);
            }
        }
        p
    }

    /// Доиграл ли файл без повтора.
    fn finished(&self) -> bool {
        !self.repeat && self.length > 0.0 && self.playing && self.since.is_some_and(|t| self.base + t.elapsed().as_secs_f64() * self.tone >= self.length)
    }

    fn settle(&mut self) {
        if self.finished() {
            self.base = self.length;
            self.playing = false;
            self.since = None;
        }
    }

    pub fn is_playing(&mut self) -> bool {
        self.settle();
        self.playing
    }

    pub fn position(&mut self) -> f64 {
        self.settle();
        (self.position_sec() * self.rate).floor()
    }
}

impl Audios {
    pub fn open(&mut self, file: String, rate: f64, length: f64) -> Option<u32> {
        if self.items.len() >= MAX_SOUNDS {
            return None;
        }
        self.next += 1;
        let id = self.next;
        let s = Sound { file, rate, length, volume: 1.0, balance: 0.0, tone: 1.0, repeat: false, playing: false, base: 0.0, since: None };
        self.items.insert(id, s);
        self.emit("open", id);
        Some(id)
    }

    fn emit(&mut self, cmd: &'static str, id: u32) {
        if let Some(s) = self.items.get_mut(&id) {
            s.settle();
            let c = AudioCmd { cmd, id, file: s.file.clone(), volume: s.volume, balance: s.balance, tone: s.tone, repeat: s.repeat, pos: s.position_sec() };
            if self.cmds.len() < 4096 {
                self.cmds.push(c);
            }
        }
    }

    pub fn play(&mut self, id: u32) {
        if let Some(s) = self.items.get_mut(&id) {
            s.settle();
            if s.length > 0.0 && !s.repeat && s.base >= s.length {
                s.base = 0.0; // доигранный файл начинается сначала
            }
            s.playing = true;
            s.since = Some(Instant::now());
        }
        self.emit("play", id);
    }

    pub fn stop(&mut self, id: u32) {
        if let Some(s) = self.items.get_mut(&id) {
            s.base = s.position_sec();
            s.playing = false;
            s.since = None;
        }
        self.emit("stop", id);
    }

    pub fn reset(&mut self, id: u32) {
        self.seek_sec(id, 0.0);
    }

    pub fn seek_sec(&mut self, id: u32, sec: f64) {
        if let Some(s) = self.items.get_mut(&id) {
            let max = if s.length > 0.0 { s.length } else { f64::MAX };
            s.base = if sec.is_finite() { sec.clamp(0.0, max) } else { 0.0 };
            if s.playing {
                s.since = Some(Instant::now());
            }
        }
        self.emit("seek", id);
    }

    /// Меняет громкость (0..1), баланс (-1..1), тональность (0.5..2) или повтор.
    pub fn set(&mut self, id: u32, what: &str, v: f64) -> bool {
        let Some(s) = self.items.get_mut(&id) else { return false };
        s.base = s.position_sec();
        if s.playing {
            s.since = Some(Instant::now());
        }
        let v = if v.is_finite() { v } else { 0.0 };
        match what {
            "volume" => s.volume = v.clamp(0.0, 1.0),
            "balance" => s.balance = v.clamp(-1.0, 1.0),
            "tone" => s.tone = v.clamp(0.5, 2.0),
            _ => s.repeat = v != 0.0,
        }
        self.emit("update", id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(rate: u32, secs: u32) -> Vec<u8> {
        let mut v = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&[1, 0, 1, 0]);
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&(rate * 2).to_le_bytes());
        v.extend_from_slice(&[2, 0, 16, 0]);
        v.extend_from_slice(b"data");
        v.extend_from_slice(&(rate * 2 * secs).to_le_bytes());
        v
    }

    #[test]
    fn wav_header_gives_rate_and_length() {
        assert_eq!(wav_info(&wav(22050, 4)), Some((22050.0, 4.0)));
        assert_eq!(wav_info(b"RIFFxxxxWAVE"), None);
        for n in 0..wav(8000, 1).len() {
            let _ = wav_info(&wav(8000, 1)[..n]);
        }
    }

    #[test]
    fn state_per_handle() {
        let mut a = Audios::default();
        let id = a.open("a.wav".into(), 1000.0, 10.0).unwrap();
        let other = a.open("b.wav".into(), 1000.0, 10.0).unwrap();
        assert!(!a.items.get_mut(&id).unwrap().is_playing());
        a.set(id, "volume", 7.0);
        assert_eq!(a.items[&id].volume, 1.0);
        a.set(id, "tone", 0.1);
        assert_eq!(a.items[&id].tone, 0.5);
        assert_eq!(a.items[&other].tone, 1.0);
        a.seek_sec(id, 2.5);
        assert_eq!(a.items.get_mut(&id).unwrap().position(), 2500.0);
        a.play(id);
        assert!(a.items.get_mut(&id).unwrap().is_playing());
        a.stop(id);
        assert!(!a.items.get_mut(&id).unwrap().is_playing());
        a.seek_sec(id, 99.0);
        assert_eq!(a.items.get_mut(&id).unwrap().position(), 10000.0);
        assert!(a.cmds.iter().any(|c| c.cmd == "play" && c.id == id));
    }
}
