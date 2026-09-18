//! Рендер графического пространства в SVG — для снимков и проверки на
//! примерах. Растры вкладываются как BMP через data:-URI.

use super::{Handle, Shape, Space};
use std::fmt::Write;

/// COLORREF (0x00BBGGRR) → `#rrggbb`.
pub fn color(c: u32) -> String {
    format!("#{:02x}{:02x}{:02x}", c & 0xFF, (c >> 8) & 0xFF, (c >> 16) & 0xFF)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn render(sp: &Space) -> String {
    let (w, h) = (sp.client.0.max(1.0), sp.client.1.max(1.0));
    let (ox, oy) = sp.origin;
    let k = sp.scale.0.max(0.001);
    let mut out = String::new();
    let _ = write!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
         width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" font-family=\"Arial, sans-serif\">\n\
         <rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n\
         <g transform=\"translate({} {}) scale({k})\">\n",
        -ox, -oy
    );
    for &hh in &sp.zorder {
        render_object(sp, hh, &mut out);
    }
    out.push_str("</g>\n</svg>\n");
    out
}

fn render_object(sp: &Space, h: Handle, out: &mut String) {
    let Some(o) = sp.objects.get(&h) else { return };
    if !o.visible || o.scheme_element {
        return;
    }
    match &o.shape {
        Shape::Group { children } => {
            for c in children {
                render_object(sp, *c, out);
            }
        }
        Shape::Polyline { pen, brush, points } => {
            if points.is_empty() {
                return;
            }
            let pen = sp.pens.get(pen);
            let brush = sp.brushes.get(brush);
            let stroke = match pen {
                // PS_NULL = 5 — невидимая линия
                Some(p) if p.style != 5 => color(p.color),
                _ => "none".into(),
            };
            let width = pen.map(|p| p.width.max(1) as f64).unwrap_or(1.0);
            let fill = match brush {
                // BS_NULL / BS_HOLLOW = 1 — без заливки
                Some(b) if b.style != 1 => color(b.color),
                _ => "none".into(),
            };
            let dash = match pen.map(|p| p.style) {
                Some(1) => " stroke-dasharray=\"6 3\"",
                Some(2) => " stroke-dasharray=\"1 2\"",
                Some(3) => " stroke-dasharray=\"6 2 1 2\"",
                _ => "",
            };
            let pts: Vec<String> = points.iter().map(|p| format!("{:.2},{:.2}", p.0, p.1)).collect();
            let tag = if points.len() > 2 && fill != "none" { "polygon" } else { "polyline" };
            let _ = write!(
                out,
                "<{tag} points=\"{}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{width}\"{dash} data-handle=\"{h}\"{}/>\n",
                pts.join(" "),
                name_attr(&o.name)
            );
        }
        Shape::Text { text } => {
            let Some(parts) = sp.texts.get(text) else { return };
            let mut y = o.y;
            for part in parts {
                let font = sp.fonts.get(&part.font);
                let size = font.map(|f| f.height.abs()).filter(|h| *h > 0).unwrap_or(12) as f64;
                let text = sp.strings.get(&part.string).cloned().unwrap_or_default();
                let weight = if font.is_some_and(|f| f.weight >= 600) { " font-weight=\"bold\"" } else { "" };
                let style = if font.is_some_and(|f| f.italic) { " font-style=\"italic\"" } else { "" };
                let family = font.map(|f| f.face.as_str()).filter(|f| !f.is_empty()).unwrap_or("Arial");
                let transform = if o.angle != 0.0 {
                    format!(" transform=\"rotate({:.2} {:.2} {:.2})\"", -o.angle.to_degrees(), o.x, o.y)
                } else {
                    String::new()
                };
                for line in text.split("\r\n").flat_map(|l| l.split('\n')) {
                    let _ = write!(
                        out,
                        "<text x=\"{:.2}\" y=\"{:.2}\" font-size=\"{size}\" font-family=\"{}\" fill=\"{}\"{weight}{style}{transform} data-handle=\"{h}\"{}>{}</text>\n",
                        o.x,
                        y + size * 0.8,
                        esc(family),
                        color(part.fg),
                        name_attr(&o.name),
                        esc(line)
                    );
                    y += size * 1.2;
                }
            }
        }
        Shape::Bitmap { dib, src, masked: _ } => {
            match sp.dibs.get(dib) {
                Some(d) if !d.bmp.is_empty() => {
                    // показывается фрагмент src растра, растянутый в габариты
                    // объекта: растр целиком масштабируется и обрезается
                    let (dw, dh) = (d.width.max(1) as f64, d.height.max(1) as f64);
                    let (sx, sy) = (src.0, src.1);
                    let sw = if src.2 > 0.0 { src.2 } else { dw };
                    let sh = if src.3 > 0.0 { src.3 } else { dh };
                    let (kx, ky) = (o.w / sw.max(1.0), o.h / sh.max(1.0));
                    let _ = write!(
                        out,
                        "<clipPath id=\"c{h}\"><rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/></clipPath>\n\
                         <image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"none\" clip-path=\"url(#c{h})\" data-handle=\"{h}\"{} xlink:href=\"data:image/bmp;base64,{}\"/>\n",
                        o.x, o.y, o.w, o.h,
                        o.x - sx * kx, o.y - sy * ky, dw * kx, dh * ky,
                        name_attr(&o.name), base64(&d.bmp)
                    );
                }
                _ => {
                    let _ = write!(
                        out,
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#e0e0e0\" stroke=\"#999\" data-handle=\"{h}\"{}/>\n",
                        o.x, o.y, o.w, o.h, name_attr(&o.name)
                    );
                }
            }
        }
        Shape::Control { class, text, .. } => {
            let _ = write!(
                out,
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#f0f0f0\" stroke=\"#606060\" data-handle=\"{h}\" data-class=\"{}\"{}/>\n\
                 <text x=\"{:.2}\" y=\"{:.2}\" font-size=\"12\" text-anchor=\"middle\" fill=\"#000\">{}</text>\n",
                o.x, o.y, o.w, o.h, esc(class), name_attr(&o.name),
                o.x + o.w / 2.0, o.y + o.h / 2.0 + 4.0, esc(text)
            );
        }
        Shape::Unknown => {}
    }
}

fn name_attr(name: &str) -> String {
    if name.is_empty() { String::new() } else { format!(" data-name=\"{}\"", esc(name)) }
}

pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}
