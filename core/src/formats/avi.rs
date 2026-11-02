//! Видеофайлы AVI (RIFF): только чтение видеопотока, кадр за кадром.
//!
//! Понимаются несжатые кадры (BI_RGB 8/16/24/32 бита), RLE8, Microsoft Video 1
//! (CRAM/MSVC/WHAM, 8 и 16 бит) и Cinepak (CVID). Остальные кодеки дают
//! `AviError::Unsupported`. Все смещения и длины из файла проверяются по
//! границам; число кадров ограничено размером файла, а не полями заголовка.

use std::fmt;

/// Потолок размера кадра (RGB, байт): 64 МБ.
pub const MAX_FRAME_BYTES: usize = 64 << 20;
/// Потолок размера файла, читаемого целиком.
/// Наибольшее число кадров и полос Cinepak в кадре (память растёт с числом).
pub const MAX_FRAMES: usize = 1 << 20;
pub const MAX_STRIPS: usize = 256;
pub const MAX_FILE_BYTES: usize = 256 << 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AviError {
    Malformed(&'static str),
    /// Кодек не поддерживается (FOURCC или код BI_*).
    Unsupported(u32),
    TooLarge,
    NoFrame,
}

impl fmt::Display for AviError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AviError::Malformed(m) => write!(f, "AVI: {m}"),
            AviError::Unsupported(c) => {
                let b = c.to_le_bytes();
                if b.iter().all(|x| x.is_ascii_graphic()) {
                    write!(f, "codec not supported: {}", String::from_utf8_lossy(&b))
                } else {
                    write!(f, "codec not supported: {c}")
                }
            }
            AviError::TooLarge => write!(f, "AVI: frame too large"),
            AviError::NoFrame => write!(f, "AVI: no such frame"),
        }
    }
}

fn u16le(d: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(i..i.checked_add(2)?)?.try_into().ok()?))
}
fn u32le(d: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(i..i.checked_add(4)?)?.try_into().ok()?))
}
fn u16be(d: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes(d.get(i..i.checked_add(2)?)?.try_into().ok()?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Codec {
    Rgb,
    Rle8,
    Msvc1,
    Cinepak,
}

#[derive(Debug, Clone)]
pub struct Avi {
    data: Vec<u8>,
    width: u32,
    height: u32,
    top_down: bool,
    bpp: u16,
    codec: Codec,
    fourcc: u32,
    palette: Vec<[u8; 3]>,
    /// Кадры видеопотока: (смещение, длина) данных в файле.
    frames: Vec<(usize, usize)>,
    fps: f64,
    // состояние декодера: последний декодированный кадр
    canvas: Vec<u8>,
    done: Option<usize>,
    cine: Vec<Codebooks>,
}

#[derive(Debug, Clone)]
struct Codebooks {
    v1: Vec<[[u8; 3]; 4]>,
    v4: Vec<[[u8; 3]; 4]>,
}

impl Codebooks {
    fn new() -> Self {
        Codebooks { v1: vec![[[0; 3]; 4]; 256], v4: vec![[[0; 3]; 4]; 256] }
    }
}

impl Avi {
    pub fn parse(data: Vec<u8>) -> Result<Avi, AviError> {
        if data.len() > MAX_FILE_BYTES {
            return Err(AviError::TooLarge);
        }
        if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..11] != b"AVI" {
            return Err(AviError::Malformed("not a RIFF/AVI file"));
        }
        let mut st = Scan::default();
        scan(&data, 12, data.len(), 0, &mut st);
        let (w, h, bpp, comp) = st.format.ok_or(AviError::Malformed("no video stream format"))?;
        let h_abs = h.unsigned_abs();
        let w_abs = w.unsigned_abs();
        if w_abs == 0 || h_abs == 0 || w_abs > 16384 || h_abs > 16384 || (w_abs as usize) * (h_abs as usize) * 3 > MAX_FRAME_BYTES {
            return Err(AviError::TooLarge);
        }
        let lc = comp.to_le_bytes().map(|b| b.to_ascii_lowercase());
        let codec = match comp {
            0 => Codec::Rgb,
            1 => Codec::Rle8,
            _ if &lc == b"cram" || &lc == b"msvc" || &lc == b"wham" => Codec::Msvc1,
            _ if &lc == b"cvid" => Codec::Cinepak,
            _ => return Err(AviError::Unsupported(comp)),
        };
        let ok_bpp = match codec {
            Codec::Rgb => matches!(bpp, 8 | 16 | 24 | 32),
            Codec::Rle8 => bpp == 8,
            Codec::Msvc1 => matches!(bpp, 8 | 16),
            Codec::Cinepak => matches!(bpp, 24 | 32),
        };
        if !ok_bpp {
            return Err(AviError::Unsupported(comp));
        }
        let mut palette = st.palette;
        palette.resize(256, [0, 0, 0]);
        let fps = if st.scale > 0 && st.rate > 0 { st.rate as f64 / st.scale as f64 } else { 15.0 };
        let px = w_abs as usize * h_abs as usize;
        let canvas_len = match codec {
            Codec::Cinepak => px * 3,
            Codec::Rle8 => px,
            Codec::Msvc1 if bpp == 8 => px,
            Codec::Msvc1 => px * 3,
            Codec::Rgb => 0,
        };
        Ok(Avi {
            data,
            width: w_abs,
            height: h_abs,
            top_down: h < 0,
            bpp,
            codec,
            fourcc: st.handler,
            palette,
            frames: st.frames,
            fps,
            canvas: vec![0; canvas_len],
            done: None,
            cine: Vec::new(),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    pub fn fps(&self) -> f64 {
        self.fps
    }
    pub fn fourcc(&self) -> u32 {
        self.fourcc
    }

    /// Кадр `index` как RGB сверху вниз. Кадры-разности (RLE8, MSVC1,
    /// Cinepak) дорисовываются поверх предыдущего; при шаге назад
    /// декодирование идёт с начала.
    pub fn frame_rgb(&mut self, index: usize) -> Result<Vec<u8>, AviError> {
        if index >= self.frames.len() {
            return Err(AviError::NoFrame);
        }
        if self.codec == Codec::Rgb {
            return self.decode_rgb(index);
        }
        let start = match self.done {
            Some(d) if d <= index => d + 1,
            _ => {
                self.canvas.iter_mut().for_each(|b| *b = 0);
                self.cine.clear();
                0
            }
        };
        for i in start..=index {
            let (off, len) = self.frames[i];
            if len == 0 {
                self.done = Some(i);
                continue;
            }
            let chunk = self.data.get(off..off + len).ok_or(AviError::Malformed("frame outside file"))?.to_vec();
            let r = match self.codec {
                Codec::Rle8 => self.rle8(&chunk),
                Codec::Msvc1 => self.msvc1(&chunk),
                _ => self.cinepak(&chunk),
            };
            if let Err(e) = r {
                self.done = None;
                return Err(e);
            }
            self.done = Some(i);
        }
        Ok(self.canvas_rgb())
    }

    fn canvas_rgb(&self) -> Vec<u8> {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut out = vec![0u8; w * h * 3];
        for y in 0..h {
            let sy = if self.codec == Codec::Cinepak || self.top_down { y } else { h - 1 - y };
            for x in 0..w {
                let px = if self.codec == Codec::Cinepak || (self.codec == Codec::Msvc1 && self.bpp == 16) {
                    let o = (sy * w + x) * 3;
                    [self.canvas[o], self.canvas[o + 1], self.canvas[o + 2]]
                } else {
                    self.palette[self.canvas[sy * w + x] as usize]
                };
                let o = (y * w + x) * 3;
                out[o..o + 3].copy_from_slice(&px);
            }
        }
        out
    }

    fn decode_rgb(&self, index: usize) -> Result<Vec<u8>, AviError> {
        let (off, len) = self.frames[index];
        let d = self.data.get(off..off + len).ok_or(AviError::Malformed("frame outside file"))?;
        let (w, h, bpp) = (self.width as usize, self.height as usize, self.bpp as usize);
        let stride = (w * bpp).div_ceil(32) * 4;
        if d.len() < stride * h {
            return Err(AviError::Malformed("short frame"));
        }
        let mut out = vec![0u8; w * h * 3];
        for y in 0..h {
            let sy = if self.top_down { y } else { h - 1 - y };
            let line = &d[sy * stride..(sy + 1) * stride];
            for x in 0..w {
                let px = match bpp {
                    8 => self.palette[line[x] as usize],
                    16 => rgb555(u16::from_le_bytes([line[x * 2], line[x * 2 + 1]])),
                    24 => [line[x * 3 + 2], line[x * 3 + 1], line[x * 3]],
                    _ => [line[x * 4 + 2], line[x * 4 + 1], line[x * 4]],
                };
                let o = (y * w + x) * 3;
                out[o..o + 3].copy_from_slice(&px);
            }
        }
        Ok(out)
    }

    fn rle8(&mut self, d: &[u8]) -> Result<(), AviError> {
        let (w, h) = (self.width as usize, self.height as usize);
        let (mut x, mut y, mut i) = (0usize, 0usize, 0usize);
        while i + 1 < d.len() && y < h {
            let (n, v) = (d[i] as usize, d[i + 1]);
            i += 2;
            if n > 0 {
                for _ in 0..n {
                    if x < w {
                        self.canvas[y * w + x] = v;
                    }
                    x += 1;
                }
                continue;
            }
            match v {
                0 => {
                    x = 0;
                    y += 1;
                }
                1 => break,
                2 => {
                    if i + 1 >= d.len() {
                        break;
                    }
                    x += d[i] as usize;
                    y += d[i + 1] as usize;
                    i += 2;
                }
                k => {
                    let k = k as usize;
                    let run = d.get(i..i + k).ok_or(AviError::Malformed("RLE8 run past end"))?;
                    for &p in run {
                        if x < w {
                            self.canvas[y * w + x] = p;
                        }
                        x += 1;
                    }
                    i += k + (k & 1);
                }
            }
        }
        Ok(())
    }

    /// Microsoft Video 1: блоки 4x4, строки блоков идут снизу вверх (порядок DIB).
    fn msvc1(&mut self, d: &[u8]) -> Result<(), AviError> {
        let (w, h) = (self.width as usize, self.height as usize);
        let sixteen = self.bpp == 16;
        let bw = w.div_ceil(4);
        let total = bw * h.div_ceil(4);
        let mut i = 0usize;
        let mut block = 0usize;
        let canvas = &mut self.canvas;
        let mut put = |bx: usize, by: usize, px: usize, py: usize, c: u16| {
            let (x, y) = (bx * 4 + px, by * 4 + py);
            if x < w && y < h {
                if sixteen {
                    let rgb = rgb555(c);
                    canvas[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(&rgb);
                } else {
                    canvas[y * w + x] = c as u8;
                }
            }
        };
        let rd = |i: &mut usize| -> Result<u16, AviError> {
            let e = AviError::Malformed("MSVC1 data ends");
            if sixteen {
                let v = u16le(d, *i).ok_or(e)?;
                *i += 2;
                Ok(v)
            } else {
                let v = *d.get(*i).ok_or(e)?;
                *i += 1;
                Ok(v as u16)
            }
        };
        while block < total && i + 1 < d.len() {
            let (a, b) = (d[i], d[i + 1]);
            i += 2;
            let (bx, by) = (block % bw, block / bw);
            if b & 0xfc == 0x84 {
                block += (((b - 0x84) as usize) << 8) + a as usize;
                continue;
            }
            if b < 0x80 {
                let mut flags = ((b as u16) << 8) | a as u16;
                let mut colors = [0u16; 8];
                colors[0] = rd(&mut i)?;
                colors[1] = rd(&mut i)?;
                let eight = if sixteen { colors[0] & 0x8000 != 0 } else { colors[1] >= 64 };
                if eight {
                    for c in colors.iter_mut().skip(2) {
                        *c = rd(&mut i)?;
                    }
                }
                for py in 0..4 {
                    for px in 0..4 {
                        let bit = (flags & 1) as usize;
                        let idx = if eight { ((py >> 1) * 2 + (px >> 1)) * 2 + bit } else { bit };
                        put(bx, by, px, py, colors[idx] & if sixteen { 0x7fff } else { 0xff });
                        flags >>= 1;
                    }
                }
            } else if b >= 0x90 {
                let c = if sixteen { (((b as u16) << 8) | a as u16) & 0x7fff } else { a as u16 };
                for py in 0..4 {
                    for px in 0..4 {
                        put(bx, by, px, py, c);
                    }
                }
            }
            block += 1;
        }
        Ok(())
    }

    fn cinepak(&mut self, d: &[u8]) -> Result<(), AviError> {
        let bad = AviError::Malformed("Cinepak data");
        let flags = *d.first().ok_or(bad.clone())?;
        let nstrips = u16be(d, 8).ok_or(bad.clone())? as usize;
        if nstrips > MAX_STRIPS {
            return Err(bad);
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let mut pos = 10usize;
        let mut y0 = 0usize;
        for si in 0..nstrips.min(d.len() / 12 + 1) {
            let sid = u16be(d, pos).ok_or(bad.clone())?;
            let ssize = u16be(d, pos + 2).ok_or(bad.clone())? as usize;
            if !(0x1000..=0x1100).contains(&sid) || ssize < 12 {
                return Err(bad);
            }
            let end = (pos + ssize).min(d.len());
            let ty = u16be(d, pos + 4).ok_or(bad.clone())? as usize;
            let by = u16be(d, pos + 8).ok_or(bad.clone())? as usize;
            let sh = if by > ty { by - ty } else { h.div_ceil(nstrips.max(1)) };
            let y1 = (y0 + sh).min(h);
            while self.cine.len() <= si {
                self.cine.push(Codebooks::new());
            }
            if si > 0 && flags & 1 == 0 {
                self.cine[si] = self.cine[si - 1].clone();
            }
            let mut p = pos + 12;
            while p + 4 <= end {
                let cid = u16be(d, p).ok_or(bad.clone())?;
                let csize = u16be(d, p + 2).ok_or(bad.clone())? as usize;
                if csize < 4 {
                    break;
                }
                let body = d.get(p + 4..(p + csize).min(end)).ok_or(bad.clone())?;
                match cid & 0xff00 {
                    0x2000 | 0x2100 | 0x2200 | 0x2300 => {
                        let cb = &mut self.cine[si];
                        let book = if cid & 0x0200 != 0 { &mut cb.v1 } else { &mut cb.v4 };
                        cine_codebook(book, body, cid);
                    }
                    0x3000 => cine_vectors(&mut self.canvas, w, y0, y1, &self.cine[si], body, cid),
                    _ => {}
                }
                p += csize;
            }
            y0 = y1;
            pos = pos.checked_add(ssize).ok_or(bad.clone())?;
        }
        Ok(())
    }
}

fn cine_codebook(book: &mut [[[u8; 3]; 4]], body: &[u8], cid: u16) {
    let gray = cid & 0x0400 != 0;
    let esize = if gray { 4 } else { 6 };
    let inter = cid & 0x0100 != 0;
    let mut p = 0usize;
    let mut flag = 0u32;
    let mut mask = 0u32;
    for entry in book.iter_mut() {
        if inter {
            mask >>= 1;
            if mask == 0 {
                let Some(b) = body.get(p..p + 4) else { return };
                flag = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
                mask = 0x8000_0000;
                p += 4;
            }
            if flag & mask == 0 {
                continue;
            }
        }
        let Some(e) = body.get(p..p + esize) else { return };
        p += esize;
        let (u, v) = if gray { (0i32, 0i32) } else { (e[4] as i8 as i32, e[5] as i8 as i32) };
        let cl = |x: i32| x.clamp(0, 255) as u8;
        for k in 0..4 {
            let y = e[k] as i32;
            entry[k] = [cl(y + v * 2), cl(y - u / 2 - v), cl(y + u * 2)];
        }
    }
}

fn cine_vectors(canvas: &mut [u8], w: usize, y0: usize, y1: usize, cb: &Codebooks, body: &[u8], cid: u16) {
    let inter = cid & 0x0100 != 0;
    let v1_only = cid & 0x0200 != 0;
    let (mut p, mut flag, mut mask) = (0usize, 0u32, 0u32);
    let mut next_bit = |p: &mut usize| -> Option<bool> {
        mask >>= 1;
        if mask == 0 {
            let b = body.get(*p..*p + 4)?;
            flag = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
            mask = 0x8000_0000;
            *p += 4;
        }
        Some(flag & mask != 0)
    };
    let mut put2 = |bx: usize, by: usize, px: [[u8; 3]; 4]| {
        for (k, c) in px.iter().enumerate() {
            let (x, y) = (bx + k % 2, by + k / 2);
            if x < w && y < y1 {
                canvas[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(c);
            }
        }
    };
    let mut by = y0;
    while by < y1 {
        let mut bx = 0;
        while bx < w {
            let update = if inter {
                match next_bit(&mut p) {
                    Some(b) => b,
                    None => return,
                }
            } else {
                true
            };
            if update {
                let v4 = if v1_only {
                    false
                } else {
                    match next_bit(&mut p) {
                        Some(b) => b,
                        None => return,
                    }
                };
                if !v4 {
                    let Some(&i) = body.get(p) else { return };
                    p += 1;
                    let e = &cb.v1[i as usize];
                    for q in 0..4 {
                        put2(bx + (q % 2) * 2, by + (q / 2) * 2, [e[q]; 4]);
                    }
                } else {
                    let Some(ix) = body.get(p..p + 4) else { return };
                    p += 4;
                    for q in 0..4 {
                        put2(bx + (q % 2) * 2, by + (q / 2) * 2, cb.v4[ix[q] as usize]);
                    }
                }
            }
            bx += 4;
        }
        by += 4;
    }
}

fn rgb555(c: u16) -> [u8; 3] {
    let e = |v: u16| ((v << 3) | (v >> 2)) as u8;
    [e((c >> 10) & 31), e((c >> 5) & 31), e(c & 31)]
}

#[derive(Default)]
struct Scan {
    format: Option<(i32, i32, u16, u32)>,
    handler: u32,
    palette: Vec<[u8; 3]>,
    frames: Vec<(usize, usize)>,
    scale: u32,
    rate: u32,
    video_stream: Option<u16>,
    streams_seen: u16,
}

/// Обходит куски RIFF в `[pos, end)`; глубина ограничена.
fn scan(d: &[u8], mut pos: usize, end: usize, depth: u32, st: &mut Scan) {
    if depth > 6 {
        return;
    }
    let end = end.min(d.len());
    while pos.saturating_add(8) <= end {
        let (Some(id), Some(size)) = (d.get(pos..pos + 4), u32le(d, pos + 4)) else { return };
        let size = size as usize;
        let body = pos + 8;
        let body_end = body.saturating_add(size).min(end);
        match id {
            b"LIST" | b"RIFF" => {
                if body + 4 <= body_end {
                    let kind = &d[body..body + 4];
                    if matches!(kind, b"hdrl" | b"strl" | b"movi" | b"rec ") {
                        scan(d, body + 4, body_end, depth + 1, st);
                    }
                }
            }
            b"strh" => {
                if d.get(body..body + 4) == Some(b"vids") && st.video_stream.is_none() {
                    st.video_stream = Some(st.streams_seen);
                    st.handler = u32le(d, body + 4).unwrap_or(0);
                    st.scale = u32le(d, body + 20).unwrap_or(0);
                    st.rate = u32le(d, body + 24).unwrap_or(0);
                }
                st.streams_seen = st.streams_seen.saturating_add(1);
            }
            b"strf" if st.format.is_none() && st.video_stream == Some(st.streams_seen.wrapping_sub(1)) => {
                let hdr = u32le(d, body).unwrap_or(0) as usize;
                if let (Some(w), Some(h), Some(bpp), Some(comp)) = (u32le(d, body + 4), u32le(d, body + 8), u16le(d, body + 14), u32le(d, body + 16)) {
                    st.format = Some((w as i32, h as i32, bpp, comp));
                    let used = u32le(d, body + 32).unwrap_or(0) as usize;
                    let n = if used == 0 { 1usize << bpp.min(8) } else { used }.min(256);
                    if bpp <= 8 {
                        if let Some(pal) = d.get(body.saturating_add(hdr.max(40))..body_end) {
                            st.palette = pal.chunks_exact(4).take(n).map(|c| [c[2], c[1], c[0]]).collect();
                        }
                    }
                }
            }
            _ if id[0..2].iter().all(|b| b.is_ascii_digit()) && matches!(&id[2..4], b"db" | b"dc") => {
                let stream = ((id[0] - b'0') * 10 + (id[1] - b'0')) as u16;
                if Some(stream) == st.video_stream.or(Some(0)) && st.frames.len() < MAX_FRAMES {
                    st.frames.push((body, size.min(d.len().saturating_sub(body))));
                }
            }
            _ => {}
        }
        let next = body.saturating_add(size).saturating_add(size & 1);
        if next <= pos {
            return;
        }
        pos = next;
    }
}

/// Собирает AVI с несжатыми 24-битными кадрами (RGB сверху вниз) - для тестов.
pub fn build_rgb24(width: u32, height: u32, frames: &[Vec<u8>], fps: u32) -> Vec<u8> {
    let stride = (width as usize * 3).div_ceil(4) * 4;
    let mut movi = b"movi".to_vec();
    for f in frames {
        let mut dib = Vec::new();
        for y in (0..height as usize).rev() {
            let start = dib.len();
            for x in 0..width as usize {
                let o = (y * width as usize + x) * 3;
                dib.extend_from_slice(&[f[o + 2], f[o + 1], f[o]]);
            }
            dib.resize(start + stride, 0);
        }
        movi.extend_from_slice(b"00db");
        movi.extend_from_slice(&(dib.len() as u32).to_le_bytes());
        movi.extend_from_slice(&dib);
    }
    let mut strh = Vec::new();
    strh.extend_from_slice(b"vids");
    strh.extend_from_slice(&0u32.to_le_bytes());
    strh.resize(20, 0);
    strh.extend_from_slice(&1u32.to_le_bytes());
    strh.extend_from_slice(&fps.to_le_bytes());
    strh.resize(56, 0);
    let mut strf = Vec::new();
    strf.extend_from_slice(&40u32.to_le_bytes());
    strf.extend_from_slice(&width.to_le_bytes());
    strf.extend_from_slice(&height.to_le_bytes());
    strf.extend_from_slice(&1u16.to_le_bytes());
    strf.extend_from_slice(&24u16.to_le_bytes());
    strf.resize(40, 0);
    let chunk = |id: &[u8], body: &[u8]| {
        let mut v = id.to_vec();
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(body);
        if body.len() & 1 == 1 {
            v.push(0);
        }
        v
    };
    let mut strl = b"strl".to_vec();
    strl.extend(chunk(b"strh", &strh));
    strl.extend(chunk(b"strf", &strf));
    let mut hdrl = b"hdrl".to_vec();
    hdrl.extend(chunk(b"LIST", &strl));
    let mut body = b"AVI ".to_vec();
    body.extend(chunk(b"LIST", &hdrl));
    body.extend(chunk(b"LIST", &movi));
    chunk(b"RIFF", &body)
}

#[cfg(test)]
mod limits {
    use super::*;

    #[test]
    fn frame_list_and_strips_are_capped() {
        let one = build_rgb24(2, 2, &[vec![0; 12]], 10);
        let mut a = one.clone();
        // много пустых кусков "00dc" в конец movi: список кадров не растёт без предела
        let at = a.windows(4).position(|w| w == b"movi").unwrap();
        a.truncate(at + 4);
        for _ in 0..(MAX_FRAMES + 1000) {
            a.extend_from_slice(b"00dc\0\0\0\0");
        }
        let n = a.len() as u32 - 8;
        a[4..8].copy_from_slice(&n.to_le_bytes());
        // внешний LIST movi охватывает весь хвост
        let lp = a.windows(4).position(|w| w == b"movi").unwrap() - 8;
        let ln = (a.len() - lp - 8) as u32;
        a[lp + 4..lp + 8].copy_from_slice(&ln.to_le_bytes());
        if let Ok(v) = Avi::parse(a) {
            assert!(v.frame_count() <= MAX_FRAMES);
        }
        // 3000 пустых полос Cinepak: у каждой свои таблицы кодов
        let mut c = vec![0u8; 10];
        c[8] = 0x0b;
        c[9] = 0xb8;
        for _ in 0..3000 {
            c.extend_from_slice(&[0x10, 0, 0, 12, 0, 0, 0, 0, 0, 2, 0, 2]);
        }
        let mut av = Avi::parse(one).unwrap();
        av.codec = Codec::Cinepak;
        av.canvas = vec![0; 2 * 2 * 3];
        assert!(av.cinepak(&c).is_err());
    }
}
