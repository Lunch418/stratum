//! Байт-код оригинала (секция `0x0d` `.cls`) -> дерево разбора.
//!
//! Оригинал исполняет сохранённый байт-код, а не текст имиджа. Обычно это
//! одно и то же, но у части имиджей корпуса код построен старым
//! компилятором и с текстом расходится: иначе сгруппированы операции,
//! константа `PI` в "Двигателе" сохранена как 2 и 3, функции названы
//! по-старому. Такие имиджи ядро исполняет по байт-коду: он разбирается
//! обратно в операторы, которые понимает интерпретатор.
//!
//! Код строит `compile.rs`, поэтому шаблоны управляющих конструкций
//! известны: `if` - условие, `jz` вперёд, [`jmp` через `else`]; `while` -
//! условие, `jz` на выход, тело, `jmp` назад на условие; `do ... until` -
//! тело, условие, `jz` назад; `break` - `jmp` на выход из цикла. Незнакомый
//! код операции - ошибка: тогда исполняется текст.

use super::ast::{BinOp, Expr, Stmt, UnOp};
use super::opcodes::FUNCTIONS;

#[derive(Debug, Clone, PartialEq)]
enum Ins {
    Push { var: u16, new: bool },
    Const(Expr),
    Assign(u16),
    ByRef(u16),
    Unary(UnOp),
    Binary(BinOp),
    Call { name: String, argc: usize },
    Jmp(usize),
    Jz(usize),
    Jnz(usize),
    End,
}

/// Разбирает байт-код; `vars` - имена переменных имиджа в порядке индексов.
pub fn decompile(code: &[u16], vars: &[String]) -> Result<Vec<Stmt>, String> {
    let (ins, at) = decode(code)?;
    let name = |i: u16| vars.get(i as usize).cloned().ok_or_else(|| format!("нет переменной с номером {i}"));
    // адрес слова -> номер инструкции
    let index_of = |addr: usize| at.iter().position(|a| *a == addr).ok_or_else(|| format!("переход в середину инструкции: {addr}"));
    let mut d = Decompiler { ins: &ins, at: &at, name: &name, index_of: &index_of, loops: Vec::new() };
    d.block(0, ins.len())
}

fn decode(code: &[u16]) -> Result<(Vec<Ins>, Vec<usize>), String> {
    let mut ins = Vec::new();
    let mut at = Vec::new();
    let mut pc = 0;
    let word = |i: usize| code.get(i).copied().ok_or_else(|| format!("код оборвался на слове {i}"));
    let string = |i: usize| -> Result<(String, usize), String> {
        let n = word(i)? as usize;
        let mut bytes = Vec::with_capacity(n * 2);
        for k in 0..n {
            let w = word(i + 1 + k)?;
            bytes.push(w as u8);
            bytes.push((w >> 8) as u8);
        }
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok((crate::formats::cp1251::decode(&bytes[..end]), i + 1 + n))
    };
    while pc < code.len() {
        at.push(pc);
        let op = code[pc];
        let (i, next) = match op {
            // 0 в конце - конец кода, внутри - exit()
            0 if pc + 1 >= code.len() => (Ins::End, pc + 1),
            0 => (Ins::Call { name: "exit".into(), argc: 0 }, pc + 1),
            1 | 3 | 120 => (Ins::Push { var: word(pc + 1)?, new: false }, pc + 2),
            2 | 4 | 121 => (Ins::Push { var: word(pc + 1)?, new: true }, pc + 2),
            10 | 11 | 12 | 13 | 123 | 157 => (Ins::Assign(word(pc + 1)?), pc + 2),
            477 => (Ins::ByRef(word(pc + 1)?), pc + 2),
            5 => {
                let v = word(pc + 1)? as u32 | (word(pc + 2)? as u32) << 16;
                (Ins::Const(Expr::Handle(v as f64)), pc + 3)
            }
            6 => {
                let mut b = 0u64;
                for k in 0..4 {
                    b |= (word(pc + 1 + k)? as u64) << (16 * k);
                }
                (Ins::Const(Expr::Number(f64::from_bits(b))), pc + 5)
            }
            122 => {
                let (s, next) = string(pc + 1)?;
                (Ins::Const(Expr::Str(s)), next)
            }
            113 => (Ins::Unary(UnOp::Neg), pc + 1),
            45 | 87 => (Ins::Unary(UnOp::Not), pc + 1),
            51 => (Ins::Jmp(word(pc + 1)? as usize), pc + 2),
            53 | 111 => (Ins::Jz(word(pc + 1)? as usize), pc + 2),
            52 | 110 => (Ins::Jnz(word(pc + 1)? as usize), pc + 2),
            // функция внешней библиотеки: за кодом - её имя
            479 => {
                let (name, next) = string(pc + 1)?;
                let lower = super::fold(&name);
                let f = FUNCTIONS.iter().find(|f| f.4 == 479 && f.0 == lower).ok_or_else(|| format!("нет функции библиотеки {name}"))?;
                let fixed = f.1.chars().filter(|c| *c != '&').count();
                let optional = f.2.chars().filter(|c| *c != '&').count();
                if optional > 0 {
                    (Ins::Call { name: f.0.to_string(), argc: fixed + word(next)? as usize }, next + 1)
                } else {
                    (Ins::Call { name: f.0.to_string(), argc: fixed }, next)
                }
            }
            // CreateControlObject2d: в таблице компилятора её нет, байт-код
            // библиотеки зовёт её с девятью аргументами
            390 => (Ins::Call { name: "createcontrolobject2d".into(), argc: 9 }, pc + 1),
            478 => {
                // имидж-функция: имя, число аргументов, их типы, тип результата
                let (name, next) = string(pc + 1)?;
                let argc = word(next)? as usize;
                (Ins::Call { name, argc }, next + 1 + argc + 1)
            }
            _ => {
                if let Some(b) = binary(op) {
                    (Ins::Binary(b), pc + 1)
                } else if let Some(f) = FUNCTIONS.iter().find(|f| f.4 == op && op != 479) {
                    let fixed = f.1.chars().filter(|c| *c != '&').count();
                    let optional = f.2.chars().filter(|c| *c != '&').count();
                    if optional > 0 {
                        (Ins::Call { name: f.0.to_string(), argc: fixed + word(pc + 1)? as usize }, pc + 2)
                    } else {
                        (Ins::Call { name: f.0.to_string(), argc: fixed }, pc + 1)
                    }
                } else {
                    return Err(format!("незнакомый код операции {op} на слове {pc}"));
                }
            }
        };
        ins.push(i);
        pc = next;
    }
    at.push(pc);
    Ok((ins, at))
}

fn binary(op: u16) -> Option<BinOp> {
    use BinOp::*;
    Some(match op {
        18 | 124 | 770 | 771 => Add,
        19 => Sub,
        21 => Mul,
        20 => Div,
        22 => Mod,
        32 => Pow,
        54 | 144 | 82 => Eq,
        55 | 145 | 83 => Ne,
        56 | 146 => Gt,
        57 | 147 => Ge,
        58 | 148 => Lt,
        59 | 149 => Le,
        105 => AndBit,
        106 => OrBit,
        43 | 85 => AndLogic,
        44 | 86 => OrLogic,
        116 => Shl,
        117 => Shr,
        _ => return None,
    })
}

struct Decompiler<'a> {
    ins: &'a [Ins],
    at: &'a [usize],
    name: &'a dyn Fn(u16) -> Result<String, String>,
    index_of: &'a dyn Fn(usize) -> Result<usize, String>,
    /// Выходы охватывающих циклов (номер инструкции) - для `break`.
    loops: Vec<usize>,
}

impl Decompiler<'_> {
    /// Операторы инструкций `[from, to)`.
    fn block(&mut self, from: usize, to: usize) -> Result<Vec<Stmt>, String> {
        let mut out: Vec<(usize, Stmt)> = Vec::new();
        let mut stack: Vec<Expr> = Vec::new();
        // где началось текущее выражение (для `while`)
        let mut expr_start = from;
        let mut i = from;
        while i < to {
            if stack.is_empty() {
                expr_start = i;
            }
            match &self.ins[i] {
                Ins::End => {
                    i += 1;
                    continue;
                }
                Ins::Push { var, new } => {
                    let v = Expr::Var((self.name)(*var)?);
                    stack.push(if *new { Expr::Unary(UnOp::Old, Box::new(v)) } else { v });
                }
                Ins::ByRef(var) => stack.push(Expr::Var((self.name)(*var)?)),
                Ins::Const(e) => stack.push(e.clone()),
                Ins::Unary(op) => {
                    let a = stack.pop().ok_or("пустой стек")?;
                    stack.push(Expr::Unary(*op, Box::new(a)));
                }
                Ins::Binary(op) => {
                    let b = stack.pop().ok_or("пустой стек")?;
                    let a = stack.pop().ok_or("пустой стек")?;
                    stack.push(Expr::Binary(*op, Box::new(a), Box::new(b)));
                }
                Ins::Call { name, argc } => {
                    if stack.len() < *argc {
                        return Err(format!("{name}: в стеке меньше {argc} аргументов"));
                    }
                    let args = stack.split_off(stack.len() - argc);
                    stack.push(Expr::Call(name.clone(), args));
                }
                Ins::Assign(var) => {
                    let value = stack.pop().ok_or("пустой стек")?;
                    self.flush(&mut stack, &mut out, expr_start);
                    out.push((expr_start, Stmt::Assign { target: (self.name)(*var)?, value }));
                }
                Ins::Jz(addr) | Ins::Jnz(addr) => {
                    let negate = matches!(self.ins[i], Ins::Jnz(_));
                    let cond = stack.pop().ok_or("условие без выражения")?;
                    self.flush(&mut stack, &mut out, expr_start);
                    let target = (self.index_of)(*addr)?;
                    if target <= i {
                        // do ... until: переход назад, пока условие ложно
                        let start = out.iter().position(|(at, _)| *at >= target).unwrap_or(out.len());
                        let body: Vec<Stmt> = out.drain(start..).map(|(_, s)| s).collect();
                        let condition = if negate { cond } else { Expr::Unary(UnOp::Not, Box::new(cond)) };
                        out.push((target, Stmt::DoUntil { body, condition }));
                        i += 1;
                        continue;
                    }
                    let condition = if negate { Expr::Unary(UnOp::Not, Box::new(cond)) } else { cond };
                    // перед целью - jmp: назад на условие - while, вперёд - else
                    let jmp_before = if target > i + 1 { match &self.ins[target - 1] { Ins::Jmp(a) => Some((self.index_of)(*a)?), _ => None } } else { None };
                    match jmp_before {
                        Some(back) if back == expr_start => {
                            self.loops.push(target);
                            let body = self.block(i + 1, target - 1)?;
                            self.loops.pop();
                            out.push((expr_start, Stmt::While { condition, body }));
                            i = target;
                        }
                        Some(end) if end >= target && end <= to && !self.loops.contains(&end) => {
                            let then_body = self.block(i + 1, target - 1)?;
                            let else_body = self.block(target, end)?;
                            out.push((expr_start, Stmt::If { condition, then_body, else_body }));
                            i = end;
                        }
                        _ => {
                            let then_body = self.block(i + 1, target.min(to))?;
                            out.push((expr_start, Stmt::If { condition, then_body, else_body: Vec::new() }));
                            i = target;
                        }
                    }
                    continue;
                }
                Ins::Jmp(addr) => {
                    self.flush(&mut stack, &mut out, expr_start);
                    let target = (self.index_of)(*addr)?;
                    if self.loops.last() == Some(&target) || self.loops.contains(&target) {
                        out.push((i, Stmt::Break));
                    } else if target > i {
                        // переход вперёд вне цикла (конец ветвей switch) - дальше код недостижим
                        i = to;
                        continue;
                    } else {
                        return Err(format!("переход назад без цикла на инструкции {i}"));
                    }
                }
            }
            i += 1;
        }
        self.flush(&mut stack, &mut out, expr_start);
        let _ = self.at;
        Ok(out.into_iter().map(|(_, s)| s).collect())
    }

    /// Выражения, оставшиеся в стеке, - это вызовы ради действия.
    fn flush(&self, stack: &mut Vec<Expr>, out: &mut Vec<(usize, Stmt)>, at: usize) {
        for e in stack.drain(..) {
            out.push((at, Stmt::Expr(e)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::compile::{compile, Env, Ty};

    fn round_trip(text: &str, vars: &[&str]) -> Vec<Stmt> {
        let model = crate::lang::parser::parse(text).unwrap();
        let known: Vec<(String, Ty)> = vars.iter().map(|v| (v.to_string(), Ty::Float)).collect();
        let env = Env { constant: &|n| (n.eq_ignore_ascii_case("pi")).then_some(std::f64::consts::PI), function: &|_| None, fold_minus: false, placeholders: false };
        let c = compile(&model, &known, &env).unwrap();
        let names: Vec<String> = c.vars.iter().map(|(n, _)| n.clone()).collect();
        decompile(&c.code, &names).unwrap()
    }

    #[test]
    fn statements_come_back() {
        let s = round_trip("x := ~x + 1\nif (x > 2)\n y := 1\nelse\n y := 2\nendif\nwhile (~i < 3)\n i := ~i + 1\n if (~i == 2)\n  break\n endif\nendwhile\nr := sin(~x)", &["x", "y", "i", "r"]);
        assert!(matches!(&s[0], Stmt::Assign { target, .. } if target == "x"));
        assert!(matches!(&s[1], Stmt::If { else_body, .. } if else_body.len() == 1));
        match &s[2] {
            Stmt::While { body, .. } => assert!(matches!(&body[1], Stmt::If { then_body, .. } if then_body == &vec![Stmt::Break])),
            other => panic!("{other:?}"),
        }
        assert!(matches!(&s[3], Stmt::Assign { value: Expr::Call(n, a), .. } if n == "sin" && a.len() == 1));
    }
}
