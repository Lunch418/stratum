//! Декодеры текстур: BMP, TGA, PNG (без чересстрочной развёртки) и
//! базовый JPEG. Всё без внешних крейтов, с пределами размеров.

use super::Tex;

const MAX_SIDE: usize = 4096;
const MAX_PIXELS: usize = 16 << 20;

fn checked(w: usize, h: usize) -> bool {
    w > 0 && h > 0 && w <= MAX_SIDE && h <= MAX_SIDE && w * h <= MAX_PIXELS
}

pub fn decode(name: &str, data: &[u8]) -> Option<Tex> {
    let lower = name.to_ascii_lowercase();
    if data.starts_with(b"BM") {
        let (w, h, rgb) = crate::gfx::decode_bmp(data)?;
        let (w, h) = (w as usize, h as usize);
        if !checked(w, h) || rgb.len() < w * h * 3 {
            return None;
        }
        let mut rgba = Vec::with_capacity(w * h * 4);
        for p in rgb.chunks_exact(3).take(w * h) {
            rgba.extend_from_slice(&[p[0], p[1], p[2], 255]);
        }
        return Some(Tex { w, h, rgba });
    }
    if data.starts_with(&[0x89, b'P', b'N', b'G']) {
        return png(data);
    }
    if data.starts_with(&[0xFF, 0xD8]) {
        return jpeg(data);
    }
    if lower.ends_with(".tga") {
        return tga(data);
    }
    None
}

fn tga(d: &[u8]) -> Option<Tex> {
    if d.len() < 18 {
        return None;
    }
    let (idlen, cmap, ty) = (d[0] as usize, d[1], d[2]);
    let (w, h) = (u16::from_le_bytes([d[12], d[13]]) as usize, u16::from_le_bytes([d[14], d[15]]) as usize);
    let bpp = d[16] as usize;
    let desc = d[17];
    if cmap != 0 || !matches!(ty, 2 | 3 | 10 | 11) || !checked(w, h) || !matches!(bpp, 8 | 24 | 32) {
        return None;
    }
    if (ty == 3 || ty == 11) != (bpp == 8) {
        return None;
    }
    let bytes = bpp / 8;
    let mut p = 18 + idlen;
    let mut px: Vec<u8> = Vec::with_capacity(w * h * bytes);
    if ty < 9 {
        px.extend_from_slice(d.get(p..p + w * h * bytes)?);
    } else {
        while px.len() < w * h * bytes {
            let c = *d.get(p)?;
            p += 1;
            let n = (c & 0x7F) as usize + 1;
            if c & 0x80 != 0 {
                let v = d.get(p..p + bytes)?;
                p += bytes;
                for _ in 0..n {
                    px.extend_from_slice(v);
                }
            } else {
                px.extend_from_slice(d.get(p..p + n * bytes)?);
                p += n * bytes;
            }
        }
        px.truncate(w * h * bytes);
    }
    let mut rgba = vec![255u8; w * h * 4];
    for y in 0..w * h {
        let (row, col) = (y / w, y % w);
        let dy = if desc & 0x20 != 0 { row } else { h - 1 - row };
        let o = (dy * w + col) * 4;
        let s = &px[y * bytes..y * bytes + bytes];
        match bytes {
            1 => rgba[o..o + 3].copy_from_slice(&[s[0], s[0], s[0]]),
            3 => rgba[o..o + 3].copy_from_slice(&[s[2], s[1], s[0]]),
            _ => rgba[o..o + 4].copy_from_slice(&[s[2], s[1], s[0], s[3]]),
        }
    }
    Some(Tex { w, h, rgba })
}

// ---- inflate ----------------------------------------------------------------

struct Bits<'a> {
    d: &'a [u8],
    p: usize,
    b: u32,
    n: u32,
}

impl Bits<'_> {
    fn need(&mut self, k: u32) -> Option<()> {
        while self.n < k {
            self.b |= (*self.d.get(self.p)? as u32) << self.n;
            self.p += 1;
            self.n += 8;
        }
        Some(())
    }
    fn get(&mut self, k: u32) -> Option<u32> {
        if k == 0 {
            return Some(0);
        }
        self.need(k)?;
        let v = self.b & ((1u32 << k) - 1);
        self.b >>= k;
        self.n -= k;
        Some(v)
    }
}

struct Huff {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huff {
    fn new(lengths: &[u8]) -> Huff {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut offs = [0u16; 16];
        for i in 1..16 {
            offs[i] = offs[i - 1] + counts[i - 1];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (s, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = s as u16;
                offs[l as usize] += 1;
            }
        }
        Huff { counts, symbols }
    }
    fn decode(&self, b: &mut Bits) -> Option<u16> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= b.get(1)? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                return self.symbols.get((index + (code - first)) as usize).copied();
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        None
    }
}

fn inflate(d: &[u8], cap: usize) -> Option<Vec<u8>> {
    const LB: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
    const LE: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
    const DB: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
    const DE: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
    let mut b = Bits { d, p: 2, b: 0, n: 0 };
    let mut out: Vec<u8> = Vec::new();
    loop {
        let last = b.get(1)?;
        let ty = b.get(2)?;
        match ty {
            0 => {
                b.b = 0;
                b.n = 0;
                let len = u16::from_le_bytes([*d.get(b.p)?, *d.get(b.p + 1)?]) as usize;
                b.p += 4;
                if out.len() + len > cap {
                    return None;
                }
                out.extend_from_slice(d.get(b.p..b.p + len)?);
                b.p += len;
            }
            1 | 2 => {
                let (lit, dist) = if ty == 1 {
                    let mut l = [0u8; 288];
                    for (i, v) in l.iter_mut().enumerate() {
                        *v = if i < 144 { 8 } else if i < 256 { 9 } else if i < 280 { 7 } else { 8 };
                    }
                    (Huff::new(&l), Huff::new(&[5u8; 30]))
                } else {
                    let hlit = b.get(5)? as usize + 257;
                    let hdist = b.get(5)? as usize + 1;
                    let hclen = b.get(4)? as usize + 4;
                    const ORD: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                    let mut cl = [0u8; 19];
                    for &o in ORD.iter().take(hclen) {
                        cl[o] = b.get(3)? as u8;
                    }
                    let ch = Huff::new(&cl);
                    let mut lens = vec![0u8; hlit + hdist];
                    let mut i = 0;
                    while i < hlit + hdist {
                        let s = ch.decode(&mut b)?;
                        if s < 16 {
                            lens[i] = s as u8;
                            i += 1;
                        } else {
                            let (v, rep) = match s {
                                16 => (*lens.get(i.checked_sub(1)?)?, 3 + b.get(2)? as usize),
                                17 => (0, 3 + b.get(3)? as usize),
                                _ => (0, 11 + b.get(7)? as usize),
                            };
                            if i + rep > lens.len() {
                                return None;
                            }
                            for _ in 0..rep {
                                lens[i] = v;
                                i += 1;
                            }
                        }
                    }
                    (Huff::new(&lens[..hlit]), Huff::new(&lens[hlit..]))
                };
                loop {
                    let s = lit.decode(&mut b)? as usize;
                    if s < 256 {
                        if out.len() >= cap {
                            return None;
                        }
                        out.push(s as u8);
                    } else if s == 256 {
                        break;
                    } else {
                        let s = s - 257;
                        if s >= 29 {
                            return None;
                        }
                        let len = LB[s] as usize + b.get(LE[s] as u32)? as usize;
                        let ds = dist.decode(&mut b)? as usize;
                        if ds >= 30 {
                            return None;
                        }
                        let dd = DB[ds] as usize + b.get(DE[ds] as u32)? as usize;
                        if dd > out.len() || out.len() + len > cap {
                            return None;
                        }
                        for _ in 0..len {
                            let v = out[out.len() - dd];
                            out.push(v);
                        }
                    }
                }
            }
            _ => return None,
        }
        if last == 1 {
            return Some(out);
        }
    }
}

fn png(d: &[u8]) -> Option<Tex> {
    let mut p = 8;
    let (mut w, mut h, mut depth, mut ct, mut interlace) = (0usize, 0usize, 0u8, 0u8, 0u8);
    let mut idat = Vec::new();
    let mut plte: Vec<u8> = Vec::new();
    let mut trns: Vec<u8> = Vec::new();
    while p + 8 <= d.len() {
        let len = u32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]) as usize;
        let ty = &d[p + 4..p + 8];
        let body = d.get(p + 8..p.checked_add(8)?.checked_add(len)?)?;
        match ty {
            b"IHDR" if body.len() >= 13 => {
                w = u32::from_be_bytes([body[0], body[1], body[2], body[3]]) as usize;
                h = u32::from_be_bytes([body[4], body[5], body[6], body[7]]) as usize;
                depth = body[8];
                ct = body[9];
                interlace = body[12];
            }
            b"PLTE" => plte = body.to_vec(),
            b"tRNS" => trns = body.to_vec(),
            b"IDAT" => {
                if idat.len() + body.len() > 64 << 20 {
                    return None;
                }
                idat.extend_from_slice(body)
            }
            b"IEND" => break,
            _ => {}
        }
        p += 12 + len;
    }
    if !checked(w, h) || interlace != 0 || !matches!(depth, 1 | 2 | 4 | 8 | 16) {
        return None;
    }
    let chans = match ct {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return None,
    };
    if (ct == 3 && depth > 8) || (ct != 0 && ct != 3 && depth < 8) {
        return None;
    }
    let bits = chans * depth as usize;
    let stride = (w * bits).div_ceil(8);
    let bpp = bits.div_ceil(8).max(1);
    let raw = inflate(&idat, (stride + 1) * h)?;
    if raw.len() < (stride + 1) * h {
        return None;
    }
    let mut img = vec![0u8; stride * h];
    for y in 0..h {
        let f = raw[y * (stride + 1)];
        let line = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        for x in 0..stride {
            let a = if x >= bpp { img[y * stride + x - bpp] as i32 } else { 0 };
            let b = if y > 0 { img[(y - 1) * stride + x] as i32 } else { 0 };
            let c = if x >= bpp && y > 0 { img[(y - 1) * stride + x - bpp] as i32 } else { 0 };
            let pr = match f {
                0 => 0,
                1 => a,
                2 => b,
                3 => (a + b) / 2,
                4 => {
                    let pa = (b - c).abs();
                    let pb = (a - c).abs();
                    let pc = (a + b - 2 * c).abs();
                    if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
                }
                _ => return None,
            };
            img[y * stride + x] = (line[x] as i32 + pr) as u8;
        }
    }
    let mut rgba = vec![255u8; w * h * 4];
    let sample = |row: &[u8], i: usize| -> u32 {
        match depth {
            8 => row[i] as u32,
            16 => row[i * 2] as u32,
            d => {
                let per = 8 / d as usize;
                let byte = row[i / per];
                let sh = 8 - d as usize * (i % per + 1);
                ((byte >> sh) & ((1 << d) - 1)) as u32
            }
        }
    };
    let scale = |v: u32| -> u8 {
        match depth {
            1 => (v * 255) as u8,
            2 => (v * 85) as u8,
            4 => (v * 17) as u8,
            _ => v as u8,
        }
    };
    for y in 0..h {
        let row = &img[y * stride..(y + 1) * stride];
        for x in 0..w {
            let o = (y * w + x) * 4;
            match ct {
                0 => {
                    let g = scale(sample(row, x));
                    rgba[o..o + 3].copy_from_slice(&[g, g, g]);
                }
                3 => {
                    let i = sample(row, x) as usize;
                    let e = plte.get(i * 3..i * 3 + 3)?;
                    rgba[o..o + 3].copy_from_slice(e);
                    if let Some(a) = trns.get(i) {
                        rgba[o + 3] = *a;
                    }
                }
                2 => {
                    for k in 0..3 {
                        rgba[o + k] = scale(sample(row, x * 3 + k));
                    }
                }
                4 => {
                    let g = scale(sample(row, x * 2));
                    rgba[o..o + 3].copy_from_slice(&[g, g, g]);
                    rgba[o + 3] = scale(sample(row, x * 2 + 1));
                }
                _ => {
                    for k in 0..4 {
                        rgba[o + k] = scale(sample(row, x * 4 + k));
                    }
                }
            }
        }
    }
    Some(Tex { w, h, rgba })
}

// ---- JPEG (базовый, последовательный) ------------------------------------------

const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22,
    15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

struct JHuff {
    mincode: [i32; 17],
    maxcode: [i32; 18],
    valptr: [i32; 17],
    vals: Vec<u8>,
}

impl JHuff {
    fn new(counts: &[u8; 16], vals: Vec<u8>) -> JHuff {
        let mut h = JHuff { mincode: [0; 17], maxcode: [-1; 18], valptr: [0; 17], vals };
        let (mut code, mut k) = (0i32, 0i32);
        for l in 1..=16 {
            h.valptr[l] = k;
            h.mincode[l] = code;
            code += counts[l - 1] as i32;
            k += counts[l - 1] as i32;
            h.maxcode[l] = if counts[l - 1] > 0 { code - 1 } else { -1 };
            code <<= 1;
        }
        h
    }
}

struct JBits<'a> {
    d: &'a [u8],
    p: usize,
    buf: u32,
    n: u32,
}

impl JBits<'_> {
    fn bit(&mut self) -> Option<u32> {
        if self.n == 0 {
            let b = *self.d.get(self.p)?;
            self.p += 1;
            if b == 0xFF {
                let nx = *self.d.get(self.p)?;
                if nx == 0 {
                    self.p += 1;
                } else {
                    return None;
                }
            }
            self.buf = b as u32;
            self.n = 8;
        }
        self.n -= 1;
        Some((self.buf >> self.n) & 1)
    }
    fn bits(&mut self, k: u32) -> Option<i32> {
        let mut v = 0;
        for _ in 0..k {
            v = (v << 1) | self.bit()? as i32;
        }
        Some(v)
    }
    fn huff(&mut self, h: &JHuff) -> Option<u8> {
        let mut code = 0i32;
        for l in 1..=16 {
            code = (code << 1) | self.bit()? as i32;
            if h.maxcode[l] >= 0 && code <= h.maxcode[l] && code >= h.mincode[l] {
                return h.vals.get((h.valptr[l] + code - h.mincode[l]) as usize).copied();
            }
        }
        None
    }
    fn align(&mut self) {
        self.n = 0;
    }
}

fn extend(v: i32, t: u32) -> i32 {
    if t == 0 { 0 } else if v < (1 << (t - 1)) { v - (1 << t) + 1 } else { v }
}

fn idct(blk: &[i32; 64], out: &mut [u8; 64]) {
    static COS: std::sync::OnceLock<[[f32; 8]; 8]> = std::sync::OnceLock::new();
    let c = COS.get_or_init(|| {
        let mut t = [[0f32; 8]; 8];
        for (x, row) in t.iter_mut().enumerate() {
            for (u, v) in row.iter_mut().enumerate() {
                let cu = if u == 0 { std::f32::consts::FRAC_1_SQRT_2 } else { 1.0 };
                *v = cu * (((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI) / 16.0).cos() / 2.0;
            }
        }
        t
    });
    let mut tmp = [0f32; 64];
    for y in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for u in 0..8 {
                s += c[x][u] * blk[y * 8 + u] as f32;
            }
            tmp[y * 8 + x] = s;
        }
    }
    for x in 0..8 {
        for y in 0..8 {
            let mut s = 0.0;
            for v in 0..8 {
                s += c[y][v] * tmp[v * 8 + x];
            }
            out[y * 8 + x] = (s + 128.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

struct Comp {
    id: u8,
    h: usize,
    v: usize,
    tq: usize,
    td: usize,
    ta: usize,
    plane: Vec<u8>,
    pw: usize,
}

fn jpeg(d: &[u8]) -> Option<Tex> {
    let mut p = 2;
    let mut qt = [[0u16; 64]; 4];
    let mut dc: [Option<JHuff>; 4] = [None, None, None, None];
    let mut ac: [Option<JHuff>; 4] = [None, None, None, None];
    let mut comps: Vec<Comp> = Vec::new();
    let (mut w, mut h) = (0usize, 0usize);
    let mut restart = 0usize;
    let (mut hmax, mut vmax) = (1, 1);
    loop {
        while *d.get(p)? != 0xFF {
            p += 1;
        }
        while *d.get(p)? == 0xFF {
            p += 1;
        }
        let m = *d.get(p)?;
        p += 1;
        if m == 0xD9 {
            return None;
        }
        if (0xD0..=0xD8).contains(&m) || m == 0x01 {
            continue;
        }
        let len = u16::from_be_bytes([*d.get(p)?, *d.get(p + 1)?]) as usize;
        let seg = d.get(p + 2..p + len)?;
        match m {
            0xDB => {
                let mut s = seg;
                while !s.is_empty() {
                    let (pq, tq) = (s[0] >> 4, (s[0] & 15) as usize);
                    if tq > 3 {
                        return None;
                    }
                    s = &s[1..];
                    for i in 0..64 {
                        qt[tq][ZIGZAG[i]] = if pq == 0 {
                            let v = *s.get(i)? as u16;
                            v
                        } else {
                            u16::from_be_bytes([*s.get(i * 2)?, *s.get(i * 2 + 1)?])
                        };
                    }
                    s = s.get(if pq == 0 { 64 } else { 128 }..)?;
                }
            }
            0xC0 | 0xC1 => {
                if *seg.first()? != 8 {
                    return None;
                }
                h = u16::from_be_bytes([seg[1], seg[2]]) as usize;
                w = u16::from_be_bytes([seg[3], seg[4]]) as usize;
                let n = seg[5] as usize;
                if !checked(w, h) || !(n == 1 || n == 3) || seg.len() < 6 + n * 3 {
                    return None;
                }
                comps.clear();
                for i in 0..n {
                    let b = &seg[6 + i * 3..9 + i * 3];
                    let (ch, cv) = ((b[1] >> 4) as usize, (b[1] & 15) as usize);
                    if !(1..=4).contains(&ch) || !(1..=4).contains(&cv) || b[2] > 3 {
                        return None;
                    }
                    hmax = hmax.max(ch);
                    vmax = vmax.max(cv);
                    comps.push(Comp { id: b[0], h: ch, v: cv, tq: b[2] as usize, td: 0, ta: 0, plane: Vec::new(), pw: 0 });
                }
            }
            0xC2 | 0xC3 | 0xC5..=0xC7 | 0xC9..=0xCF => return None,
            0xC4 => {
                let mut s = seg;
                while s.len() >= 17 {
                    let (tc, th) = (s[0] >> 4, (s[0] & 15) as usize);
                    let mut counts = [0u8; 16];
                    counts.copy_from_slice(&s[1..17]);
                    let total: usize = counts.iter().map(|&c| c as usize).sum();
                    let vals = s.get(17..17 + total)?.to_vec();
                    if th > 3 {
                        return None;
                    }
                    let t = JHuff::new(&counts, vals);
                    if tc == 0 { dc[th] = Some(t) } else { ac[th] = Some(t) }
                    s = &s[17 + total..];
                }
            }
            0xDD => restart = u16::from_be_bytes([*seg.first()?, *seg.get(1)?]) as usize,
            0xDA => {
                let n = *seg.first()? as usize;
                if comps.is_empty() || n != comps.len() || seg.len() < 1 + n * 2 {
                    return None;
                }
                for i in 0..n {
                    let (cid, t) = (seg[1 + i * 2], seg[2 + i * 2]);
                    let c = comps.iter_mut().find(|c| c.id == cid)?;
                    c.td = (t >> 4) as usize & 3;
                    c.ta = (t & 15) as usize & 3;
                }
                p += len;
                return jpeg_scan(d, p, w, h, hmax, vmax, restart, &qt, &dc, &ac, comps);
            }
            _ => {}
        }
        p += len;
    }
}

#[allow(clippy::too_many_arguments)]
fn jpeg_scan(d: &[u8], p: usize, w: usize, h: usize, hmax: usize, vmax: usize, restart: usize, qt: &[[u16; 64]; 4], dc: &[Option<JHuff>; 4], ac: &[Option<JHuff>; 4], mut comps: Vec<Comp>) -> Option<Tex> {
    let (mw, mh) = (8 * hmax, 8 * vmax);
    let (mcx, mcy) = (w.div_ceil(mw), h.div_ceil(mh));
    for c in comps.iter_mut() {
        c.pw = mcx * c.h * 8;
        c.plane = vec![0u8; c.pw * mcy * c.v * 8];
    }
    let mut b = JBits { d, p, buf: 0, n: 0 };
    let mut pred = vec![0i32; comps.len()];
    let mut count = 0usize;
    for my in 0..mcy {
        for mx in 0..mcx {
            if restart > 0 && count > 0 && count % restart == 0 {
                b.align();
                while *b.d.get(b.p)? != 0xFF {
                    b.p += 1;
                }
                b.p += 2;
                pred.iter_mut().for_each(|v| *v = 0);
            }
            count += 1;
            for (ci, c) in comps.iter_mut().enumerate() {
                let (dct, act) = (dc[c.td].as_ref()?, ac[c.ta].as_ref()?);
                for by in 0..c.v {
                    for bx in 0..c.h {
                        let mut blk = [0i32; 64];
                        let t = b.huff(dct)? as u32;
                        if t > 15 {
                            return None;
                        }
                        let diff = extend(b.bits(t)?, t);
                        pred[ci] += diff;
                        blk[0] = pred[ci] * qt[c.tq][0] as i32;
                        let mut k = 1;
                        while k < 64 {
                            let rs = b.huff(act)?;
                            let (r, s) = ((rs >> 4) as usize, (rs & 15) as u32);
                            if s == 0 {
                                if r == 15 {
                                    k += 16;
                                    continue;
                                }
                                break;
                            }
                            k += r;
                            if k > 63 {
                                return None;
                            }
                            let v = extend(b.bits(s)?, s);
                            blk[ZIGZAG[k]] = v * qt[c.tq][ZIGZAG[k]] as i32;
                            k += 1;
                        }
                        let mut out = [0u8; 64];
                        idct(&blk, &mut out);
                        let (ox, oy) = ((mx * c.h + bx) * 8, (my * c.v + by) * 8);
                        for yy in 0..8 {
                            let row = (oy + yy) * c.pw + ox;
                            c.plane[row..row + 8].copy_from_slice(&out[yy * 8..yy * 8 + 8]);
                        }
                    }
                }
            }
        }
    }
    let mut rgba = vec![255u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let s = |c: &Comp| c.plane[(y * c.v / vmax) * c.pw + x * c.h / hmax] as f32;
            let o = (y * w + x) * 4;
            if comps.len() == 1 {
                let g = s(&comps[0]) as u8;
                rgba[o..o + 3].copy_from_slice(&[g, g, g]);
            } else {
                let (yy, cb, cr) = (s(&comps[0]), s(&comps[1]) - 128.0, s(&comps[2]) - 128.0);
                rgba[o] = (yy + 1.402 * cr).round().clamp(0.0, 255.0) as u8;
                rgba[o + 1] = (yy - 0.344136 * cb - 0.714136 * cr).round().clamp(0.0, 255.0) as u8;
                rgba[o + 2] = (yy + 1.772 * cb).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Some(Tex { w, h, rgba })
}
