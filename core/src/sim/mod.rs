//! Планировщик симуляции: дерево экземпляров, связи, такты.
//!
//! Модель — дерево имиджей. Корневой имидж проекта разворачивается в дерево
//! экземпляров: у каждого экземпляра свои переменные, а связанные переменные
//! двух экземпляров делят одну ячейку — запись в одном сразу видна в другом.
//!
//! Такт: в начале снимается буфер «старых» значений (его читает `~`), затем
//! тексты имиджей исполняются в порядке вычисления.

pub mod equations;
pub mod interp;

use crate::formats::{Class, LoadedProject};
use crate::lang::{self, ast::{Expr, Model, Stmt}};
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
#[derive(Clone)]
struct CompiledClass {
    name: String,
    model: Model,
    /// Текст в разделяемой обёртке: по сообщению имидж может исполняться,
    /// пока другой экземпляр того же класса ещё считается.
    body: std::sync::Arc<Vec<Stmt>>,
    /// Ошибка разбора текста; мешает только если имидж попал на схему.
    parse_error: Option<lang::ParseError>,
    /// Уравнения и неизвестные текста — для решателя.
    equations: equations::ClassEquations,
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
    /// Номер описания имиджа в проекте (по имени класса).
    fn class_index_in(&self, project: &LoadedProject) -> Option<usize> {
        project.classes.iter().position(|c| c.name.eq_ignore_ascii_case(&self.class_name))
    }

    pub fn var_names(&self) -> &[String] {
        &self.order
    }
}

/// Подписка имиджа на сообщение окна (`RegisterObject`).
#[derive(Debug, Clone)]
pub struct Registration {
    pub instance: usize,
    pub space: crate::gfx::Handle,
    /// Объект, над которым должна быть мышь (0 — любой).
    pub object: crate::gfx::Handle,
    pub msg: u32,
    pub flags: u32,
}

pub mod wm {
    pub const KEYDOWN: u32 = 256;
    pub const KEYUP: u32 = 257;
    pub const MOUSEMOVE: u32 = 512;
    pub const LBUTTONDOWN: u32 = 513;
    pub const LBUTTONUP: u32 = 514;
    pub const LBUTTONDBLCLK: u32 = 515;
    pub const RBUTTONDOWN: u32 = 516;
    pub const RBUTTONUP: u32 = 517;
    pub const MBUTTONDOWN: u32 = 519;
    pub const MBUTTONUP: u32 = 520;
    pub const ALLMOUSEMESSAGE: u32 = 1536;
    pub const ALLKEYMESSAGE: u32 = 1537;
    pub const SPACEDONE: u32 = 1539;
    pub const CONTROLNOTIFY: u32 = 1544;
    pub const SPACEINIT: u32 = 1540;
    /// Флаг регистрации: сообщение только когда мышь над объектом.
    pub const FLAG_OVER_OBJECT: u32 = 1;
    /// Флаг регистрации: доставлять и на паузе.
    pub const FLAG_ALWAYS: u32 = 256;
}

#[derive(Clone)]
pub struct Simulation {
    classes: Vec<CompiledClass>,
    pub registrations: Vec<Registration>,
    /// Описания имиджей — для экземпляров, создаваемых на ходу (функции).
    class_meta: Vec<Class>,
    /// Экземпляр-«стек» для каждого имиджа-функции (по номеру класса).
    function_instances: HashMap<usize, usize>,
    /// Нажатые сейчас виртуальные клавиши (для `GetAsyncKeyState`).
    pub keys_down: std::collections::HashSet<u32>,
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
    /// Время текста каждого экземпляра за последний такт, наносекунды.
    pub profile: Vec<u64>,
    /// Отложенные присваивания `::=` (экземпляр, переменная, значение).
    deferred: Vec<(usize, String, Value)>,
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
            let body = std::sync::Arc::new(model.body.clone());
            let equations = equations::collect(&model.body);
            classes.push(CompiledClass { name: cls.name.clone(), model, body, parse_error, equations });
        }

        let root = project
            .root()
            .ok_or_else(|| BuildError::MissingRoot(project.project.root.clone()))?;

        let mut sim = Simulation {
            classes,
            registrations: Vec::new(),
            class_meta: project.classes.clone(),
            function_instances: HashMap::new(),
            keys_down: std::collections::HashSet::new(),
            class_dirs: HashMap::new(),
            instances: Vec::new(),
            cells: Vec::new(),
            old: Vec::new(),
            types: Vec::new(),
            order: Vec::new(),
            effects: Effects::default(),
            tick: 0,
            stopped: false,
            profile: Vec::new(),
            deferred: Vec::new(),
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
        sim.old = sim.cells.clone();
        // служебные переменные со свойствами объекта на схеме (в оригинале
        // заполняются по «Стоп» и попадают в снимок; у нас — всегда)
        for i in 0..sim.instances.len() {
            let (handle, name, class) = (sim.instances[i].handle, sim.instances[i].name.clone(), sim.instances[i].class_name.clone());
            let pos = sim.instances[i].parent.and_then(|p| {
                let parent_class = &project.classes[sim.instances[p].class_index_in(project)?];
                parent_class.children.iter().find(|c| c.handle == handle).map(|c| (c.x, c.y))
            });
            sim.set_var(i, "_hobject", Value::Handle(handle as f64));
            sim.set_var(i, "_objname", Value::Str(name));
            sim.set_var(i, "_classname", Value::Str(class));
            if let Some((x, y)) = pos {
                sim.set_var(i, "orgx", Value::Float(x));
                sim.set_var(i, "orgy", Value::Float(y));
            }
        }
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
    /// Имиджи с `_enable = 0` или `_disable = 1` пропускаются вместе со своей
    /// подсхемой — они работают только по сообщениям.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        self.old.clone_from(&self.cells);
        self.solve_equations()?;
        let order = self.order.clone();
        let mut disabled_roots: Vec<usize> = Vec::new();
        for index in order {
            if self.stopped {
                break;
            }
            if self.is_disabled(index) {
                disabled_roots.push(index);
                continue;
            }
            if disabled_roots.iter().any(|&d| self.descends_from(index, d)) {
                continue;
            }
            let started = std::time::Instant::now();
            self.run_instance(index)?;
            if self.profile.len() != self.instances.len() {
                self.profile.resize(self.instances.len(), 0);
            }
            self.profile[index] = started.elapsed().as_nanos() as u64;
        }
        // отложенные присваивания — после всех текстов такта
        let deferred = std::mem::take(&mut self.deferred);
        for (instance, name, value) in deferred {
            if instance < self.instances.len() {
                self.set_var(instance, &name, value);
            }
        }
        self.tick += 1;
        self.effects.gfx.flush_dibs();
        Ok(())
    }

    fn is_disabled(&self, index: usize) -> bool {
        let vars = &self.instances[index].vars;
        let enable = vars.get("_enable").map(|&c| self.cells[c].as_float());
        let disable = vars.get("_disable").map(|&c| self.cells[c].as_float());
        enable == Some(0.0) || disable.is_some_and(|d| d != 0.0)
    }

    fn descends_from(&self, mut index: usize, ancestor: usize) -> bool {
        while let Some(p) = self.instances[index].parent {
            if p == ancestor {
                return true;
            }
            index = p;
        }
        false
    }

    /// Исполняет текст одного экземпляра (в такте или по сообщению).
    fn run_instance(&mut self, index: usize) -> Result<(), RuntimeError> {
        let class = self.instances[index].class;
        let body = std::sync::Arc::clone(&self.classes[class].body);
        if body.is_empty() {
            return Ok(());
        }
        self.effects.clear_exit();
        let result = {
            let immediate = self.classes[class].model.is_function;
            let mut frame = Frame { sim: self, instance: index, immediate };
            let mut interp = Interpreter::new();
            interp.run(&body, &mut frame).map(|_| ()).map_err(|mut e| {
                if e.line == 0 {
                    e.line = interp.line;
                }
                e.instance.get_or_insert(index);
                e
            })
        };
        // exit() прерывает только текущий имидж
        self.effects.clear_exit();
        if self.effects.stop_requested {
            self.stopped = true;
        }
        result
    }

    /// Уравнения всех экземпляров решаются как одна система относительно
    /// неизвестных (`? x, y`); значения ложатся в ячейки до текстов такта.
    fn solve_equations(&mut self) -> Result<(), RuntimeError> {
        // (экземпляр, левая, правая)
        let mut eqs: Vec<(usize, Expr, Expr)> = Vec::new();
        let mut cells: Vec<usize> = Vec::new();
        for i in 0..self.instances.len() {
            if self.is_disabled(i) {
                continue;
            }
            let class = self.instances[i].class;
            if self.classes[class].equations.equations.is_empty() {
                continue;
            }
            let ce = self.classes[class].equations.clone();
            for (l, r) in ce.equations {
                eqs.push((i, l, r));
            }
            for u in &ce.unknowns {
                if let Some(&c) = self.instances[i].vars.get(u) {
                    if !cells.contains(&c) {
                        cells.push(c);
                    }
                }
            }
        }
        if eqs.is_empty() || cells.is_empty() {
            return Ok(());
        }
        let residuals = |sim: &mut Simulation| -> Result<Vec<f64>, RuntimeError> {
            let mut out = Vec::with_capacity(eqs.len());
            for (i, l, r) in &eqs {
                let immediate = sim.classes[sim.instances[*i].class].model.is_function;
                let mut frame = Frame { sim, instance: *i, immediate };
                let mut interp = Interpreter::new();
                let a = interp.eval(l, &mut frame)?.as_float();
                let b = interp.eval(r, &mut frame)?.as_float();
                out.push(a - b);
            }
            Ok(out)
        };
        for _ in 0..8 {
            let r0 = residuals(self)?;
            if r0.iter().all(|r| r.abs() < 1e-9) {
                break;
            }
            let mut jac = vec![vec![0.0; cells.len()]; eqs.len()];
            for (k, &cell) in cells.iter().enumerate() {
                let x = self.cells[cell].as_float();
                let h = 1e-6 * x.abs().max(1.0);
                self.cells[cell] = Value::Float(x + h);
                self.old[cell] = Value::Float(x + h);
                let r1 = residuals(self)?;
                self.cells[cell] = Value::Float(x);
                self.old[cell] = Value::Float(x);
                for (row, (a, b)) in r1.iter().zip(&r0).enumerate() {
                    jac[row][k] = (a - b) / h;
                }
            }
            let Some(dx) = equations::solve_step(&jac, &r0) else { break };
            let mut moved = false;
            for (k, &cell) in cells.iter().enumerate() {
                if dx[k].is_finite() && dx[k] != 0.0 {
                    let v = self.cells[cell].as_float() + dx[k];
                    self.cells[cell] = Value::Float(v);
                    self.old[cell] = Value::Float(v);
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        Ok(())
    }

    /// Живое редактирование: новый текст имиджа подменяется в работающей
    /// модели. Переменные, появившиеся в тексте, получают ячейки у всех
    /// экземпляров; прочее состояние (значения, связи, графика) сохраняется.
    pub fn hot_swap_text(&mut self, class_name: &str, model: Model) -> bool {
        let Some(class) = self.class_index(class_name) else { return false };
        let body = std::sync::Arc::new(model.body.clone());
        let declarations = model.declarations.clone();
        self.classes[class].equations = equations::collect(&model.body);
        self.classes[class].model = model;
        self.classes[class].body = body;
        self.classes[class].parse_error = None;
        for i in 0..self.instances.len() {
            if self.instances[i].class != class {
                continue;
            }
            for decl in &declarations {
                let ty = ValueType::from_name(&decl.var_type);
                for var_name in &decl.names {
                    let key = var_name.to_ascii_lowercase();
                    if self.instances[i].vars.contains_key(&key) {
                        continue;
                    }
                    let cell = self.cells.len();
                    self.cells.push(ty.default_value());
                    self.old.push(ty.default_value());
                    self.types.push(ty);
                    self.instances[i].vars.insert(key, cell);
                    self.instances[i].order.push(var_name.clone());
                }
            }
        }
        if self.profile.len() != self.instances.len() {
            self.profile.resize(self.instances.len(), 0);
        }
        true
    }

    /// Вычисляет выражение на языке Stratum в контексте экземпляра: для
    /// условных точек останова и окна наблюдения.
    pub fn eval_in(&mut self, instance: usize, src: &str) -> Result<Value, String> {
        if instance >= self.instances.len() {
            return Err("нет такого экземпляра".into());
        }
        let model = crate::lang::parse(&format!("__dbg := ({src})")).map_err(|e| e.to_string())?;
        let expr = model
            .body
            .iter()
            .find_map(|st| match st {
                crate::lang::ast::Stmt::Assign { value, .. } => Some(value.clone()),
                _ => None,
            })
            .ok_or("не выражение")?;
        let immediate = self.classes[self.instances[instance].class].model.is_function;
        let mut frame = Frame { sim: self, instance, immediate };
        let mut interp = Interpreter::new();
        interp.eval(&expr, &mut frame).map_err(|e| e.message)
    }

    /// Событие мыши в пространстве `space`: координаты в единицах
    /// пространства, `keys` — состояние кнопок (MK_LBUTTON=1, MK_RBUTTON=2).
    pub fn mouse(&mut self, space: crate::gfx::Handle, msg: u32, x: f64, y: f64, keys: u32) -> Result<(), RuntimeError> {
        let under = self.effects.gfx.space(space).and_then(|sp| sp.object_at(x, y));
        self.effects.gfx.last_primary = under.unwrap_or(0);
        let targets: Vec<Registration> = self
            .registrations
            .iter()
            .filter(|r| r.space == space && message_matches(r.msg, msg))
            .filter(|r| {
                if r.flags & wm::FLAG_OVER_OBJECT != 0 && r.object != 0 {
                    self.effects.gfx.space(space).is_some_and(|sp| covers(sp, r.object, under))
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        for r in targets {
            self.set_var(r.instance, "msg", Value::Float(msg as f64));
            self.set_var(r.instance, "xPos", Value::Float(x));
            self.set_var(r.instance, "yPos", Value::Float(y));
            self.set_var(r.instance, "fwKeys", Value::Float(keys as f64));
            self.run_instance(r.instance)?;
        }
        Ok(())
    }

    /// Уведомление от контрола (кнопка нажата, текст изменён, выбор в
    /// списке): `code` — wNotifyCode (0 = BN_CLICKED, 768 = EN_CHANGE,
    /// 1 = LBN_SELCHANGE). Текст и состояние уже применены к объекту.
    pub fn control_notify(&mut self, space: crate::gfx::Handle, object: crate::gfx::Handle, code: u32) -> Result<(), RuntimeError> {
        let targets: Vec<Registration> = self
            .registrations
            .iter()
            .filter(|r| r.space == space && (r.object == object || r.object == 0) && message_matches(r.msg, wm::CONTROLNOTIFY))
            .cloned()
            .collect();
        for r in targets {
            self.set_var(r.instance, "msg", Value::Float(wm::CONTROLNOTIFY as f64));
            self.set_var(r.instance, "wNotifyCode", Value::Float(code as f64));
            self.set_var(r.instance, "wParam", Value::Float(code as f64));
            self.set_var(r.instance, "lParam", Value::Float(object as f64));
            self.run_instance(r.instance)?;
        }
        Ok(())
    }

    /// Событие клавиатуры: `vk` — виртуальный код клавиши Windows.
    pub fn key(&mut self, space: crate::gfx::Handle, msg: u32, vk: u32) -> Result<(), RuntimeError> {
        if msg == wm::KEYDOWN {
            self.keys_down.insert(vk);
        } else if msg == wm::KEYUP {
            self.keys_down.remove(&vk);
        }
        let targets: Vec<Registration> = self
            .registrations
            .iter()
            .filter(|r| r.space == space && message_matches(r.msg, msg))
            .cloned()
            .collect();
        for r in targets {
            self.set_var(r.instance, "msg", Value::Float(msg as f64));
            self.set_var(r.instance, "wVkey", Value::Float(vk as f64));
            self.set_var(r.instance, "fwKeys", Value::Float(vk as f64));
            self.run_instance(r.instance)?;
        }
        Ok(())
    }

    /// Устанавливает переменную, если она есть у экземпляра (обе фазы).
    fn set_var(&mut self, instance: usize, name: &str, value: Value) {
        if let Some(&cell) = self.instances[instance].vars.get(&name.to_ascii_lowercase()) {
            let v = value.cast_to(self.types[cell]);
            self.cells[cell] = v.clone();
            self.old[cell] = v;
        }
    }

    /// `SendMessage` от `sender`: переменные копируются в приёмник, его
    /// текст исполняется, значения копируются обратно.
    fn send_message(&mut self, sender: usize, object: &str, class: &str, pairs: &[(String, String)]) -> Result<(), RuntimeError> {
        let mut targets: Vec<usize> = Vec::new();
        if !object.is_empty() {
            if let Some(t) = self.resolve_path(sender, object) {
                targets.push(t);
            }
        } else if !class.is_empty() {
            targets.extend((0..self.instances.len()).filter(|&i| self.instances[i].class_name.eq_ignore_ascii_case(class)));
        }
        for target in targets {
            if target == sender {
                continue;
            }
            for (from, to) in pairs {
                if let Some(v) = self.get_var_value(sender, from) {
                    self.set_var(target, to, v);
                }
            }
            self.run_instance(target)?;
            for (from, to) in pairs {
                if let Some(v) = self.get_var_value(target, to) {
                    self.set_var(sender, from, v);
                }
            }
        }
        Ok(())
    }

    fn get_var_value(&self, instance: usize, name: &str) -> Option<Value> {
        let cell = *self.instances[instance].vars.get(&name.to_ascii_lowercase())?;
        Some(self.cells[cell].clone())
    }

    /// Путь к экземпляру (справка, «Путь»): "" — сам, ".." — родитель,
    /// "\\" — корень, "Имя" — ребёнок в своей схеме, "#3" — ребёнок по
    /// handle на схеме, "..\\Имя" — брат; последним может стоять "*" или
    /// шаблон с "?" — тогда берётся первый подходящий.
    fn resolve_path(&self, from: usize, path: &str) -> Option<usize> {
        let path = path.trim().replace('/', "\\");
        if path.is_empty() {
            return Some(from);
        }
        let mut current = if let Some(rest) = path.strip_prefix('\\') {
            let root = 0;
            return self.walk_path(root, rest);
        } else {
            from
        };
        // относительный путь ищется в схеме текущего имиджа
        if !path.starts_with("..") {
            return self.walk_path(current, &path).or_else(|| self.find(&path));
        }
        for part in path.split('\\').filter(|p| !p.is_empty()) {
            current = match part {
                ".." => self.instances[current].parent?,
                _ => self.child_named(current, part).or_else(|| self.find(part))?,
            };
        }
        Some(current)
    }

    fn walk_path(&self, mut current: usize, rest: &str) -> Option<usize> {
        for part in rest.split('\\').filter(|p| !p.is_empty()) {
            current = match part {
                ".." => self.instances[current].parent?,
                _ => self.child_named(current, part)?,
            };
        }
        Some(current)
    }

    fn child_named(&self, parent: usize, part: &str) -> Option<usize> {
        if let Some(h) = part.strip_prefix('#') {
            let h: u16 = h.parse().ok()?;
            return (0..self.instances.len()).find(|&i| self.instances[i].parent == Some(parent) && self.instances[i].handle == h);
        }
        (0..self.instances.len()).find(|&i| {
            self.instances[i].parent == Some(parent)
                && (glob_match(part, &self.instances[i].name) || glob_match(part, &self.instances[i].class_name))
        })
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

/// Шаблон с "*" и "?", без учёта регистра.
fn glob_match(pattern: &str, text: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (
        pattern.to_lowercase().chars().collect(),
        text.to_lowercase().chars().collect(),
    );
    fn go(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some('*'), _) => go(&p[1..], t) || (!t.is_empty() && go(p, &t[1..])),
            (Some('?'), Some(_)) => go(&p[1..], &t[1..]),
            (Some(a), Some(b)) if a == b => go(&p[1..], &t[1..]),
            _ => false,
        }
    }
    go(&p, &t)
}

/// Подходит ли зарегистрированное сообщение под пришедшее.
fn message_matches(registered: u32, msg: u32) -> bool {
    registered == msg
        || (registered == wm::ALLMOUSEMESSAGE && (wm::MOUSEMOVE..=521).contains(&msg))
        || (registered == wm::ALLKEYMESSAGE && (msg == wm::KEYDOWN || msg == wm::KEYUP))
}

/// Объект `wanted` (или группа с ним) находится под мышью.
fn covers(sp: &crate::gfx::Space, wanted: crate::gfx::Handle, under: Option<crate::gfx::Handle>) -> bool {
    let mut cur = under;
    while let Some(h) = cur {
        if h == wanted {
            return true;
        }
        cur = sp.objects.get(&h).and_then(|o| o.parent);
    }
    false
}

/// Доступ к переменным одного экземпляра во время исполнения его текста.
struct Frame<'a> {
    sim: &'a mut Simulation,
    instance: usize,
    /// Имидж-функция: присваивание видно сразу, без фаз.
    immediate: bool,
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

    fn set_deferred(&mut self, name: &str, value: Value) {
        if self.immediate {
            self.set(name, value);
        } else {
            self.sim.deferred.push((self.instance, name.to_string(), value));
        }
    }

    fn set(&mut self, name: &str, value: Value) {
        let key = name.to_ascii_lowercase();
        match self.sim.instances[self.instance].vars.get(&key).copied() {
            Some(cell) => {
                self.sim.cells[cell] = value.cast_to(self.sim.types[cell]);
                if self.immediate {
                    self.sim.old[cell] = self.sim.cells[cell].clone();
                }
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

    fn effects(&mut self) -> &mut Effects {
        &mut self.sim.effects
    }

    fn call_special(&mut self, name: &str, args: &[Value]) -> Option<Value> {
        let arg = |i: usize| args.get(i).map(|v| v.as_string()).unwrap_or_default();
        let lower = name.to_ascii_lowercase();
        Some(match lower.as_str() {
            // GetClassName("") — имя своего класса, ".." — родителя
            "getclassname" => {
                let target = self.resolve_arg(args.first())?;
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
            "registerobject" => {
                // RegisterObject(HSpace | WindowName, hObject, path, msg, flags)
                let space = match args.first() {
                    Some(Value::Str(name)) => self.sim.effects.gfx.window_space(name).unwrap_or(0),
                    Some(v) => v.as_float() as crate::gfx::Handle,
                    None => 0,
                };
                let object = args.get(1).map(|v| v.as_float() as crate::gfx::Handle).unwrap_or(0);
                let Some(target) = self.sim.resolve_path(self.instance, &arg(2)) else { return Some(Value::Float(0.0)) };
                let msg = args.get(3).map(|v| v.as_float() as u32).unwrap_or(0);
                let flags = args.get(4).map(|v| v.as_float() as u32).unwrap_or(0);
                let dup = self.sim.registrations.iter().any(|r| r.instance == target && r.space == space && r.msg == msg);
                if space != 0 && !dup {
                    self.sim.registrations.push(Registration { instance: target, space, object, msg, flags });
                }
                Value::Float(1.0)
            }
            "unregisterobject" => {
                let space = args.first().map(|v| v.as_float() as crate::gfx::Handle).unwrap_or(0);
                let msg = args.get(3).map(|v| v.as_float() as u32);
                let target = self.sim.resolve_path(self.instance, &arg(2)).unwrap_or(self.instance);
                self.sim.registrations.retain(|r| !(r.instance == target && r.space == space && msg.is_none_or(|m| m == r.msg)));
                Value::Float(1.0)
            }
            "setcapture" => {
                let space = args.first().map(|v| v.as_float() as crate::gfx::Handle).unwrap_or(0);
                let target = self.sim.resolve_path(self.instance, &arg(1)).unwrap_or(self.instance);
                if space != 0 {
                    self.sim.registrations.push(Registration { instance: target, space, object: 0, msg: wm::ALLMOUSEMESSAGE, flags: 0x10000 });
                }
                Value::Float(1.0)
            }
            "releasecapture" => {
                self.sim.registrations.retain(|r| r.flags & 0x10000 == 0);
                Value::Float(1.0)
            }
            "getasynckeystate" => {
                let vk = args.first().map(|v| v.as_float() as u32).unwrap_or(0);
                Value::Float(if self.sim.keys_down.contains(&vk) { -32768.0 } else { 0.0 })
            }
            "sendmessage" => {
                let object = arg(0);
                let class = arg(1);
                let pairs: Vec<(String, String)> = args[2.min(args.len())..]
                    .chunks(2)
                    .filter(|c| c.len() == 2)
                    .map(|c| (c[0].as_string(), c[1].as_string()))
                    .collect();
                if let Err(e) = self.sim.send_message(self.instance, &object, &class, &pairs) {
                    self.sim.effects.log.push(format!("SendMessage: {}", e.message));
                }
                Value::Float(1.0)
            }
            "getvarf" | "getvars" | "getvarh" | "getvarc" => {
                let target = self.resolve_arg(args.first())?;
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
                let target = self.resolve_arg(args.first())?;
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
            // ── состав схемы имиджа: экземпляры, связи, переменные ─────────
            "getobjectcount" => {
                let target = self.resolve_arg(args.first())?;
                Value::Float((0..self.sim.instances.len()).filter(|&i| self.sim.instances[i].parent == Some(target)).count() as f64)
            }
            "gethobjectbynum" => {
                let target = self.resolve_arg(args.first())?;
                let n = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as usize;
                let kids: Vec<usize> = (0..self.sim.instances.len()).filter(|&i| self.sim.instances[i].parent == Some(target)).collect();
                Value::Handle(kids.get(n).map(|&i| self.sim.instances[i].handle as f64).unwrap_or(0.0))
            }
            "getobjectclass" | "getnamebyhandle" => {
                let target = self.resolve_arg(args.first())?;
                let h = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as u16;
                let child = (0..self.sim.instances.len()).find(|&i| self.sim.instances[i].parent == Some(target) && self.sim.instances[i].handle == h);
                Value::Str(child.map(|i| if lower == "getobjectclass" { self.sim.instances[i].class_name.clone() } else { self.sim.instances[i].name.clone() }).unwrap_or_default())
            }
            "setobjectname" => {
                let target = self.resolve_arg(args.first())?;
                let h = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as u16;
                let name = arg(2);
                let child = (0..self.sim.instances.len()).find(|&i| self.sim.instances[i].parent == Some(target) && self.sim.instances[i].handle == h);
                match child {
                    Some(i) => {
                        self.sim.instances[i].name = name;
                        Value::Float(1.0)
                    }
                    None => Value::Float(0.0),
                }
            }
            "getclassfile" => {
                let class = arg(0);
                Value::Str(self.sim.class_meta.iter().find(|c| c.name.eq_ignore_ascii_case(&class)).map(|c| {
                    std::path::Path::new(&c.source).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
                }).unwrap_or_default())
            }
            "getvarcount" => {
                let class = arg(0);
                Value::Float(self.sim.class_meta.iter().find(|c| c.name.eq_ignore_ascii_case(&class)).map(|c| c.vars.len() as f64).unwrap_or(0.0))
            }
            // GetVarInfo(class, n, &name, &type, &default, &description)
            "getvarinfo" => {
                let class = arg(0);
                let n = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as usize;
                match self.sim.class_meta.iter().find(|c| c.name.eq_ignore_ascii_case(&class)).and_then(|c| c.vars.get(n)).cloned() {
                    Some(v) => {
                        let fx = &mut self.sim.effects;
                        fx.outputs.push((2, Value::Str(v.name)));
                        fx.outputs.push((3, Value::Str(v.var_type)));
                        fx.outputs.push((4, Value::Str(v.default)));
                        fx.outputs.push((5, Value::Str(v.description)));
                        Value::Float(1.0)
                    }
                    None => Value::Float(0.0),
                }
            }
            "getprojectclasses" => {
                let names: Vec<String> = self.sim.class_meta.iter().map(|c| c.name.clone()).collect();
                let h = self.sim.effects.arrays.new_array();
                if let Some(arr) = self.sim.effects.arrays.get_mut(h) {
                    for n in names {
                        let mut el = crate::runtime::data::Element { type_name: "STRING".into(), ..Default::default() };
                        el.set("", Value::Str(n));
                        arr.push(el);
                    }
                }
                Value::Handle(h as f64)
            }
            // GetModelText(class, HStream) — текст в поток; SetModelText — из потока
            "getmodeltext" => {
                let class = arg(0);
                let h = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as u32;
                let text = self.sim.class_meta.iter().find(|c| c.name.eq_ignore_ascii_case(&class)).map(|c| c.text.clone());
                match (text, self.sim.effects.streams.items.get_mut(&h)) {
                    (Some(t), Some(st)) => {
                        let bytes = crate::formats::cp1251::encode(&t);
                        let start = st.pos.min(st.data.len());
                        st.data.truncate(start);
                        st.data.extend_from_slice(&bytes);
                        Value::Float(1.0)
                    }
                    _ => Value::Float(0.0),
                }
            }
            "setmodeltext" => {
                let class = arg(0);
                let h = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as u32;
                let Some(text) = self.sim.effects.streams.items.get(&h).map(|st| crate::formats::cp1251::decode(&st.data)) else { return Some(Value::Float(0.0)) };
                match crate::lang::parse(&text) {
                    Ok(m) => {
                        if let Some(c) = self.sim.class_meta.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&class)) {
                            c.text = text;
                        }
                        Value::Float(if self.sim.hot_swap_text(&class, m) { 1.0 } else { 0.0 })
                    }
                    Err(e) => {
                        self.sim.effects.log.push(format!("SetModelText {class}: {e}"));
                        Value::Float(0.0)
                    }
                }
            }
            "setvarstodefault" => {
                let target = self.resolve_arg(args.first())?;
                let class = self.sim.instances[target].class;
                let meta = self.sim.class_meta.get(class).cloned();
                if let Some(meta) = meta {
                    for v in &meta.vars {
                        if let Some(&c) = self.sim.instances[target].vars.get(&v.name.to_ascii_lowercase()) {
                            self.sim.cells[c] = Value::parse_default(&v.default, self.sim.types[c]);
                        }
                    }
                }
                Value::Float(1.0)
            }
            "getcalcorder" => {
                let target = self.resolve_arg(args.first())?;
                let h = args.get(1).map(|v| v.as_float()).unwrap_or(0.0) as u16;
                let kids: Vec<usize> = (0..self.sim.instances.len()).filter(|&i| self.sim.instances[i].parent == Some(target)).collect();
                Value::Float(kids.iter().position(|&i| self.sim.instances[i].handle == h).map(|p| p as f64 + 1.0).unwrap_or(0.0))
            }
            "setcalcorder" | "createlink" | "removelink" | "setlinkvars" | "createobject" | "deleteobject" | "createclass" | "deleteclass" | "openclassscheme" | "closeclassscheme" | "loadobjectstate" | "saveobjectstate" | "loadproject" | "unloadproject" | "setactiveproject" => {
                // перестройка схемы на ходу: принимаем без действия, о чём сообщаем один раз
                *self.sim.effects.missing.entry(name.to_string()).or_insert(0) += 1;
                Value::Float(0.0)
            }
            "getlink" => Value::Handle(0.0),
            "getuniqueclassname" => {
                let base = arg(0);
                let mut n = 1;
                while self.sim.class_meta.iter().any(|c| c.name.eq_ignore_ascii_case(&format!("{base}{n}"))) {
                    n += 1;
                }
                Value::Str(format!("{base}{n}"))
            }
            "isprojectexist" => Value::Float(1.0),
            _ => {
                let class = self.sim.classes.iter().position(|c| c.model.is_function && c.name.eq_ignore_ascii_case(name))?;
                return Some(self.sim.call_function(class, args));
            }
        })
    }
}

impl Simulation {
    /// Вызов имиджа-функции: параметры по порядку объявления, текст,
    /// значение `return`.
    fn call_function(&mut self, class: usize, args: &[Value]) -> Value {
        let index = match self.function_instances.get(&class) {
            Some(&i) => i,
            None => {
                let meta = self.class_meta[class].clone();
                let Ok(i) = self.add_instance_bare(&meta) else { return Value::Float(0.0) };
                self.function_instances.insert(class, i);
                i
            }
        };
        let params: Vec<String> = self.classes[class]
            .model
            .declarations
            .iter()
            .filter(|d| d.parameter)
            .flat_map(|d| d.names.clone())
            .collect();
        for (name, value) in params.iter().zip(args) {
            self.set_var(index, name, value.clone());
        }
        let body = std::sync::Arc::clone(&self.classes[class].body);
        let mut frame = Frame { sim: self, instance: index, immediate: true };
        let mut interp = Interpreter::new();
        match interp.run(&body, &mut frame) {
            Ok(interp::Flow::Return(Some(v))) => v,
            _ => Value::Float(0.0),
        }
    }

    /// Экземпляр без родителя и схемы — для имиджей-функций.
    fn add_instance_bare(&mut self, cls: &Class) -> Result<usize, BuildError> {
        let class = self.class_index(&cls.name).ok_or_else(|| BuildError::MissingRoot(cls.name.clone()))?;
        let index = self.instances.len();
        let mut instance = Instance {
            name: cls.name.clone(),
            class_name: cls.name.clone(),
            path: format!("<{}>", cls.name),
            parent: None,
            handle: 0,
            class,
            vars: HashMap::new(),
            order: Vec::new(),
        };
        for v in &cls.vars {
            let ty = ValueType::from_name(&v.var_type);
            let cell = self.cells.len();
            self.cells.push(Value::parse_default(&v.default, ty));
            self.old.push(ty.default_value());
            self.types.push(ty);
            if instance.vars.insert(v.name.to_ascii_lowercase(), cell).is_none() {
                instance.order.push(v.name.clone());
            }
        }
        for decl in self.classes[class].model.declarations.clone() {
            let ty = ValueType::from_name(&decl.var_type);
            for name in &decl.names {
                let key = name.to_ascii_lowercase();
                if instance.vars.contains_key(&key) {
                    continue;
                }
                let cell = self.cells.len();
                self.cells.push(ty.default_value());
                self.old.push(ty.default_value());
                self.types.push(ty);
                instance.vars.insert(key, cell);
                instance.order.push(name.clone());
            }
        }
        self.instances.push(instance);
        Ok(index)
    }
}

impl Frame<'_> {
    /// Экземпляр по пути: "" — сам, ".." — родитель, иначе имя класса или
    /// экземпляра на схеме.
    fn resolve(&self, path: &str) -> Option<usize> {
        self.sim.resolve_path(self.instance, path)
    }

    /// Первый аргумент — путь строкой или дескриптор экземпляра
    /// (то, что вернул `GetHObject`/`GetHObjectByName`).
    fn resolve_arg(&self, arg: Option<&Value>) -> Option<usize> {
        match arg {
            Some(Value::Handle(h)) | Some(Value::Float(h)) => {
                let i = *h as usize;
                (i < self.sim.instances.len()).then_some(i)
            }
            Some(v) => self.resolve(&v.as_string()),
            None => Some(self.instance),
        }
    }
}
