//! Планировщик симуляции: дерево экземпляров, связи, такты.
//!
//! Модель — дерево имиджей. Корневой имидж проекта разворачивается в дерево
//! экземпляров: у каждого экземпляра свои переменные, а связанные переменные
//! двух экземпляров делят одну ячейку — запись в одном сразу видна в другом.
//!
//! Такт: в начале снимается буфер «старых» значений (его читает `~`), затем
//! тексты имиджей исполняются в порядке вычисления.

pub mod interp;

use crate::formats::{Class, LoadedProject};
use crate::lang::{self, ast::Model};
use crate::runtime::builtins::Effects;
use crate::runtime::constants;
use crate::runtime::value::{Value, ValueType};
use interp::{Interpreter, RuntimeError, Vars};
use std::collections::HashMap;

#[derive(Debug)]
pub enum BuildError {
    MissingRoot(String),
    MissingClass { parent: String, class: String },
    Parse { class: String, error: lang::ParseError },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::MissingRoot(name) => write!(f, "корневой имидж {name:?} не найден"),
            BuildError::MissingClass { parent, class } => {
                write!(f, "имидж {parent:?} ссылается на {class:?}, которого нет в проекте")
            }
            BuildError::Parse { class, error } => write!(f, "имидж {class:?}: {error}"),
        }
    }
}

/// Имидж с разобранным текстом.
struct CompiledClass {
    name: String,
    model: Model,
    /// Ошибка разбора текста; мешает только если имидж попал на схему.
    parse_error: Option<lang::ParseError>,
}

/// Экземпляр имиджа на схеме.
#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub class_name: String,
    /// Путь от корня: `Root.планета`.
    pub path: String,
    pub parent: Option<usize>,
    /// handle экземпляра на схеме родителя (0 у корня).
    pub handle: u16,
    class: usize,
    /// Имя переменной в нижнем регистре → номер ячейки.
    vars: HashMap<String, usize>,
    /// Порядок объявления — для печати.
    order: Vec<String>,
}

impl Instance {
    pub fn var_names(&self) -> &[String] {
        &self.order
    }
}

pub struct Simulation {
    classes: Vec<CompiledClass>,
    /// Папка файла каждого имиджа (имя в нижнем регистре) — для
    /// `GetClassDirectory`.
    class_dirs: HashMap<String, String>,
    instances: Vec<Instance>,
    /// Текущие значения всех ячеек.
    cells: Vec<Value>,
    /// Значения на начало такта: их читает оператор `~`.
    old: Vec<Value>,
    types: Vec<ValueType>,
    /// Порядок обхода экземпляров в такте.
    order: Vec<usize>,
    pub effects: Effects,
    pub tick: u64,
    pub stopped: bool,
}

impl Simulation {
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    pub fn tick_number(&self) -> u64 {
        self.tick
    }

    /// Значение переменной экземпляра.
    pub fn value(&self, instance: usize, var: &str) -> Option<&Value> {
        let cell = *self.instances[instance].vars.get(&var.to_ascii_lowercase())?;
        Some(&self.cells[cell])
    }

    pub fn set_value(&mut self, instance: usize, var: &str, value: Value) -> bool {
        let Some(&cell) = self.instances[instance].vars.get(&var.to_ascii_lowercase()) else {
            return false;
        };
        self.cells[cell] = value.cast_to(self.types[cell]);
        true
    }

    /// Ищет экземпляр по имени или по пути `Root.планета`; регистр не важен.
    pub fn find(&self, needle: &str) -> Option<usize> {
        self.instances
            .iter()
            .position(|i| i.path.eq_ignore_ascii_case(needle))
            .or_else(|| self.instances.iter().position(|i| i.name.eq_ignore_ascii_case(needle)))
            .or_else(|| {
                self.instances.iter().position(|i| i.class_name.eq_ignore_ascii_case(needle))
            })
    }

    /// Собирает модель проекта: разбирает тексты, строит дерево экземпляров,
    /// сливает связанные переменные в общие ячейки.
    pub fn build(project: &LoadedProject) -> Result<Simulation, BuildError> {
        let mut classes = Vec::with_capacity(project.classes.len());
        for cls in &project.classes {
            let (model, parse_error) = match lang::parse(&cls.text) {
                Ok(m) => (m, None),
                Err(e) => (Model::default(), Some(e)),
            };
            classes.push(CompiledClass { name: cls.name.clone(), model, parse_error });
        }

        let root = project
            .root()
            .ok_or_else(|| BuildError::MissingRoot(project.project.root.clone()))?;

        let mut sim = Simulation {
            classes,
            class_dirs: HashMap::new(),
            instances: Vec::new(),
            cells: Vec::new(),
            old: Vec::new(),
            types: Vec::new(),
            order: Vec::new(),
            effects: Effects::default(),
            tick: 0,
            stopped: false,
        };

        // рисунки имиджей нужны окнам модели (OpenSchemeWindow)
        sim.effects.gfx.project_dir = project.dir.clone();
        sim.effects.gfx.library_dirs = project.library_dirs.clone();
        for cls in &project.classes {
            if let Some(blob) = &cls.image {
                if let Ok(pic) = crate::formats::vdr::parse(blob, &cls.name) {
                    sim.effects.gfx.pictures.insert(cls.name.to_lowercase(), pic);
                }
            }
        }
        sim.class_dirs = project
            .classes
            .iter()
            .map(|c| {
                let dir = std::path::Path::new(&c.source)
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                (c.name.to_lowercase(), dir)
            })
            .collect();

        // ячейки объединяются по связям; пока строим дерево, копим пары
        let mut merges: Vec<(usize, usize)> = Vec::new();
        let root_index = sim.add_instance(project, root, &root.name, None, "", 0)?;
        sim.expand(project, root, root_index, &mut merges)?;
        sim.apply_links(&merges);
        sim.order = (0..sim.instances.len()).collect();
        sim.apply_state(project);
        sim.old = sim.cells.clone();
        Ok(sim)
    }

    fn class_index(&self, name: &str) -> Option<usize> {
        self.classes.iter().position(|c| c.name.eq_ignore_ascii_case(name))
    }

    fn add_instance(
        &mut self,
        project: &LoadedProject,
        cls: &Class,
        name: &str,
        parent: Option<usize>,
        parent_path: &str,
        handle: u16,
    ) -> Result<usize, BuildError> {
        let class = self
            .class_index(&cls.name)
            .ok_or_else(|| BuildError::MissingRoot(cls.name.clone()))?;
        if let Some(error) = &self.classes[class].parse_error {
            return Err(BuildError::Parse { class: cls.name.clone(), error: error.clone() });
        }
        let path = if parent_path.is_empty() {
            name.to_string()
        } else {
            format!("{parent_path}.{name}")
        };
        let index = self.instances.len();
        let mut instance = Instance {
            name: name.to_string(),
            class_name: cls.name.clone(),
            path,
            parent,
            handle,
            class,
            vars: HashMap::new(),
            order: Vec::new(),
        };

        // объявленные в файле переменные — со своим типом и значением по умолчанию
        for v in &cls.vars {
            let ty = ValueType::from_name(&v.var_type);
            let value = Value::parse_default(&v.default, ty);
            let cell = self.cells.len();
            self.cells.push(value);
            self.types.push(ty);
            let key = v.name.to_ascii_lowercase();
            if instance.vars.insert(key, cell).is_none() {
                instance.order.push(v.name.clone());
            }
        }
        // переменные, объявленные только в тексте
        let declarations = self.classes[class].model.declarations.clone();
        for decl in declarations {
            let ty = ValueType::from_name(&decl.var_type);
            for var_name in &decl.names {
                let key = var_name.to_ascii_lowercase();
                if instance.vars.contains_key(&key) {
                    continue;
                }
                let cell = self.cells.len();
                self.cells.push(ty.default_value());
                self.types.push(ty);
                instance.vars.insert(key, cell);
                instance.order.push(var_name.clone());
            }
        }
        let _ = project;
        self.instances.push(instance);
        Ok(index)
    }

    /// Разворачивает схему имиджа в экземпляры детей (рекурсивно).
    fn expand(
        &mut self,
        project: &LoadedProject,
        cls: &Class,
        parent_index: usize,
        merges: &mut Vec<(usize, usize)>,
    ) -> Result<(), BuildError> {
        let parent_path = self.instances[parent_index].path.clone();
        let mut by_handle: HashMap<u16, usize> = HashMap::new();

        for child in &cls.children {
            let child_class = project.class(&child.class_name).ok_or_else(|| {
                BuildError::MissingClass {
                    parent: cls.name.clone(),
                    class: child.class_name.clone(),
                }
            })?;
            let name = Class::instance_name(child).to_string();
            let index = self.add_instance(
                project,
                child_class,
                &name,
                Some(parent_index),
                &parent_path,
                child.handle,
            )?;
            by_handle.insert(child.handle, index);
            self.expand(project, child_class, index, merges)?;
        }

        // связи схемы: пары переменных двух экземпляров делят ячейку
        for link in &cls.links {
            let source = by_handle.get(&link.source).copied();
            let target = by_handle.get(&link.target).copied();
            // handle корневого имиджа в его собственной схеме не встречается,
            // поэтому неизвестный конец связи означает сам родительский имидж
            let source = source.unwrap_or(parent_index);
            let target = target.unwrap_or(parent_index);
            for (src_var, dst_var) in &link.vars {
                let a = self.cell_of(source, src_var);
                let b = self.cell_of(target, dst_var);
                if let (Some(a), Some(b)) = (a, b) {
                    merges.push((a, b));
                }
            }
        }
        Ok(())
    }

    fn cell_of(&self, instance: usize, var: &str) -> Option<usize> {
        self.instances[instance].vars.get(&var.to_ascii_lowercase()).copied()
    }

    /// Сливает связанные ячейки: обе стороны связи начинают указывать в одну.
    fn apply_links(&mut self, merges: &[(usize, usize)]) {
        let mut parent: Vec<usize> = (0..self.cells.len()).collect();
        fn root(parent: &mut Vec<usize>, mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        for &(a, b) in merges {
            let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
            if ra != rb {
                // сохраняем ячейку с меньшим номером: она создана раньше
                let (keep, drop) = if ra < rb { (ra, rb) } else { (rb, ra) };
                parent[drop] = keep;
            }
        }
        for instance in &mut self.instances {
            for cell in instance.vars.values_mut() {
                let mut i = *cell;
                while parent[i] != i {
                    i = parent[i];
                }
                *cell = i;
            }
        }
    }

    /// Стартовые значения из `_preload.stt`.
    fn apply_state(&mut self, project: &LoadedProject) {
        let Some(state) = &project.state else { return };
        for image in &state.images {
            // запись адресует экземпляр по handle на схеме; корень — по имени
            let Some(index) = self
                .instances
                .iter()
                .position(|i| {
                    i.class_name.eq_ignore_ascii_case(&image.class_name)
                        && (i.handle == image.handle || i.parent.is_none())
                })
            else {
                continue;
            };
            for (name, text) in &image.vars {
                let Some(&cell) = self.instances[index].vars.get(&name.to_ascii_lowercase()) else {
                    continue;
                };
                self.cells[cell] = Value::parse_default(text, self.types[cell]);
            }
        }
    }

    /// Один такт: снимок старых значений, затем тексты в порядке вычисления.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        self.old.clone_from(&self.cells);
        let mut effects = std::mem::take(&mut self.effects);
        effects.clear_exit();
        let order = self.order.clone();
        let mut result = Ok(());
        for index in order {
            let class = self.instances[index].class;
            if self.classes[class].model.body.is_empty() {
                continue;
            }
            // текст временно забирается из имиджа, чтобы кадр мог занять
            // симуляцию под запись переменных
            let body = std::mem::take(&mut self.classes[class].model.body);
            {
                let mut frame = Frame { sim: self, instance: index };
                let mut interp = Interpreter::new(&mut effects);
                if let Err(e) = interp.run(&body, &mut frame) {
                    result = Err(e);
                }
            }
            self.classes[class].model.body = body;
            if result.is_err() {
                break;
            }
            // exit() прерывает только текущий имидж
            effects.clear_exit();
            if effects.stop_requested {
                self.stopped = true;
                break;
            }
        }
        self.effects = effects;
        self.tick += 1;
        result
    }

    pub fn run(&mut self, ticks: u64) -> Result<(), RuntimeError> {
        for _ in 0..ticks {
            if self.stopped {
                break;
            }
            self.step()?;
        }
        Ok(())
    }
}

/// Доступ к переменным одного экземпляра во время исполнения его текста.
struct Frame<'a> {
    sim: &'a mut Simulation,
    instance: usize,
}

impl Vars for Frame<'_> {
    fn get(&self, name: &str) -> Option<Value> {
        let cell = *self.sim.instances[self.instance].vars.get(&name.to_ascii_lowercase())?;
        Some(self.sim.cells[cell].clone())
    }

    fn get_old(&self, name: &str) -> Option<Value> {
        let cell = *self.sim.instances[self.instance].vars.get(&name.to_ascii_lowercase())?;
        Some(self.sim.old[cell].clone())
    }

    fn set(&mut self, name: &str, value: Value) {
        let key = name.to_ascii_lowercase();
        match self.sim.instances[self.instance].vars.get(&key).copied() {
            Some(cell) => {
                self.sim.cells[cell] = value.cast_to(self.sim.types[cell]);
            }
            None => {
                let cell = self.sim.cells.len();
                let ty = value.value_type();
                self.sim.types.push(ty);
                self.sim.cells.push(value);
                self.sim.old.push(ty.default_value());
                let instance = &mut self.sim.instances[self.instance];
                instance.vars.insert(key, cell);
                instance.order.push(name.to_string());
            }
        }
    }

    fn constant(&self, name: &str) -> Option<Value> {
        constants::lookup(name).map(Value::Float)
    }

    fn call_special(&mut self, name: &str, args: &[Value]) -> Option<Value> {
        let arg = |i: usize| args.get(i).map(|v| v.as_string()).unwrap_or_default();
        let lower = name.to_ascii_lowercase();
        Some(match lower.as_str() {
            // GetClassName("") — имя своего класса, ".." — родителя
            "getclassname" => {
                let target = self.resolve(&arg(0))?;
                Value::Str(self.sim.instances[target].class_name.clone())
            }
            "getclassdirectory" => {
                let class = arg(0);
                let dir = self.sim.class_dirs.get(&class.to_lowercase()).cloned().unwrap_or_default();
                Value::Str(dir)
            }
            // дескриптор экземпляра на схеме родителя
            "gethobject" => Value::Handle(self.instance as f64),
            "gethobjectbyname" => match self.sim.find(&arg(0)) {
                Some(i) => Value::Handle(i as f64),
                None => Value::Handle(0.0),
            },
            "getvarf" | "getvars" | "getvarh" | "getvarc" => {
                let target = self.resolve(&arg(0))?;
                let var = arg(1).to_ascii_lowercase();
                let cell = self.sim.instances[target].vars.get(&var).copied();
                match (lower.as_str(), cell) {
                    (_, None) => Value::Float(0.0),
                    ("getvars", Some(c)) => Value::Str(self.sim.cells[c].as_string()),
                    ("getvarh", Some(c)) => Value::Handle(self.sim.cells[c].as_float()),
                    (_, Some(c)) => Value::Float(self.sim.cells[c].as_float()),
                }
            }
            "setvar" => {
                let target = self.resolve(&arg(0))?;
                let var = arg(1).to_ascii_lowercase();
                let value = args.get(2).cloned().unwrap_or(Value::Float(0.0));
                match self.sim.instances[target].vars.get(&var).copied() {
                    Some(c) => {
                        self.sim.cells[c] = value.cast_to(self.sim.types[c]);
                        Value::Float(1.0)
                    }
                    None => Value::Float(0.0),
                }
            }
            _ => return None,
        })
    }
}

impl Frame<'_> {
    /// Экземпляр по пути: "" — сам, ".." — родитель, иначе имя класса или
    /// экземпляра на схеме.
    fn resolve(&self, path: &str) -> Option<usize> {
        let path = path.trim();
        if path.is_empty() {
            return Some(self.instance);
        }
        if path == ".." {
            return self.sim.instances[self.instance].parent;
        }
        self.sim.find(path)
    }
}
