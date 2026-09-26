//! Родной текстовый формат проекта Stratum Modern.
//!
//! Папка проекта:
//! ```text
//! project.json                 корневой имидж, свойства, переменные, список имиджей
//! state.json                   снимок значений (`_preload.stt`), если был
//! classes/<Имя>.strat.json     описание имиджа: переменные, дети, связи
//! classes/<Имя>.strat          текст имиджа
//! classes/<Имя>.icon.vdr       иконка, рисунок и схема — блоки `.vdr` как есть
//! classes/<Имя>.image.vdr
//! classes/<Имя>.scheme.vdr
//! classes/<Имя>.eq.bin         уравнения (библиотеки цепей)
//! ```
//! Всё, что нужно человеку и git, лежит текстом; двоичными остаются только
//! векторные картинки, которые в Stratum 2000 хранились так же.

use std::path::{Path, PathBuf};

use super::json::{self, object, Json};
use super::project::{Project, ProjectVariable, Property, PropertyValue, State, StateImage};
use super::{Child, Class, FormatError, Link, LinkStyle, LoadedProject, Pad, SheetOptions, Variable};

pub const FORMAT: &str = "stratum-modern/1";
pub const PROJECT_FILE: &str = "project.json";

/// Папка выглядит как проект в родном формате.
pub fn is_native(path: &Path) -> bool {
    if path.is_dir() {
        path.join(PROJECT_FILE).is_file()
    } else {
        path.file_name().is_some_and(|n| n == PROJECT_FILE)
    }
}

/// Имя файла для имиджа: убираем символы, недопустимые в Windows и в URL.
pub fn file_stem(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .collect();
    let trimmed = s.trim_matches(|c| c == '.' || c == ' ');
    if trimmed.is_empty() { "image".into() } else { trimmed.to_string() }
}

/// Уникальные имена файлов для имиджей проекта (регистр не различается —
/// Windows и macOS).
fn unique_stems(project: &LoadedProject, sep: &str) -> Vec<String> {
    let mut used: Vec<String> = Vec::new();
    for cls in &project.classes[..project.own_classes] {
        let base = file_stem(&cls.name);
        let mut stem = base.clone();
        let mut n = 2;
        while used.iter().any(|u| u.eq_ignore_ascii_case(&stem)) {
            stem = format!("{base}{sep}{n}");
            n += 1;
        }
        used.push(stem);
    }
    used
}

/// Записывает проект целиком (только собственные имиджи, без библиотек).
pub fn save(dir: &Path, project: &LoadedProject) -> std::io::Result<()> {
    let classes_dir = dir.join("classes");
    std::fs::create_dir_all(&classes_dir)?;

    let stems = unique_stems(project, "~");
    let mut list = Vec::new();
    for (cls, stem) in project.classes[..project.own_classes].iter().zip(&stems) {
        save_class(&classes_dir, stem, cls)?;
        list.push(object(vec![("name", Json::Str(cls.name.clone())), ("file", Json::Str(stem.clone()))]));
    }

    let p = &project.project;
    let properties = p
        .properties
        .iter()
        .map(|pr| match &pr.value {
            PropertyValue::Int(i) => object(vec![("key", Json::Str(pr.key.clone())), ("int", Json::Number(*i as f64))]),
            PropertyValue::Text(t) => object(vec![("key", Json::Str(pr.key.clone())), ("text", Json::Str(t.clone()))]),
        })
        .collect();
    let variables = p
        .variables
        .iter()
        .map(|v| {
            let mut pairs = vec![
                ("kind", Json::Number(v.kind as f64)),
                ("flags", Json::Number(v.flags as f64)),
                ("name", Json::Str(v.name.clone())),
                ("description", Json::Str(v.description.clone())),
            ];
            if let Some(h) = v.handle {
                pairs.push(("handle", Json::Number(h as f64)));
            }
            object(pairs)
        })
        .collect();
    // в файл попадают только библиотеки внутри папки проекта: пути на
    // машине разработчика — настройка окружения, а не часть проекта
    let libraries = project
        .library_dirs
        .iter()
        .filter_map(|d| d.strip_prefix(&project.dir).ok())
        .map(|d| Json::Str(d.display().to_string().replace('\\', "/")))
        .collect();
    let root = object(vec![
        ("format", Json::Str(FORMAT.into())),
        ("root", Json::Str(p.root.clone())),
        ("properties", Json::Array(properties)),
        ("variables", Json::Array(variables)),
        ("classes", Json::Array(list)),
        ("libraries", Json::Array(libraries)),
    ]);
    std::fs::write(dir.join(PROJECT_FILE), root.pretty())?;

    let state_path = dir.join("state.json");
    match &project.state {
        Some(state) => {
            let images = state
                .images
                .iter()
                .map(|im| {
                    object(vec![
                        ("class", Json::Str(im.class_name.clone())),
                        ("ref", Json::Number(im.reference as f64)),
                        ("handle", Json::Number(im.handle as f64)),
                        ("vars", pairs_json(&im.vars)),
                    ])
                })
                .collect();
            let j = object(vec![("root", Json::Str(state.root.clone())), ("images", Json::Array(images))]);
            std::fs::write(state_path, j.pretty())?;
        }
        None => {
            let _ = std::fs::remove_file(state_path);
        }
    }
    Ok(())
}

fn pairs_json(pairs: &[(String, String)]) -> Json {
    Json::Array(pairs.iter().map(|(a, b)| Json::Array(vec![Json::Str(a.clone()), Json::Str(b.clone())])).collect())
}

fn pairs_from(j: Option<&Json>) -> Vec<(String, String)> {
    j.and_then(Json::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|p| {
                    let p = p.as_array()?;
                    Some((p.first()?.as_str()?.to_string(), p.get(1)?.as_str()?.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn save_class(dir: &Path, stem: &str, cls: &Class) -> std::io::Result<()> {
    let vars = cls
        .vars
        .iter()
        .map(|v| {
            object(vec![
                ("name", Json::Str(v.name.clone())),
                ("type", Json::Str(v.var_type.clone())),
                ("default", Json::Str(v.default.clone())),
                ("description", Json::Str(v.description.clone())),
                ("flags", Json::Number(v.flags as f64)),
            ])
        })
        .collect();
    let children = cls
        .children
        .iter()
        .map(|c| {
            object(vec![
                ("handle", Json::Number(c.handle as f64)),
                ("class", Json::Str(c.class_name.clone())),
                ("name", Json::Str(c.name.clone())),
                ("x", Json::Number(c.x)),
                ("y", Json::Number(c.y)),
                ("flags", Json::Number(c.flags as f64)),
            ])
        })
        .collect();
    let links = cls
        .links
        .iter()
        .map(|l| {
            let mut pairs = vec![
                ("handle", Json::Number(l.handle as f64)),
                ("source", Json::Number(l.source as f64)),
                ("target", Json::Number(l.target as f64)),
                ("flags", Json::Number(l.flags as f64)),
                ("vars", pairs_json(&l.vars)),
            ];
            if !l.style.is_default() {
                pairs.push(("style", link_style_json(&l.style)));
            }
            if l.pad != 0 {
                pairs.push(("pad", Json::Number(l.pad as f64)));
            }
            object(pairs)
        })
        .collect();
    let mut pairs = vec![
        ("name", Json::Str(cls.name.clone())),
        ("description", Json::Str(cls.description.clone())),
        ("vars", Json::Array(vars)),
        ("children", Json::Array(children)),
        ("links", Json::Array(links)),
    ];
    if let Some(f) = cls.flags {
        pairs.push(("flags", Json::Number(f as f64)));
    }
    if let Some(sh) = &cls.sheet {
        pairs.push(("sheet", sheet_json(sh)));
    }
    if !cls.pads.is_empty() {
        let pads = cls.pads.iter().map(|p| object(vec![("id", Json::Number(p.id as f64)), ("x", Json::Number(p.x)), ("y", Json::Number(p.y))])).collect();
        pairs.push(("pads", Json::Array(pads)));
    }
    if let Some(f) = &cls.icon_file {
        pairs.push(("iconFile", Json::Str(f.clone())));
    }
    if let Some(i) = cls.icon_index {
        pairs.push(("iconIndex", Json::Number(i as f64)));
    }
    if let Some(t) = cls.timestamp {
        pairs.push(("timestamp", Json::Number(t as f64)));
    }
    std::fs::write(dir.join(format!("{stem}.strat.json")), object(pairs).pretty())?;

    // текст — отдельным файлом, чтобы редактировать и смотреть diff как код
    let text_path = dir.join(format!("{stem}.strat"));
    if cls.text.is_empty() {
        let _ = std::fs::remove_file(text_path);
    } else {
        std::fs::write(text_path, cls.text.replace("\r\n", "\n"))?;
    }
    for (suffix, blob) in [
        ("icon.vdr", &cls.icon),
        ("image.vdr", &cls.image),
        ("scheme.vdr", &cls.scheme),
        ("eq.bin", &cls.equations),
    ] {
        let path = dir.join(format!("{stem}.{suffix}"));
        match blob {
            Some(b) => std::fs::write(path, b)?,
            None => {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    Ok(())
}

/// Читает проект из папки родного формата (без библиотек — их подключает
/// `load_project`).
pub fn load(dir: &Path) -> std::io::Result<Result<LoadedProject, FormatError>> {
    let dir = if dir.is_dir() { dir.to_path_buf() } else { dir.parent().unwrap_or(Path::new(".")).to_path_buf() };
    let path = dir.join(PROJECT_FILE);
    let text = std::fs::read_to_string(&path)?;
    let fail = |message: String| FormatError { path: path.display().to_string(), offset: 0, message };
    let root = match json::parse(&text) {
        Ok(j) => j,
        Err(e) => return Ok(Err(fail(e))),
    };
    if root.get("format").and_then(Json::as_str) != Some(FORMAT) {
        return Ok(Err(fail(format!("ожидался формат {FORMAT}"))));
    }
    let mut project = Project { root: root.str_or("root", ""), ..Default::default() };
    for p in root.get("properties").and_then(Json::as_array).into_iter().flatten() {
        let key = p.str_or("key", "");
        let value = match p.get("int").and_then(Json::as_f64) {
            Some(i) => PropertyValue::Int(i as u32),
            None => PropertyValue::Text(p.str_or("text", "")),
        };
        project.properties.push(Property { key, value });
    }
    for v in root.get("variables").and_then(Json::as_array).into_iter().flatten() {
        project.variables.push(ProjectVariable {
            kind: v.num_or("kind", 0.0) as u16,
            flags: v.num_or("flags", 0.0) as u16,
            handle: v.get("handle").and_then(Json::as_f64).map(|h| h as u16),
            name: v.str_or("name", ""),
            description: v.str_or("description", ""),
        });
    }

    let classes_dir = dir.join("classes");
    let mut classes = Vec::new();
    for entry in root.get("classes").and_then(Json::as_array).into_iter().flatten() {
        let stem = entry.str_or("file", "");
        // имя файла имиджа, а не путь: «../../…» из чужого project.json
        // читал бы файлы вне папки проекта
        if stem.is_empty() || stem.contains(['/', '\\', ':']) || stem.starts_with('.') {
            return Ok(Err(fail(format!("недопустимое имя файла имиджа «{stem}»"))));
        }
        match load_class(&classes_dir, &stem)? {
            Ok(c) => classes.push(c),
            Err(e) => return Ok(Err(e)),
        }
    }

    let library_dirs: Vec<PathBuf> = root
        .get("libraries")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter_map(|l| l.as_str().filter(|l| !l.contains(':')).map(PathBuf::from))
        // только папки внутри проекта, как их и пишет save: библиотека
        // расширяет песочницу модели на чтение, и «/» или «../..» из чужого
        // project.json открывали бы модели весь диск
        .filter(|d| d.components().all(|c| matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir)))
        .collect();

    let mut state = None;
    let state_path = dir.join("state.json");
    if state_path.is_file() {
        let text = std::fs::read_to_string(&state_path)?;
        if let Ok(j) = json::parse(&text) {
            let images = j
                .get("images")
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
                .map(|im| StateImage {
                    class_name: im.str_or("class", ""),
                    reference: im.num_or("ref", 0.0) as u32,
                    handle: im.num_or("handle", 0.0) as u16,
                    vars: pairs_from(im.get("vars")),
                })
                .collect();
            state = Some(State { root: j.str_or("root", ""), images });
        }
    }

    Ok(Ok(LoadedProject { dir, project, own_classes: classes.len(), classes, state, library_dirs }))
}

fn load_class(dir: &Path, stem: &str) -> std::io::Result<Result<Class, FormatError>> {
    let meta_path = dir.join(format!("{stem}.strat.json"));
    let text = std::fs::read_to_string(&meta_path)?;
    let j = match json::parse(&text) {
        Ok(j) => j,
        Err(e) => return Ok(Err(FormatError { path: meta_path.display().to_string(), offset: 0, message: e })),
    };
    let read_opt = |suffix: &str| -> std::io::Result<Option<Vec<u8>>> {
        let p = dir.join(format!("{stem}.{suffix}"));
        if p.is_file() { std::fs::read(p).map(Some) } else { Ok(None) }
    };
    let source_path = dir.join(format!("{stem}.strat"));
    let source = if source_path.is_file() { std::fs::read_to_string(&source_path)? } else { String::new() };

    let vars = j
        .get("vars")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|v| Variable {
            name: v.str_or("name", ""),
            var_type: v.str_or("type", "FLOAT"),
            default: v.str_or("default", ""),
            description: v.str_or("description", ""),
            flags: v.num_or("flags", 0.0) as u32,
        })
        .collect();
    let children = j
        .get("children")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|c| Child {
            handle: c.num_or("handle", 0.0) as u16,
            class_name: c.str_or("class", ""),
            name: c.str_or("name", ""),
            x: c.num_or("x", 0.0),
            y: c.num_or("y", 0.0),
            flags: c.num_or("flags", 0.0) as u8,
        })
        .collect();
    let links = j
        .get("links")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|l| Link {
            handle: l.num_or("handle", 0.0) as u16,
            source: l.num_or("source", 0.0) as u16,
            target: l.num_or("target", 0.0) as u16,
            flags: l.num_or("flags", 0.0) as u32,
            vars: pairs_from(l.get("vars")),
            style: l.get("style").map(link_style_from).unwrap_or_default(),
            pad: l.num_or("pad", 0.0) as u16,
        })
        .collect();
    let pads: Vec<Pad> = j
        .get("pads")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .map(|p| Pad { id: p.num_or("id", 0.0) as u16, x: p.num_or("x", 0.0), y: p.num_or("y", 0.0) })
        .collect();
    let has_pads = j.get("pads").is_some();

    let mut cls = Class {
        name: j.str_or("name", stem),
        source: meta_path.display().to_string(),
        version: 0x3003,
        description: j.str_or("description", ""),
        vars,
        text: source,
        children,
        links,
        icon_file: j.get("iconFile").and_then(Json::as_str).map(str::to_string),
        icon_index: j.get("iconIndex").and_then(Json::as_f64).map(|i| i as u16),
        icon: read_opt("icon.vdr")?,
        image: read_opt("image.vdr")?,
        scheme: read_opt("scheme.vdr")?,
        bytecode: None,
        bytecode_text: None,
        sheet: j.get("sheet").map(sheet_from),
        equations: read_opt("eq.bin")?,
        timestamp: j.get("timestamp").and_then(Json::as_f64).map(|t| t as u32),
        flags: j.get("flags").and_then(Json::as_f64).map(|f| f as u32),
        pads,
    };
    // проекты, сконвертированные до появления площадок: найти их по графике
    if !has_pads {
        cls.detect_pads();
    }
    Ok(Ok(cls))
}

/// Экспорт в формат Stratum 2000: `project.spj` и `.cls` рядом.
pub fn export_stratum2000(dir: &Path, project: &LoadedProject) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("project.spj"), super::project::write_project(&project.project))?;
    if let Some(state) = &project.state {
        std::fs::write(dir.join("_preload.stt"), super::project::write_state(state))?;
    }
    let compiled = compile_classes(project);
    for (cls, stem) in compiled.iter().zip(unique_stems(project, "")) {
        let mut cls = cls.clone();
        cls.draw_pads();
        std::fs::write(dir.join(format!("{stem}.cls")), super::cls::write(&cls))?;
    }
    Ok(())
}

/// Собственные имиджи проекта с байт-кодом для Stratum 2000: оригинал
/// исполняет байт-код, а не текст. Байт-код, прочитанный из `.cls`,
/// сохраняется, если текст с тех пор не менялся (в корпусе есть имиджи со
/// старыми именами функций, которые текущий компилятор не знает). Новые
/// переменные из текста дописываются в таблицу: оригинал адресует их по
/// индексам. Имидж, который не компилируется, пишется без байт-кода.
/// Имиджи-функции проекта и библиотек по имени (в нижнем регистре).
pub fn image_functions(project: &LoadedProject) -> std::collections::HashMap<String, crate::lang::compile::ImageFunction> {
    project
        .classes
        .iter()
        .filter_map(|c| {
            let m = crate::lang::parse(&c.text).ok()?;
            m.is_function.then(|| (crate::lang::fold(&c.name), crate::lang::compile::image_function(c, &m)))
        })
        .collect()
}

/// Проверка текста компилятором в правилах Stratum 2000: неизвестные
/// функции, неподходящие аргументы, недопустимые операции. Ошибка значит,
/// что оригинал этот текст не примет.
pub fn check_text(project: &LoadedProject, cls: &Class, model: &crate::lang::Model) -> Result<(), crate::lang::compile::CompileError> {
    use crate::lang::compile::{compile, Env, Ty};
    let functions = image_functions(project);
    let constant = |n: &str| crate::runtime::constants::lookup(n);
    let function = |n: &str| functions.get(&crate::lang::fold(n)).cloned();
    let env = Env { constant: &constant, function: &function, fold_minus: true, placeholders: false };
    let known: Vec<(String, Ty)> = cls.vars.iter().map(|v| (v.name.clone(), Ty::from_name(&v.var_type))).collect();
    compile(model, &known, &env).map(|_| ())
}

pub fn compile_classes(project: &LoadedProject) -> Vec<Class> {
    use crate::lang::compile::{compile, Env, Ty};
    let functions = image_functions(project);
    let constant = |n: &str| crate::runtime::constants::lookup(n);
    let function = |n: &str| functions.get(&crate::lang::fold(n)).cloned();
    let env = Env { constant: &constant, function: &function, fold_minus: true, placeholders: false };
    project.classes[..project.own_classes]
        .iter()
        .map(|cls| {
            let mut out = cls.clone();
            if cls.bytecode.is_some() && cls.bytecode_text.as_deref() == Some(cls.text.as_str()) {
                return out;
            }
            let Ok(model) = crate::lang::parse(&cls.text) else {
                out.bytecode = None;
                return out;
            };
            let known: Vec<(String, Ty)> = cls.vars.iter().map(|v| (v.name.clone(), Ty::from_name(&v.var_type))).collect();
            match compile(&model, &known, &env) {
                Ok(c) => {
                    for (name, ty) in &c.vars[known.len()..] {
                        out.vars.push(Variable { name: name.clone(), description: String::new(), default: String::new(), var_type: ty.name().into(), flags: 0x20000 });
                    }
                    out.bytecode = Some(c.code.iter().flat_map(|w| w.to_le_bytes()).collect());
                    out.bytecode_text = Some(cls.text.clone());
                }
                Err(_) => out.bytecode = None,
            }
            out
        })
        .collect()
}

pub fn link_style_json(st: &LinkStyle) -> Json {
    object(vec![
        ("color", Json::Str(st.color.clone())),
        ("width", Json::Number(st.width as f64)),
        ("disabled", Json::Bool(st.disabled)),
        ("arrows", Json::Bool(st.arrows)),
        ("layer", Json::Number(st.layer as f64)),
    ])
}

pub fn link_style_from(j: &Json) -> LinkStyle {
    LinkStyle {
        color: j.str_or("color", ""),
        width: j.num_or("width", 0.0) as u8,
        disabled: j.get("disabled").and_then(Json::as_bool).unwrap_or(false),
        arrows: j.get("arrows").and_then(Json::as_bool).unwrap_or(false),
        layer: j.num_or("layer", 0.0) as u8,
    }
}

pub fn sheet_json(sh: &SheetOptions) -> Json {
    object(vec![
        ("gridOrigin", Json::Array(vec![Json::Number(sh.grid_origin.0), Json::Number(sh.grid_origin.1)])),
        ("gridStep", Json::Array(vec![Json::Number(sh.grid_step.0), Json::Number(sh.grid_step.1)])),
        ("gridVisible", Json::Bool(sh.grid_visible)),
        ("gridSnap", Json::Bool(sh.grid_snap)),
        ("windowStyle", Json::Str(sh.window_style.clone())),
        ("windowSize", Json::Str(sh.window_size.clone())),
        ("windowWh", Json::Array(vec![Json::Number(sh.window_wh.0), Json::Number(sh.window_wh.1)])),
        ("windowFixed", Json::Bool(sh.window_fixed)),
        ("hscroll", Json::Bool(sh.hscroll)),
        ("vscroll", Json::Bool(sh.vscroll)),
        ("autoOrigin", Json::Bool(sh.auto_origin)),
        ("layers", Json::Number(sh.layers as f64)),
        ("noSubwindows", Json::Bool(sh.no_subwindows)),
    ])
}

pub fn sheet_from(j: &Json) -> SheetOptions {
    let d = SheetOptions::default();
    let pair = |k: &str, def: (f64, f64)| -> (f64, f64) {
        match j.get(k).and_then(Json::as_array) {
            Some(a) if a.len() == 2 => (a[0].as_f64().unwrap_or(def.0), a[1].as_f64().unwrap_or(def.1)),
            _ => def,
        }
    };
    let flag = |k: &str, def: bool| j.get(k).and_then(Json::as_bool).unwrap_or(def);
    SheetOptions {
        grid_origin: pair("gridOrigin", d.grid_origin),
        grid_step: pair("gridStep", d.grid_step),
        grid_visible: flag("gridVisible", d.grid_visible),
        grid_snap: flag("gridSnap", d.grid_snap),
        window_style: j.str_or("windowStyle", &d.window_style),
        window_size: j.str_or("windowSize", &d.window_size),
        window_wh: pair("windowWh", d.window_wh),
        window_fixed: flag("windowFixed", d.window_fixed),
        hscroll: flag("hscroll", d.hscroll),
        vscroll: flag("vscroll", d.vscroll),
        auto_origin: flag("autoOrigin", d.auto_origin),
        // из JS маска может прийти со знаком (-1 = все слои)
        layers: (j.num_or("layers", d.layers as f64) as i64) as u32,
        no_subwindows: flag("noSubwindows", d.no_subwindows),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_style_and_sheet_survive_round_trip() {
        let dir = std::env::temp_dir().join(format!("stratum-native-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut cls = Class { name: "Лист".into(), version: 0x3003, ..Default::default() };
        cls.links.push(Link {
            source: 1,
            target: 2,
            handle: 1,
            flags: 0,
            vars: vec![("a".into(), "b".into())],
            style: LinkStyle { color: "#ff0000".into(), width: 2, disabled: true, arrows: true, layer: 3 },
            pad: 0,
        });
        cls.links.push(Link { source: 0, target: 2, handle: 3, vars: vec![("x".into(), "y".into())], pad: 4, ..Default::default() });
        cls.pads.push(Pad { id: 4, x: -16.0, y: 40.5 });
        cls.sheet = Some(SheetOptions { grid_visible: true, grid_step: (20.0, 25.0), layers: 0xffff_fffe, window_size: "fixed".into(), window_wh: (300.0, 200.0), ..Default::default() });
        save_class(&dir, "list", &cls).unwrap();
        let back = load_class(&dir, "list").unwrap().unwrap();
        assert_eq!(back.links[0].style, cls.links[0].style);
        assert_eq!(back.sheet, cls.sheet);
        assert_eq!(back.links[1].pad, 4);
        assert_eq!(back.pads, cls.pads);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn foreign_project_json_cannot_point_outside_the_project() {
        let dir = std::env::temp_dir().join(format!("stratum-native-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let project = LoadedProject {
            dir: dir.clone(),
            project: Project { root: "Main".into(), ..Default::default() },
            classes: vec![Class { name: "Main".into(), version: 0x3003, ..Default::default() }],
            own_classes: 1,
            state: None,
            library_dirs: vec![dir.join("lib")],
        };
        save(&dir, &project).unwrap();
        let json_path = dir.join(PROJECT_FILE);
        let text = std::fs::read_to_string(&json_path).unwrap();
        // библиотеки: наружу — отбрасываются, внутри проекта — остаются
        let hostile = text.replace("\"lib\"", "\"lib\", \"/\", \"../..\", \"C:/\", \"sub/../..\"");
        assert_ne!(hostile, text, "в project.json нет библиотеки lib");
        std::fs::write(&json_path, &hostile).unwrap();
        let loaded = load(&dir).unwrap().unwrap();
        assert_eq!(loaded.library_dirs, vec![PathBuf::from("lib")]);
        // имя файла имиджа с путём — ошибка
        let hostile = text.replace("\"file\": \"Main\"", "\"file\": \"../../etc/x\"");
        assert_ne!(hostile, text, "в project.json нет имиджа Main");
        std::fs::write(&json_path, &hostile).unwrap();
        assert!(load(&dir).unwrap().is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(file_stem("Root2787"), "Root2787");
        assert_eq!(file_stem("a/b:c?"), "a_b_c_");
        assert_eq!(file_stem("..."), "image");
    }
}
