//! Язык моделирования Stratum: лексер, парсер, дерево разбора.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{BinOp, Expr, Model, Stmt, UnOp};
pub use parser::{parse, ParseError};
