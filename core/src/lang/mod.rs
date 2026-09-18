//! Язык моделирования Stratum: лексер, парсер, дерево разбора.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{BinOp, Expr, Model, Stmt, UnOp};
pub use parser::{parse, ParseError};

/// Зависимости переменных внутри текста: для каждого присваивания `x := e`
/// пары (переменная из `e`, `x`); условия управляющих конструкций тоже
/// считаются причинами присваиваний в их теле.
pub fn dependencies(model: &Model) -> Vec<(String, String)> {
    fn vars(e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Var(v) => out.push(v.to_ascii_lowercase()),
            Expr::Unary(_, a) => vars(a, out),
            Expr::Binary(_, a, b) => {
                vars(a, out);
                vars(b, out);
            }
            Expr::Call(_, args) => args.iter().for_each(|a| vars(a, out)),
            _ => {}
        }
    }
    fn walk(body: &[Stmt], context: &[String], out: &mut Vec<(String, String)>) {
        for st in body {
            match st {
                Stmt::Assign { target, value } | Stmt::AssignDeferred { target, value } => {
                    let mut from = context.to_vec();
                    vars(value, &mut from);
                    let target = target.to_ascii_lowercase();
                    for f in from {
                        if f != target {
                            out.push((f, target.clone()));
                        }
                    }
                }
                Stmt::If { condition, then_body, else_body } => {
                    let mut ctx = context.to_vec();
                    vars(condition, &mut ctx);
                    walk(then_body, &ctx, out);
                    walk(else_body, &ctx, out);
                }
                Stmt::While { condition, body } | Stmt::DoUntil { body, condition } => {
                    let mut ctx = context.to_vec();
                    vars(condition, &mut ctx);
                    walk(body, &ctx, out);
                }
                Stmt::Switch { arms, default } => {
                    for arm in arms {
                        let mut ctx = context.to_vec();
                        vars(&arm.condition, &mut ctx);
                        walk(&arm.body, &ctx, out);
                    }
                    walk(default, context, out);
                }
                Stmt::Equation { left, right } => {
                    let (mut l, mut r) = (Vec::new(), Vec::new());
                    vars(left, &mut l);
                    vars(right, &mut r);
                    for a in &l {
                        for b in &r {
                            out.push((a.clone(), b.clone()));
                            out.push((b.clone(), a.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(&model.body, &[], &mut out);
    out.sort();
    out.dedup();
    out
}
