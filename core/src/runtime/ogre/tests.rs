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
    c(fx, "RenderWindow_Create2", &[Value::Handle(0.0), st("t"), f(800.0), f(600.0), f(0.0), f(0.0), f(0.0), f(0.0)]);
    c(fx, "Scene_Create", &[st("OctreeSceneManager")])
}

#[test]
fn every_function_is_implemented() {
    assert_eq!(function_names().len(), 315);
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

fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
    let mut v = id.to_le_bytes().to_vec();
    v.extend_from_slice(&((body.len() + 6) as u32).to_le_bytes());
    v.extend_from_slice(body);
    v
}
fn f32s(a: &[f32]) -> Vec<u8> {
    a.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn skinned_files(dir: &std::path::Path) {
    // сетка: один треугольник, все вершины привязаны к кости 1
    let mut geom = 3u32.to_le_bytes().to_vec();
    let mut elem = Vec::new();
    for v in [0u16, 2, 1, 0, 0] {
        elem.extend_from_slice(&v.to_le_bytes());
    }
    geom.extend(chunk(0x5100, &chunk(0x5110, &elem)));
    let mut buf = 0u16.to_le_bytes().to_vec();
    buf.extend_from_slice(&12u16.to_le_bytes());
    buf.extend(chunk(0x5210, &f32s(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0])));
    geom.extend(chunk(0x5200, &buf));
    let mut sub = b"mat\n".to_vec();
    sub.push(0);
    sub.extend_from_slice(&3u32.to_le_bytes());
    sub.push(0);
    for i in [0u16, 1, 2] {
        sub.extend_from_slice(&i.to_le_bytes());
    }
    sub.extend(chunk(0x5000, &geom));
    for v in 0..3u32 {
        let mut asg = v.to_le_bytes().to_vec();
        asg.extend_from_slice(&1u16.to_le_bytes());
        asg.extend_from_slice(&1f32.to_le_bytes());
        sub.extend(chunk(0x4100, &asg));
    }
    let mut mesh_body = vec![1u8];
    mesh_body.extend(chunk(0x4000, &sub));
    mesh_body.extend(chunk(0x6000, b"anim.skeleton\n"));
    let mut mesh = vec![0x00, 0x10];
    mesh.extend_from_slice(b"[MeshSerializer_v1.40]\n");
    mesh.extend(chunk(0x3000, &mesh_body));
    std::fs::write(dir.join("tri.mesh"), mesh).unwrap();
    // скелет: корень и кость 1; анимация сдвигает кость 1 на +2 по X
    let bone = |name: &str, h: u16| {
        let mut b = format!("{name}\n").into_bytes();
        b.extend_from_slice(&h.to_le_bytes());
        b.extend(f32s(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]));
        chunk(0x2000, &b)
    };
    let mut sk = vec![0x00, 0x10];
    sk.extend_from_slice(b"[Serializer_v1.10]\n");
    sk.extend(bone("root", 0));
    sk.extend(bone("arm", 1));
    let mut par = 1u16.to_le_bytes().to_vec();
    par.extend_from_slice(&0u16.to_le_bytes());
    sk.extend(chunk(0x3000, &par));
    let key = |t: f32, x: f32| {
        let mut k = f32s(&[t, 0.0, 0.0, 0.0, 1.0, x, 0.0, 0.0]);
        k.truncate(32);
        chunk(0x4110, &k)
    };
    let mut track = 1u16.to_le_bytes().to_vec();
    track.extend(key(0.0, 0.0));
    track.extend(key(1.0, 2.0));
    let mut anim = b"move\n".to_vec();
    anim.extend(f32s(&[1.0]));
    anim.extend(chunk(0x4100, &track));
    sk.extend(chunk(0x4000, &anim));
    std::fs::write(dir.join("anim.skeleton"), sk).unwrap();
}

#[test]
fn skeletal_animation_skins_vertices() {
    let dir = std::env::temp_dir().join(format!("stratum-ogre-skin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    skinned_files(&dir);
    let mut fx = Effects::default();
    fx.gfx.project_dir = dir.clone();
    let sc = boot(&mut fx);
    let e = c(&mut fx, "Entity_Create", &[hv(&sc), st("e"), st("tri.mesh")]);
    assert!(e.as_float() >= 1.0, "{:?}", fx.log);
    let n = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("n")]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&n), hv(&e)]);
    assert_eq!(c(&mut fx, "Entity_GetAnimationState", &[hv(&e), st("nope")]).as_float(), 0.0);
    let a = c(&mut fx, "Entity_GetAnimationState", &[hv(&e), st("move")]);
    assert!(a.as_float() >= 1.0);
    assert_eq!(c(&mut fx, "AnimationState_GetLength", &[hv(&a)]).as_float(), 1.0);
    assert!(c(&mut fx, "Entity_GetBone", &[hv(&e), st("arm")]).as_float() >= 1.0);
    let x0 = fx.ogre.world_tris(e.as_float() as u32)[0][1][0];
    assert!((x0 - 1.0).abs() < 1e-6);
    c(&mut fx, "AnimationState_SetEnabled", &[hv(&a), f(1.0)]);
    c(&mut fx, "AnimationState_AddTime", &[hv(&a), f(0.5)]);
    let x1 = fx.ogre.world_tris(e.as_float() as u32)[0][1][0];
    assert!((x1 - 2.0).abs() < 1e-5, "{x1}");
    // цикл: 1.25 -> 0.25
    c(&mut fx, "AnimationState_AddTime", &[hv(&a), f(0.75)]);
    assert!((c(&mut fx, "AnimationState_GetTimePosition", &[hv(&a)]).as_float() - 0.25).abs() < 1e-9);
    c(&mut fx, "AnimationState_SetLoop", &[hv(&a), f(0.0)]);
    c(&mut fx, "AnimationState_AddTime", &[hv(&a), f(5.0)]);
    assert_eq!(c(&mut fx, "AnimationState_GetTimePosition", &[hv(&a)]).as_float(), 1.0);
    let _ = std::fs::remove_dir_all(dir);
}

fn fixture_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/PROJECTS/samples/Ogre_3D")
}

#[test]
fn scene_load_dotscene() {
    let dir = fixture_dir().join("Project2");
    if !dir.exists() {
        return;
    }
    let mut fx = Effects::default();
    fx.gfx.project_dir = dir;
    let sc = boot(&mut fx);
    c(&mut fx, "Root_AddResourceLocation", &[st("TestScene"), st("FileSystem"), st("General"), f(1.0)]);
    c(&mut fx, "Root_InitialiseAllResourceGroups", &[]);
    c(&mut fx, "Scene_Load", &[hv(&sc), st("TestScene/Lesson2.scene")]);
    assert_eq!(fx.ogre.errors, 0, "{:?}", fx.log);
    let t = c(&mut fx, "Scene_GetEntity", &[hv(&sc), st("Teapot01")]);
    assert!(t.as_float() >= 1.0);
    assert!(c(&mut fx, "Entity_GetVertexCount", &[hv(&t), f(0.0)]).as_float() > 100.0);
    assert!(c(&mut fx, "Scene_GetLight", &[hv(&sc), st("Omni01")]).as_float() >= 1.0);
    assert!(c(&mut fx, "Scene_GetEntity", &[hv(&sc), st("Plane01")]).as_float() >= 1.0);
    assert!(c(&mut fx, "Material_Get", &[st("01-Default")]).as_float() >= 1.0);
}

#[test]
fn texture_decoders_on_fixtures() {
    let root = fixture_dir();
    for (rel, min) in [("CompositorDemo/Data/RustyBarrel.png", 8), ("ManualMaterial/Data/MRAMOR.jpg", 8), ("CompositorDemo/Data/WaterNormal1.tga", 8)] {
        let p = root.join(rel);
        let Ok(d) = std::fs::read(&p) else { continue };
        let t = image::decode(rel, &d).unwrap_or_else(|| panic!("{rel}"));
        assert!(t.w >= min && t.h >= min && t.rgba.len() == t.w * t.h * 4, "{rel}");
        // не сплошной цвет
        assert!(t.rgba.chunks_exact(4).any(|p| p[..3] != t.rgba[..3]), "{rel} is flat");
    }
    // мусор не падает
    for n in ["a.png", "a.jpg", "a.tga", "a.bmp"] {
        assert!(image::decode(n, &[0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0]).is_none());
        assert!(image::decode(n, &[0xFF, 0xD8, 0xFF, 0xDA, 0, 2]).is_none());
        assert!(image::decode(n, &[]).is_none());
    }
}

#[test]
fn texture_fuzz_does_not_panic() {
    let root = fixture_dir();
    let mut x = 99u64;
    for rel in ["CompositorDemo/Data/RustyBarrel.png", "ManualMaterial/Data/MRAMOR.jpg", "CompositorDemo/Data/WaterNormal1.tga"] {
        let Ok(base) = std::fs::read(root.join(rel)) else { continue };
        for _ in 0..40 {
            let mut d = base.clone();
            for _ in 0..6 {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let i = (x >> 33) as usize % d.len();
                d[i] = (x >> 20) as u8;
            }
            let _ = image::decode(rel, &d);
            let _ = image::decode(rel, &d[..d.len() / 2]);
        }
    }
}

#[test]
fn fog_and_texture_affect_pixels() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    let mo = c(&mut fx, "ManualObject_Create", &[hv(&sc), st("q")]);
    c(&mut fx, "ManualObject_Begin", &[hv(&mo), st(""), f(4.0)]);
    for p in [[-5.0, -5.0, 0.0], [5.0, -5.0, 0.0], [5.0, 5.0, 0.0], [-5.0, 5.0, 0.0]] {
        c(&mut fx, "ManualObject_Position", &[hv(&mo), f(p[0]), f(p[1]), f(p[2])]);
    }
    for i in [0.0, 1.0, 2.0, 0.0, 2.0, 3.0] {
        c(&mut fx, "ManualObject_Index", &[hv(&mo), f(i)]);
    }
    c(&mut fx, "ManualObject_End", &[hv(&mo)]);
    let n = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("n")]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&n), hv(&mo)]);
    let cam = c(&mut fx, "Camera_Create", &[hv(&sc), st("c")]);
    let cn = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("cn")]);
    c(&mut fx, "Node_SetPosition", &[hv(&cn), f(0.0), f(0.0), f(20.0)]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&cn), hv(&cam)]);
    c(&mut fx, "Camera_SetNearClipDistance", &[hv(&cam), f(1.0)]);
    let win = c(&mut fx, "RenderWindow_GetPrimary", &[]);
    c(&mut fx, "Viewport_Create", &[hv(&win), hv(&cam), f(0.0)]);
    c(&mut fx, "Scene_SetAmbientLight", &[hv(&sc), f(1.0), f(1.0), f(1.0)]);
    let mid = |fr: &Frame| {
        let i = (fr.height / 2 * fr.width + fr.width / 2) * 3;
        [fr.rgb[i], fr.rgb[i + 1], fr.rgb[i + 2]]
    };
    let plain = mid(&fx.ogre.render(None).unwrap());
    assert!(plain[0] > 150, "{plain:?}");
    // сплошной туман: цвет тумана
    c(&mut fx, "Scene_SetFog", &[hv(&sc), f(3.0), f(1.0), f(0.0), f(0.0), f(0.0), f(1.0), f(2.0)]);
    let fogged = mid(&fx.ogre.render(None).unwrap());
    assert_eq!(fogged, [255, 0, 0]);
    // точечный свет без окружения
    c(&mut fx, "Scene_SetFog", &[hv(&sc), f(0.0), f(0.0), f(0.0), f(0.0), f(0.0), f(0.0), f(0.0)]);
    c(&mut fx, "Scene_SetAmbientLight", &[hv(&sc), f(0.0), f(0.0), f(0.0)]);
    let dark = mid(&fx.ogre.render(None).unwrap());
    assert_eq!(dark, [0, 0, 0]);
    let lt = c(&mut fx, "Light_Create", &[hv(&sc), st("l")]);
    let ln = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("ln")]);
    c(&mut fx, "Node_SetPosition", &[hv(&ln), f(0.0), f(0.0), f(5.0)]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&ln), hv(&lt)]);
    c(&mut fx, "Light_SetAttenuation", &[hv(&lt), f(100.0), f(1.0), f(0.0), f(0.0)]);
    let lit = mid(&fx.ogre.render(None).unwrap());
    assert!(lit[0] > 100, "{lit:?}");
}

#[test]
fn sinbad_skeleton_from_zip() {
    let dir = fixture_dir().join("SkeletalAnimationDemo");
    if !dir.join("Data/Sinbad.zip").exists() {
        return;
    }
    let mut fx = Effects::default();
    fx.gfx.project_dir = dir;
    let sc = boot(&mut fx);
    c(&mut fx, "Root_AddResourceLocationFromConfigFile", &[st("Resources.cfg")]);
    let e = c(&mut fx, "Entity_Create", &[hv(&sc), st("sinbad"), st("Sinbad.mesh")]);
    assert!(e.as_float() >= 1.0, "{:?}", fx.log);
    let n = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("n")]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&n), hv(&e)]);
    assert!(c(&mut fx, "Entity_GetSubEntityCount", &[hv(&e)]).as_float() >= 1.0);
    let bounds = |fx: &mut Effects| {
        let mut mn = [f64::MAX; 3];
        let mut mx = [f64::MIN; 3];
        for t in fx.ogre.world_tris(e.as_float() as u32) {
            for v in t {
                for k in 0..3 {
                    assert!(v[k].is_finite());
                    mn[k] = mn[k].min(v[k]);
                    mx[k] = mx[k].max(v[k]);
                }
            }
        }
        (mn, mx)
    };
    if let Some(m) = &fx.ogre.objs[&(e.as_float() as u32)].mesh {
        eprintln!("mesh bounds {:?} {:?}", m.min, m.max);
        for s in &m.subs {
            eprintln!("sub {} v={} i={} bones={} nrm={} uv={}", s.material, s.verts.len(), s.idx.len(), s.bones.len(), s.nrm.len(), s.uv.len());
        }
    }
    let (bmin, bmax) = bounds(&mut fx);
    eprintln!("bind: {bmin:?} {bmax:?}");
    let a = c(&mut fx, "Entity_GetAnimationState", &[hv(&e), st("RunBase")]);
    assert!(a.as_float() >= 1.0, "animation RunBase");
    let len = c(&mut fx, "AnimationState_GetLength", &[hv(&a)]).as_float();
    assert!(len > 0.1);
    c(&mut fx, "AnimationState_SetEnabled", &[hv(&a), f(1.0)]);
    c(&mut fx, "AnimationState_SetTimePosition", &[hv(&a), f(len / 3.0)]);
    let (amin, amax) = bounds(&mut fx);
    eprintln!("anim: {amin:?} {amax:?}");
    // кожа не разваливается: размеры поз соизмеримы
    for k in 0..3 {
        let (b, an) = (bmax[k] - bmin[k], amax[k] - amin[k]);
        assert!(an > b * 0.3 && an < b * 3.0, "axis {k}: bind {b} anim {an}");
    }
    assert!(c(&mut fx, "Entity_GetBone", &[hv(&e), st("Handle.L")]).as_float() >= 0.0);
}


fn crc_less_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut cd = Vec::new();
    for (name, data) in entries {
        let off = out.len() as u32;
        out.extend_from_slice(&0x04034b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        cd.extend_from_slice(&0x02014b50u32.to_le_bytes());
        cd.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        cd.extend_from_slice(&0u32.to_le_bytes());
        cd.extend_from_slice(&(data.len() as u32).to_le_bytes());
        cd.extend_from_slice(&(data.len() as u32).to_le_bytes());
        cd.extend_from_slice(&(name.len() as u16).to_le_bytes());
        cd.extend_from_slice(&[0; 12]);
        cd.extend_from_slice(&off.to_le_bytes());
        cd.extend_from_slice(name.as_bytes());
    }
    let cdoff = out.len() as u32;
    out.extend_from_slice(&cd);
    out.extend_from_slice(&0x06054b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(cd.len() as u32).to_le_bytes());
    out.extend_from_slice(&cdoff.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[test]
fn zip_stored_roundtrip_and_path_guards() {
    let z = super::zip::Zip::open(crc_less_zip(&[("a/b.txt", b"hello"), ("../evil.txt", b"x")])).expect("zip");
    assert_eq!(z.read("a/b.txt").as_deref(), Some(&b"hello"[..]));
    assert!(z.read("../evil.txt").is_none());
    assert!(z.names().iter().all(|n| !n.contains("..")));
    assert!(z.read("missing").is_none());
}

#[test]
fn zip_fuzz_does_not_panic() {
    let base = crc_less_zip(&[("a.txt", b"hello world"), ("b.txt", b"0123456789")]);
    let mut x = 99u64;
    for _ in 0..500 {
        let mut d = base.clone();
        for _ in 0..4 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let i = (x >> 33) as usize % d.len();
            d[i] = (x >> 20) as u8;
        }
        if let Some(z) = super::zip::Zip::open(d) {
            for n in z.names() {
                let _ = z.read(&n);
            }
        }
    }
}

fn td(name: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/ogre/testdata").join(name)).unwrap()
}

#[test]
fn adam7_png_equals_plain_png() {
    for (a, b) in [("adam7.png", "plain.png"), ("adam7_p4.png", "plain_p4.png"), ("adam7_1bit.png", "plain_1bit.png")] {
        let (ta, tb) = (image::decode(a, &td(a)).unwrap_or_else(|| panic!("{a}")), image::decode(b, &td(b)).unwrap());
        assert_eq!((ta.w, ta.h), (tb.w, tb.h));
        assert!(ta.rgba == tb.rgba, "{a}");
    }
}

#[test]
fn progressive_jpeg_equals_baseline() {
    for (a, b) in [("prog.jpg", "base.jpg"), ("prog420.jpg", "base420.jpg"), ("prog_g.jpg", "base_g.jpg")] {
        let (ta, tb) = (image::decode(a, &td(a)).unwrap_or_else(|| panic!("{a}")), image::decode(b, &td(b)).unwrap());
        assert_eq!((ta.w, ta.h), (tb.w, tb.h));
        let diff = ta.rgba.iter().zip(&tb.rgba).filter(|(x, y)| x != y).count();
        assert!(diff * 100 < ta.rgba.len(), "{a}: {diff} differing bytes");
    }
}

#[test]
fn progressive_and_interlaced_fuzz() {
    let mut x = 7u64;
    for n in ["prog.jpg", "prog420.jpg", "prog_g.jpg", "adam7.png", "adam7_p4.png"] {
        let base = td(n);
        for _ in 0..300 {
            let mut d = base.clone();
            for _ in 0..6 {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let i = (x >> 33) as usize % d.len();
                d[i] = (x >> 20) as u8;
            }
            let _ = image::decode(n, &d);
            let _ = image::decode(n, &d[..d.len() * 2 / 3]);
        }
    }
}

fn quad(fx: &mut Effects, sc: &Value, name: &str, z: f64, h: f64) -> Value {
    let mo = c(fx, "ManualObject_Create", &[hv(sc), st(name)]);
    c(fx, "ManualObject_Begin", &[hv(&mo), st(""), f(4.0)]);
    for p in [[-h, -h, z], [h, -h, z], [h, h, z], [-h, h, z]] {
        c(fx, "ManualObject_Position", &[hv(&mo), f(p[0]), f(p[1]), f(p[2])]);
    }
    for i in [0.0, 1.0, 2.0, 0.0, 2.0, 3.0] {
        c(fx, "ManualObject_Index", &[hv(&mo), f(i)]);
    }
    c(fx, "ManualObject_End", &[hv(&mo)]);
    let n = c(fx, "SceneNode_Create", &[hv(sc), st(&format!("n_{name}"))]);
    c(fx, "SceneNode_AttachObject", &[hv(&n), hv(&mo)]);
    mo
}

fn view(fx: &mut Effects, sc: &Value) -> Value {
    let cam = c(fx, "Camera_Create", &[hv(sc), st("c")]);
    let cn = c(fx, "SceneNode_Create", &[hv(sc), st("cn")]);
    c(fx, "Node_SetPosition", &[hv(&cn), f(0.0), f(0.0), f(20.0)]);
    c(fx, "SceneNode_AttachObject", &[hv(&cn), hv(&cam)]);
    c(fx, "Camera_SetNearClipDistance", &[hv(&cam), f(1.0)]);
    let win = c(fx, "RenderWindow_GetPrimary", &[]);
    c(fx, "Viewport_Create", &[hv(&win), hv(&cam), f(0.0)])
}

#[test]
fn compositor_passes_and_shader_log() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    quad(&mut fx, &sc, "q", 0.0, 5.0);
    let vp = view(&mut fx, &sc);
    c(&mut fx, "Scene_SetAmbientLight", &[hv(&sc), f(1.0), f(0.0), f(0.0)]);
    fx.ogre.ingest_script(
        "material",
        "material Ogre/Compositor/BlackAndWhite { technique { pass { fragment_program_ref BW_ps { } } } }",
    );
    assert!(fx.ogre.pending_log.iter().any(|m| m.contains("шейдер")), "{:?}", fx.ogre.pending_log);
    fx.ogre.ingest_script(
        "compositor",
        "compositor BW { technique { texture rt0 target_width target_height PF_A8R8G8B8 target rt0 { input previous } \\
         target_output { input none pass render_quad { material Ogre/Compositor/BlackAndWhite input 0 rt0 } } } }
         compositor Blue { technique { target_output { input none pass clear { colour_value 0 0 1 1 } } } }",
    );
    let mid = |fx: &Effects| {
        let fr = fx.ogre.render(None).unwrap();
        let i = (fr.height / 2 * fr.width + fr.width / 2) * 3;
        [fr.rgb[i], fr.rgb[i + 1], fr.rgb[i + 2]]
    };
    let plain = mid(&fx);
    assert!(plain[0] > 200 && plain[1] == 0, "{plain:?}");
    let bw = c(&mut fx, "Viewport_AddCompositor", &[hv(&vp), st("BW"), f(-1.0)]);
    assert!(bw.as_float() >= 1.0, "{:?}", fx.log);
    // выключен: картинка не меняется
    assert_eq!(mid(&fx), plain);
    c(&mut fx, "Compositor_SetEnabled", &[hv(&bw), f(1.0)]);
    let g = mid(&fx);
    assert!(g[0] == g[1] && g[1] == g[2] && g[0] > 20, "{g:?}");
    c(&mut fx, "Compositor_SetEnabled", &[hv(&bw), f(0.0)]);
    let blue = c(&mut fx, "Viewport_AddCompositor", &[hv(&vp), st("Blue"), f(-1.0)]);
    c(&mut fx, "Compositor_SetEnabled", &[hv(&blue), f(1.0)]);
    assert_eq!(mid(&fx), [0, 0, 255]);
}

#[test]
fn shadows_darken_receivers_deterministically() {
    let mut fx = Effects::default();
    let sc = boot(&mut fx);
    quad(&mut fx, &sc, "floor", 0.0, 8.0);
    quad(&mut fx, &sc, "caster", 3.0, 2.0);
    view(&mut fx, &sc);
    c(&mut fx, "Scene_SetAmbientLight", &[hv(&sc), f(0.2), f(0.2), f(0.2)]);
    let lt = c(&mut fx, "Light_Create", &[hv(&sc), st("l")]);
    c(&mut fx, "Light_SetType", &[hv(&lt), f(1.0)]);
    let ln = c(&mut fx, "SceneNode_Create", &[hv(&sc), st("ln")]);
    c(&mut fx, "Node_SetRotationEulerXYZ", &[hv(&ln), f(0.0), f(0.5), f(0.0)]);
    c(&mut fx, "SceneNode_AttachObject", &[hv(&ln), hv(&lt)]);
    let off = fx.ogre.render(None).unwrap();
    c(&mut fx, "Scene_SetShadowTechnique", &[hv(&sc), f(18.0)]);
    let on = fx.ogre.render(None).unwrap();
    let on2 = fx.ogre.render(None).unwrap();
    assert!(on.rgb == on2.rgb, "not deterministic");
    let darker = on.rgb.chunks_exact(3).zip(off.rgb.chunks_exact(3)).filter(|(a, b)| a[0] < b[0]).count();
    let brighter = on.rgb.chunks_exact(3).zip(off.rgb.chunks_exact(3)).filter(|(a, b)| a[0] > b[0]).count();
    assert!(darker > 50, "darker {darker}");
    assert_eq!(brighter, 0);
}

#[test]
fn cyrillic_glyphs_are_drawn() {
    let set = |s: &str| {
        let mut v = std::collections::BTreeSet::new();
        fx::draw_text(s, 0.0, 0.0, 7.0, |x, y| {
            v.insert((x, y));
        });
        v
    };
    let q = set("?");
    for ch in ["Ж", "ж", "Б", "ю", "Я", "Ё", "щ"] {
        let g = set(ch);
        assert!(g.len() > 8 && g != q, "{ch}");
    }
    assert_eq!(set("ж"), set("Ж"));
    assert_eq!(set("а"), set("A"));
    assert_ne!(set("Привет"), set("??????"));
}

/// Порядок углов и перемножения Node_SetRotationEuler* - числа оригинала (Wine,
/// tools/verify/ogre_euler.txt), углы 0.3, 0.5, 0.7.
#[test]
fn euler_order_matches_original() {
    let want = [
        ("xyz", [0.887272, 0.219896, 0.180146, 0.363237]),
        ("zyx", [0.912627, 0.293777, 0.279444, 0.0521324]),
        ("yxz", [0.912627, 0.279444, 0.0521324, 0.293777]),
        ("xzy", [0.912627, 0.0521324, 0.293777, 0.279444]),
        ("yzx", [0.887272, 0.363237, 0.219896, 0.180146]),
        ("zxy", [0.887272, 0.180146, 0.363237, 0.219896]),
    ];
    for (order, q) in want {
        let got = super::euler(order, [0.3, 0.5, 0.7]);
        for i in 0..4 {
            assert!((got[i] - q[i]).abs() < 2e-6, "{order}: {got:?} vs {q:?}");
        }
    }
}
