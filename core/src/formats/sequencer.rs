//! Общее для MIDI и MOD: готовый звук (PCM -> WAV) и кэш отрисовки.
//! Модель проигрывает такие файлы как обычный WAV: ядро отдаёт страницe
//! отрисованный звук, а длина и частота берутся из его заголовка.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const RATE: u32 = 22050;
/// Предел длины отрисованного звука, секунды.
pub const MAX_SECONDS: f64 = 360.0;
/// Предел размера исходного файла.
pub const MAX_SOURCE: u64 = 4 << 20;

pub struct Pcm {
    pub channels: u16,
    /// Чередующиеся отсчёты.
    pub samples: Vec<i16>,
}

pub fn to_wav(p: &Pcm) -> Vec<u8> {
    let data = p.samples.len() * 2;
    let mut v = Vec::with_capacity(44 + data);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&((36 + data) as u32).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&p.channels.to_le_bytes());
    v.extend_from_slice(&RATE.to_le_bytes());
    v.extend_from_slice(&(RATE * 2 * p.channels as u32).to_le_bytes());
    v.extend_from_slice(&(2 * p.channels).to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&(data as u32).to_le_bytes());
    for s in &p.samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}

/// Это файл секвенсора (MIDI или MOD) по расширению.
pub fn is_sequence(path: &Path) -> bool {
    matches!(ext(path).as_str(), "mid" | "midi" | "mod")
}

fn ext(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).unwrap_or_default()
}

/// Отрисовывает файл в WAV; `None` - не MIDI/MOD, слишком большой или повреждён.
pub fn render_file(path: &Path) -> Option<Vec<u8>> {
    if std::fs::metadata(path).ok()?.len() > MAX_SOURCE {
        return None;
    }
    let data = std::fs::read(path).ok()?;
    let pcm = match ext(path).as_str() {
        "mid" | "midi" => super::midi::render(&super::midi::parse(&data)?),
        "mod" => super::tracker::render(&super::tracker::parse(&data)?),
        _ => return None,
    };
    Some(to_wav(&pcm))
}

type Cache = Vec<(PathBuf, std::time::SystemTime, Arc<Vec<u8>>)>;
static CACHE: Mutex<Cache> = Mutex::new(Vec::new());

/// То же, но с кэшем на несколько последних файлов (по пути и времени изменения).
pub fn render_cached(path: &Path) -> Option<Arc<Vec<u8>>> {
    let mt = std::fs::metadata(path).ok()?.modified().ok()?;
    {
        let c = CACHE.lock().ok()?;
        if let Some(e) = c.iter().find(|e| e.0 == path && e.1 == mt) {
            return Some(e.2.clone());
        }
    }
    let wav = Arc::new(render_file(path)?);
    let mut c = CACHE.lock().ok()?;
    if c.len() >= 4 {
        c.remove(0);
    }
    c.push((path.to_path_buf(), mt, wav.clone()));
    Some(wav)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_roundtrip() {
        let w = to_wav(&Pcm { channels: 2, samples: vec![0; RATE as usize * 2 * 3] });
        assert_eq!(crate::runtime::audio::wav_info(&w), Some((RATE as f64, 3.0)));
    }
}
