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
