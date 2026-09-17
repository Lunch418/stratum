//! Чтение файлов Stratum 2000: имиджи, проекты, снимки состояния.
//!
//! Все форматы восстановлены по корпусу; спецификации — в `docs/formats/`.

pub mod cls;
pub mod cp1251;
pub mod project;
pub mod reader;

pub use cls::{Child, Class, Link, Variable};
pub use project::{Project, State};
pub use reader::{FormatError, Result};

use std::path::{Path, PathBuf};

/// Проект целиком: `project.spj`, все `.cls` рядом с ним и `_preload.stt`.
#[derive(Debug, Default)]
pub struct LoadedProject {
    pub dir: PathBuf,
    pub project: Project,
    /// Имиджи проекта, затем имиджи подключённых библиотек.
    pub classes: Vec<Class>,
    /// Сколько первых элементов `classes` принадлежат самому проекту.
    pub own_classes: usize,
    pub state: Option<State>,
}

impl LoadedProject {
    /// Имидж по имени, без учёта регистра — язык регистронезависим.
    pub fn class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn root(&self) -> Option<&Class> {
        self.class(&self.project.root)
    }
}

/// Открывает проект по пути к `.spj` или к папке проекта. `libraries` —
/// папки стандартной библиотеки (`library/`, `add.lib/`): их имиджи
/// подключаются после имиджей проекта и не перекрывают одноимённые.
pub fn load_project(
    path: &Path,
    libraries: &[PathBuf],
) -> std::io::Result<std::result::Result<LoadedProject, FormatError>> {
    let (dir, spj) = if path.is_dir() {
        let spj = std::fs::read_dir(path)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("spj")));
        match spj {
            Some(spj) => (path.to_path_buf(), spj),
            None => return Ok(Err(FormatError {
                path: path.display().to_string(),
                offset: 0,
                message: "в папке нет файла проекта .spj".into(),
            })),
        }
    } else {
        (path.parent().unwrap_or(Path::new(".")).to_path_buf(), path.to_path_buf())
    };

    let data = std::fs::read(&spj)?;
    let project = match project::parse_project(&data, &spj.display().to_string()) {
        Ok(p) => p,
        Err(e) => return Ok(Err(e)),
    };

    let mut loaded = LoadedProject { dir: dir.clone(), project, ..Default::default() };
    if let Err(e) = load_classes(&dir, &mut loaded.classes)? {
        return Ok(Err(e));
    }
    loaded.own_classes = loaded.classes.len();
    for lib in libraries {
        if !lib.is_dir() {
            continue;
        }
        let mut found = Vec::new();
        if let Err(e) = load_classes(lib, &mut found)? {
            return Ok(Err(e));
        }
        for c in found {
            if loaded.class(&c.name).is_none() {
                loaded.classes.push(c);
            }
        }
    }

    for name in ["_preload.stt", "_PRELOAD.STT"] {
        let p = dir.join(name);
        if p.exists() {
            let data = std::fs::read(&p)?;
            if let Ok(s) = project::parse_state(&data, &p.display().to_string()) {
                loaded.state = Some(s);
            }
            break;
        }
    }
    Ok(Ok(loaded))
}

fn load_classes(dir: &Path, out: &mut Vec<Class>) -> std::io::Result<std::result::Result<(), FormatError>> {
    let mut entries: Vec<PathBuf> = Vec::new();
    collect_classes(dir, &mut entries)?;
    entries.sort();
    for file in entries {
        let data = std::fs::read(&file)?;
        if data.len() < 2 || &data[0..2] != b"SB" {
            continue;
        }
        match cls::parse(&data, &file.display().to_string()) {
            Ok(c) => out.push(c),
            Err(e) => return Ok(Err(e)),
        }
    }
    Ok(Ok(()))
}

/// Папки библиотек по умолчанию: `STRATUM_LIBRARY` (список через `:`),
/// иначе `fixtures/library` и `fixtures/add.lib` рядом с исполняемым файлом
/// или в текущем каталоге, иначе установленный Stratum в Wine.
pub fn default_library_dirs() -> Vec<PathBuf> {
    if let Ok(env) = std::env::var("STRATUM_LIBRARY") {
        return env.split(':').filter(|s| !s.is_empty()).map(PathBuf::from).collect();
    }
    let mut candidates = Vec::new();
    for base in [Path::new("."), Path::new(".."), Path::new("../..")] {
        for sub in ["fixtures/library", "fixtures/add.lib"] {
            candidates.push(base.join(sub));
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let wine = Path::new(&home).join(".wine32/drive_c/Program Files/Stratum");
        candidates.push(wine.join("library"));
        candidates.push(wine.join("add.lib"));
    }
    candidates.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect_classes(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_classes(&path, out)?;
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cls")) {
            out.push(path);
        }
    }
    Ok(())
}
