//! Ogre3D из исходного текста модели: компилятор -> симулятор -> кадр.

use stratum_core::formats::{Class, LoadedProject, Project, Variable};
use stratum_core::sim::Simulation;

fn var(name: &str, ty: &str) -> Variable {
    Variable { name: name.into(), description: String::new(), default: String::new(), var_type: ty.into(), flags: 0 }
}

fn build(vars: &[(&str, &str)], text: &str) -> Simulation {
    let main = Class { name: "Main".into(), version: 0x3003, vars: vars.iter().map(|(n, t)| var(n, t)).collect(), text: text.into(), ..Default::default() };
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes: vec![main], own_classes: 1, dir: std::path::PathBuf::new(), ..Default::default() };
    Simulation::build(&project).expect("build")
}

pub const CUBE_SRC: &str = r#"
if(~!init)
  init := 1
  Root_Create("", "", "")
  Root_Initialise()
  sc := Scene_Create("OctreeSceneManager")
  Scene_SetAmbientLight(~sc, 0.25, 0.25, 0.3)
  mo := ManualObject_Create(~sc, "cube")
  ManualObject_Begin(~mo, "", 4)
  ManualObject_Position(~mo, -1, -1, 1)
  ManualObject_Position(~mo, 1, -1, 1)
  ManualObject_Position(~mo, 1, 1, 1)
  ManualObject_Position(~mo, -1, 1, 1)
  ManualObject_Position(~mo, -1, -1, -1)
  ManualObject_Position(~mo, 1, -1, -1)
  ManualObject_Position(~mo, 1, 1, -1)
  ManualObject_Position(~mo, -1, 1, -1)
  FACES
  ManualObject_End(~mo)
  n := SceneNode_Create(~sc, "cubeNode")
  Node_AddChild(Scene_GetRootSceneNode(~sc), ~n)
  SceneNode_AttachObject(~n, ~mo)
  cam := Camera_Create(~sc, "cam")
  Camera_SetNearClipDistance(~cam, 0.5)
  cn := SceneNode_Create(~sc, "camNode")
  Node_AddChild(Scene_GetRootSceneNode(~sc), ~cn)
  Node_SetPosition(~cn, 0, 0, 8)
  SceneNode_AttachObject(~cn, ~cam)
  lt := Light_Create(~sc, "sun")
  Light_SetType(~lt, 1)
  ln := SceneNode_Create(~sc, "lightNode")
  Node_AddChild(Scene_GetRootSceneNode(~sc), ~ln)
  Node_SetRotationEulerXYZ(~ln, 0.5, -0.6, 0)
  SceneNode_AttachObject(~ln, ~lt)
  win := RenderWindow_Create2(#0, "Ogre", 800, 600, 0, 0, 0, 0)
  vp := Viewport_Create(~win, ~cam, 0)
  Viewport_SetBackgroundColour(~vp, 0.1, 0.1, 0.4)
endif
ang := ~ang + 0.05
Node_SetRotationEulerXYZ(~n, ~ang, ~ang * 0.6, 0)
Root_RenderOneFrame()
"#;

pub fn cube_text() -> String {
    // грани против часовой стрелки снаружи
    let faces = [0, 1, 2, 0, 2, 3, 5, 4, 7, 5, 7, 6, 1, 5, 6, 1, 6, 2, 4, 0, 3, 4, 3, 7, 3, 2, 6, 3, 6, 7, 4, 5, 1, 4, 1, 0];
    let idx: Vec<String> = faces.iter().map(|i| format!("  ManualObject_Index(~mo, {i})")).collect();
    CUBE_SRC.replace("  FACES", &idx.join("\n"))
}

pub fn cube_vars() -> Vec<(&'static str, &'static str)> {
    vec![("init", "FLOAT"), ("ang", "FLOAT"), ("sc", "HANDLE"), ("mo", "HANDLE"), ("n", "HANDLE"), ("cam", "HANDLE"), ("cn", "HANDLE"), ("lt", "HANDLE"), ("ln", "HANDLE"), ("win", "HANDLE"), ("vp", "HANDLE")]
}

#[test]
fn cube_scene_renders_from_source() {
    let mut sim = build(&cube_vars(), &cube_text());
    sim.step().unwrap();
    sim.step().unwrap();
    assert!(sim.effects.missing.is_empty(), "{:?}", sim.effects.missing);
    assert_eq!(sim.effects.ogre.errors, 0, "{:?}", sim.effects.log);
    let fr = sim.effects.ogre.render(None).expect("frame");
    assert_eq!((fr.width, fr.height), (800, 600));
    let bg = [25u8, 25, 102];
    let bgcount = fr.rgb.chunks_exact(3).filter(|p| **p == bg).count();
    let total = fr.width * fr.height;
    assert!(bgcount < total, "cube not drawn");
    assert!(bgcount > total / 3, "cube covers too much: bg {bgcount}/{total}");
    // центр кадра - куб
    let mid = (fr.height / 2 * fr.width + fr.width / 2) * 3;
    assert_ne!(&fr.rgb[mid..mid + 3], &bg);
    // кадр для IDE
    let (w, h, svg) = sim.effects.ogre.frame_svg().expect("svg");
    assert_eq!((w, h), (800.0, 600.0));
    assert!(svg.contains("data:image/bmp;base64,"));
    // BMP читается обратно
    let bmp = fr.to_bmp();
    let (bw, bh, px) = stratum_core::gfx::decode_bmp(&bmp).expect("bmp");
    assert_eq!((bw, bh), (800, 600));
    assert_eq!(px, fr.rgb);
}

#[test]
fn ray_pick_from_source() {
    let mut src = cube_text();
    src.push_str("Collision_RayCast(~sc, 0, 0, 20, 0, 0, -1)\nhits := Collision_GetResultCount()\nd := Collision_GetDistance(0)\n");
    let mut vars = cube_vars();
    vars.push(("hits", "FLOAT"));
    vars.push(("d", "FLOAT"));
    let mut sim = build(&vars, &src);
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.value(0, "hits").unwrap().as_float(), 1.0);
    let d = sim.value(0, "d").unwrap().as_float();
    assert!(d > 4.0 && d < 20.0, "{d}");
}

/// Выгрузка кадров для просмотра глазами: OGRE_DUMP=папка cargo test --test ogre_e2e dump
#[test]
fn dump_frames_when_asked() {
    let Ok(dir) = std::env::var("OGRE_DUMP") else { return };
    let mut sim = build(&cube_vars(), &cube_text());
    for _ in 0..14 {
        sim.step().unwrap();
    }
    std::fs::write(std::path::Path::new(&dir).join("cube.bmp"), sim.effects.ogre.render(None).unwrap().to_bmp()).unwrap();
}

/// Пишет демонстрационный проект в родном формате: OGRE_DEMO_OUT=папка.
#[test]
fn write_demo_project_when_asked() {
    let Ok(dir) = std::env::var("OGRE_DEMO_OUT") else { return };
    let vars: Vec<Variable> = cube_vars().iter().map(|(n, t)| var(n, t)).collect();
    let main = Class { name: "Main".into(), version: 0x3003, vars, text: cube_text(), ..Default::default() };
    let project = LoadedProject { project: Project { root: "Main".into(), ..Default::default() }, classes: vec![main], own_classes: 1, ..Default::default() };
    std::fs::create_dir_all(&dir).unwrap();
    stratum_core::formats::native::save(std::path::Path::new(&dir), &project).unwrap();
}
