//! Встроенные функции вне графики: строки второго эшелона, файлы и папки,
//! потоки (FILE/MEMORY), системные сведения, диалоги (без окна — ответ по
//! умолчанию и запись в сообщения), аудио через очередь звука, заглушки
//! баз данных, анализатора текста и плагинов (Ogre3D, NUI, ключи).

use super::builtins::Effects;
use super::data;
use super::Value;
use crate::formats::cp1251;
use std::collections::BTreeMap;

fn f(args: &[Value], i: usize) -> f64 {
    args.get(i).map(|v| v.as_float()).unwrap_or(0.0)
}
fn s(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.as_string()).unwrap_or_default()
}
fn num(v: f64) -> Value {
    Value::Float(v)
}
fn ok(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}
fn text(t: impl Into<String>) -> Value {
    Value::Str(t.into())
}

/// Поток: байты в памяти; файловый поток записывается при закрытии.
#[derive(Debug, Clone, Default)]
pub struct Stream {
    pub data: Vec<u8>,
    pub pos: usize,
    pub file: Option<std::path::PathBuf>,
    pub writable: bool,
    /// Ширина числа при `Read`/`Write` (0 — как текст до разделителя).
    pub width: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Streams {
    pub items: BTreeMap<u32, Stream>,
    next: u32,
}

impl Streams {
    fn open(&mut self, st: Stream) -> u32 {
        self.next += 1;
        self.items.insert(self.next, st);
        self.next
    }
}

/// Список строк как динамический массив элементов STRING.
fn string_list(fx: &mut Effects, items: Vec<String>) -> Value {
    let h = fx.arrays.new_array();
    if let Some(arr) = fx.arrays.get_mut(h) {
        for it in items {
            let mut el = data::Element { type_name: "STRING".into(), ..Default::default() };
            el.set("", Value::Str(it));
            arr.push(el);
        }
    }
    Value::Handle(h as f64)
}

/// Путь из модели: относительный — от папки проекта, `C:\…` — тоже туда.
fn resolve_path(fx: &Effects, name: &str) -> std::path::PathBuf {
    let name = name.replace('\\', "/");
    if name.len() > 2 && name.as_bytes()[1] == b':' {
        return fx.gfx.project_dir.join(name[3..].trim_start_matches('/'));
    }
    let p = std::path::Path::new(&name);
    if p.is_absolute() { p.to_path_buf() } else { fx.gfx.project_dir.join(p) }
}

/// Путь без `.` и `..`, от корня файловой системы — чтобы сравнивать папки
/// до обращения к диску (файла может ещё не быть).
fn normalize(p: &std::path::Path) -> std::path::PathBuf {
    use std::path::Component;
    let abs = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(p) };
    let mut out = std::path::PathBuf::new();
    for c in abs.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Песочница файловых функций модели (ТЗ §10): писать можно только в папку
/// проекта и во временную папку, читать — ещё из папок библиотек. Проект,
/// ещё не сохранённый на диск, пишет только во временную папку. Отказ
/// попадает в «Сообщения», функция возвращает неудачу.
pub(crate) fn sandboxed(fx: &mut Effects, name: &str, write: bool) -> Option<std::path::PathBuf> {
    let path = normalize(&resolve_path(fx, name));
    let mut roots = vec![normalize(&std::env::temp_dir())];
    if !fx.gfx.project_dir.as_os_str().is_empty() {
        roots.push(normalize(&fx.gfx.project_dir));
    }
    if !write {
        roots.extend(fx.gfx.library_dirs.iter().map(|d| normalize(d)));
        roots.extend(fx.gfx.library_dirs.iter().filter_map(|d| d.parent()).map(|d| normalize(&d.join("ICONS"))));
    }
    if roots.iter().any(|r| path.starts_with(r)) {
        Some(path)
    } else {
        fx.log.push(format!("файл вне папки проекта — {}: {}", if write { "запись запрещена" } else { "чтение запрещено" }, path.display()));
        None
    }
}

fn glob_match(pat: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match (p.first(), n.first()) {
            (None, None) => true,
            (Some('*'), _) => go(&p[1..], n) || (!n.is_empty() && go(p, &n[1..])),
            (Some('?'), Some(_)) => go(&p[1..], &n[1..]),
            (Some(a), Some(b)) if a.to_lowercase().eq(b.to_lowercase()) => go(&p[1..], &n[1..]),
            _ => false,
        }
    }
    let p: Vec<char> = pat.chars().collect();
    let n: Vec<char> = name.chars().collect();
    go(&p, &n)
}

pub fn call(name: &str, args: &[Value], fx: &mut Effects) -> Option<Value> {
    let lower = name.to_ascii_lowercase();
    let v = match lower.as_str() {
        // ── строки ───────────────────────────────────────────────────────
        "left" => text(s(args, 0).chars().take(f(args, 1).max(0.0) as usize).collect::<String>()),
        "right" => {
            let t = s(args, 0);
            let n = f(args, 1).max(0.0) as usize;
            let len = t.chars().count();
            text(t.chars().skip(len.saturating_sub(n)).collect::<String>())
        }
        "ltrim" => text(s(args, 0).trim_start()),
        "rtrim" => text(s(args, 0).trim_end()),
        "alltrim" => text(s(args, 0).trim()),
        "ansi_to_oem" | "oem_to_ansi" => text(s(args, 0)),
        "compare" => num((s(args, 0).cmp(&s(args, 1)) as i8) as f64),
        "comparei" => {
            let n = (f(args, 2).max(1.0)) as usize;
            let a: String = s(args, 0).to_lowercase().chars().take(n).collect();
            let b: String = s(args, 1).to_lowercase().chars().take(n).collect();
            num((a.cmp(&b) as i8) as f64)
        }
        "systemstr" => text(""),
        "setstringbuffermode" => num(0.0),

        // ── файлы и папки ────────────────────────────────────────────────
        "createdir" => ok(sandboxed(fx, &s(args, 0), true).is_some_and(|p| std::fs::create_dir_all(p).is_ok())),
        "deletedir" => ok(sandboxed(fx, &s(args, 0), true).is_some_and(|p| std::fs::remove_dir_all(p).is_ok())),
        "filerename" => {
            let (a, b) = (sandboxed(fx, &s(args, 0), true), sandboxed(fx, &s(args, 1), true));
            ok(matches!((&a, &b), (Some(a), Some(b)) if std::fs::rename(a, b).is_ok()))
        }
        "filecopy" => {
            let (a, b) = (sandboxed(fx, &s(args, 0), false), sandboxed(fx, &s(args, 1), true));
            ok(matches!((&a, &b), (Some(a), Some(b)) if std::fs::copy(a, b).is_ok()))
        }
        "filedelete" => ok(sandboxed(fx, &s(args, 0), true).is_some_and(|p| std::fs::remove_file(p).is_ok())),
        "getfilelist" => {
            let spec = s(args, 0).replace('\\', "/");
            let (dir, mask) = match spec.rsplit_once('/') {
                Some((d, m)) => (d.to_string(), m.to_string()),
                None => (String::new(), spec.clone()),
            };
            let mask = if mask.is_empty() { "*".to_string() } else { mask };
            let dir = if dir.is_empty() { fx.gfx.project_dir.to_string_lossy().to_string() } else { dir };
            let Some(dir) = sandboxed(fx, &dir, false) else { return Some(string_list(fx, Vec::new())) };
            let mut names: Vec<String> = std::fs::read_dir(&dir)
                .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| glob_match(&mask, n)).collect())
                .unwrap_or_default();
            names.sort();
            string_list(fx, names)
        }
        "getromdrivenames" => text(""),
        "getwindowsdirectory" | "getsystemdirectory" => text("/"),
        "getstratumdirectory" => text(std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default()),
        "gettempdirectory" => text(std::env::temp_dir().display().to_string()),

        // ── текст контрола через поток (варианты …2ds) ────────────────────
        "getcontroltext2ds" | "setcontroltext2ds" | "lbgetselindexs" => {
            let (sp, obj) = (f(args, 0) as crate::gfx::Handle, f(args, 1) as crate::gfx::Handle);
            let control = fx.gfx.space_mut(sp).and_then(|s| s.objects.get_mut(&obj));
            let Some(crate::gfx::Object { shape: crate::gfx::Shape::Control { text, style, .. }, .. }) = control else { return Some(num(0.0)) };
            match lower.as_str() {
                "getcontroltext2ds" => {
                    let bytes = crate::formats::cp1251::encode(text);
                    match fx.streams.items.get_mut(&(f(args, 2) as u32)) {
                        Some(st) => {
                            st.data = bytes;
                            st.pos = 0;
                            num(1.0)
                        }
                        None => num(0.0),
                    }
                }
                "setcontroltext2ds" => match fx.streams.items.get(&(f(args, 2) as u32)) {
                    Some(st) => {
                        *text = crate::formats::cp1251::decode(&st.data);
                        num(1.0)
                    }
                    None => num(0.0),
                },
                // выбранные строки списка — поток с номерами через перевод строки
                _ => {
                    let sel = if text.is_empty() { String::new() } else { format!("{}\n", *style >> 24) };
                    Value::Handle(fx.streams.open(Stream { data: sel.into_bytes(), writable: true, ..Default::default() }) as f64)
                }
            }
        }

        // ── потоки ───────────────────────────────────────────────────────
        "createstream" => {
            let kind = s(args, 0).to_ascii_uppercase();
            let name = s(args, 1);
            let flags = s(args, 2).to_ascii_uppercase();
            let st = match kind.as_str() {
                "MEMORY" => Some(Stream { writable: true, ..Default::default() }),
                "FILE" => {
                    let write = flags.contains("CREATE") || !flags.contains("READONLY");
                    let Some(path) = sandboxed(fx, &name, write) else { return Some(Value::Handle(0.0)) };
                    if flags.contains("CREATE") {
                        Some(Stream { file: Some(path), writable: true, ..Default::default() })
                    } else {
                        std::fs::read(&path).ok().map(|data| Stream { data, file: Some(path), writable: !flags.contains("READONLY"), ..Default::default() })
                    }
                }
                _ => None,
            };
            match st {
                Some(st) => Value::Handle(fx.streams.open(st) as f64),
                None => Value::Handle(0.0),
            }
        }
        "closestream" => {
            let h = f(args, 0) as u32;
            match fx.streams.items.remove(&h) {
                Some(st) => {
                    if let (Some(p), true) = (&st.file, st.writable) {
                        let _ = std::fs::write(p, &st.data);
                    }
                    ok(true)
                }
                None => ok(false),
            }
        }
        "streamstatus" => num(if fx.streams.items.contains_key(&(f(args, 0) as u32)) { 1.0 } else { 0.0 }),
        "seek" => ok(fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| st.pos = (f(args, 1).max(0.0) as usize).min(st.data.len())).is_some()),
        "getpos" => num(fx.streams.items.get(&(f(args, 0) as u32)).map(|st| st.pos as f64).unwrap_or(0.0)),
        "getsize" => num(fx.streams.items.get(&(f(args, 0) as u32)).map(|st| st.data.len() as f64).unwrap_or(0.0)),
        "eof" => num(fx.streams.items.get(&(f(args, 0) as u32)).map(|st| if st.pos >= st.data.len() { 1.0 } else { 0.0 }).unwrap_or(1.0)),
        "setwidth" => ok(fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| st.width = f(args, 1).max(0.0) as usize).is_some()),
        "truncate" => ok(fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| st.data.truncate(st.pos)).is_some()),
        "readln" | "getline" => {
            let stop = if lower == "getline" { s(args, 2) } else { "\n".to_string() };
            let limit = if lower == "getline" { f(args, 1).max(0.0) as usize } else { usize::MAX };
            let out = fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| {
                let start = st.pos.min(st.data.len());
                let rest = &st.data[start..];
                let stop_b = stop.as_bytes();
                let mut n = 0;
                let (mut end, mut consumed) = (rest.len(), rest.len());
                while n < rest.len() && n < limit {
                    if !stop_b.is_empty() && rest[n..].starts_with(stop_b) {
                        end = n;
                        consumed = n + stop_b.len();
                        break;
                    }
                    n += 1;
                }
                if n >= limit && n < rest.len() {
                    end = n;
                    consumed = n;
                }
                let line = cp1251::decode(&rest[..end]);
                st.pos = start + consumed;
                line.trim_end_matches('\r').to_string()
            });
            text(out.unwrap_or_default())
        }
        "read" => {
            let out = fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| {
                let start = st.pos.min(st.data.len());
                let rest = &st.data[start..];
                let taken: usize = if st.width > 0 {
                    st.width.min(rest.len())
                } else {
                    let mut i = 0;
                    while i < rest.len() && (rest[i] as char).is_whitespace() {
                        i += 1;
                    }
                    while i < rest.len() && !(rest[i] as char).is_whitespace() {
                        i += 1;
                    }
                    // один разделитель после числа тоже съедаем
                    if i < rest.len() && (rest[i] as char).is_whitespace() {
                        i += 1;
                    }
                    i
                };
                let t = String::from_utf8_lossy(&rest[..taken]).trim().to_string();
                st.pos = start + taken;
                t.parse::<f64>().unwrap_or(0.0)
            });
            num(out.unwrap_or(0.0))
        }
        "write" | "writeln" => {
            let payload = if lower == "write" {
                let v = f(args, 1);
                let t = if v == v.trunc() && v.abs() < 1e15 { format!("{}", v as i64) } else { v.to_string() };
                t + " "
            } else {
                s(args, 1) + "\r\n"
            };
            let bytes = cp1251::encode(&payload);
            ok(fx.streams.items.get_mut(&(f(args, 0) as u32)).map(|st| {
                let end = (st.pos + bytes.len()).min(st.data.len());
                let start = st.pos.min(st.data.len());
                st.data.splice(start..end, bytes.iter().copied());
                st.pos = start + bytes.len();
            }).is_some())
        }
        "copyblock" => {
            let (src, dst, from, count) = (f(args, 0) as u32, f(args, 1) as u32, f(args, 2).max(0.0) as usize, f(args, 3).max(0.0) as usize);
            let chunk: Option<Vec<u8>> = fx.streams.items.get(&src).map(|st| st.data.iter().skip(from).take(count).copied().collect());
            match (chunk, fx.streams.items.get_mut(&dst)) {
                (Some(c), Some(st)) => {
                    let start = st.pos.min(st.data.len());
                    let end = (start + c.len()).min(st.data.len());
                    st.data.splice(start..end, c.iter().copied());
                    st.pos = start + c.len();
                    ok(true)
                }
                _ => ok(false),
            }
        }
        "encryptstream" | "decryptstream" => ok(true),
        // vSave/vLoad: массив в поток текстом «тип\tполе=значение…» построчно
        "vsave" => {
            let h = f(args, 1) as u32;
            let Some(items) = fx.arrays.get(f(args, 0) as u32).cloned() else { return Some(ok(false)) };
            let mut buf = String::new();
            for el in items {
                let mut line = el.type_name.clone();
                let mut fields: Vec<_> = el.fields.iter().collect();
                fields.sort_by(|a, b| a.0.cmp(b.0));
                for (k, v) in fields {
                    line.push('\t');
                    line.push_str(k);
                    line.push('=');
                    line.push_str(&v.as_string());
                }
                buf.push_str(&line);
                buf.push('\n');
            }
            let bytes = cp1251::encode(&buf);
            ok(fx.streams.items.get_mut(&h).map(|st| {
                let start = st.pos.min(st.data.len());
                st.data.truncate(start);
                st.data.extend_from_slice(&bytes);
                st.pos = st.data.len();
            }).is_some())
        }
        "vload" => {
            let Some(st) = fx.streams.items.get(&(f(args, 0) as u32)) else { return Some(Value::Handle(0.0)) };
            let textual = cp1251::decode(&st.data[st.pos.min(st.data.len())..]);
            let h = fx.arrays.new_array();
            if let Some(arr) = fx.arrays.get_mut(h) {
                for line in textual.lines().filter(|l| !l.trim().is_empty()) {
                    let mut parts = line.split('\t');
                    let mut el = data::Element { type_name: parts.next().unwrap_or("").to_string(), ..Default::default() };
                    for kv in parts {
                        if let Some((k, v)) = kv.split_once('=') {
                            el.set(k, match v.parse::<f64>() { Ok(n) => Value::Float(n), Err(_) => Value::Str(v.to_string()) });
                        }
                    }
                    arr.push(el);
                }
            }
            Value::Handle(h as f64)
        }

        // ── системные ────────────────────────────────────────────────────
        // местные дата и время
        "getdate" => {
            let (y, m, d, ..) = super::clock::split(super::clock::local_now());
            fx.outputs.push((0, num(y as f64)));
            fx.outputs.push((1, num(m as f64)));
            fx.outputs.push((2, num(d as f64)));
            num(1.0)
        }
        "gettime" => {
            let (.., hh, mm, ss) = super::clock::split(super::clock::local_now());
            let hundredths = super::clock::hundredths();
            fx.outputs.push((0, num(hh as f64)));
            fx.outputs.push((1, num(mm as f64)));
            fx.outputs.push((2, num(ss as f64)));
            fx.outputs.push((3, num(hundredths as f64)));
            num(1.0)
        }
        "getkeyboardlayout" => num(1049.0),
        "gettitleheight" => num(30.0),
        "getsmalltitleheight" => num(22.0),
        "getfixedframewidth" | "getfixedframeheight" | "getsizeframewidth" | "getsizeframeheight" => num(4.0),
        "getmousepos" => {
            fx.outputs.push((1, num(0.0)));
            fx.outputs.push((2, num(0.0)));
            num(1.0)
        }
        "showcursor" | "setstandartcursor" | "loadcursor" | "sethyperjump" | "stdhyperjump" | "windowintaskbar" | "setwindowowner" | "setwindowparent" | "setwindowregion" | "setscrollrange" => num(1.0),
        "cascadewindows" | "tile" | "arrangeicons" => num(1.0),
        "winexecute" | "shell" | "shellwait" | "sendsms" | "sendmail" | "senddata" => {
            fx.log.push(format!("{name}: {}", s(args, 0)));
            num(0.0)
        }
        "joygetx" | "joygety" | "joygetz" | "joygetbuttons" => num(0.0),
        "loadmenu" | "deletemenu" | "checkmenuitem" | "enablemenuitem" => num(1.0),
        "screenshot" => Value::Handle(0.0),

        // ── диалоги: окна нет, ответ по умолчанию и запись в сообщения ────
        "messagebox" => {
            let style = f(args, 2) as u32;
            // ответ по умолчанию: IDOK (1); для «да/нет» — IDYES (6)
            let default = if style & 0xf == 4 || style & 0xf == 3 { 6.0 } else { 1.0 };
            fx.ask(super::builtins::DialogRequest { kind: "message", title: s(args, 1), text: s(args, 0), style, default: String::new() }, num(default))
        }
        "dialog" | "dialogex" | "dialogbox" => num(1.0),
        // файловые диалоги: заголовок, путь/имя по умолчанию, фильтр расширений
        "fileloaddialog" | "filesavedialog" | "chosefolderdialog" | "choosefolderdialog" => {
            let kind = match lower.as_str() { "filesavedialog" => "save", "fileloaddialog" => "open", _ => "folder" };
            let d = s(args, 1);
            fx.ask(super::builtins::DialogRequest { kind, title: s(args, 0), text: s(args, 2), style: 0, default: d.clone() }, text(d))
        }
        "chosecolordialog" => {
            let c = f(args, 1);
            fx.ask(super::builtins::DialogRequest { kind: "color", title: s(args, 0), text: String::new(), style: 0, default: format!("{}", c as u32) }, Value::Color(c))
        }
        "meditor" | "maddcolumn" | "maddrow" => num(1.0),

        // ── аудио: через очередь звука плеера ─────────────────────────────
        "audioopensound" => {
            let file = s(args, 0);
            let n = fx.mci.len() + 1;
            fx.mci.insert(format!("audio{n}"), file);
            Value::Handle(n as f64)
        }
        "audioplay" => {
            let key = format!("audio{}", f(args, 0) as u32);
            if let Some(file) = fx.mci.get(&key).cloned() {
                fx.sounds.push(("play".into(), file, false));
            }
            num(1.0)
        }
        "audiostop" | "audioreset" => {
            let key = format!("audio{}", f(args, 0) as u32);
            if let Some(file) = fx.mci.get(&key).cloned() {
                fx.sounds.push(("stop".into(), file, false));
            }
            num(1.0)
        }
        "audioisplaying" | "audiogetrepeat" | "audiogetbalance" | "audiogettone" | "audiogetposition" | "audiogetlength" | "audioisseekable" => num(0.0),
        "audiogetvolume" => num(1.0),
        "audiosetrepeat" | "audiosetbalance" | "audiosetposition" => num(1.0),
        "mcisendstringstr" | "mcisendstringex" | "getmcierrorstr" => text(""),
        "getlastmcierror" | "getlastmcieerror" => num(0.0),
        // видео: кадры не показываем
        "openvideo" | "createvideoframe2d" | "framegetvideo2d" => Value::Handle(0.0),
        "videosetpos2d" | "framesetpos2d" | "videoplay2d" | "videopause2d" | "videoresume2d" | "videostop2d" | "framesetsrcrect2d" | "videogetpos2d" | "getvideomarker" => num(0.0),

        // ── базы данных, анализатор текста, ключи, NUI: подсистем нет ─────
        // Отвечаем неудачей и считаем вызовы — плеер покажет их списком
        // неподдерживаемых, а не будет молча делать вид, что всё работает.
        n if n.starts_with("db") => {
            *fx.missing.entry(name.to_string()).or_insert(0) += 1;
            if n.ends_with("str") || n.contains("name") || n == "dbgetfield" { text("") } else { num(0.0) }
        }
        // InitAnalyzer: 0 — подключились к базе MySQL, 1 — нет
        "initanalyzer" => {
            *fx.missing.entry(name.to_string()).or_insert(0) += 1;
            num(1.0)
        }
        "analyseword" | "getsentancetree" | "morphdivide" | "worddivide" | "getwordform" | "getwordinfo" | "getwordproperty" | "getwordpropertyinsent" | "getwordinsentbyrole" | "getanswer"
        | "getuserkeyvalue" | "getuserkeyfullvalue" | "nui_getdevicename" => {
            *fx.missing.entry(name.to_string()).or_insert(0) += 1;
            text("")
        }
        "setmorphdivide" | "findnextword" | "findprevword" | "getwordformcount" | "searchwords"
        | "senduserresult" | "copyuserresult" | "userkeyisautorized" | "readuserkey" | "readprojectkey"
        | "nui_destroyinstance" | "nui_getskeletonpositions" => {
            *fx.missing.entry(name.to_string()).or_insert(0) += 1;
            num(0.0)
        }
        // Ogre3D — отдельный движок; в ядре не поддерживается, считаем вызовы
        n if is_ogre(n) => {
            *fx.missing.entry(name.to_string()).or_insert(0) += 1;
            Value::Handle(0.0)
        }
        // служебные опкоды компилятора и решателя из таблиц — не функции языка
        "jmp" | "jnz" | "jz" | "push" | "push_new" | "push_cst" | "vfunction" | "dllfunction" | "getelement" | "setelement" | "diff0" | "diff1" | "diff2" | "equation" | "dequation" => num(0.0),

        _ => return None,
    };
    Some(v)
}

fn is_ogre(n: &str) -> bool {
    const PREFIXES: [&str; 22] = [
        "animationstate_", "billboardset_", "billboard_", "camera_", "entity_", "light_", "material_", "materialmanager_", "mesh_", "node_",
        "overlay_", "overlayelement_", "overlaymanager_", "particlesystem_", "renderwindow_", "root_", "scenemanager_", "scenenode_",
        "textureunitstate_", "viewport_", "compositor", "skeleton_",
    ];
    PREFIXES.iter().any(|p| n.starts_with(p))
}

/// Год, месяц, день по системным часам (григорианский календарь).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_stream_round_trip() {
        let mut fx = Effects::default();
        let h = call("CreateStream", &[Value::Str("MEMORY".into()), Value::Str("".into()), Value::Str("".into())], &mut fx).unwrap().as_float();
        call("WriteLn", &[Value::Float(h), Value::Str("привет".into())], &mut fx);
        call("Write", &[Value::Float(h), Value::Float(42.0)], &mut fx);
        call("Seek", &[Value::Float(h), Value::Float(0.0)], &mut fx);
        assert_eq!(call("ReadLn", &[Value::Float(h)], &mut fx).unwrap().as_string(), "привет");
        assert_eq!(call("Read", &[Value::Float(h)], &mut fx).unwrap().as_float(), 42.0);
        assert_eq!(call("Eof", &[Value::Float(h)], &mut fx).unwrap().as_float(), 1.0);
    }

    #[test]
    fn glob_and_strings() {
        assert!(glob_match("*.cls", "Root.CLS"));
        assert!(!glob_match("*.cls", "Root.spj"));
        let mut fx = Effects::default();
        assert_eq!(call("Right", &[Value::Str("Stratum".into()), Value::Float(3.0)], &mut fx).unwrap().as_string(), "tum");
        assert_eq!(call("Alltrim", &[Value::Str("  a b  ".into())], &mut fx).unwrap().as_string(), "a b");
    }
}

#[cfg(test)]
mod sandbox_tests {
    use super::*;

    #[test]
    fn model_files_stay_inside_the_project() {
        let project = std::env::temp_dir().join(format!("stratum-sandbox-{}", std::process::id())).join("project");
        std::fs::create_dir_all(&project).unwrap();
        let mut fx = Effects::default();
        fx.gfx.project_dir = project.clone();
        assert!(sandboxed(&mut fx, "data.txt", true).is_some());
        assert!(sandboxed(&mut fx, "C:\\SC3\\data.txt", true).is_some(), "диск Windows — в папку проекта");
        // временная папка разрешена, выход через .. — только в её пределах
        assert!(sandboxed(&mut fx, "../outside.txt", true).is_some());
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".into());
        assert!(sandboxed(&mut fx, &format!("{home}/.bashrc"), true).is_none());
        assert!(sandboxed(&mut fx, "/etc/passwd", false).is_none());
        assert!(fx.log.iter().any(|l| l.contains("запрещен")));
        let _ = std::fs::remove_dir_all(project.parent().unwrap());
    }
}
