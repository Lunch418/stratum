use super::*;

fn c(fx: &mut Effects, name: &str, args: &[Value]) -> Value {
    super::super::extra::call(name, args, fx).unwrap_or_else(|| panic!("no {name}"))
}
fn hv(v: &Value) -> Value {
    Value::Handle(v.as_float())
}
fn st(s: &str) -> Value {
    Value::Str(s.into())
}
fn f(x: f64) -> Value {
    Value::Float(x)
}

fn boot(fx: &mut Effects) -> Value {
    c(fx, "Root_Create", &[st(""), st("none.cfg"), st("")]);
    c(fx, "Root_Initialise", &[]);
    c(fx, "Scene_Create", &[st("OctreeSceneManager")])
}

#[test]
fn every_function_is_implemented() {
    assert_eq!(function_names().len(), 299);
    let mut fx = Effects::default();
    for n in function_names() {
        let sig = sigs().get(&n).unwrap();
        let args: Vec<Value> = sig.params.iter().map(|p| if p.0 == 'S' { st("x") } else { f(0.0) }).collect();
        // не должно паниковать на нулевых дескрипторах
        assert!(super::super::extra::call(&n, &args, &mut fx).is_some(), "{n}");
    }
    assert!(fx.missing.is_empty());
}

#[test]
fn handles_lifetime_and_errors() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    assert!(sc.as_float() >= 1.0);
    assert_eq!(c(&mut fx, "Root_IsInitialised", &[]).as_float(), 1.0);
    let cam = c(&mut fx, "Camera_Create", &[hv(&sc), st("Main")]);
    assert!(cam.as_float() >= 1.0);
    // дубликат имени и стадия
    assert_eq!(c(&mut fx, "Camera_Create", &[hv(&sc), st("main")]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "Scene_GetCamera", &[hv(&sc), st("MAIN")]).as_float(), cam.as_float());
    assert!((c(&mut fx, "Camera_GetFOV", &[hv(&cam)]).as_float() - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
    c(&mut fx, "Camera_SetFOV", &[hv(&cam), f(1.0)]);
    assert_eq!(c(&mut fx, "Camera_GetFOV", &[hv(&cam)]).as_float(), 1.0);
    // NaN и отрицательное значение отклоняются
    c(&mut fx, "Camera_SetFOV", &[hv(&cam), f(f64::NAN)]);
    c(&mut fx, "Camera_SetFOV", &[hv(&cam), f(-2.0)]);
    assert_eq!(c(&mut fx, "Camera_GetFOV", &[hv(&cam)]).as_float(), 1.0);
    // чужой тип дескриптора
    c(&mut fx, "Light_SetPowerScale", &[hv(&cam), f(5.0)]);
    c(&mut fx, "Camera_Destroy", &[hv(&cam)]);
    assert_eq!(c(&mut fx, "Scene_GetCamera", &[hv(&sc), st("Main")]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "Camera_GetFOV", &[hv(&cam)]).as_float(), 0.0);
    assert!(fx.ogre.errors >= 3);
}

#[test]
fn scene_requires_root_and_valid_type() {
    let mut fx = Effects::default();
    assert_eq!(c(&mut fx, "Scene_Create", &[st("OctreeSceneManager")]).as_float(), 0.0);
    c(&mut fx, "Root_Create", &[st(""), st(""), st("")]);
    assert_eq!(c(&mut fx, "Scene_Create", &[st("Bogus")]).as_float(), 0.0);
    assert!(c(&mut fx, "Scene_Create", &[st("BSPSceneManager")]).as_float() >= 1.0);
}

#[test]
fn node_tree_and_transforms() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    let root = c(&mut fx, "Scene_GetRootSceneNode", &[hv(&sc)]);
    let a = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("")]);
    let b = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("b")]);
    c(&mut fx, "Node_AddChild", &[hv(&root), hv(&a)]);
    c(&mut fx, "Node_AddChild", &[hv(&a), hv(&b)]);
    // цикл отклоняется
    c(&mut fx, "Node_AddChild", &[hv(&b), hv(&a)]);
    assert_eq!(c(&mut fx, "Node_GetParent", &[hv(&a)]).as_float(), root.as_float());
    c(&mut fx, "Node_SetPosition", &[hv(&a), f(1.0), f(2.0), f(3.0)]);
    c(&mut fx, "Node_SetPosition", &[hv(&b), f(1.0), f(0.0), f(0.0)]);
    c(&mut fx, "Node_SetRotationAxis", &[hv(&a), f(0.0), f(1.0), f(0.0), f(std::f64::consts::FRAC_PI_2)]);
    c(&mut fx, "Node_GetDerivedPosition", &[hv(&b), f(0.0), f(0.0), f(0.0)]);
    let o: Vec<f64> = fx.outputs.iter().map(|x| x.1.as_float()).collect();
    assert!((o[0] - 1.0).abs() < 1e-9 && (o[1] - 2.0).abs() < 1e-9 && (o[2] - 2.0).abs() < 1e-9, "{o:?}");
    assert_eq!(c(&mut fx, "Node_GetNumChildren", &[hv(&root)]).as_float(), 1.0);
    c(&mut fx, "Node_RemoveChild", &[hv(&root), hv(&a)]);
    assert_eq!(c(&mut fx, "Node_GetParent", &[hv(&a)]).as_float(), 0.0);
}

#[test]
fn manual_object_mesh_ray_and_render() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    let mo = c(&mut fx, "ManualObject_Create", &[hv(&sc), st("tri")]);
    c(&mut fx, "ManualObject_Begin", &[hv(&mo), st(""), f(4.0)]);
    for p in [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]] {
        c(&mut fx, "ManualObject_Position", &[hv(&mo), f(p[0]), f(p[1]), f(p[2])]);
    }
    c(&mut fx, "ManualObject_End", &[hv(&mo)]);
    let n = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("n")]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&n), hv(&mo)]);
    c(&mut fx, "Collision_RayCast", &[hv(&sc), f(0.0), f(0.0), f(5.0), f(0.0), f(0.0), f(-1.0)]);
    assert_eq!(c(&mut fx, "Collision_GetResultCount", &[]).as_float(), 1.0);
    assert!((c(&mut fx, "Collision_GetDistance", &[f(0.0)]).as_float() - 5.0).abs() < 1e-9);
    assert_eq!(c(&mut fx, "Collision_GetObject", &[f(0.0)]).as_float(), mo.as_float());
    // камера и кадр
    let cam = c(&mut fx, "Camera_Create", &[hv(&sc), st("c")]);
    let cn = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("cn")]);
    c(&mut fx, "Node_SetPosition", &[hv(&cn), f(0.0), f(0.0), f(5.0)]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&cn), hv(&cam)]);
    c(&mut fx, "Camera_SetNearClipDistance", &[hv(&cam), f(0.5)]);
    let win = c(&mut fx, "RenderWindow_GetPrimary", &[]);
    let vp = c(&mut fx, "Viewport_Create", &[hv(&win), hv(&cam), f(0.0)]);
    assert!(vp.as_float() >= 1.0);
    assert_eq!(c(&mut fx, "Viewport_Create", &[hv(&win), hv(&cam), f(0.0)]).as_float(), 0.0);
    c(&mut fx, "Viewport_SetBackgroundColour", &[hv(&vp), f(0.0), f(0.0), f(1.0)]);
    let fr = fx.ogre.render(None).unwrap();
    let mid = (fr.height / 2 * fr.width + fr.width / 2) * 3;
    assert_ne!(&fr.rgb[mid..mid + 3], &[0, 0, 255]);
    assert_eq!(&fr.rgb[0..3], &[0, 0, 255]);
    assert_eq!(c(&mut fx, "ManualObject_ConvertToMesh", &[hv(&mo), st("m1")]).as_float(), 0.0);
    let e = c(&mut fx, "Entity_Create", &[hv(&sc), st("e"), st("m1")]);
    assert_eq!(c(&mut fx, "Entity_GetVertexCount", &[hv(&e), f(0.0)]).as_float(), 3.0);
    assert_eq!(c(&mut fx, "Entity_GetIndexCount", &[hv(&e), f(0.0)]).as_float(), 3.0);
}

#[test]
fn caps_and_bad_input() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    assert_eq!(c(&mut fx, "BillboardSet_Create", &[hv(&sc), st("b"), f(1e9)]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "RenderTexture_Create", &[st("t"), f(1e9), f(1.0)]).as_float(), 0.0);
    let mo = c(&mut fx, "ManualObject_Create", &[hv(&sc), st("m")]);
    c(&mut fx, "ManualObject_Position", &[hv(&mo), f(0.0), f(0.0), f(0.0)]);
    c(&mut fx, "ManualObject_Begin", &[hv(&mo), st(""), f(99.0)]);
    c(&mut fx, "ManualObject_End", &[hv(&mo)]);
    assert_eq!(c(&mut fx, "Entity_Create", &[hv(&sc), st("e"), st("../../etc/passwd")]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "Collision_GetObject", &[f(f64::NAN)]).as_float(), 0.0);
    c(&mut fx, "Node_SetPosition", &[f(1e300), f(1.0), f(1.0), f(f64::INFINITY)]);
    assert!(fx.ogre.errors > 4);
    // сброс
    c(&mut fx, "Root_Destroy", &[]);
    assert_eq!(c(&mut fx, "Root_IsInitialised", &[]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "Scene_GetCamera", &[hv(&sc), st("x")]).as_float(), 0.0);
}

#[test]
fn materials_and_animation() {
    let mut fx = Effects::default();
    boot(&mut fx);
    let m = c(&mut fx, "Material_Create", &[st("Mat")]);
    assert_eq!(c(&mut fx, "Material_Create", &[st("mat")]).as_float(), 0.0);
    assert_eq!(c(&mut fx, "Material_Get", &[st("MAT")]).as_float(), m.as_float());
    let t = c(&mut fx, "Material_GetBestTechnique", &[hv(&m)]);
    let p = c(&mut fx, "Technique_GetPassByIndex", &[hv(&t), f(0.0)]);
    assert!(p.as_float() >= 1.0);
    assert_eq!(c(&mut fx, "Pass_GetLightingEnabled", &[hv(&p)]).as_float(), 1.0);
    c(&mut fx, "Pass_SetLightingEnabled", &[hv(&p), f(7.0)]);
    assert_eq!(c(&mut fx, "Pass_GetLightingEnabled", &[hv(&p)]).as_float(), 1.0);
    c(&mut fx, "Pass_SetLightingEnabled", &[hv(&p), f(0.0)]);
    assert_eq!(c(&mut fx, "Pass_GetLightingEnabled", &[hv(&p)]).as_float(), 0.0);
    let txt = "material A : B // c\n{ technique { pass { diffuse 0.5 0.25 1 lighting off } } }";
    fx.ogre.parse_materials(txt);
    let a = c(&mut fx, "Material_Get", &[st("A")]);
    let t = c(&mut fx, "Material_GetTechniqueByIndex", &[hv(&a), f(0.0)]);
    let p = c(&mut fx, "Technique_GetPassByIndex", &[hv(&t), f(0.0)]);
    c(&mut fx, "Pass_GetDiffuse", &[hv(&p), f(0.0), f(0.0), f(0.0), f(0.0)]);
    let o: Vec<f64> = fx.outputs.iter().map(|x| x.1.as_float()).collect();
    assert_eq!(&o[o.len() - 4..], &[0.5, 0.25, 1.0, 1.0]);
}

#[test]
fn parses_fixture_meshes() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples/Ogre_3D");
    let mut n = 0;
    let mut stack = vec![dir];
    let mut bad = Vec::new();
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "mesh") {
                n += 1;
                match parse_mesh(&std::fs::read(&p).unwrap()) {
                    Some(m) if !m.subs.is_empty() && m.subs.iter().all(|s| !s.verts.is_empty()) => {}
                    _ => bad.push(p),
                }
            } else if p.extension().is_some_and(|x| x == "skeleton") {
                assert!(parse_skeleton(&std::fs::read(&p).unwrap()).is_some(), "{p:?}");
            }
        }
    }
    eprintln!("meshes: {n}, unparsed: {bad:?}");
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn mesh_parser_fuzz() {
    let mut x = 12345u64;
    let base = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples/Ogre_3D/Project2/TestScene/Teapot01.mesh")).unwrap_or_default();
    for _ in 0..300 {
        let mut d = base.clone();
        for _ in 0..8 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if !d.is_empty() {
                let i = (x >> 33) as usize % d.len();
                d[i] = (x >> 20) as u8;
            }
        }
        let _ = parse_mesh(&d);
        let _ = parse_skeleton(&d);
        let _ = parse_mesh(&d[..d.len() / 3]);
    }
}
