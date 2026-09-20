//! Решатель уравнений на проекте, собранном в памяти: кольцо из источника
//! и двух резисторов, связанных как в библиотеке CHAINS.

use stratum_core::formats::{Child, Class, Link, LoadedProject, Project, Variable};
use stratum_core::sim::Simulation;

fn var(name: &str, ty: &str, default: &str) -> Variable {
    Variable { name: name.into(), description: String::new(), default: default.into(), var_type: ty.into(), flags: 0 }
}

#[test]
fn resistor_ring_is_solved_each_tick() {
    // источник: f1 - f2 = e; резистор: f1 - f2 = i*r; ток общий по кольцу
    let source = Class {
        name: "Source".into(),
        version: 0x3003,
        vars: vec![var("e", "FLOAT", "12"), var("f1", "FLOAT", "0"), var("f2", "FLOAT", "0"), var("i", "FLOAT", "0")],
        text: "f1 - f2 = ~e\n?f1, f2, i".into(),
        ..Default::default()
    };
    let resistor = Class {
        name: "Resistor".into(),
        version: 0x3003,
        vars: vec![var("r", "FLOAT", "1"), var("f1", "FLOAT", "0"), var("f2", "FLOAT", "0"), var("i", "FLOAT", "0"), var("u", "FLOAT", "0")],
        text: "f1 - f2 = i*~r\n?f1, f2, i\nu := ~f1 - ~f2".into(),
        ..Default::default()
    };
    let ground = Class {
        name: "Ground".into(),
        version: 0x3003,
        vars: vec![var("f", "FLOAT", "0")],
        text: "f = 0\n?f".into(),
        ..Default::default()
    };
    let link = |handle, source, target, a: &str, b: &str| Link { handle, source, target, flags: 0, vars: vec![(a.into(), b.into())], style: Default::default() };
    let root = Class {
        name: "Root".into(),
        version: 0x3003,
        children: vec![
            Child { class_name: "Source".into(), handle: 1, name: "src".into(), x: 0.0, y: 0.0, flags: 0 },
            Child { class_name: "Resistor".into(), handle: 2, name: "r1".into(), x: 0.0, y: 0.0, flags: 0 },
            Child { class_name: "Resistor".into(), handle: 3, name: "r2".into(), x: 0.0, y: 0.0, flags: 0 },
            Child { class_name: "Ground".into(), handle: 4, name: "gnd".into(), x: 0.0, y: 0.0, flags: 0 },
        ],
        links: vec![
            // кольцо: src.f1 → r1.f1, r1.f2 → r2.f1, r2.f2 → src.f2 → gnd; ток общий
            link(1, 1, 2, "f1", "f1"),
            link(2, 2, 3, "f2", "f1"),
            link(3, 3, 1, "f2", "f2"),
            link(4, 1, 4, "f2", "f"),
            link(5, 1, 2, "i", "i"),
            link(6, 2, 3, "i", "i"),
        ],
        ..Default::default()
    };
    let project = LoadedProject {
        project: Project { root: "Root".into(), ..Default::default() },
        classes: vec![root, source, resistor, ground],
        own_classes: 4,
        ..Default::default()
    };
    let mut sim = Simulation::build(&project).unwrap();
    sim.step().unwrap();
    let r1 = sim.find("r1").unwrap();
    let i = sim.value(r1, "i").unwrap().as_float();
    let u = sim.value(r1, "u").unwrap().as_float();
    // 12 В на два резистора по 1 Ом: ток 6 А, на каждом по 6 В
    assert!((i - 6.0).abs() < 1e-6, "ток {i}");
    assert!((u - 6.0).abs() < 1e-6, "падение {u}");
}
