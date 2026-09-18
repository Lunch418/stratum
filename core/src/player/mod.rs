//! Плеер: локальный HTTP-сервер, отдающий страницу с транспортом и живым
//! окном модели. Внешних зависимостей нет — HTTP разбирается вручную,
//! ровно настолько, насколько нужно странице.
//!
//! Симуляция крутится в отдельном потоке; страница опрашивает `/frame`
//! и отправляет команды и события мыши/клавиатуры на `/event`.

pub mod api;

use crate::formats::{self, LoadedProject};
use crate::gfx::svg;
use crate::lang;
use crate::sim::{wm, Simulation};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PAGE: &str = include_str!("player.html");

#[derive(Debug, Clone)]
enum Event {
    Run,
    Pause,
    Step,
    Reset,
    Speed(u32),
    Mouse { window: String, msg: u32, x: f64, y: f64, keys: u32 },
    Key { msg: u32, vk: u32 },
}

pub struct Shared {
    pub sim: Simulation,
    pub running: bool,
    pub fps: u32,
    events: VecDeque<Event>,
    pub error: Option<String>,
    /// Проект в памяти: правки IDE ложатся сюда, симуляция собирается из него.
    pub project: LoadedProject,
    /// Разобранные тексты по имени имиджа (в нижнем регистре).
    pub models: HashMap<String, lang::Model>,
    /// Есть несохранённые правки.
    pub dirty: bool,
}

pub struct Options {
    pub project: PathBuf,
    pub libraries: Vec<PathBuf>,
    pub port: u16,
    pub fps: u32,
    /// Папка со сборкой IDE (`app/dist`); без неё отдаётся страница плеера.
    pub static_dir: Option<PathBuf>,
}

fn load(project: &PathBuf, libraries: &[PathBuf]) -> Result<LoadedProject, String> {
    formats::load_project(project, libraries)
        .map_err(|e| format!("{}: {e}", project.display()))?
        .map_err(|e| e.to_string())
}

/// Запускает плеер и не возвращается, пока сервер жив.
pub fn serve(opts: Options) -> Result<(), String> {
    let project = load(&opts.project, &opts.libraries)?;
    let sim = Simulation::build(&project).map_err(|e| e.to_string())?;
    let models = project
        .classes
        .iter()
        .filter_map(|c| lang::parse(&c.text).ok().map(|m| (c.name.to_lowercase(), m)))
        .collect();
    let shared = Arc::new(Mutex::new(Shared {
        sim,
        running: false,
        fps: opts.fps.max(1),
        events: VecDeque::new(),
        error: None,
        project,
        models,
        dirty: false,
    }));
    let static_dir = opts.static_dir.clone();

    // поток симуляции: события, затем такт по расписанию
    let worker = Arc::clone(&shared);
    std::thread::spawn(move || loop {
        let started = Instant::now();
        let fps = {
            let mut s = worker.lock().unwrap();
            while let Some(ev) = s.events.pop_front() {
                apply_event(&mut s, ev);
            }
            if s.running && !s.sim.stopped && s.error.is_none() {
                if let Err(e) = s.sim.step() {
                    s.error = Some(e.message);
                    s.running = false;
                }
            }
            s.fps
        };
        let budget = Duration::from_millis((1000 / fps.max(1)) as u64);
        let spent = started.elapsed();
        if spent < budget {
            std::thread::sleep(budget - spent);
        }
    });

    let listener = TcpListener::bind(("127.0.0.1", opts.port)).map_err(|e| format!("порт {}: {e}", opts.port))?;
    println!(
        "{}: http://127.0.0.1:{}/  (Ctrl+C — выход)",
        if static_dir.is_some() { "IDE" } else { "плеер" },
        opts.port
    );
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let shared = Arc::clone(&shared);
        let static_dir = static_dir.clone();
        std::thread::spawn(move || {
            let _ = handle(stream, &shared, static_dir.as_deref());
        });
    }
    Ok(())
}

fn apply_event(s: &mut Shared, ev: Event) {
    match ev {
        Event::Run => s.running = true,
        Event::Pause => s.running = false,
        Event::Step => {
            s.running = false;
            if let Err(e) = s.sim.step() {
                s.error = Some(e.message);
            }
        }
        Event::Reset => match Simulation::build(&s.project) {
            Ok(sim) => {
                s.sim = sim;
                s.running = false;
                s.error = None;
            }
            Err(e) => s.error = Some(e.to_string()),
        },
        Event::Speed(fps) => s.fps = fps.clamp(1, 1000),
        Event::Mouse { window, msg, x, y, keys } => {
            if let Some(space) = s.sim.effects.gfx.window_space(&window) {
                // координаты страницы → координаты пространства
                let (ox, oy, k) = s
                    .sim
                    .effects
                    .gfx
                    .space(space)
                    .map(|sp| (sp.origin.0, sp.origin.1, sp.scale.0.max(0.001)))
                    .unwrap_or((0.0, 0.0, 1.0));
                let (sx, sy) = ((x + ox) / k, (y + oy) / k);
                if let Err(e) = s.sim.mouse(space, msg, sx, sy, keys) {
                    s.error = Some(e.message);
                }
            }
        }
        Event::Key { msg, vk } => {
            let spaces: Vec<_> = s.sim.effects.gfx.spaces.keys().copied().collect();
            for space in spaces {
                if let Err(e) = s.sim.key(space, msg, vk) {
                    s.error = Some(e.message);
                }
            }
        }
    }
}

fn handle(mut stream: TcpStream, shared: &Arc<Mutex<Shared>>, static_dir: Option<&std::path::Path>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body_bytes = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body_bytes)?;
    }
    let body_text = String::from_utf8_lossy(&body_bytes).into_owned();

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let (status, mime, body): (&str, &str, Vec<u8>) = match (method.as_str(), path.as_str()) {
        ("GET", "/frame") => ("200 OK", "application/json; charset=utf-8", frame_json(shared).into_bytes()),
        ("POST", "/event") | ("GET", "/event") => {
            if let Some(ev) = parse_event(&query) {
                shared.lock().unwrap().events.push_back(ev);
            }
            ("200 OK", "text/plain", b"ok".to_vec())
        }
        (m, p) if p.starts_with("/api/") => {
            let r = api::handle(m, p, &query, &body_text, shared);
            (r.status, r.mime, r.body.into_bytes())
        }
        ("GET", p) => match static_file(static_dir, p) {
            Some((mime, data)) => ("200 OK", mime, data),
            // одностраничное приложение: любой путь ведёт на index.html;
            // без сборки IDE отдаётся встроенная страница плеера
            None => match static_file(static_dir, "/index.html") {
                Some((mime, data)) => ("200 OK", mime, data),
                None => ("200 OK", "text/html; charset=utf-8", PAGE.as_bytes().to_vec()),
            },
        },
        _ => ("404 Not Found", "text/plain", "нет такой страницы".as_bytes().to_vec()),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()
}

/// Файл из папки сборки IDE; пути вне папки не отдаются.
fn static_file(dir: Option<&std::path::Path>, path: &str) -> Option<(&'static str, Vec<u8>)> {
    let dir = dir?;
    let rel = path.trim_start_matches('/');
    if rel.is_empty() || rel.contains("..") {
        return None;
    }
    let file = dir.join(rel);
    let data = std::fs::read(&file).ok()?;
    let mime = match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    };
    Some((mime, data))
}

pub(crate) fn param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| kv.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

pub(crate) fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn parse_event(query: &str) -> Option<Event> {
    let num = |k: &str| param(query, k).and_then(|v| v.parse::<f64>().ok());
    Some(match param(query, "type")? {
        "run" => Event::Run,
        "pause" => Event::Pause,
        "step" => Event::Step,
        "reset" => Event::Reset,
        "speed" => Event::Speed(num("fps")? as u32),
        "mouse" => Event::Mouse {
            window: url_decode(param(query, "win")?),
            msg: num("msg")? as u32,
            x: num("x")?,
            y: num("y")?,
            keys: num("keys").unwrap_or(0.0) as u32,
        },
        "key" => Event::Key { msg: num("msg").unwrap_or(wm::KEYDOWN as f64) as u32, vk: num("vk")? as u32 },
        _ => return None,
    })
}

pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn frame_json(shared: &Arc<Mutex<Shared>>) -> String {
    let s = shared.lock().unwrap();
    let gfx = &s.sim.effects.gfx;
    let mut windows = Vec::new();
    for (i, name) in gfx.window_order.iter().enumerate() {
        let Some(sp) = gfx.window_space(name).and_then(|h| gfx.space(h)) else { continue };
        if !sp.visible {
            continue;
        }
        windows.push(format!(
            "{{\"id\":{i},\"name\":{},\"w\":{},\"h\":{},\"svg\":{}}}",
            json_string(name),
            sp.client.0,
            sp.client.1,
            json_string(&svg::render(sp))
        ));
    }
    let mut log: Vec<String> = s.sim.effects.log.iter().rev().take(8).map(|l| json_string(l)).collect();
    if let Some(e) = &s.error {
        log.insert(0, json_string(&format!("ошибка: {e}")));
    }
    format!(
        "{{\"tick\":{},\"running\":{},\"stopped\":{},\"windows\":[{}],\"log\":[{}]}}",
        s.sim.tick_number(),
        s.running,
        s.sim.stopped,
        windows.join(","),
        log.join(",")
    )
}
