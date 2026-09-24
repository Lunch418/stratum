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
