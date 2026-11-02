//! Видеопотоки и видеофреймы (`OpenVideo`, `CreateVideoFrame2d`, ...).
//!
//! Видеофрейм - обычный растровый объект: его растр (`Dib`) перезаписывается
//! декодированным кадром, поэтому рисуется тем же путём, что и все растры.
//! Одновременно в памяти один кадр на поток; позиция - номер кадра.

use super::{encode_bmp24, Dib, Gfx, Handle, Object, Shape};
use crate::formats::avi::Avi;
use crate::runtime::value::Value;

/// Наибольшее число открытых потоков и видеофреймов одного потока.
pub const MAX_VIDEOS: usize = 16;
pub const MAX_VIDEO_FRAMES: usize = 64;

#[derive(Debug, Clone)]
pub struct Video {
    pub avi: Avi,
    pub pos: usize,
    /// Видеофреймы потока: (пространство, объект, растр).
    pub frames: Vec<(Handle, Handle, Handle)>,
    pub play: Option<Play>,
}

#[derive(Debug, Clone, Copy)]
pub struct Play {
    pub to: usize,
    pub cycled: bool,
    pub paused: bool,
}

fn f(args: &[Value], i: usize) -> f64 {
    args.get(i).map(|v| v.as_float()).unwrap_or(0.0)
}
fn h(args: &[Value], i: usize) -> Handle {
    f(args, i).max(0.0) as Handle
}
fn num(v: f64) -> Value {
    Value::Float(v)
}
fn handle(v: Handle) -> Value {
    Value::Handle(v as f64)
}

impl Gfx {
    /// Открывает AVI; при ошибке (нет файла, кодек не поддерживается) - 0 и
    /// причина в `video_error`.
    pub fn open_video(&mut self, name: &str) -> Handle {
        let Some(path) = self.find_file(name) else {
            self.video_error = format!("file not found: {name}");
            return 0;
        };
        if self.videos.len() >= MAX_VIDEOS {
            self.video_error = "too many open videos".into();
            return 0;
        }
        let too_big = std::fs::metadata(&path).map(|m| m.len() > crate::formats::avi::MAX_FILE_BYTES as u64).unwrap_or(true);
        let data = if too_big { None } else { std::fs::read(&path).ok() };
        let Some(data) = data else {
            self.video_error = format!("cannot read: {name}");
            return 0;
        };
        match Avi::parse(data) {
            Ok(avi) => {
                self.video_error.clear();
                let hnd = super::lowest_free(&self.videos);
                self.videos.insert(hnd, Video { avi, pos: 0, frames: Vec::new(), play: None });
                hnd
            }
            Err(e) => {
                self.video_error = e.to_string();
                0
            }
        }
    }

    /// Переходит на кадр `pos` и обновляет растры всех его видеофреймов.
    /// Возвращает установленную позицию (в пределах потока).
    pub fn video_seek(&mut self, video: Handle, pos: f64) -> Option<usize> {
        let v = self.videos.get_mut(&video)?;
        let last = v.avi.frame_count().checked_sub(1)?;
        let pos = if pos.is_nan() { 0 } else { (pos.max(0.0) as usize).min(last) };
        if v.frames.is_empty() {
            v.pos = pos;
            return Some(pos);
        }
        let rgb = match v.avi.frame_rgb(pos) {
            Ok(r) => r,
            Err(e) => {
                self.video_error = e.to_string();
                return Some(v.pos);
            }
        };
        v.pos = pos;
        let (w, hh) = (v.avi.width(), v.avi.height());
        let bmp = encode_bmp24(w, hh, &rgb);
        let targets = v.frames.clone();
        for (sp, _, dib) in targets {
            if let Some(d) = self.spaces.get_mut(&sp).and_then(|s| s.dibs.get_mut(&dib)) {
                d.bmp.clone_from(&bmp);
                d.pixels = None;
                d.dirty = false;
                d.width = w;
                d.height = hh;
            }
        }
        Some(pos)
    }

    /// Раз в такт: проигрываемые потоки идут на кадр вперёд.
    pub fn advance_videos(&mut self) {
        let ids: Vec<Handle> = self.videos.iter().filter(|(_, v)| v.play.is_some_and(|p| !p.paused)).map(|(k, _)| *k).collect();
        for id in ids {
            let Some(v) = self.videos.get_mut(&id) else { continue };
            let Some(pl) = v.play else { continue };
            let next = v.pos + 1;
            if next > pl.to.min(v.avi.frame_count().saturating_sub(1)) {
                if pl.cycled {
                    self.video_seek(id, 0.0);
                } else if let Some(v) = self.videos.get_mut(&id) {
                    v.play = None;
                }
            } else {
                self.video_seek(id, next as f64);
            }
        }
    }

    fn frame_video(&self, space: Handle, frame: Handle) -> Option<Handle> {
        self.videos.iter().find(|(_, v)| v.frames.iter().any(|(s, o, _)| *s == space && *o == frame)).map(|(k, _)| *k)
    }
}

pub fn call(name: &str, args: &[Value], gfx: &mut Gfx) -> Option<Value> {
    let (a0, a1) = (h(args, 0), h(args, 1));
    Some(match name {
        "openvideo" => {
            let file = args.first().map(|v| v.as_string()).unwrap_or_default();
            handle(gfx.open_video(&file))
        }
        "closevideo" => {
            let used = gfx.videos.get(&a0).is_some_and(|v| !v.frames.is_empty());
            num(if !used && gfx.videos.remove(&a0).is_some() { 1.0 } else { 0.0 })
        }
        "createvideoframe2d" => {
            let (x, y, mut w, mut hh) = (f(args, 2), f(args, 3), f(args, 4), f(args, 5));
            let Some((vw, vh, pos)) = gfx.videos.get(&a1).map(|v| (v.avi.width() as f64, v.avi.height() as f64, v.pos)) else { return Some(handle(0)) };
            if w < 0.0 {
                w = vw;
            }
            if hh < 0.0 {
                hh = vh;
            }
            if gfx.videos.get(&a1).is_some_and(|v| v.frames.len() >= MAX_VIDEO_FRAMES) {
                return Some(handle(0));
            }
            let Some(sp) = gfx.space_mut(a0) else { return Some(handle(0)) };
            let dib = sp.add_dib(Dib::new(Vec::new(), Vec::new(), None));
            let obj = sp.add_object(Object::new(0, x, y, w, hh, Shape::Bitmap { dib, src: (0.0, 0.0, vw, vh), masked: false }));
            if let Some(v) = gfx.videos.get_mut(&a1) {
                v.frames.push((a0, obj, dib));
            }
            gfx.video_seek(a1, pos as f64);
            handle(obj)
        }
        "framegetvideo2d" => handle(gfx.frame_video(a0, a1).unwrap_or(0)),
        "videosetpos2d" => num(gfx.video_seek(a0, f(args, 1)).map(|p| p as f64).unwrap_or(0.0)),
        "videogetpos2d" => num(gfx.videos.get(&a0).map(|v| v.pos as f64).unwrap_or(0.0)),
        "framesetpos2d" => {
            let Some(v) = gfx.frame_video(a0, a1) else { return Some(num(0.0)) };
            num(gfx.video_seek(v, f(args, 2)).map(|p| p as f64).unwrap_or(0.0))
        }
        "framegetpos2d" => num(gfx.frame_video(a0, a1).and_then(|v| gfx.videos.get(&v)).map(|v| v.pos as f64).unwrap_or(0.0)),
        "videoplay2d" => {
            let Some(vid) = gfx.frame_video(a0, a1) else { return Some(num(0.0)) };
            let last = gfx.videos.get(&vid).map(|v| v.avi.frame_count().saturating_sub(1)).unwrap_or(0);
            let (from, to) = (f(args, 2), f(args, 3));
            let to = if to < 0.0 { last } else { (to as usize).min(last) };
            let cycled = (f(args, 5) as i64) & 512 != 0;
            gfx.video_seek(vid, from);
            if let Some(v) = gfx.videos.get_mut(&vid) {
                v.play = Some(Play { to, cycled, paused: false });
            }
            num(1.0)
        }
        "videopause2d" | "videoresume2d" | "videostop2d" => {
            let Some(vid) = gfx.frame_video(a0, a1) else { return Some(num(0.0)) };
            if let Some(v) = gfx.videos.get_mut(&vid) {
                match name {
                    "videostop2d" => v.play = None,
                    _ => {
                        if let Some(p) = v.play.as_mut() {
                            p.paused = name == "videopause2d";
                        }
                    }
                }
            }
            num(1.0)
        }
        "framesetsrcrect2d" => {
            let rect = (f(args, 2), f(args, 3), f(args, 4), f(args, 5));
            let ok = gfx.space_mut(a0).and_then(|sp| sp.objects.get_mut(&a1)).map(|o| if let Shape::Bitmap { src, .. } = &mut o.shape { *src = rect }).is_some();
            num(if ok { 1.0 } else { 0.0 })
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_video_table_is_capped() {
        let dir = std::env::temp_dir().join(format!("stratum_vid_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.avi"), crate::formats::avi::build_rgb24(2, 2, &[vec![0; 12]], 10)).unwrap();
        let mut g = Gfx { project_dir: dir.clone(), ..Default::default() };
        let opened = (0..MAX_VIDEOS + 8).filter(|_| g.open_video("a.avi") != 0).count();
        assert_eq!(opened, MAX_VIDEOS);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
