//! Дерево разбора текста имиджа.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndBit,
    OrBit,
    AndLogic,
    OrLogic,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    /// `~x` — значение на начало такта.
    Old,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Str(String),
    Handle(f64),
    /// Переменная или константа; имя хранится как написано.
    Var(String),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub var_type: String,
    pub local: bool,
    /// `parameter` — аргумент имиджа-функции.
    pub parameter: bool,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Declare(Declaration),
    /// `x := выражение`; цепочка `s := r := …` разворачивается в несколько
    /// присваиваний с одним и тем же правым выражением.
    Assign { target: String, value: Expr },
    /// `x ::= выражение` — присваивание, выполняемое в конце такта.
    AssignDeferred { target: String, value: Expr },
    Expr(Expr),
    If { condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
    While { condition: Expr, body: Vec<Stmt> },
    DoUntil { body: Vec<Stmt>, condition: Expr },
    Switch { arms: Vec<CaseArm>, default: Vec<Stmt> },
    Break,
    Return(Option<Expr>),
    /// Уравнение `выражение = выражение`, решается средой, а не по порядку.
    Equation { left: Expr, right: Expr },
    /// `? x, y` — список неизвестных для уравнений выше.
    Unknowns(Vec<String>),
}

/// Разобранный текст имиджа.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Model {
    pub body: Vec<Stmt>,
    /// Объявления, собранные по всему тексту — удобно для таблицы переменных.
    pub declarations: Vec<Declaration>,
    /// true, если текст содержит `function` — имидж является функцией.
    pub is_function: bool,
}
