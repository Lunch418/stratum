//! Исполнение текста имиджа в такте.
//!
//! У каждой переменной две фазы (справка, «Оператор Тильда»): значение на
//! начало шага и новое, присваиваемое в этом шаге. Обычное имя в выражении
//! читает фазу начала шага, `~имя` — новую; присваивание пишет в новую.
//! В конце такта новая фаза становится значением начала следующего.

use crate::lang::ast::*;
use crate::runtime::builtins::{self, Effects};
use crate::runtime::value::{Value, ValueType};

/// Фаза, из которой читаются переменные выражения.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Значение на начало шага — так читается обычное имя.
    Start,
    /// Новое значение этого шага — так читается `~имя`.
    New,
}

/// Доступ к переменным одного экземпляра: связанные переменные делят ячейку.
pub trait Vars {
    /// Новое значение этого шага; `None`, если переменной нет.
    fn get(&self, name: &str) -> Option<Value>;
    /// Значение на начало шага.
    fn get_old(&self, name: &str) -> Option<Value>;
    /// Записывает значение, создавая переменную при необходимости.
    fn set(&mut self, name: &str, value: Value);
    /// `::=` — запись откладывается до конца такта.
    fn set_deferred(&mut self, name: &str, value: Value) {
        self.set(name, value);
    }
    /// Значение именованной константы языка.
    fn constant(&self, name: &str) -> Option<Value>;
    /// Функции, зависящие от текущего имиджа (`GetClassName`, `GetVarF`…).
    fn call_special(&mut self, _name: &str, _args: &[Value]) -> Option<Value> {
        None
    }
    /// Побочные эффекты и графика модели.
    fn effects(&mut self) -> &mut Effects;
}

#[derive(Debug)]
pub enum Flow {
    Normal,
    Break,
    /// `exit()` — прекратить текст имиджа в этом такте.
    Exit,
    Return(Option<Value>),
}

pub struct Interpreter {
    /// Предохранитель от зацикливания: такт не должен подвешивать среду.
    pub max_steps: u64,
    steps: u64,
    /// Строка текущего оператора (по меткам `Stmt::At`).
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    /// Строка текста имиджа, на которой произошла ошибка (0 — неизвестно).
    pub line: u32,
    /// Экземпляр, в котором произошла ошибка (заполняет симуляция).
    pub instance: Option<usize>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        RuntimeError { message: message.into(), line: 0, instance: None }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { max_steps: 5_000_000, steps: 0, line: 0 }
    }

    pub fn run(&mut self, body: &[Stmt], vars: &mut dyn Vars) -> Result<Flow, RuntimeError> {
        self.steps = 0;
        self.block(body, vars)
    }

    fn block(&mut self, body: &[Stmt], vars: &mut dyn Vars) -> Result<Flow, RuntimeError> {
        for stmt in body {
            match self.statement(stmt, vars)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn tick_budget(&mut self) -> Result<(), RuntimeError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(RuntimeError {
                message: format!("такт не завершился за {} шагов: похоже на вечный цикл", self.max_steps),
                line: self.line,
                instance: None,
            });
        }
        Ok(())
    }

    fn statement(&mut self, stmt: &Stmt, vars: &mut dyn Vars) -> Result<Flow, RuntimeError> {
        self.tick_budget()?;
        match stmt {
            Stmt::At(line) => self.line = *line,
            Stmt::Declare(decl) => {
                let ty = ValueType::from_name(&decl.var_type);
                for name in &decl.names {
                    if vars.get(name).is_none() {
                        vars.set(name, ty.default_value());
                    }
                }
            }
            Stmt::Assign { target, value } => {
                let v = self.eval(value, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
                vars.set(target, v);
            }
            // ::= выполняется в конце такта
            Stmt::AssignDeferred { target, value } => {
                let v = self.eval(value, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
                vars.set_deferred(target, v);
            }
            Stmt::Expr(e) => {
                self.eval(e, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
            }
            Stmt::If { condition, then_body, else_body } => {
                let c = self.eval(condition, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
                return self.block(if c.is_true() { then_body } else { else_body }, vars);
            }
            Stmt::While { condition, body } => loop {
                self.tick_budget()?;
                let c = self.eval(condition, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
                if !c.is_true() {
                    break;
                }
                match self.block(body, vars)? {
                    Flow::Normal => {}
                    Flow::Break => break,
                    other => return Ok(other),
                }
            },
            Stmt::DoUntil { body, condition } => loop {
                self.tick_budget()?;
                match self.block(body, vars)? {
                    Flow::Normal => {}
                    Flow::Break => break,
                    other => return Ok(other),
                }
                let c = self.eval(condition, vars)?;
                if vars.effects().exit_requested {
                    return Ok(Flow::Exit);
                }
                // цикл идёт, пока условие истинно
                if !c.is_true() {
                    break;
                }
            },
            Stmt::Switch { arms, default } => {
                // проверяются все ветви подряд, пока не встретится break
                let mut matched = false;
                for arm in arms {
                    let c = self.eval(&arm.condition, vars)?;
                    if vars.effects().exit_requested {
                        return Ok(Flow::Exit);
                    }
                    if c.is_true() {
                        matched = true;
                        match self.block(&arm.body, vars)? {
                            Flow::Normal => {}
                            Flow::Break => return Ok(Flow::Normal),
                            other => return Ok(other),
                        }
                    }
                }
                if !matched {
                    match self.block(default, vars)? {
                        Flow::Normal => {}
                        Flow::Break => return Ok(Flow::Normal),
                        other => return Ok(other),
                    }
                }
            }
            Stmt::Break => return Ok(Flow::Break),
            Stmt::Return(value) => {
                let v = match value {
                    Some(e) => Some(self.eval(e, vars)?),
                    None => None,
                };
                return Ok(Flow::Return(v));
            }
            // уравнения решаются отдельным решателем, он появится на этапе 2
            Stmt::Equation { .. } | Stmt::Unknowns(_) => {}
        }
        Ok(Flow::Normal)
    }

    pub fn eval(&mut self, expr: &Expr, vars: &mut dyn Vars) -> Result<Value, RuntimeError> {
        self.eval_in(expr, vars, Phase::Start)
    }

    /// `~` переключает на новую фазу всё выражение под собой.
    fn eval_in(&mut self, expr: &Expr, vars: &mut dyn Vars, phase: Phase) -> Result<Value, RuntimeError> {
        self.tick_budget()?;
        Ok(match expr {
            Expr::Number(n) => Value::Float(*n),
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Handle(h) => Value::Handle(*h),
            Expr::Var(name) => {
                let found = match phase {
                    Phase::Start => vars.get_old(name),
                    Phase::New => vars.get(name),
                };
                match found {
                    Some(v) => v,
                    None => match vars.constant(name) {
                        Some(v) => v,
                        None => {
                            // неизвестные переменные создаются на лету, как в оригинале
                            vars.set(name, Value::Float(0.0));
                            Value::Float(0.0)
                        }
                    },
                }
            }
            Expr::Unary(UnOp::Old, inner) => self.eval_in(inner, vars, Phase::New)?,
            Expr::Unary(op, inner) => {
                let v = self.eval_in(inner, vars, phase)?;
                match op {
                    UnOp::Neg => Value::Float(-v.as_float()),
                    UnOp::Not => Value::Float(if v.is_true() { 0.0 } else { 1.0 }),
                    UnOp::Old => unreachable!(),
                }
            }
            Expr::Binary(op, l, r) => {
                let a = self.eval_in(l, vars, phase)?;
                let b = self.eval_in(r, vars, phase)?;
                binary(*op, a, b)
            }
            Expr::Call(name, args) => {
                let mut values = Vec::with_capacity(args.len());
                for a in args {
                    values.push(self.eval_in(a, vars, phase)?);
                }
                vars.effects().outputs.clear();
                let result = match vars.call_special(name, &values) {
                    Some(v) => v,
                    None => match builtins::call(name, &values, vars.effects()) {
                        Some(v) => v,
                        None => builtins::call_stub(name, vars.effects()),
                    },
                };
                // выходные аргументы пишутся в переменные, стоящие на их местах
                let outputs = std::mem::take(&mut vars.effects().outputs);
                for (index, value) in outputs {
                    if let Some(target) = args.get(index).and_then(var_name) {
                        vars.set(target, value);
                    }
                }
                result
            }
        })
    }
}

/// Имя переменной в выражении-аргументе: `x` или `~x`.
fn var_name(e: &Expr) -> Option<&str> {
    match e {
        Expr::Var(name) => Some(name),
        Expr::Unary(UnOp::Old, inner) => var_name(inner),
        _ => None,
    }
}

fn binary(op: BinOp, a: Value, b: Value) -> Value {
    use BinOp::*;
    let strings = matches!(a, Value::Str(_)) || matches!(b, Value::Str(_));
    match op {
        Add if strings => Value::Str(a.as_string() + &b.as_string()),
        Add => Value::Float(a.as_float() + b.as_float()),
        Sub => Value::Float(a.as_float() - b.as_float()),
        Mul => Value::Float(a.as_float() * b.as_float()),
        Div => {
            let d = b.as_float();
            Value::Float(if d == 0.0 { 0.0 } else { a.as_float() / d })
        }
        Mod => {
            let d = b.as_float();
            Value::Float(if d == 0.0 { 0.0 } else { a.as_float() % d })
        }
        Pow => Value::Float(a.as_float().powf(b.as_float())),
        Eq | Ne | Lt | Le | Gt | Ge => {
            let result = if strings {
                let (x, y) = (a.as_string(), b.as_string());
                match op {
                    Eq => x == y,
                    Ne => x != y,
                    Lt => x < y,
                    Le => x <= y,
                    Gt => x > y,
                    _ => x >= y,
                }
            } else {
                let (x, y) = (a.as_float(), b.as_float());
                match op {
                    Eq => x == y,
                    Ne => x != y,
                    Lt => x < y,
                    Le => x <= y,
                    Gt => x > y,
                    _ => x >= y,
                }
            };
            Value::Float(if result { 1.0 } else { 0.0 })
        }
        AndBit => Value::Float(((a.as_float() as i64) & (b.as_float() as i64)) as f64),
        OrBit => Value::Float(((a.as_float() as i64) | (b.as_float() as i64)) as f64),
        AndLogic => Value::Float(if a.is_true() && b.is_true() { 1.0 } else { 0.0 }),
        OrLogic => Value::Float(if a.is_true() || b.is_true() { 1.0 } else { 0.0 }),
        Shl => Value::Float((((a.as_float() as i64) << (b.as_float() as i64 & 63)) as f64).trunc()),
        Shr => Value::Float((((a.as_float() as i64) >> (b.as_float() as i64 & 63)) as f64).trunc()),
    }
}
