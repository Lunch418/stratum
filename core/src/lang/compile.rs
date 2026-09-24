//! Компилятор текста имиджа в байт-код Stratum 2000 (секция `0x0d` `.cls`).
//!
//! Зачем: оригинал исполняет не текст, а этот байт-код, поэтому проект,
//! экспортированный без него, в Stratum 2000 «молчит». И второе: байт-код
//! всех имиджей корпуса построен компилятором оригинала, так что совпадение
//! нашего вывода слово в слово проверяет разбор, приоритеты операций, фазы
//! значений (`x` — `push`, `~x` — `push_new`), выбор перегрузок и порядок
//! переменных — без запуска оригинала (`stratum bytecode`).
//!
//! Формат: поток `u16`. Операнды — индекс переменной (`push 1,i`),
//! абсолютный номер слова (`jmp 51,a`), константа (`push_cst 6` + f64 в
//! четырёх словах; `122 n` + строка CP1251 с нулём, по два байта в слове;
//! `5` + u32 в двух словах). В конце — `0`.

use super::ast::{BinOp, Expr, Model, Stmt, UnOp};
use super::opcodes::FUNCTIONS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Float,
    Str,
    Handle,
    Color,
    Int,
}

impl Ty {
    pub fn from_name(name: &str) -> Ty {
        match name.to_ascii_uppercase().as_str() {
            "STRING" => Ty::Str,
            "HANDLE" => Ty::Handle,
            "COLORREF" => Ty::Color,
            "INTEGER" => Ty::Int,
            _ => Ty::Float,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Ty::Float => "FLOAT",
            Ty::Str => "STRING",
            Ty::Handle => "HANDLE",
            Ty::Color => "COLORREF",
            Ty::Int => "INTEGER",
        }
    }

    fn code(c: char) -> Option<Ty> {
        Some(match c {
            'F' => Ty::Float,
            'S' => Ty::Str,
            'H' => Ty::Handle,
            'C' => Ty::Color,
            'I' => Ty::Int,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub line: u32,
    pub message: String,
}

/// Результат: байт-код и таблица переменных в порядке индексов.
#[derive(Debug, Clone, Default)]
pub struct Compiled {
    pub code: Vec<u16>,
    pub vars: Vec<(String, Ty)>,
}

type Function = (&'static str, &'static str, &'static str, char, u16);

/// Имидж-функция проекта или библиотеки: типы параметров и результата.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageFunction {
    pub params: Vec<Ty>,
    pub ret: Option<Ty>,
}

/// Окружение компиляции: константы, имиджи-функции и режим сворачивания.
pub struct Env<'a> {
    pub constant: &'a dyn Fn(&str) -> Option<f64>,
    pub function: &'a dyn Fn(&str) -> Option<ImageFunction>,
    pub fold_minus: bool,
}

impl Ty {
    /// Код типа в вызове имиджа-функции (VFunction).
    fn vcode(self) -> u16 {
        match self {
            Ty::Float => 0xFFFF,
            Ty::Str => 0xFFFE,
            Ty::Handle => 0xFFFD,
            Ty::Color => 0xFFFC,
            Ty::Int => 0xFFFB,
        }
    }
}

struct Compiler<'a> {
    code: Vec<u16>,
    vars: Vec<(String, Ty)>,
    line: u32,
    /// Имидж-функция: одна фаза значений — `~` не действует, присваивания
    /// идут в «старую» ячейку (`:=_old`, коды 10/157/12).
    function: bool,
    /// Внутри `~(…)`: переменные читаются в новой фазе (`~!x` — это `!~x`).
    new_phase: bool,
    /// Сворачивать `-число` в отрицательную константу. В корпусе есть имиджи,
    /// скомпилированные и так, и так (видимо, настройка оптимизации среды);
    /// на результат вычислений это не влияет.
    fold_minus: bool,
    /// Для `break`: адреса переходов, которые надо дописать концом цикла.
    breaks: Vec<Vec<usize>>,
    constant: &'a dyn Fn(&str) -> Option<f64>,
    function_of: &'a dyn Fn(&str) -> Option<ImageFunction>,
}

/// Компилирует текст. `known` — переменные имиджа в порядке `.cls` (их
/// индексы сохраняются), новые из текста добавляются в конец по мере
/// появления в коде. `constant` — значения именованных констант (`pi`…).
pub fn compile(model: &Model, known: &[(String, Ty)], env: &Env) -> Result<Compiled, CompileError> {
    let mut c = Compiler {
        code: Vec::new(),
        vars: known.to_vec(),
        line: 0,
        function: model.is_function,
        new_phase: false,
        fold_minus: env.fold_minus,
        breaks: Vec::new(),
        constant: env.constant,
        function_of: env.function,
    };
    // объявления в тексте регистрируют переменные раньше всех выражений
    for d in &model.declarations {
        for n in &d.names {
            c.var(n, Some(Ty::from_name(&d.var_type)));
        }
    }
    c.block(&model.body)?;
    c.code.push(0);
    Ok(Compiled { code: c.code, vars: c.vars })
}

impl Compiler<'_> {
    fn err<T>(&self, message: impl Into<String>) -> Result<T, CompileError> {
        Err(CompileError { line: self.line, message: message.into() })
    }

    /// Индекс переменной; новая регистрируется с типом `ty` (или FLOAT).
    fn var(&mut self, name: &str, ty: Option<Ty>) -> (u16, Ty) {
        if let Some(found) = self.known(name) {
            return found;
        }
        let t = ty.unwrap_or(Ty::Float);
        self.vars.push((name.to_string(), t));
        ((self.vars.len() - 1) as u16, t)
    }

    fn known(&self, name: &str) -> Option<(u16, Ty)> {
        self.vars.iter().position(|(n, _)| super::same_name(n, name)).map(|i| (i as u16, self.vars[i].1))
    }

    fn block(&mut self, body: &[Stmt]) -> Result<(), CompileError> {
        for s in body {
            self.stmt(s)?;
        }
        Ok(())
    }

    fn emit(&mut self, w: u16) {
        self.code.push(w);
    }

    /// Переход с адресом, который допишется позже; возвращает место адреса.
    fn jump(&mut self, op: u16) -> usize {
        self.emit(op);
        self.emit(0);
        self.code.len() - 1
    }

    fn patch(&mut self, at: usize) {
        self.code[at] = self.code.len() as u16;
    }

    fn cond_jump(&mut self, cond: &Expr, when_true: bool) -> Result<usize, CompileError> {
        let ty = self.expr(cond)?;
        let op = match (ty, when_true) {
            (Ty::Handle, false) => 111,
            (Ty::Handle, true) => 110,
            (_, false) => 53,
            (_, true) => 52,
        };
        Ok(self.jump(op))
    }

    fn stmt(&mut self, s: &Stmt) -> Result<(), CompileError> {
        match s {
            Stmt::At(line) => self.line = *line,
            Stmt::Declare(_) => {}
            // `x ::= …` выполняется в конце такта и в байт-код такта не входит
            Stmt::AssignDeferred { .. } => {}
            Stmt::Assign { target, value } => {
                let ty = self.expr(value)?;
                let (i, vt) = self.var(target, Some(ty));
                let op = match (vt, self.function) {
                    (Ty::Float, false) => 11,
                    (Ty::Str, false) => 123,
                    (_, false) => 13,
                    (Ty::Float, true) => 10,
                    (Ty::Str, true) => 157,
                    (_, true) => 12,
                };
                self.emit(op);
                self.emit(i);
            }
            Stmt::Expr(e) => {
                self.expr(e)?;
            }
            Stmt::If { condition, then_body, else_body } => {
                let skip = self.cond_jump(condition, false)?;
                self.block(then_body)?;
                if else_body.is_empty() {
                    self.patch(skip);
                } else {
                    let end = self.jump(51);
                    self.patch(skip);
                    self.block(else_body)?;
                    self.patch(end);
                }
            }
            Stmt::While { condition, body } => {
                let start = self.code.len() as u16;
                let exit = self.cond_jump(condition, false)?;
                self.breaks.push(vec![exit]);
                self.block(body)?;
                self.emit(51);
                self.emit(start);
                for at in self.breaks.pop().unwrap_or_default() {
                    self.patch(at);
                }
            }
            Stmt::DoUntil { body, condition } => {
                let start = self.code.len() as u16;
                self.breaks.push(Vec::new());
                self.block(body)?;
                let ty = self.expr(condition)?;
                self.emit(if ty == Ty::Handle { 111 } else { 53 });
                self.emit(start);
                for at in self.breaks.pop().unwrap_or_default() {
                    self.patch(at);
                }
            }
            Stmt::Switch { arms, default } => {
                let mut ends = Vec::new();
                for arm in arms {
                    let next = self.cond_jump(&arm.condition, false)?;
                    self.block(&arm.body)?;
                    ends.push(self.jump(51));
                    self.patch(next);
                }
                self.block(default)?;
                for at in ends {
                    self.patch(at);
                }
            }
            Stmt::Break => {
                let at = self.jump(51);
                match self.breaks.last_mut() {
                    Some(list) => list.push(at),
                    None => return self.err("break вне цикла"),
                }
            }
            // `return x` в имидже-функции только называет переменную-результат,
            // кода не порождает; досрочный выход — `exit()`
            Stmt::Return(_) => {}
            // уравнения идут в свою секцию (0x1e), не в байт-код такта
            Stmt::Equation { .. } | Stmt::Unknowns(_) => {}
        }
        Ok(())
    }

    fn push_var(&mut self, name: &str, new: bool) -> Ty {
        let new = (new || self.new_phase) && !self.function;
        let (i, ty) = self.var(name, None);
        let op = match (ty, new) {
            (Ty::Float, false) => 1,
            (Ty::Float, true) => 2,
            (Ty::Str, false) => 120,
            (Ty::Str, true) => 121,
            (_, false) => 3,
            (_, true) => 4,
        };
        self.emit(op);
        self.emit(i);
        ty
    }

    /// Вызов имиджа-функции: аргументы, `478`, имя, их число, типы
    /// параметров в обратном порядке (как в стеке) и тип результата.
    fn call_image(&mut self, name: &str, args: &[Expr], f: &ImageFunction) -> Result<Ty, CompileError> {
        if args.len() != f.params.len() {
            return self.err(format!("{name}: ожидается аргументов {}, передано {}", f.params.len(), args.len()));
        }
        for a in args {
            self.expr(a)?;
        }
        self.emit(478);
        self.push_string_body(name);
        self.emit(args.len() as u16);
        for t in f.params.iter().rev() {
            self.emit(t.vcode());
        }
        self.emit(f.ret.map(Ty::vcode).unwrap_or(0));
        Ok(f.ret.unwrap_or(Ty::Float))
    }

    /// Строка в коде: число слов, затем байты CP1251 с нулём, по два в слове.
    fn push_string_body(&mut self, s: &str) {
        let mut bytes = crate::formats::cp1251::encode(s);
        bytes.push(0);
        if bytes.len() % 2 == 1 {
            bytes.push(0);
        }
        self.emit((bytes.len() / 2) as u16);
        for pair in bytes.chunks(2) {
            self.emit(pair[0] as u16 | (pair[1] as u16) << 8);
        }
    }

    fn push_number(&mut self, v: f64) {
        self.emit(6);
        let b = v.to_bits();
        for k in 0..4 {
            self.emit((b >> (16 * k)) as u16);
        }
    }

    fn expr(&mut self, e: &Expr) -> Result<Ty, CompileError> {
        match e {
            Expr::Number(v) => {
                self.push_number(*v);
                Ok(Ty::Float)
            }
            Expr::Handle(v) => {
                let u = *v as u32;
                self.emit(5);
                self.emit(u as u16);
                self.emit((u >> 16) as u16);
                Ok(Ty::Handle)
            }
            Expr::Str(s) => {
                self.emit(122);
                self.push_string_body(s);
                Ok(Ty::Str)
            }
            Expr::Var(name) => {
                if self.known(name).is_none() {
                    if let Some(v) = (self.constant)(name) {
                        self.push_number(v);
                        return Ok(Ty::Float);
                    }
                }
                Ok(self.push_var(name, false))
            }
            Expr::Unary(UnOp::Old, inner) => match inner.as_ref() {
                Expr::Var(name) => Ok(self.push_var(name, true)),
                other => {
                    let saved = std::mem::replace(&mut self.new_phase, true);
                    let r = self.expr(other);
                    self.new_phase = saved;
                    r
                }
            },
            // минус перед числом компилятор сворачивает в отрицательную константу
            Expr::Unary(UnOp::Neg, inner) if self.fold_minus && matches!(inner.as_ref(), Expr::Number(_)) => {
                let Expr::Number(v) = inner.as_ref() else { unreachable!() };
                self.push_number(-v);
                Ok(Ty::Float)
            }
            Expr::Unary(op, inner) => {
                let ty = self.expr(inner)?;
                let code = match (op, ty) {
                    (UnOp::Neg, _) => 113,
                    (UnOp::Not, Ty::Handle) => 87,
                    _ => 45,
                };
                self.emit(code);
                Ok(Ty::Float)
            }
            Expr::Binary(op, a, b) => {
                let ta = self.expr(a)?;
                let tb = self.expr(b)?;
                let Some((code, ty)) = binary(*op, ta, tb) else {
                    return self.err(format!("операция {op:?} для {} и {}", ta.name(), tb.name()));
                };
                self.emit(code);
                Ok(ty)
            }
            Expr::Call(name, args) => self.call(name, args),
        }
    }

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<Ty, CompileError> {
        let lower = super::fold(name);
        let overloads: Vec<&Function> = FUNCTIONS.iter().filter(|f| f.0 == lower).collect();
        if overloads.is_empty() {
            if let Some(f) = (self.function_of)(name) {
                return self.call_image(name, args, &f);
            }
            return self.err(format!("неизвестная функция {name}"));
        }
        // типы аргументов узнаём, компилируя их во временный буфер
        let mut arg_types = Vec::new();
        let mut arg_code = Vec::new();
        for a in args {
            let saved = std::mem::take(&mut self.code);
            let t = self.expr(a)?;
            arg_code.push(std::mem::replace(&mut self.code, saved));
            arg_types.push(t);
        }
        let fits = |f: &&&Function| -> bool {
            let fixed: Vec<Ty> = f.1.chars().filter_map(Ty::code).collect();
            let opt: Vec<Ty> = f.2.chars().filter_map(Ty::code).collect();
            if arg_types.len() < fixed.len() || (opt.is_empty() && arg_types.len() != fixed.len()) {
                return false;
            }
            arg_types.iter().enumerate().all(|(i, t)| {
                let want = if i < fixed.len() { fixed[i] } else { opt[(i - fixed.len()) % opt.len()] };
                compatible(*t, want)
            })
        };
        let Some(f) = overloads.iter().find(fits) else {
            let got: Vec<&str> = arg_types.iter().map(|t| t.name()).collect();
            return self.err(format!("{name}: нет варианта для аргументов ({})", got.join(", ")));
        };
        // аргументы по ссылке (&FLOAT…) передаются адресом переменной: 477 i
        let byref: Vec<bool> = {
            let mut v = Vec::new();
            let mut amp = false;
            for c in f.1.chars().chain(f.2.chars()) {
                if c == '&' {
                    amp = true;
                } else {
                    v.push(amp);
                    amp = false;
                }
            }
            v
        };
        let fixed = f.1.chars().filter(|c| *c != '&').count();
        let optional = f.2.chars().filter(|c| *c != '&').count();
        for (k, code) in arg_code.into_iter().enumerate() {
            let by_ref = if k < fixed { byref.get(k) } else { byref.get(fixed + (k - fixed) % optional.max(1)) };
            let target = match &args[k] {
                Expr::Var(n) => Some(n),
                Expr::Unary(UnOp::Old, inner) => match inner.as_ref() {
                    Expr::Var(n) => Some(n),
                    _ => None,
                },
                _ => None,
            };
            match (by_ref, target) {
                (Some(true), Some(n)) => {
                    let (i, _) = self.var(n, None);
                    self.emit(477);
                    self.emit(i);
                }
                _ => self.code.extend(code),
            }
        }
        self.emit(f.4);
        // функция внешней библиотеки (.TDL): за DLLFunction идёт её имя
        if f.4 == 479 {
            self.push_string_body(name);
        }
        // у функций с повторяемыми аргументами за кодом идёт их число
        if optional > 0 {
            self.emit((args.len() - fixed) as u16);
        }
        Ok(Ty::code(f.3).unwrap_or(Ty::Float))
    }
}

/// HANDLE, INTEGER и COLORREF в машине оригинала — одно и то же целое.
fn compatible(have: Ty, want: Ty) -> bool {
    have == want || (have != Ty::Float && have != Ty::Str && want != Ty::Float && want != Ty::Str)
}

fn binary(op: BinOp, a: Ty, b: Ty) -> Option<(u16, Ty)> {
    use BinOp::*;
    use Ty::*;
    let int = |t: Ty| matches!(t, Handle | Color | Int);
    Some(match (op, a, b) {
        (Add, Float, Float) => (18, Float),
        (Add, Str, Str) => (124, Str),
        (Add, Str, Float) => (770, Str),
        (Add, Float, Str) => (771, Str),
        (Sub, Float, Float) => (19, Float),
        (Mul, Float, Float) => (21, Float),
        (Div, Float, Float) => (20, Float),
        (Mod, Float, Float) => (22, Float),
        (Pow, Float, Float) => (32, Float),
        (Eq, Float, Float) => (54, Float),
        (Ne, Float, Float) => (55, Float),
        (Gt, Float, Float) => (56, Float),
        (Ge, Float, Float) => (57, Float),
        (Lt, Float, Float) => (58, Float),
        (Le, Float, Float) => (59, Float),
        (Eq, Str, Str) => (144, Float),
        (Ne, Str, Str) => (145, Float),
        (Gt, Str, Str) => (146, Float),
        (Ge, Str, Str) => (147, Float),
        (Lt, Str, Str) => (148, Float),
        (Le, Str, Str) => (149, Float),
        (Eq, x, y) if int(x) && int(y) => (82, Float),
        (Ne, x, y) if int(x) && int(y) => (83, Float),
        (AndBit, Float, Float) => (105, Float),
        (OrBit, Float, Float) => (106, Float),
        (AndLogic, Float, Float) => (43, Float),
        (OrLogic, Float, Float) => (44, Float),
        (AndLogic, x, y) if int(x) && int(y) => (85, Handle),
        (OrLogic, x, y) if int(x) && int(y) => (86, Handle),
        (Shl, Float, Float) => (116, Float),
        (Shr, Float, Float) => (117, Float),
        _ => return None,
    })
}

/// Сигнатура имиджа-функции: параметры — переменные с флагом PARAMETER
/// (0x200) в порядке таблицы, результат — тип переменной из `return x`.
pub fn image_function(cls: &crate::formats::Class, model: &Model) -> ImageFunction {
    let params = cls.vars.iter().filter(|v| v.flags & 0x200 != 0).map(|v| Ty::from_name(&v.var_type)).collect();
    fn ret_of(body: &[Stmt]) -> Option<String> {
        body.iter().find_map(|s| match s {
            Stmt::Return(Some(Expr::Var(n))) => Some(n.clone()),
            Stmt::If { then_body, else_body, .. } => ret_of(then_body).or_else(|| ret_of(else_body)),
            _ => None,
        })
    }
    let ret = ret_of(&model.body).map(|n| {
        cls.vars
            .iter()
            .find(|v| super::same_name(&v.name, &n))
            .map(|v| Ty::from_name(&v.var_type))
            .or_else(|| model.declarations.iter().find(|d| d.names.iter().any(|x| super::same_name(x, &n))).map(|d| Ty::from_name(&d.var_type)))
            .unwrap_or(Ty::Float)
    });
    ImageFunction { params, ret }
}
