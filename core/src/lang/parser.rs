//! Парсер текста имиджа.
//!
//! Приоритеты операторов взяты из раздела справки «Операторы» и сверены с
//! таблицей компилятора `template/COMPILER.TPL` (поле `imp`).

use super::ast::*;
use super::lexer::{tokenize, LexError, Tok, Token};
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "строка {}, позиция {}: {}", self.line, self.column, self.message)
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError { message: e.message, line: e.line, column: e.column }
    }
}

const TYPE_NAMES: &[&str] = &[
    "FLOAT", "STRING", "HANDLE", "COLORREF", "INTEGER", "WORD", "BYTE", "POINTER",
];

/// Слова, которые нельзя принимать за имя переменной в начале оператора.
const KEYWORDS: &[&str] = &[
    "if", "else", "endif", "while", "endwhile", "do", "until", "switch", "case",
    "default", "endswitch", "break", "function", "return", "local",
];

pub fn parse(src: &str) -> Result<Model, ParseError> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0, is_function: false };
    let body = p.block(&[])?;
    p.expect_eof()?;
    let mut model = Model { is_function: p.is_function, declarations: Vec::new(), body };
    collect_declarations(&model.body, &mut model.declarations);
    Ok(model)
}

fn collect_declarations(body: &[Stmt], out: &mut Vec<Declaration>) {
    for stmt in body {
        match stmt {
            Stmt::Declare(d) => out.push(d.clone()),
            Stmt::If { then_body, else_body, .. } => {
                collect_declarations(then_body, out);
                collect_declarations(else_body, out);
            }
            Stmt::While { body, .. } | Stmt::DoUntil { body, .. } => collect_declarations(body, out),
            Stmt::Switch { arms, default } => {
                for arm in arms {
                    collect_declarations(&arm.body, out);
                }
                collect_declarations(default, out);
            }
            _ => {}
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    is_function: bool,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
    }

    fn advance(&mut self) -> Tok {
        let t = self.tokens[self.pos].tok.clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        let t = &self.tokens[self.pos];
        Err(ParseError { message: message.into(), line: t.line, column: t.column })
    }

    fn skip_eol(&mut self) {
        while matches!(self.peek(), Tok::Eol) {
            self.advance();
        }
    }

    fn eat_op(&mut self, op: &str) -> bool {
        if matches!(self.peek(), Tok::Op(o) if *o == op) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_op(&mut self, op: &str) -> Result<(), ParseError> {
        if self.eat_op(op) {
            Ok(())
        } else {
            self.error(format!("ожидалось {:?}, встречено {}", op, self.peek()))
        }
    }

    /// Ключевое слово без учёта регистра; при совпадении токен съедается.
    fn eat_word(&mut self, word: &str) -> bool {
        if self.word_is(word) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn word_is(&self, word: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s.eq_ignore_ascii_case(word))
    }

    fn at_any_word(&self, words: &[&str]) -> bool {
        words.iter().any(|w| self.word_is(w))
    }

    fn expect_eof(&self) -> Result<(), ParseError> {
        if self.at_eof() {
            Ok(())
        } else {
            self.error(format!("лишний текст: {}", self.peek()))
        }
    }

    /// Разбирает операторы до одного из завершающих слов (не съедая его).
    fn block(&mut self, terminators: &[&str]) -> Result<Vec<Stmt>, ParseError> {
        let mut body = Vec::new();
        loop {
            self.skip_eol();
            if self.at_eof() || self.at_any_word(terminators) {
                return Ok(body);
            }
            let before = self.pos;
            let line = self.tokens[self.pos].line;
            let mark = body.len();
            self.statement(&mut body)?;
            if body.len() > mark {
                body.insert(mark, Stmt::At(line));
            }
            if self.pos == before {
                return self.error(format!("не удалось разобрать оператор: {}", self.peek()));
            }
        }
    }

    fn statement(&mut self, out: &mut Vec<Stmt>) -> Result<(), ParseError> {
        if self.word_is("if") {
            self.advance();
            let condition = self.parenthesized()?;
            let then_body = self.block(&["else", "endif"])?;
            let else_body = if self.eat_word("else") {
                self.block(&["endif"])?
            } else {
                Vec::new()
            };
            if !self.eat_word("endif") {
                return self.error("не найден endif");
            }
            out.push(Stmt::If { condition, then_body, else_body });
            return Ok(());
        }
        if self.word_is("while") {
            self.advance();
            let condition = self.parenthesized()?;
            let body = self.block(&["endwhile"])?;
            if !self.eat_word("endwhile") {
                return self.error("не найден endwhile");
            }
            out.push(Stmt::While { condition, body });
            return Ok(());
        }
        if self.word_is("do") {
            self.advance();
            let body = self.block(&["until"])?;
            if !self.eat_word("until") {
                return self.error("не найден until");
            }
            let condition = self.parenthesized()?;
            out.push(Stmt::DoUntil { body, condition });
            return Ok(());
        }
        if self.word_is("switch") {
            self.advance();
            let mut arms = Vec::new();
            let mut default = Vec::new();
            loop {
                self.skip_eol();
                if self.eat_word("case") {
                    let condition = self.parenthesized()?;
                    let body = self.block(&["case", "default", "endswitch"])?;
                    arms.push(CaseArm { condition, body });
                } else if self.eat_word("default") {
                    default = self.block(&["case", "endswitch"])?;
                } else if self.eat_word("endswitch") {
                    break;
                } else if self.at_eof() {
                    return self.error("не найден endswitch");
                } else {
                    return self.error(format!("внутри switch ожидались case/default/endswitch, встречено {}", self.peek()));
                }
            }
            out.push(Stmt::Switch { arms, default });
            return Ok(());
        }
        if self.word_is("break") {
            self.advance();
            out.push(Stmt::Break);
            return Ok(());
        }
        if self.word_is("return") {
            self.advance();
            let value = if matches!(self.peek(), Tok::Eol | Tok::Eof) {
                None
            } else {
                Some(self.expression()?)
            };
            out.push(Stmt::Return(value));
            return Ok(());
        }
        if self.word_is("function") {
            // `function` в начале текста: имидж вызывается как функция
            self.advance();
            self.is_function = true;
            return Ok(());
        }
        // список неизвестных для уравнений: ? x, y
        if self.eat_op("?") {
            let mut names = Vec::new();
            loop {
                match self.advance() {
                    Tok::Ident(name) => names.push(name),
                    other => return self.error(format!("ожидалось имя переменной, встречено {other}")),
                }
                if !self.eat_op(",") {
                    break;
                }
            }
            out.push(Stmt::Unknowns(names));
            return Ok(());
        }
        // объявление переменных
        if let Tok::Ident(word) = self.peek().clone() {
            if TYPE_NAMES.iter().any(|t| word.eq_ignore_ascii_case(t)) && self.looks_like_declaration() {
                self.advance();
                // модификаторы: local, parameter (аргумент функции), nosave
                let mut local = false;
                let mut parameter = false;
                loop {
                    if self.eat_word("local") {
                        local = true;
                    } else if self.eat_word("parameter") {
                        parameter = true;
                    } else if self.eat_word("nosave") {
                    } else {
                        break;
                    }
                }
                let mut names = Vec::new();
                loop {
                    match self.advance() {
                        Tok::Ident(name) => names.push(name),
                        other => return self.error(format!("ожидалось имя переменной, встречено {other}")),
                    }
                    if !self.eat_op(",") {
                        break;
                    }
                }
                out.push(Stmt::Declare(Declaration {
                    var_type: word.to_uppercase(),
                    local,
                    parameter,
                    names,
                }));
                return Ok(());
            }
            if KEYWORDS.iter().any(|k| word.eq_ignore_ascii_case(k)) {
                return self.error(format!("неожиданное ключевое слово {word}"));
            }
        }

        // присваивание, уравнение или вызов функции
        let first = self.expression()?;
        if self.eat_op(":=") || self.eat_op("::=") {
            let deferred = matches!(self.tokens[self.pos - 1].tok, Tok::Op("::="));
            // цепочка s := r := выражение
            let mut targets = vec![assign_target(&first).ok_or_else(|| ParseError {
                message: "слева от := должна быть переменная".into(),
                line: self.tokens[self.pos - 1].line,
                column: self.tokens[self.pos - 1].column,
            })?];
            let mut value = self.expression()?;
            while self.eat_op(":=") {
                let next = self.expression()?;
                let name = assign_target(&value).ok_or_else(|| ParseError {
                    message: "слева от := должна быть переменная".into(),
                    line: self.tokens[self.pos - 1].line,
                    column: self.tokens[self.pos - 1].column,
                })?;
                targets.push(name);
                value = next;
            }
            for target in targets {
                out.push(if deferred {
                    Stmt::AssignDeferred { target, value: value.clone() }
                } else {
                    Stmt::Assign { target, value: value.clone() }
                });
            }
            return Ok(());
        }
        if self.eat_op("=") {
            let right = self.expression()?;
            out.push(Stmt::Equation { left: first, right });
            return Ok(());
        }
        out.push(Stmt::Expr(first));
        Ok(())
    }

    /// Отличает объявление `FLOAT x` от использования функции приведения
    /// `FLOAT(h)` или переменной с именем типа.
    fn looks_like_declaration(&self) -> bool {
        match &self.tokens[self.pos + 1].tok {
            Tok::Ident(next) => {
                // FLOAT local x / FLOAT x
                !matches!(&self.tokens[self.pos + 2].tok, Tok::Op(":=") | Tok::Op("::="))
                    || ["local", "parameter", "nosave"].iter().any(|m| next.eq_ignore_ascii_case(m))
            }
            _ => false,
        }
    }

    fn parenthesized(&mut self) -> Result<Expr, ParseError> {
        self.expect_op("(")?;
        let e = self.expression()?;
        self.expect_op(")")?;
        Ok(e)
    }

    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.binary(0)
    }

    fn binary(&mut self, min_level: u8) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            let (op, level) = match self.peek() {
                Tok::Op(o) => match binop(o) {
                    Some(pair) => pair,
                    None => break,
                },
                _ => break,
            };
            if level < min_level {
                break;
            }
            self.advance();
            // все бинарные операторы левоассоциативны
            let right = self.binary(level + 1)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.eat_op("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        if self.eat_op("+") {
            return self.unary();
        }
        if self.eat_op("!") {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?)));
        }
        if self.eat_op("~") {
            return Ok(Expr::Unary(UnOp::Old, Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Tok::Number(n) => Ok(Expr::Number(n)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Handle(h) => Ok(Expr::Handle(h)),
            Tok::Ident(name) => {
                if self.eat_op("(") {
                    let mut args = Vec::new();
                    if !self.eat_op(")") {
                        loop {
                            self.skip_eol();
                            args.push(self.expression()?);
                            self.skip_eol();
                            if self.eat_op(",") {
                                continue;
                            }
                            self.expect_op(")")?;
                            break;
                        }
                    }
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Tok::Op("(") => {
                let e = self.expression()?;
                self.expect_op(")")?;
                Ok(e)
            }
            other => {
                self.pos = self.pos.saturating_sub(1);
                self.error(format!("ожидалось выражение, встречено {other}"))
            }
        }
    }
}

fn assign_target(e: &Expr) -> Option<String> {
    match e {
        Expr::Var(name) => Some(name.clone()),
        _ => None,
    }
}

/// Оператор и его уровень приоритета (больше — сильнее связывает).
fn binop(op: &str) -> Option<(BinOp, u8)> {
    Some(match op {
        "^" => (BinOp::Pow, 9),
        "*" => (BinOp::Mul, 8),
        "/" => (BinOp::Div, 8),
        "%" => (BinOp::Mod, 8),
        "+" => (BinOp::Add, 7),
        "-" => (BinOp::Sub, 7),
        ">" => (BinOp::Gt, 6),
        ">=" => (BinOp::Ge, 6),
        "<" => (BinOp::Lt, 6),
        "<=" => (BinOp::Le, 6),
        "==" => (BinOp::Eq, 5),
        "!=" => (BinOp::Ne, 5),
        "&" => (BinOp::AndBit, 4),
        "|" => (BinOp::OrBit, 3),
        "&&" => (BinOp::AndLogic, 2),
        "<<" => (BinOp::Shl, 2),
        "||" => (BinOp::OrLogic, 1),
        ">>" => (BinOp::Shr, 1),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Операторы без меток строк — тестам важна структура.
    fn body(src: &str) -> Vec<Stmt> {
        parse(src).unwrap().body.into_iter().filter(|s| !matches!(s, Stmt::At(_))).collect()
    }

    fn one(src: &str) -> Stmt {
        let b = body(src);
        assert_eq!(b.len(), 1, "{:?}", b);
        b.into_iter().next().unwrap()
    }

    #[test]
    fn assignment_and_precedence() {
        let stmt = one("x := 1 + 2 * 3");
        match stmt {
            Stmt::Assign { target, value } => {
                assert_eq!(target, "x");
                assert_eq!(
                    value,
                    Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::Number(1.0)),
                        Box::new(Expr::Binary(
                            BinOp::Mul,
                            Box::new(Expr::Number(2.0)),
                            Box::new(Expr::Number(3.0))
                        ))
                    )
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn tilde_binds_to_the_variable() {
        let stmt = one("y := ~x + 1");
        let Stmt::Assign { value, .. } = stmt else { panic!() };
        assert_eq!(
            value,
            Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Unary(UnOp::Old, Box::new(Expr::Var("x".into())))),
                Box::new(Expr::Number(1.0))
            )
        );
    }

    #[test]
    fn declarations() {
        let stmt = one("FLOAT local xPos,yPos,msg");
        assert_eq!(
            stmt,
            Stmt::Declare(Declaration {
                var_type: "FLOAT".into(),
                local: true,
                parameter: false,
                names: vec!["xPos".into(), "yPos".into(), "msg".into()],
            })
        );
    }

    #[test]
    fn float_call_is_not_a_declaration() {
        let stmt = one("x := FLOAT(~h)");
        let Stmt::Assign { value, .. } = stmt else { panic!("{stmt:?}") };
        assert!(matches!(value, Expr::Call(name, _) if name.eq_ignore_ascii_case("float")));
    }

    #[test]
    fn if_else_endif() {
        let m_body = body("if (a > 1)\n b := 2\nelse\n b := 3\nendif");
        let Stmt::If { then_body, else_body, .. } = &m_body[0] else { panic!() };
        let real = |b: &Vec<Stmt>| b.iter().filter(|s| !matches!(s, Stmt::At(_))).count();
        assert_eq!(real(then_body), 1);
        assert_eq!(real(else_body), 1);
    }

    #[test]
    fn switch_with_cases() {
        let m_body = body("switch\n case (~msg == 1); a := 1\n case (~msg == 2); a := 2\n default;\nendswitch");
        let Stmt::Switch { arms, default } = &m_body[0] else { panic!() };
        assert_eq!(arms.len(), 2);
        assert!(default.is_empty());
    }

    #[test]
    fn chained_assignment() {
        let m_body = body("s := r := a * 2");
        assert_eq!(m_body.len(), 2);
    }

    #[test]
    fn equation_and_unknowns() {
        let m_body = body("x + y = 10\n? x, y");
        assert!(matches!(m_body[0], Stmt::Equation { .. }));
        assert_eq!(m_body[1], Stmt::Unknowns(vec!["x".into(), "y".into()]));
    }

    #[test]
    fn call_without_assignment() {
        let m_body = body("exit()");
        assert_eq!(m_body[0], Stmt::Expr(Expr::Call("exit".into(), vec![])));
    }
}
