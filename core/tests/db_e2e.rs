//! Db* из исходного текста через компилятор (не только call() на уровне Rust).

use stratum_core::formats::{Class, LoadedProject, Project, Variable};
use stratum_core::sim::Simulation;

fn var(name: &str, ty: &str) -> Variable {
    Variable { name: name.into(), description: String::new(), default: String::new(), var_type: ty.into(), flags: 0 }
}

fn build(vars: &[(&str, &str)], text: &str) -> Result<Simulation, String> {
    build_in(std::path::PathBuf::new(), vars, text)
}

fn build_in(dir: std::path::PathBuf, vars: &[(&str, &str)], text: &str) -> Result<Simulation, String> {
    let main = Class {
        name: "Main".into(),
        version: 0x3003,
        vars: vars.iter().map(|(n, t)| var(n, t)).collect(),
        text: text.into(),
        ..Default::default()
    };
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes: vec![main], own_classes: 1, dir, ..Default::default() };
    let mut sim = Simulation::build(&project).map_err(|e| format!("{e:?}"))?;
    sim.step().map_err(|e| format!("{e:?}"))?;
    Ok(sim)
}

#[test]
fn create_table_compiles_with_declared_types() {
    let src = "h := DbCreateTable(#0, \"demo.dbf\", \"NAME:C:10,AGE:N:3:0\", \"1251\", 0)";
    let r = build(&[("h", "HANDLE")], src);
    assert!(r.is_ok(), "{:?}", r.err());
}

fn val(sim: &Simulation, n: &str) -> String {
    sim.value(0, n).unwrap().to_string()
}

#[test]
fn db_functions_from_source_text() {
    let dir = std::env::temp_dir().join(format!("stratum-dbe2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let vars = [("b", "HANDLE"), ("h", "HANDLE"), ("again", "HANDLE"), ("over", "HANDLE"), ("r1", "FLOAT"), ("r2", "FLOAT"), ("r3", "FLOAT"), ("r4", "FLOAT"), ("r5", "FLOAT"), ("cnt", "FLOAT"), ("first", "STRING"), ("hq", "HANDLE"), ("qn", "FLOAT")];
    let src = r#"
b := DbOpenBase("", "", "", "")
r1 := DbSetDir(~b, "($project)")
h := DbCreateTable(~b, "demo.dbf", "NAME:C:10,AGE:N:3:0", "1251", 0)
r1 := DbAppendRecord(~h)
r1 := DbSetField(~h, "NAME", "Zed")
r1 := DbSetField(~h, "AGE", 30)
r1 := DbAppendRecord(~h)
r1 := DbSetField(~h, "NAME", "Abe")
r1 := DbSetField(~h, "AGE", 20)
again := DbCreateTable(~b, "demo.dbf", "NAME:C:10", "", 0)
over := DbCreateTable(~b, "over.dbf", "NAME:C:10", "", 1)
over := DbCreateTable(~b, "over.dbf", "NAME:C:10", "", 1)
r2 := DbAddIndex(~h, "ia", 1, "", 0, 0, "", "", "AGE")
r3 := DbSwitchToIndex(~h, "ia", 1, "", 0)
r4 := DbGoTop(~h)
first := DbGetField(~h, "NAME")
r5 := DbSetToKey(~h, "30")
r1 := DbSortTable(~h, 0, "NAME")
cnt := DbCopyTo(~h, "copy.dbf", "dBASE")
qn := DbCopyTo(~h, "copy.db", "PARADOX")
hq := DbSQL(~b, "SELECT NAME FROM demo ORDER BY NAME", 2)
r1 := DbBrowse("W", ~h, "")
r1 := DbCloseTable(~h)
"#;
    let sim = build_in(dir.clone(), &vars, src).unwrap();
    assert_eq!(val(&sim, "again"), "#0", "existing table must not be overwritten without flag");
    assert_ne!(val(&sim, "over"), "#0", "flag 1 overwrites");
    assert_eq!(val(&sim, "r2"), "1");
    assert_eq!(val(&sim, "r3"), "1");
    assert_eq!(val(&sim, "first"), "Abe");
    assert_eq!(val(&sim, "r5"), "1");
    assert_eq!(val(&sim, "cnt"), "1");
    assert_eq!(val(&sim, "qn"), "0");
    assert_ne!(val(&sim, "hq"), "#0");
    assert!(dir.join("copy.dbf").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

fn check(vars: &[(&str, &str)], text: &str) -> Result<(), String> {
    let main = Class {
        name: "Main".into(),
        version: 0x3003,
        vars: vars.iter().map(|(n, t)| var(n, t)).collect(),
        text: text.into(),
        ..Default::default()
    };
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes: vec![main.clone()], own_classes: 1, ..Default::default() };
    let model = stratum_core::lang::parse(text).map_err(|e| format!("{e:?}"))?;
    stratum_core::formats::native::check_text(&project, &main, &model).map_err(|e| e.message)
}

#[test]
fn checker_accepts_declared_call_and_explains_mismatch() {
    let ok = "h := DbCreateTable(#0, \"demo.dbf\", \"NAME:C:10,AGE:N:3:0\", \"1251\", 0)";
    assert_eq!(check(&[("h", "HANDLE")], ok), Ok(()));
    // числовой литерал - не дескриптор: оригинал тоже требует HANDLE (#0)
    let e = check(&[("h", "HANDLE")], "h := DbCreateTable(0, \"a.dbf\", \"A:C:3\", \"\", 0)").unwrap_err();
    assert!(e.contains("(FLOAT, STRING, STRING, STRING, FLOAT)") && e.contains("ожидается (HANDLE"), "{e}");
    let e = check(&[("h", "HANDLE")], "h := DbCreateTable(0, \"a.dbf\", \"A:C:3\", 0)").unwrap_err();
    assert!(e.contains("(FLOAT, STRING, STRING, FLOAT)") && e.contains("HANDLE, STRING, STRING, STRING, FLOAT"), "{e}");
}
