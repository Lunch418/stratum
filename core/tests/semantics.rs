//! Семантика языка на маленьких проектах в памяти: регистр имён, фазы
//! значений, встроенные функции. Эталоны здесь — документированное
//! поведение оригинала (справка SC3.HLP), а не вывод этого же ядра.

use stratum_core::formats::{Class, LoadedProject, Project, Variable};
use stratum_core::sim::Simulation;

fn var(name: &str, ty: &str) -> Variable {
    Variable { name: name.into(), description: String::new(), default: String::new(), var_type: ty.into(), flags: 0 }
}

/// Один имидж `Main` с переменными и текстом; прогон одного такта.
fn run(vars: &[(&str, &str)], text: &str) -> Simulation {
    let main = Class {
        name: "Main".into(),
        version: 0x3003,
        vars: vars.iter().map(|(n, t)| var(n, t)).collect(),
        text: text.into(),
        ..Default::default()
    };
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes: vec![main], own_classes: 1, ..Default::default() };
    let mut sim = Simulation::build(&project).unwrap();
    sim.step().unwrap();
    sim
}

fn value(sim: &Simulation, name: &str) -> String {
    sim.value(0, name).unwrap().to_string()
}

#[test]
fn cyrillic_names_ignore_case() {
    let sim = run(&[("Скорость", "FLOAT"), ("x", "FLOAT")], "Скорость := 5\nx := ~скорость + 1");
    assert_eq!(value(&sim, "x"), "6");
    assert_eq!(value(&sim, "СКОРОСТЬ"), "5");
}

#[test]
fn latin_names_ignore_case() {
    let sim = run(&[("Speed", "FLOAT"), ("x", "FLOAT")], "SPEED := 2\nx := ~speed * 3");
    assert_eq!(value(&sim, "x"), "6");
}

/// Подставные значения при математических ошибках — эталоны получены от
/// Stratum 2000 в режиме «Не замечать» (tools/verify/matherr.txt, power.txt).
#[test]
fn math_errors_give_the_originals_values() {
    let cases = [
        ("1/0", "0"),
        ("5%0", "0"),
        ("sqrt(-1)", "0"),
        ("ln(0)", "-1.7e+308"),
        ("lg(0)", "-7.38301e+307"),
        ("exp(1000)", "1.7e+308"),
        ("(-4)^0.5", "2"),
        ("(-2)^3", "-8"),
        ("0^0", "0"),
        ("1e308 * 10", "1e+308"),
        ("10 * 5e307", "10"),
    ];
    for (expr, want) in cases {
        let sim = run(&[("x", "STRING")], &format!("x := String({expr})"));
        assert_eq!(value(&sim, "x"), want, "{expr}");
        assert!(!sim.effects.math_errors.is_empty() || expr == "(-2)^3", "{expr}: ошибка должна регистрироваться");
    }
}

fn class(name: &str, vars: Vec<Variable>, text: &str, children: &[(&str, &str, u16)], links: &[(u16, u16, &str, &str)]) -> Class {
    use stratum_core::formats::cls::{Child, Link};
    Class {
        name: name.into(),
        version: 0x3003,
        vars,
        text: text.into(),
        children: children.iter().map(|&(c, n, h)| Child { class_name: c.into(), handle: h, name: n.into(), x: 0.0, y: 0.0, flags: 0 }).collect(),
        links: links.iter().map(|&(s, t, a, b)| Link { source: s, target: t, handle: 100 + s, flags: 0, vars: vec![(a.into(), b.into())], ..Default::default() }).collect(),
        ..Default::default()
    }
}

fn build(classes: Vec<Class>) -> Simulation {
    let n = classes.len();
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes, own_classes: n, ..Default::default() };
    Simulation::build(&project).unwrap()
}

/// Оригинал считает сначала детей по порядку схемы, потом сам имидж:
/// журнал «LKLKM» (сверено в Wine).
#[test]
fn children_are_calculated_before_their_parent() {
    let leaf = class("Leaf", vec![], "SetVar(\"..\\..\", \"log\", GetVarS(\"..\\..\", \"log\") + \"L\")", &[], &[]);
    let kid = class("Kid", vec![], "SetVar(\"..\", \"log\", GetVarS(\"..\", \"log\") + \"K\")", &[("Leaf", "L1", 1)], &[]);
    let main = class("Main", vec![var("log", "STRING")], "log := ~log + \"M\"", &[("Kid", "K1", 1), ("Kid", "K2", 2)], &[]);
    let mut sim = build(vec![main, kid, leaf]);
    sim.step().unwrap();
    assert_eq!(value(&sim, "log"), "LKLKM");
}

/// Связанные переменные с разными значениями по умолчанию: побеждает
/// последнее непустое в обходе «дети, потом родитель»; направление связи
/// не важно (сверено в Wine, tools/verify_links.py).
#[test]
fn linked_defaults_follow_the_originals_order() {
    let v = |name: &str, d: &str| class(name, vec![Variable { default: d.into(), ..var("x", "FLOAT") }], "", &[], &[]);
    let main = class(
        "Main",
        vec![Variable { default: "7".into(), ..var("p", "FLOAT") }],
        "",
        &[("V1", "A", 1), ("V2", "B", 2), ("V2", "H", 3), ("V1", "I", 4), ("V1", "N", 5), ("V0", "O", 6), ("V2", "P", 7)],
        &[(1, 2, "x", "x"), (3, 4, "x", "x"), (5, 6, "x", "x"), (7, 0, "x", "p")],
    );
    let sim = build(vec![main, v("V0", ""), v("V1", "1"), v("V2", "2")]);
    let x = |child: usize| sim.value(child, "x").unwrap().to_string();
    // экземпляры: 0 — Main, дальше дети по порядку схемы
    assert_eq!((x(1), x(2)), ("2".into(), "2".into()));
    assert_eq!((x(3), x(4)), ("1".into(), "1".into()));
    assert_eq!((x(5), x(6)), ("1".into(), "1".into()));
    assert_eq!((x(7), value(&sim, "p")), ("7".into(), "7".into()));
}
