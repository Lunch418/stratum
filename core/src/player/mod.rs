//! Плеер: локальный HTTP-сервер, отдающий страницу с транспортом и живым
//! окном модели. Внешних зависимостей нет — HTTP разбирается вручную,
//! ровно настолько, насколько нужно странице.
//!
//! Симуляция крутится в отдельном потоке; страница опрашивает `/frame`
//! и отправляет команды и события мыши/клавиатуры на `/event`.

pub mod api;

use crate::formats::{Class, self, LoadedProject};
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
    Back,
    Speed(u32),
    Mouse { window: String, msg: u32, x: f64, y: f64, keys: u32 },
    Key { msg: u32, vk: u32 },
    /// Действие в контроле окна модели: код уведомления и новое значение.
    Control { window: String, handle: u32, code: u32, text: Option<String>, checked: Option<bool> },
}

/// Наблюдаемая переменная: значения по тактам для графика.
pub struct Trace {
    pub id: u32,
    pub instance: usize,
    pub var: String,
    pub points: VecDeque<(u64, f64)>,
}

pub const TRACE_LEN: usize = 2000;

/// Условная точка останова: выражение на языке Stratum в контексте
/// экземпляра (`index`) или всех экземпляров имиджа (`class`).
pub struct Breakpoint {
    pub id: u32,
    pub index: Option<usize>,
    pub class: Option<String>,
    pub expr: String,
    pub enabled: bool,
}

/// Где остановилась модель по точке останова или ошибке.
#[derive(Clone)]
pub struct Halt {
    pub kind: &'static str,
    pub message: String,
    pub instance: Option<usize>,
    pub line: u32,
}

pub const HISTORY_LEN: usize = 60;

pub struct Shared {
    pub sim: Simulation,
    /// Снимки перед каждым тактом — для шага назад.
    pub past: VecDeque<Simulation>,
    pub breakpoints: Vec<Breakpoint>,
    pub next_breakpoint: u32,
    /// Последняя остановка: ошибка или точка останова.
    pub halt: Option<Halt>,
    pub traces: Vec<Trace>,
    pub next_trace: u32,
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
    /// Есть правки, не записанные на диск.
    pub unsaved: bool,
    /// Снимки собственных имиджей для Undo/Redo (правки IDE).
    pub history: Vec<Vec<Class>>,
    pub future: Vec<Vec<Class>>,
}

impl Shared {
    /// Один такт с историей, обработкой ошибки и точками останова.
    pub fn advance(&mut self) {
        self.past.push_back(self.sim.clone());
        if self.past.len() > HISTORY_LEN {
            self.past.pop_front();
        }
        if let Err(e) = self.sim.step() {
            self.running = false;
            self.error = Some(describe_error(&self.sim, &e));
            self.halt = Some(Halt { kind: "error", message: e.message.clone(), instance: e.instance, line: e.line });
        }
        self.sample_traces();
        self.check_breakpoints();
    }

    pub fn step_back(&mut self) -> bool {
        let Some(prev) = self.past.pop_back() else { return false };
        self.sim = prev;
        self.running = false;
        self.error = None;
        self.halt = None;
        let tick = self.sim.tick_number();
        for t in &mut self.traces {
            while t.points.back().is_some_and(|p| p.0 >= tick) {
                t.points.pop_back();
            }
        }
        true
    }

    fn check_breakpoints(&mut self) {
        if self.breakpoints.is_empty() || self.error.is_some() {
            return;
        }
        let n = self.sim.instances().len();
        let bps: Vec<(u32, Option<usize>, Option<String>, String)> = self
            .breakpoints
            .iter()
            .filter(|b| b.enabled)
            .map(|b| (b.id, b.index, b.class.clone(), b.expr.clone()))
            .collect();
        for (id, index, class, expr) in bps {
            let targets: Vec<usize> = match (index, class) {
                (Some(i), _) if i < n => vec![i],
                (_, Some(c)) => (0..n).filter(|&i| self.sim.instances()[i].class_name.eq_ignore_ascii_case(&c)).collect(),
                _ => Vec::new(),
            };
            for i in targets {
                match self.sim.eval_in(i, &expr) {
                    Ok(v) if v.as_float() != 0.0 => {
                        self.running = false;
                        let path = self.sim.instances()[i].path.clone();
                        self.halt = Some(Halt { kind: "breakpoint", message: format!("точка останова #{id}: {expr} — {path}"), instance: Some(i), line: 0 });
                        self.sim.effects.log.push(format!("остановлено: {expr} в {path} (такт {})", self.sim.tick_number()));
                        return;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        self.sim.effects.log.push(format!("точка останова #{id}: {e}"));
                        if let Some(b) = self.breakpoints.iter_mut().find(|b| b.id == id) {
                            b.enabled = false;
                        }
                    }
                }
            }
        }
    }

    /// После каждого такта дописывает значения наблюдаемых переменных.
    pub fn sample_traces(&mut self) {
        let tick = self.sim.tick_number();
        let n = self.sim.instances().len();
        for t in &mut self.traces {
            if t.instance >= n {
                continue;
            }
            let Some(v) = self.sim.value(t.instance, &t.var) else { continue };
            let f = v.as_float();
            if t.points.back().is_some_and(|p| p.0 == tick) {
                continue;
            }
            t.points.push_back((tick, f));
            if t.points.len() > TRACE_LEN {
                t.points.pop_front();
            }
        }
    }

    /// Вызывается перед каждой правкой проекта: запоминает состояние.
    pub fn remember(&mut self) {
        let own = self.project.classes[..self.project.own_classes].to_vec();
        self.history.push(own);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.future.clear();
        self.dirty = true;
        self.unsaved = true;
    }


    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.history.pop() else { return false };
        let lib_count = self.project.classes.len() - self.project.own_classes;
        let current = self.swap_snapshot_keeping(snapshot, lib_count);
        self.future.push(current);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.future.pop() else { return false };
        let lib_count = self.project.classes.len() - self.project.own_classes;
        let current = self.swap_snapshot_keeping(snapshot, lib_count);
        self.history.push(current);
        true
    }

    fn swap_snapshot_keeping(&mut self, snapshot: Vec<Class>, lib_count: usize) -> Vec<Class> {
        let n = self.project.own_classes;
        let new_len = snapshot.len();
        let current: Vec<Class> = self.project.classes.splice(..n, snapshot).collect();
        self.project.own_classes = new_len;
        debug_assert_eq!(self.project.classes.len() - new_len, lib_count);
        self.reparse();
        self.dirty = true;
        self.unsaved = true;
        current
    }

    /// Перечитывает тексты собственных имиджей после Undo/Redo.
    fn reparse(&mut self) {
        for c in &self.project.classes[..self.project.own_classes] {
            match lang::parse(&c.text) {
                Ok(m) => {
                    self.models.insert(c.name.to_lowercase(), m);
                }
                Err(_) => {
                    self.models.remove(&c.name.to_lowercase());
                }
            }
        }
    }
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
    serve_with(opts, |_| {})
}

/// То же, но с уведомлением о фактическом порте (при `port = 0` порт
/// выбирает система) — нужно оболочке Tauri, чтобы открыть окно.
pub fn serve_with(opts: Options, on_ready: impl FnOnce(u16)) -> Result<(), String> {
    let project = load(&opts.project, &opts.libraries)?;
    let sim = Simulation::build(&project).map_err(|e| e.to_string())?;
    let models = project
        .classes
        .iter()
        .filter_map(|c| lang::parse(&c.text).ok().map(|m| (c.name.to_lowercase(), m)))
        .collect();
    let shared = Arc::new(Mutex::new(Shared {
        sim,
        past: VecDeque::new(),
        breakpoints: Vec::new(),
        next_breakpoint: 1,
        halt: None,
        traces: Vec::new(),
        next_trace: 1,
        running: false,
        fps: opts.fps.max(1),
        events: VecDeque::new(),
        error: None,
        project,
        models,
        dirty: false,
        unsaved: false,
        history: Vec::new(),
        future: Vec::new(),
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
                s.advance();
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
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(opts.port);
    println!(
        "{}: http://127.0.0.1:{}/  (Ctrl+C — выход)",
        if static_dir.is_some() { "IDE" } else { "плеер" },
        port
    );
    on_ready(port);
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
            s.halt = None;
            if s.error.is_none() {
                s.advance();
            }
        }
        Event::Back => {
            s.step_back();
        }
        Event::Reset => match Simulation::build(&s.project) {
            Ok(sim) => {
                s.sim = sim;
                s.past.clear();
                s.running = false;
                s.error = None;
                s.halt = None;
                for t in &mut s.traces {
                    t.points.clear();
                }
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
        Event::Control { window, handle, code, text, checked } => {
            if let Some(space) = s.sim.effects.gfx.window_space(&window) {
                if let Some(o) = s.sim.effects.gfx.space_mut(space).and_then(|sp| sp.objects.get_mut(&handle)) {
                    if let crate::gfx::Shape::Control { text: t, checked: c, .. } = &mut o.shape {
                        if let Some(text) = text {
                            *t = text;
                        }
                        if let Some(checked) = checked {
                            *c = checked;
                        }
                    }
                }
                if let Err(e) = s.sim.control_notify(space, handle, code) {
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
        "back" => Event::Back,
        "speed" => Event::Speed(num("fps")? as u32),
        "mouse" => Event::Mouse {
            window: url_decode(param(query, "win")?),
            msg: num("msg")? as u32,
            x: num("x")?,
            y: num("y")?,
            keys: num("keys").unwrap_or(0.0) as u32,
        },
        "key" => Event::Key { msg: num("msg").unwrap_or(wm::KEYDOWN as f64) as u32, vk: num("vk")? as u32 },
        "control" => Event::Control {
            window: url_decode(param(query, "win")?),
            handle: num("handle")? as u32,
            code: num("code").unwrap_or(0.0) as u32,
            text: param(query, "text").map(url_decode),
            checked: param(query, "checked").map(|v| v == "1"),
        },
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
        // контролы отдаются отдельно: страница кладёт поверх SVG настоящие
        // кнопки и поля ввода
        // как в svg::render: translate(-origin) scale(k)
        let (ox, oy, k) = (sp.origin.0, sp.origin.1, sp.scale.0.max(0.001));
        let controls: Vec<String> = sp
            .objects
            .values()
            .filter(|o| o.visible)
            .filter_map(|o| match &o.shape {
                crate::gfx::Shape::Control { class, text, style, checked, enabled, .. } => Some(format!(
                    "{{\"handle\":{},\"class\":{},\"text\":{},\"style\":{},\"checked\":{},\"enabled\":{},\"x\":{},\"y\":{},\"w\":{},\"h\":{}}}",
                    o.handle, json_string(class), json_string(text), style, checked, enabled,
                    o.x * k - ox, o.y * k - oy, o.w * k, o.h * k
                )),
                _ => None,
            })
            .collect();
        windows.push(format!(
            "{{\"id\":{i},\"name\":{},\"w\":{},\"h\":{},\"svg\":{},\"controls\":[{}]}}",
            json_string(name),
            sp.client.0,
            sp.client.1,
            json_string(&svg::render(sp)),
            controls.join(",")
        ));
    }
    let mut log: Vec<String> = s.sim.effects.log.iter().rev().take(8).map(|l| json_string(l)).collect();
    if let Some(e) = &s.error {
        log.insert(0, json_string(&format!("ошибка: {e}")));
    }
    let halt = match &s.halt {
        Some(h) => {
            let inst = h.instance.and_then(|i| s.sim.instances().get(i));
            format!(
                "{{\"kind\":{},\"message\":{},\"instance\":{},\"path\":{},\"class\":{},\"line\":{}}}",
                json_string(h.kind),
                json_string(&h.message),
                h.instance.map(|i| i.to_string()).unwrap_or("null".into()),
                json_string(inst.map(|i| i.path.as_str()).unwrap_or("")),
                json_string(inst.map(|i| i.class_name.as_str()).unwrap_or("")),
                h.line
            )
        }
        None => "null".into(),
    };
    format!(
        "{{\"tick\":{},\"running\":{},\"stopped\":{},\"canBack\":{},\"halt\":{},\"windows\":[{}],\"log\":[{}]}}",
        s.sim.tick_number(),
        s.running,
        s.sim.stopped,
        !s.past.is_empty(),
        halt,
        windows.join(","),
        log.join(",")
    )
}

/// Текст ошибки с именем экземпляра и строкой.
fn describe_error(sim: &Simulation, e: &crate::sim::interp::RuntimeError) -> String {
    let mut out = e.message.clone();
    if let Some(inst) = e.instance.and_then(|i| sim.instances().get(i)) {
        out = format!("{} [{}]: {out}", inst.path, inst.class_name);
    }
    if e.line > 0 {
        out = format!("{out} (строка {})", e.line);
    }
    out
}
