//! Семейство функций Ogre 3D: состояние сцены в ядре.
//!
//! Модель держит дескрипторы объектов (сцены, камеры, источники света,
//! модели, узлы, материалы, области рендеринга...). Здесь хранится их
//! состояние, а `render` рисует сцену программным растеризатором (плоская
//! закраска, буфер глубины) в кадр окна рендеринга. Сигнатуры функций
//! берутся из таблицы `ogre_sigs.txt` (из docs/lang/functions.md).
//!
//! Допущения, которых нет в справке: углы в радианах; кватернион (w, x, y, z);
//! Node_SetRotationEuler*: аргументы (рыскание Y, тангаж X, крен Z), буквы в
//! имени задают порядок перемножения поворотов; объект Billboard адресуется
//! дескриптором своего BillboardSet (первая плоскость); SceneNode_Create не
//! подключает узел к корню (как createSceneNode в Ogre).

use super::builtins::Effects;
use super::value::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

const MAX_OBJECTS: usize = 50_000;
const MAX_NAME: usize = 256;
const MAX_TEXT: usize = 65_536;
const MAX_VERTS: usize = 1 << 20;
const MAX_INDICES: usize = 3 << 20;
const MAX_RESULTS: usize = 10_000;
const MAX_LOCATIONS: usize = 256;
const MAX_FILE: u64 = 64 << 20;
const MAX_PARAMS: usize = 256;
const MAX_ERRORS_LOGGED: u32 = 50;
const MAX_DEPTH: usize = 1000;
const MAX_TRIS_TESTED: usize = 5_000_000;
const MAX_SURFACE: usize = 16384;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Scene,
    Camera,
    Light,
    Entity,
    Node,
    BillboardSet,
    Billboard,
    ManualObject,
    MovableText,
    ParticleSystem,
    AnimState,
    Bone,
    Material,
    Technique,
    Pass,
    TextureUnit,
    Overlay,
    Element,
    Viewport,
    Window,
    Texture,
    Compositor,
}

impl Kind {
    fn is_movable(self) -> bool {
        matches!(self, Kind::Camera | Kind::Light | Kind::Entity | Kind::BillboardSet | Kind::ManualObject | Kind::MovableText | Kind::ParticleSystem)
    }
}

type V3 = [f64; 3];
type Q = [f64; 4];

#[derive(Debug, Clone, Default)]
pub struct SubMesh {
    pub verts: Vec<[f32; 3]>,
    pub idx: Vec<u32>,
    pub material: String,
    pub op: u16,
    pub nrm: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    /// Привязки вершин к костям: на вершину пары (кость, вес).
    pub bones: Vec<Vec<(u16, f32)>>,
}

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub subs: Vec<SubMesh>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub skeleton: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BoneDef {
    pub name: String,
    pub handle: u16,
    pub parent: Option<usize>,
    pub pos: V3,
    pub rot: Q,
}

#[derive(Debug, Clone, Default)]
pub struct Key {
    pub t: f64,
    pub rot: Q,
    pub pos: V3,
}

#[derive(Debug, Clone, Default)]
pub struct Anim {
    pub name: String,
    pub length: f64,
    /// Дорожки: индекс кости, ключи по времени.
    pub tracks: Vec<(usize, Vec<Key>)>,
}

#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub bones: Vec<BoneDef>,
    pub anims: Vec<Anim>,
}

/// Текстура: RGBA построчно сверху вниз.
#[derive(Debug, Clone)]
pub struct Tex {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
struct Section {
    material: String,
    op: u16,
    verts: Vec<[f32; 3]>,
    norms: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    cols: Vec<[f32; 4]>,
    idx: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
struct Manual {
    sections: Vec<Section>,
    cur: Option<Section>,
    normal: [f32; 3],
    uv: [f32; 2],
    col: [f32; 4],
}

#[derive(Debug, Clone)]
struct Obj {
    kind: Kind,
    name: String,
    scene: u32,
    parent: u32,
    children: Vec<u32>,
    objects: Vec<u32>,
    f: HashMap<String, Vec<f64>>,
    st: HashMap<String, String>,
    params: Vec<(String, String)>,
    mesh: Option<Arc<Mesh>>,
    sub_mat: Vec<String>,
    manual: Option<Manual>,
    track: u32,
    track_off: V3,
    container: bool,
    skel: Option<Arc<Skeleton>>,
}

impl Obj {
    fn new(kind: Kind, name: &str, scene: u32) -> Obj {
        Obj {
            kind,
            name: name.to_string(),
            scene,
            parent: 0,
            children: Vec::new(),
            objects: Vec::new(),
            f: HashMap::new(),
            st: HashMap::new(),
            params: Vec::new(),
            mesh: None,
            sub_mat: Vec::new(),
            manual: None,
            track: 0,
            track_off: [0.0; 3],
            container: false,
            skel: None,
        }
    }
}

/// Результат кадра: размеры и пиксели RGB.
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct Ogre {
    objs: BTreeMap<u32, Obj>,
    next: u32,
    root_created: bool,
    initialised: bool,
    config: String,
    config_saved: bool,
    started: Option<std::time::Instant>,
    pub frames: u64,
    locations: Vec<(String, bool)>,
    materials: BTreeMap<String, u32>,
    overlays: BTreeMap<String, u32>,
    elements: BTreeMap<String, u32>,
    texts: BTreeMap<String, u32>,
    meshes: BTreeMap<String, Arc<Mesh>>,
    skeletons: BTreeMap<String, Arc<Skeleton>>,
    compositors: BTreeSet<String>,
    hits: Vec<(f64, u32)>,
    /// Сколько вызовов отклонено (неверный дескриптор, предел, NaN).
    pub errors: u32,
    pending_log: Vec<String>,
    keys: BTreeSet<u32>,
    mouse: [bool; 3],
    cursor: [f64; 2],
    wheel: f64,
    textures: BTreeMap<String, Option<Arc<Tex>>>,
    zips: BTreeMap<String, Option<Arc<zip::Zip>>>,
    zip_bytes: u64,
    ptemplates: BTreeMap<String, Arc<fx::PTemplate>>,
    pstate: BTreeMap<u32, fx::PState>,
    pending_tex: Vec<String>,
    cached: FrameCache,
}

/// Кэш картинки кадра: не копируется при снимках состояния.
#[derive(Debug, Default)]
struct FrameCache(BTreeMap<u32, (u64, f64, f64, String)>);

impl Clone for FrameCache {
    fn clone(&self) -> Self {
        FrameCache(BTreeMap::new())
    }
}

struct Sig {
    ret: char,
    params: Vec<(char, bool)>,
}

fn sigs() -> &'static HashMap<String, Sig> {
    static S: OnceLock<HashMap<String, Sig>> = OnceLock::new();
    S.get_or_init(|| {
        let mut m = HashMap::new();
        for line in include_str!("ogre_sigs.txt").lines() {
            let mut it = line.split(' ');
            let (Some(name), Some(ret)) = (it.next(), it.next()) else { continue };
            let ps = it.next().unwrap_or("");
            let mut params = Vec::new();
            let mut out = false;
            for c in ps.chars() {
                if c == '&' {
                    out = true;
                } else {
                    params.push((c, out));
                    out = false;
                }
            }
            m.insert(name.to_ascii_lowercase(), Sig { ret: ret.chars().next().unwrap_or('V'), params });
        }
        m
    })
}

/// Принадлежит ли имя семейству Ogre 3D (есть в таблице сигнатур).
pub fn is_ogre(lower: &str) -> bool {
    sigs().contains_key(lower)
}

/// Все имена функций семейства (для проверок полноты).
pub fn function_names() -> Vec<String> {
    sigs().keys().cloned().collect()
}

/// Вызов функции Ogre. `None` - имя не из семейства.
pub fn call(lower: &str, args: &[Value], fx: &mut Effects) -> Option<Value> {
    let sig = sigs().get(lower)?;
    static TRACE: OnceLock<bool> = OnceLock::new();
    if *TRACE.get_or_init(|| std::env::var_os("STRATUM_OGRE_TRACE").is_some()) {
        eprintln!("ogre: {lower}({})", args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "));
    }
    let mut og = std::mem::take(&mut fx.ogre);
    let mut outs = Vec::new();
    // недостающие хвостовые аргументы добираем значениями по умолчанию
    // (альфа цвета - 1, остальное - 0 или пустая строка)
    let padded: Vec<Value>;
    let args = if args.len() < sig.params.len() {
        let colour = ["colour", "color", "diffuse", "ambient", "specular", "selfillum"].iter().any(|k| lower.contains(k));
        let mut v = args.to_vec();
        let mut floats = sig.params[..v.len()].iter().filter(|p| p.0 == 'F' && !p.1).count();
        for &(c, out) in &sig.params[v.len()..] {
            v.push(match c {
                'S' => Value::Str(String::new()),
                'H' => Value::Handle(0.0),
                _ => {
                    floats += if out { 0 } else { 1 };
                    Value::Float(if colour && !out && floats == 4 { 1.0 } else { 0.0 })
                }
            });
        }
        padded = v;
        &padded[..]
    } else {
        args
    };
    let r = og.run(fx, lower, sig, args, &mut outs);
    for m in og.pending_log.drain(..) {
        if fx.log.len() < 1000 {
            fx.log.push(m);
        }
    }
    og.flush_textures(fx);
    for m in og.pending_log.drain(..) {
        if fx.log.len() < 1000 {
            fx.log.push(m);
        }
    }
    fx.ogre = og;
    fx.outputs.extend(outs);
    Some(r)
}

fn zero(sig: &Sig) -> Value {
    match sig.ret {
        'H' => Value::Handle(0.0),
        'S' => Value::Str(String::new()),
        _ => Value::Float(0.0),
    }
}

fn h(id: u32) -> Value {
    Value::Handle(id as f64)
}
fn fl(v: f64) -> Value {
    Value::Float(v)
}
fn bl(b: bool) -> Value {
    Value::Float(if b { 1.0 } else { 0.0 })
}

// ---- математика ------------------------------------------------------

fn qmul(a: Q, b: Q) -> Q {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}
fn qnorm(q: Q) -> Q {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n < 1e-12 || !n.is_finite() {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
    }
}
fn qconj(q: Q) -> Q {
    [q[0], -q[1], -q[2], -q[3]]
}
fn qrot(q: Q, v: V3) -> V3 {
    let p = qmul(qmul(q, [0.0, v[0], v[1], v[2]]), qconj(q));
    [p[1], p[2], p[3]]
}
fn qaxis(angle: f64, axis: V3) -> Q {
    let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if n < 1e-12 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let (s, c) = (angle / 2.0).sin_cos();
    [c, axis[0] / n * s, axis[1] / n * s, axis[2] / n * s]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: V3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: V3) -> V3 {
    let l = len(a);
    if l < 1e-12 || !l.is_finite() {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// Кватернион из ортонормированного базиса (столбцы x, y, z).
fn q_from_basis(x: V3, y: V3, z: V3) -> Q {
    let (m00, m01, m02) = (x[0], y[0], z[0]);
    let (m10, m11, m12) = (x[1], y[1], z[1]);
    let (m20, m21, m22) = (x[2], y[2], z[2]);
    let tr = m00 + m11 + m22;
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [0.25 * s, (m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [(m21 - m12) / s, 0.25 * s, (m01 + m10) / s, (m02 + m20) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m02 - m20) / s, (m01 + m10) / s, 0.25 * s, (m12 + m21) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m10 - m01) / s, (m02 + m20) / s, (m12 + m21) / s, 0.25 * s]
    };
    qnorm(q)
}

fn look_rotation(dir: V3) -> Q {
    let z = unit([-dir[0], -dir[1], -dir[2]]);
    if len(z) == 0.0 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let mut x = unit(cross([0.0, 1.0, 0.0], z));
    if len(x) == 0.0 {
        x = [1.0, 0.0, 0.0];
    }
    let y = cross(z, x);
    q_from_basis(x, y, z)
}

fn euler(order: &str, yaw: f64, pitch: f64, roll: f64) -> Q {
    let mut q = [1.0, 0.0, 0.0, 0.0];
    for c in order.chars() {
        let r = match c {
            'x' => qaxis(pitch, [1.0, 0.0, 0.0]),
            'y' => qaxis(yaw, [0.0, 1.0, 0.0]),
            _ => qaxis(roll, [0.0, 0.0, 1.0]),
        };
        // справка: "поворот вокруг осей в порядке Z, Y, X" - последовательно, вокруг неподвижных осей
        q = qmul(r, q);
    }
    qnorm(q)
}

// ---- значения по умолчанию ------------------------------------------------

fn default_of(key: &str) -> Vec<f64> {
    let d: &[f64] = match key {
        "camera.fov" => &[std::f64::consts::FRAC_PI_4],
        "camera.aspectratio" => &[4.0 / 3.0],
        "camera.nearclipdistance" => &[100.0],
        "camera.farclipdistance" => &[100000.0],
        "camera.polygonmode" => &[3.0],
        "camera.projectiontype" => &[1.0],
        "light.powerscale" => &[1.0],
        "light.attenuation" => &[100000.0, 1.0, 0.0, 0.0],
        "light.diffusecolour" | "light.specularcolour" => &[1.0, 1.0, 1.0],
        "movable.visible" | "movable.castshadows" => &[1.0],
        "node.scale" => &[1.0, 1.0, 1.0],
        "node.rot" => &[1.0, 0.0, 0.0, 0.0],
        "pass.ambient" | "pass.diffuse" => &[1.0, 1.0, 1.0, 1.0],
        "pass.specular" | "pass.selfillumination" => &[0.0, 0.0, 0.0, 1.0],
        "pass.lightingenabled" | "pass.depthcheckenabled" | "pass.depthwriteenabled" | "pass.colourwriteenabled" => &[1.0],
        "pass.depthfunction" => &[3.0],
        "pass.cullingmode" => &[2.0],
        "pass.shadingmode" => &[1.0],
        "pass.polygonmode" => &[3.0],
        "pass.sceneblending" => &[0.0, 1.0],
        "pass.alpharejectsettings" => &[1.0, 0.0],
        "overlay.scale" => &[1.0, 1.0],
        "overlayelement.visible" => &[1.0],
        "overlayelement.colour" => &[1.0, 1.0, 1.0, 1.0],
        "billboard.colour" => &[1.0, 1.0, 1.0, 1.0],
        "billboardset.defaultdimensions" => &[100.0, 100.0],
        "billboardset.commonupvector" => &[0.0, 1.0, 0.0],
        "movabletext.characterheight" => &[1.0],
        "movabletext.color" => &[1.0, 1.0, 1.0, 1.0],
        "animationstate.weight" | "animationstate.loop" => &[1.0],
        "viewport.overlaysenabled" => &[1.0],
        _ => &[],
    };
    d.to_vec()
}

const BOOL_PROPS: [&str; 11] = [
    "visible", "enabled", "loop", "castshadows", "lightingenabled", "depthcheckenabled", "depthwriteenabled", "colourwriteenabled",
    "overlaysenabled", "manuallycontrolled", "dynamic",
];
const POSITIVE_PROPS: [&str; 4] = ["camera.fov", "camera.nearclipdistance", "camera.farclipdistance", "camera.aspectratio"];

fn allowed_kinds(prefix: &str) -> Option<&'static [Kind]> {
    use Kind::*;
    Some(match prefix {
        "camera" => &[Camera],
        "light" => &[Light],
        "billboardset" => &[BillboardSet],
        "billboard" => &[Billboard, BillboardSet],
        "animationstate" => &[AnimState],
        "bone" => &[Bone],
        "entity" => &[Entity],
        "manualobject" => &[ManualObject],
        "material" => &[Material],
        "movabletext" => &[MovableText],
        "movable" => &[Camera, Light, Entity, BillboardSet, ManualObject, MovableText, ParticleSystem],
        "node" => &[Node, Bone],
        "scenenode" => &[Node],
        "overlay" => &[Overlay],
        "overlaycontainer" | "overlayelement" => &[Element],
        "particlesystem" => &[ParticleSystem],
        "pass" => &[Pass],
        "technique" => &[Technique],
        "textureunitstate" => &[TextureUnit],
        "viewport" => &[Viewport],
        "renderwindow" => &[Window],
        "rendertexture" => &[Texture],
        "scene" => &[Scene],
        "compositor" => &[Compositor],
        "stringinterface" => &[
            Scene, Camera, Light, Entity, Node, BillboardSet, ManualObject, MovableText, ParticleSystem, Material, Pass, Technique, Overlay,
            Element, Viewport, Window, Texture, Compositor, TextureUnit,
        ],
        _ => return None,
    })
}

fn clean_name(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(MAX_NAME).collect()
}

impl Ogre {
    fn fail(&mut self, m: String) {
        self.errors = self.errors.saturating_add(1);
        if self.errors <= MAX_ERRORS_LOGGED {
            self.pending_log.push(format!("Ogre: {m}"));
        }
    }

    fn get(&self, v: &Value) -> Option<u32> {
        let x = v.as_float();
        if x.is_finite() && x >= 1.0 && x <= u32::MAX as f64 && x.fract() == 0.0 && self.objs.contains_key(&(x as u32)) {
            Some(x as u32)
        } else {
            None
        }
    }

    fn get_kind(&self, v: &Value, kinds: &[Kind]) -> Option<u32> {
        let id = self.get(v)?;
        kinds.contains(&self.objs[&id].kind).then_some(id)
    }

    fn add(&mut self, o: Obj) -> u32 {
        if self.objs.len() >= MAX_OBJECTS {
            self.fail("предел числа объектов".into());
            return 0;
        }
        self.next = self.next.saturating_add(1).max(1);
        let id = self.next;
        self.objs.insert(id, o);
        id
    }

    fn name_taken(&self, scene: u32, kind: Kind, name: &str) -> bool {
        self.objs.values().any(|o| o.kind == kind && o.scene == scene && o.name.eq_ignore_ascii_case(name))
    }

    /// Создаёт объект сцены с уникальным именем (пустое имя - автоматическое).
    fn create_in_scene(&mut self, scene: u32, kind: Kind, name: &str) -> u32 {
        let mut name = clean_name(name);
        if name.is_empty() {
            name = format!("Unnamed_{}", self.next + 1);
        } else if self.name_taken(scene, kind, &name) {
            self.fail(format!("имя '{name}' уже занято"));
            return 0;
        }
        self.add(Obj::new(kind, &name, scene))
    }

    fn pf(&self, id: u32, key: &str) -> Vec<f64> {
        self.objs.get(&id).and_then(|o| o.f.get(key)).cloned().unwrap_or_else(|| default_of(key))
    }

    fn scene_of(&self, id: u32) -> u32 {
        self.objs.get(&id).map(|o| o.scene).unwrap_or(0)
    }

    fn detach(&mut self, id: u32) {
        let Some(p) = self.objs.get(&id).map(|o| o.parent) else { return };
        if p != 0 {
            if let Some(po) = self.objs.get_mut(&p) {
                po.objects.retain(|&x| x != id);
                po.children.retain(|&x| x != id);
            }
        }
        if let Some(o) = self.objs.get_mut(&id) {
            o.parent = 0;
        }
    }

    fn is_ancestor(&self, anc: u32, mut id: u32) -> bool {
        for _ in 0..MAX_DEPTH {
            if id == anc {
                return true;
            }
            match self.objs.get(&id) {
                Some(o) if o.parent != 0 => id = o.parent,
                _ => return false,
            }
        }
        true
    }

    fn set_parent_node(&mut self, child: u32, parent: u32) -> bool {
        if parent != 0 && self.is_ancestor(child, parent) {
            self.fail("цикл в иерархии узлов".into());
            return false;
        }
        self.detach(child);
        if parent != 0 {
            self.objs.get_mut(&child).unwrap().parent = parent;
            self.objs.get_mut(&parent).unwrap().children.push(child);
        }
        true
    }

    fn attach_object(&mut self, node: u32, obj: u32) {
        self.detach(obj);
        self.objs.get_mut(&obj).unwrap().parent = node;
        self.objs.get_mut(&node).unwrap().objects.push(obj);
    }

    fn remove_obj(&mut self, id: u32) {
        self.detach(id);
        let Some(o) = self.objs.remove(&id) else { return };
        if o.kind == Kind::BillboardSet {
            for c in &o.children {
                self.objs.remove(c);
            }
        }
        for c in o.children {
            if let Some(co) = self.objs.get_mut(&c) {
                if o.kind == Kind::Node || o.kind == Kind::Bone {
                    co.parent = 0;
                }
            }
        }
        for m in o.objects {
            if let Some(mo) = self.objs.get_mut(&m) {
                mo.parent = 0;
            }
        }
        for v in self.objs.values_mut() {
            if v.kind == Kind::Viewport && v.f.get("camera").is_some_and(|c| c[0] == id as f64) {
                v.f.remove("camera");
            }
            if v.track == id {
                v.track = 0;
            }
        }
        self.hits.retain(|h| h.1 != id);
        if o.kind == Kind::Material {
            self.materials.retain(|_, v| *v != id);
        }
    }

    fn remove_tree(&mut self, id: u32) {
        let kids: Vec<u32> = self.objs.get(&id).map(|o| o.children.clone()).unwrap_or_default();
        for k in kids {
            self.remove_tree(k);
        }
        self.remove_obj(id);
    }

    fn remove_scene(&mut self, scene: u32, keep_scene: bool) {
        let ids: Vec<u32> = self.objs.iter().filter(|(_, o)| o.scene == scene).map(|(i, _)| *i).collect();
        let root = self.root_node(scene);
        for id in ids {
            if keep_scene && id == root {
                continue;
            }
            self.remove_obj(id);
        }
        if keep_scene {
            if let Some(r) = self.objs.get_mut(&root) {
                r.children.clear();
                r.objects.clear();
            }
        } else {
            self.remove_obj(scene);
        }
    }

    fn root_node(&self, scene: u32) -> u32 {
        self.objs.get(&scene).and_then(|o| o.f.get("rootnode")).map(|v| v[0] as u32).unwrap_or(0)
    }

    // ---- преобразования -------------------------------------------------

    /// Положение, поворот и масштаб узла в мировых координатах.
    fn derived(&self, id: u32) -> (V3, Q, V3) {
        self.derived_inner(id, true)
    }

    fn derived_inner(&self, id: u32, tracking: bool) -> (V3, Q, V3) {
        let mut chain = Vec::new();
        let mut cur = id;
        for _ in 0..MAX_DEPTH {
            let Some(o) = self.objs.get(&cur) else { break };
            if !matches!(o.kind, Kind::Node | Kind::Bone) {
                break;
            }
            chain.push(cur);
            if o.parent == 0 {
                break;
            }
            cur = o.parent;
        }
        let (mut p, mut r, mut s) = ([0.0; 3], [1.0, 0.0, 0.0, 0.0], [1.0; 3]);
        for &n in chain.iter().rev() {
            let o = &self.objs[&n];
            let lp = o.f.get("node.position").map(|v| [v[0], v[1], v[2]]).unwrap_or([0.0; 3]);
            let lr = o.f.get("node.rot").map(|v| [v[0], v[1], v[2], v[3]]).unwrap_or([1.0, 0.0, 0.0, 0.0]);
            let ls = o.f.get("node.scale").map(|v| [v[0], v[1], v[2]]).unwrap_or([1.0; 3]);
            let sp = [s[0] * lp[0], s[1] * lp[1], s[2] * lp[2]];
            let rp = qrot(r, sp);
            p = [p[0] + rp[0], p[1] + rp[1], p[2] + rp[2]];
            r = qnorm(qmul(r, lr));
            s = [s[0] * ls[0], s[1] * ls[1], s[2] * ls[2]];
            if tracking && o.track != 0 && o.track != n && self.objs.contains_key(&o.track) {
                let (tp, _, _) = self.derived_inner(o.track, false);
                r = look_rotation(sub([tp[0] + o.track_off[0], tp[1] + o.track_off[1], tp[2] + o.track_off[2]], p));
            }
        }
        (p, r, s)
    }

    fn set_rot(&mut self, id: u32, q: Q) {
        self.objs.get_mut(&id).unwrap().f.insert("node.rot".into(), qnorm(q).to_vec());
    }

    // ---- ресурсы --------------------------------------------------------

    fn find_resource(&mut self, fx: &mut Effects, name: &str) -> Option<PathBuf> {
        if name.is_empty() || name.len() > 1024 {
            return None;
        }
        let mut cands = vec![name.to_string()];
        for (l, _) in self.locations.clone() {
            cands.push(format!("{}/{}", l.trim_end_matches(['/', '\\']), name));
        }
        for c in cands {
            if let Some(p) = super::extra::sandboxed(fx, &c, false) {
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        let want = name.rsplit(['/', '\\']).next().unwrap_or(name).to_ascii_lowercase();
        let mut budget = 4000usize;
        for (l, rec) in self.locations.clone() {
            if !rec {
                continue;
            }
            let Some(dir) = super::extra::sandboxed(fx, &l, false) else { continue };
            if let Some(p) = walk_find(&dir, &want, 8, &mut budget) {
                return Some(p);
            }
        }
        None
    }

    fn get_zip(&mut self, fx: &mut Effects, loc: &str) -> Option<Arc<zip::Zip>> {
        let key = loc.to_ascii_lowercase();
        if let Some(z) = self.zips.get(&key) {
            return z.clone();
        }
        let total: u64 = self.zip_bytes;
        let z = (|| {
            let p = super::extra::sandboxed(fx, loc, false)?;
            let len = std::fs::metadata(&p).ok()?.len();
            if len > zip::MAX_ARCHIVE || total + len > 2 * zip::MAX_ARCHIVE {
                return None;
            }
            self.zip_bytes += len;
            zip::Zip::open(std::fs::read(p).ok()?).map(Arc::new)
        })();
        if z.is_none() && self.zips.len() < MAX_LOCATIONS {
            self.pending_log.push(format!("Ogre: архив '{loc}' не прочитан"));
        }
        if self.zips.len() < MAX_LOCATIONS {
            self.zips.insert(key, z.clone());
        }
        z
    }

    fn zip_locations(&self) -> Vec<String> {
        self.locations.iter().filter(|(l, _)| l.to_ascii_lowercase().ends_with(".zip")).map(|(l, _)| l.clone()).collect()
    }

    fn read_file(&mut self, fx: &mut Effects, name: &str) -> Option<Vec<u8>> {
        let Some(p) = self.find_resource(fx, name) else {
            for l in self.zip_locations() {
                if let Some(d) = self.get_zip(fx, &l).and_then(|z| z.read(name)) {
                    return Some(d);
                }
            }
            return None;
        };
        if std::fs::metadata(&p).ok()?.len() > MAX_FILE {
            self.fail(format!("файл '{name}' слишком велик"));
            return None;
        }
        std::fs::read(p).ok()
    }

    fn load_mesh(&mut self, fx: &mut Effects, name: &str) -> Option<Arc<Mesh>> {
        let key = name.to_ascii_lowercase();
        if let Some(m) = self.meshes.get(&key) {
            return Some(m.clone());
        }
        let data = self.read_file(fx, name)?;
        let mesh = Arc::new(parse_mesh(&data).unwrap_or_else(|| Mesh { subs: vec![SubMesh::default()], ..Default::default() }));
        self.meshes.insert(key, mesh.clone());
        Some(mesh)
    }

    fn load_skeleton(&mut self, fx: &mut Effects, name: &str) -> Option<Arc<Skeleton>> {
        let key = name.to_ascii_lowercase();
        if let Some(s) = self.skeletons.get(&key) {
            return Some(s.clone());
        }
        let data = self.read_file(fx, name)?;
        let sk = Arc::new(parse_skeleton(&data)?);
        self.skeletons.insert(key, sk.clone());
        Some(sk)
    }

    fn scan_resources(&mut self, fx: &mut Effects) {
        for l in self.zip_locations() {
            let Some(z) = self.get_zip(fx, &l) else { continue };
            for n in z.names() {
                let ext = n.rsplit('.').next().unwrap_or("");
                if matches!(ext, "material" | "compositor" | "particle" | "overlay") {
                    if let Some(d) = z.read(&n) {
                        self.ingest_script(ext, &String::from_utf8_lossy(&d));
                    }
                }
            }
        }
        let mut files = 0usize;
        for (l, rec) in self.locations.clone() {
            let Some(dir) = super::extra::sandboxed(fx, &l, false) else { continue };
            let mut found = Vec::new();
            collect_files(&dir, rec, 6, &mut found, &mut files);
            for p in found {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
                if !matches!(ext.as_str(), "material" | "compositor" | "particle" | "overlay") {
                    continue;
                }
                let Ok(md) = std::fs::metadata(&p) else { continue };
                if md.len() > (4 << 20) {
                    continue;
                }
                let Ok(text) = std::fs::read(&p) else { continue };
                self.ingest_script(&ext, &String::from_utf8_lossy(&text));
            }
        }
    }

    fn ingest_script(&mut self, ext: &str, text: &str) {
        match ext {
            "material" => self.parse_materials(text),
            "compositor" => {
                let toks = tokenize(text);
                for w in toks.windows(2) {
                    if w[0] == "compositor" && self.compositors.len() < MAX_OBJECTS {
                        self.compositors.insert(w[1].to_ascii_lowercase());
                    }
                }
            }
            "overlay" => self.parse_overlays(text),
            _ => self.parse_particles(text),
        }
    }

    fn new_material(&mut self, name: &str) -> u32 {
        let id = self.add(Obj::new(Kind::Material, name, 0));
        if id == 0 {
            return 0;
        }
        self.materials.insert(name.to_ascii_lowercase(), id);
        let t = self.add_child(id, Kind::Technique, "");
        if t != 0 {
            self.add_child(t, Kind::Pass, "");
        }
        id
    }

    fn add_child(&mut self, parent: u32, kind: Kind, name: &str) -> u32 {
        let id = self.add(Obj::new(kind, &clean_name(name), 0));
        if id != 0 {
            self.objs.get_mut(&id).unwrap().parent = parent;
            self.objs.get_mut(&parent).unwrap().children.push(id);
        }
        id
    }

    fn parse_materials(&mut self, text: &str) {
        let t = tokenize(text);
        let mut i = 0;
        while i < t.len() {
            if t[i] == "material" && i + 1 < t.len() {
                let name = t[i + 1].clone();
                let mut j = i + 2;
                while j < t.len() && t[j] != "{" {
                    j += 1;
                }
                if j >= t.len() {
                    break;
                }
                if self.materials.contains_key(&name.to_ascii_lowercase()) || self.objs.len() > MAX_OBJECTS - 64 {
                    i = skip_block(&t, j);
                    continue;
                }
                let m = self.new_material(&name);
                let end = skip_block(&t, j);
                if m != 0 {
                    self.fill_material(m, &t[j + 1..end.saturating_sub(1).max(j + 1)]);
                }
                i = end;
            } else {
                i += 1;
            }
        }
    }

    fn fill_material(&mut self, mat: u32, t: &[String]) {
        // сбросить техники по умолчанию, если в скрипте есть своя
        let mut tech = 0u32;
        let mut pass = 0u32;
        let mut tu = 0u32;
        let mut depth = 0usize;
        let mut first = true;
        let mut stack: Vec<&str> = Vec::new();
        let mut i = 0;
        while i < t.len() {
            let w = t[i].as_str();
            match w {
                "{" => {
                    depth += 1;
                    i += 1;
                    continue;
                }
                "}" => {
                    depth = depth.saturating_sub(1);
                    stack.pop();
                    i += 1;
                    continue;
                }
                _ => {}
            }
            let args: Vec<&str> = {
                let mut v = Vec::new();
                let mut j = i + 1;
                while j < t.len() && t[j] != "{" && t[j] != "}" && v.len() < 8 && !is_keyword(&t[j]) {
                    v.push(t[j].as_str());
                    j += 1;
                }
                v
            };
            let nums: Vec<f64> = args.iter().filter_map(|a| a.parse::<f64>().ok()).collect();
            match w {
                "technique" => {
                    if first {
                        let old: Vec<u32> = self.objs[&mat].children.clone();
                        for o in old {
                            self.remove_tree(o);
                        }
                        first = false;
                    }
                    tech = self.add_child(mat, Kind::Technique, args.first().copied().unwrap_or(""));
                    pass = 0;
                    tu = 0;
                    stack.push("technique");
                }
                "pass" if tech != 0 => {
                    pass = self.add_child(tech, Kind::Pass, args.first().copied().unwrap_or(""));
                    tu = 0;
                    stack.push("pass");
                }
                "texture_unit" if pass != 0 => {
                    tu = self.add_child(pass, Kind::TextureUnit, args.first().copied().unwrap_or(""));
                    stack.push("tu");
                }
                _ if pass != 0 => {
                    let o = self.objs.get_mut(&pass).unwrap();
                    let col = |n: &[f64], d: f64| -> Vec<f64> { vec![n[0], n.get(1).copied().unwrap_or(0.0), n.get(2).copied().unwrap_or(0.0), n.get(3).copied().unwrap_or(d)] };
                    let onoff = |a: &[&str]| a.first().is_some_and(|x| *x == "on");
                    match w {
                        "ambient" if nums.len() >= 3 => {
                            o.f.insert("pass.ambient".into(), col(&nums, 1.0));
                        }
                        "diffuse" if nums.len() >= 3 => {
                            o.f.insert("pass.diffuse".into(), col(&nums, 1.0));
                        }
                        "specular" if nums.len() >= 3 => {
                            let n = col(&nums, 1.0);
                            o.f.insert("pass.specular".into(), n);
                            if nums.len() >= 5 {
                                o.f.insert("pass.shininess".into(), vec![nums[4]]);
                            } else if nums.len() == 4 {
                                o.f.insert("pass.shininess".into(), vec![nums[3]]);
                            }
                        }
                        "emissive" if nums.len() >= 3 => {
                            o.f.insert("pass.selfillumination".into(), col(&nums, 1.0));
                        }
                        "lighting" => {
                            o.f.insert("pass.lightingenabled".into(), vec![bl(onoff(&args)).as_float()]);
                        }
                        "depth_check" => {
                            o.f.insert("pass.depthcheckenabled".into(), vec![bl(onoff(&args)).as_float()]);
                        }
                        "depth_write" => {
                            o.f.insert("pass.depthwriteenabled".into(), vec![bl(onoff(&args)).as_float()]);
                        }
                        "cull_hardware" => {
                            let v = match args.first().copied() {
                                Some("none") => 1.0,
                                Some("anticlockwise") => 3.0,
                                _ => 2.0,
                            };
                            o.f.insert("pass.cullingmode".into(), vec![v]);
                        }
                        "scene_blend" => {
                            let v = match args.first().copied() {
                                Some("add") => vec![0.0, 0.0],
                                Some("modulate") => vec![2.0, 1.0],
                                Some("alpha_blend") => vec![7.0, 9.0],
                                Some("colour_blend") => vec![3.0, 5.0],
                                _ => vec![0.0, 1.0],
                            };
                            o.f.insert("pass.sceneblending".into(), v);
                        }
                        "cubic_texture" if tu != 0 => {
                            let names: Vec<String> = args.iter().filter(|a| a.contains('.')).map(|a| a.to_string()).collect();
                            let faces: Vec<String> = if names.len() == 1 {
                                let (stem, ext) = names[0].rsplit_once('.').unwrap_or((&names[0], ""));
                                ["fr", "bk", "lf", "rt", "up", "dn"].iter().map(|s| format!("{stem}_{s}.{ext}")).collect()
                            } else {
                                names.into_iter().take(6).collect()
                            };
                            if faces.len() == 6 {
                                self.objs.get_mut(&tu).unwrap().st.insert("textureunitstate.cubic".into(), faces.join("|"));
                                for f in &faces {
                                    self.want_texture(f);
                                }
                            }
                        }
                        "texture" if tu != 0 => {
                            if let Some(a) = args.first() {
                                self.objs.get_mut(&tu).unwrap().st.insert("textureunitstate.texture".into(), a.to_string());
                                self.want_texture(a);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            let _ = depth;
            i += 1;
        }
    }

    // ---- основной разбор ---------------------------------------------------

    fn run(&mut self, fx: &mut Effects, name: &str, sig: &Sig, a: &[Value], outs: &mut Vec<(usize, Value)>) -> Value {
        let s = |i: usize| a.get(i).map(|v| v.as_string()).unwrap_or_default();
        let n = |i: usize| a.get(i).map(|v| v.as_float()).unwrap_or(0.0);
        // нечисловой ввод не должен попадать в состояние
        for (i, (c, out)) in sig.params.iter().enumerate() {
            if *c == 'F' && !out && !n(i).is_finite() {
                self.fail(format!("{name}: аргумент {} не число", i + 1));
                return zero(sig);
            }
        }
        let (prefix, verb) = name.split_once('_').unwrap_or((name, ""));
        // проверка дескриптора первого аргумента
        let first_handle = sig.params.first().is_some_and(|p| p.0 == 'H');
        let id = if first_handle && prefix != "collision" && !matches!(name, "rendertexture_createex") {
            match allowed_kinds(prefix).and_then(|k| self.get_kind(&a[0], k)) {
                Some(i) => i,
                None => {
                    // создающие функции принимают сцену/узел: проверяются ниже
                    if !is_creator(name) {
                        self.fail(format!("{name}: неверный дескриптор"));
                        return zero(sig);
                    }
                    0
                }
            }
        } else {
            0
        };
        let out_f = |outs: &mut Vec<(usize, Value)>, start: usize, vals: &[f64]| {
            for (k, v) in vals.iter().enumerate() {
                outs.push((start + k, Value::Float(*v)));
            }
        };

        match name {
            // ---- Root -------------------------------------------------
            "root_create" => {
                self.root_created = true;
                self.initialised = false;
                self.config = s(1);
                self.started = Some(std::time::Instant::now());
                fl(0.0)
            }
            "root_destroy" => {
                let (errs, frames) = (self.errors, self.frames);
                *self = Ogre::default();
                self.errors = errs;
                self.frames = frames;
                fl(0.0)
            }
            "root_initialise" => {
                if self.root_created && !self.initialised {
                    self.initialised = true;
                } else if !self.root_created {
                    self.fail("Root_Initialise до Root_Create".into());
                }
                fl(0.0)
            }
            "root_isinitialised" => bl(self.initialised),
            "root_gettime" => fl(self.started.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)),
            "root_showconfigdialog" => bl(self.root_created),
            "root_restoreconfig" => {
                let ok = self.root_created && (self.config_saved || {
                    let c = self.config.clone();
                    !c.is_empty() && super::extra::sandboxed(fx, &c, false).is_some_and(|p| p.is_file())
                });
                bl(ok)
            }
            "root_saveconfig" => {
                self.config_saved = self.root_created;
                fl(0.0)
            }
            "root_addresourcelocation" => {
                if self.locations.len() >= MAX_LOCATIONS {
                    self.fail("слишком много ресурсных путей".into());
                } else {
                    self.locations.push((clean_name(&s(0)), n(3) != 0.0));
                }
                fl(0.0)
            }
            "root_addresourcelocationfromconfigfile" => {
                if let Some(p) = super::extra::sandboxed(fx, &s(0), false) {
                    if std::fs::metadata(&p).map(|m| m.len() < (1 << 20)).unwrap_or(false) {
                        if let Ok(text) = std::fs::read(&p) {
                            for line in String::from_utf8_lossy(&text).lines() {
                                let line = line.trim();
                                if let Some((k, v)) = line.split_once('=') {
                                    if (k.trim().eq_ignore_ascii_case("FileSystem") || k.trim().eq_ignore_ascii_case("Zip")) && self.locations.len() < MAX_LOCATIONS {
                                        self.locations.push((clean_name(v.trim()), true));
                                    }
                                }
                            }
                        }
                    }
                }
                fl(0.0)
            }
            "root_initialiseallresourcegroups" => {
                self.scan_resources(fx);
                fl(0.0)
            }
            "root_renderoneframe" => {
                if self.initialised && self.objs.values().any(|o| o.kind == Kind::Window) {
                    self.frames += 1;
                    // шаг частиц фиксированный (30 кадров в секунду): ход детерминирован
                    self.step_particles(1.0 / 30.0);
                    fl(1.0)
                } else {
                    fl(0.0)
                }
            }
            // ---- Scene -----------------------------------------------------
            "scene_create" => {
                let t = s(0).to_ascii_lowercase();
                if !self.root_created {
                    self.fail("Scene_Create до Root_Create".into());
                    return h(0);
                }
                if !t.is_empty() && !matches!(t.as_str(), "octreescenemanager" | "bspscenemanager" | "terrainscenemanager" | "defaultscenemanager") {
                    self.fail(format!("неизвестный тип сцены '{t}'"));
                    return h(0);
                }
                let sc = self.add(Obj::new(Kind::Scene, &t, 0));
                if sc == 0 {
                    return h(0);
                }
                let rn = self.add(Obj::new(Kind::Node, "Ogre/SceneRoot", sc));
                if rn == 0 {
                    self.objs.remove(&sc);
                    return h(0);
                }
                self.objs.get_mut(&rn).unwrap().scene = sc;
                self.objs.get_mut(&sc).unwrap().f.insert("rootnode".into(), vec![rn as f64]);
                h(sc)
            }
            "scene_destroy" => {
                self.remove_scene(id, false);
                fl(0.0)
            }
            "scene_clear" => {
                self.remove_scene(id, true);
                fl(0.0)
            }
            "scene_getrootscenenode" => h(self.root_node(id)),
            "scene_getcamera" | "scene_getentity" | "scene_getlight" | "scene_getparticlesystem" | "scene_getscenenode" => {
                let kind = match name {
                    "scene_getcamera" => Kind::Camera,
                    "scene_getentity" => Kind::Entity,
                    "scene_getlight" => Kind::Light,
                    "scene_getparticlesystem" => Kind::ParticleSystem,
                    _ => Kind::Node,
                };
                let want = s(1);
                h(self.objs.iter().find(|(_, o)| o.kind == kind && o.scene == id && o.name.eq_ignore_ascii_case(&want)).map(|(i, _)| *i).unwrap_or(0))
            }
            "scene_setambientlight" => {
                self.objs.get_mut(&id).unwrap().f.insert("scene.ambientlight".into(), vec![n(1), n(2), n(3)]);
                fl(0.0)
            }
            "scene_setfog" => {
                let v: Vec<f64> = (1..8).map(n).collect();
                self.objs.get_mut(&id).unwrap().f.insert("scene.fog".into(), v);
                fl(0.0)
            }
            "scene_setskybox" => {
                let o = self.objs.get_mut(&id).unwrap();
                o.f.insert("scene.skybox".into(), vec![n(1), n(3)]);
                o.st.insert("scene.skybox".into(), clean_name(&s(2)));
                fl(0.0)
            }
            "scene_setworldgeometry" => {
                self.objs.get_mut(&id).unwrap().st.insert("scene.worldgeometry".into(), clean_name(&s(1)));
                fl(0.0)
            }
            "scene_load" => {
                let file = s(1);
                self.objs.get_mut(&id).unwrap().st.insert("scene.loaded".into(), clean_name(&file));
                self.load_scene(fx, id, &file);
                fl(0.0)
            }
            // ---- создание объектов сцены ------------------------------------
            "camera_create" | "light_create" | "entity_create" | "scenenode_create" | "billboardset_create" | "manualobject_create"
            | "particlesystem_create" | "particlesystem_createex" => {
                let Some(sc) = self.get_kind(&a[0], &[Kind::Scene]) else {
                    self.fail(format!("{name}: нужна сцена"));
                    return h(0);
                };
                let kind = match prefix {
                    "camera" => Kind::Camera,
                    "light" => Kind::Light,
                    "entity" => Kind::Entity,
                    "scenenode" => Kind::Node,
                    "billboardset" => Kind::BillboardSet,
                    "manualobject" => Kind::ManualObject,
                    _ => Kind::ParticleSystem,
                };
                if kind == Kind::ParticleSystem && !self.ptemplates.is_empty() && !self.ptemplates.contains_key(&s(2).to_ascii_lowercase()) {
                    self.fail(format!("ParticleSystem_Create: шаблон '{}' не найден", s(2)));
                    return h(0);
                }
                if kind == Kind::BillboardSet {
                    let pool = n(2);
                    if !(1.0..=100_000.0).contains(&pool) {
                        self.fail("BillboardSet_Create: размер пула 1..100000".into());
                        return h(0);
                    }
                }
                let mesh = if kind == Kind::Entity {
                    match self.load_mesh(fx, &s(2)) {
                        Some(m) => Some(m),
                        None => {
                            self.fail(format!("Entity_Create: сетка '{}' не найдена", s(2)));
                            return h(0);
                        }
                    }
                } else {
                    None
                };
                let skel = match mesh.as_ref().and_then(|m| m.skeleton.clone()) {
                    Some(k) => self.load_skeleton(fx, &k),
                    None => None,
                };
                let nid = self.create_in_scene(sc, kind, &s(1));
                if nid == 0 {
                    return h(0);
                }
                let o = self.objs.get_mut(&nid).unwrap();
                if let Some(m) = mesh {
                    o.sub_mat = m.subs.iter().map(|x| x.material.clone()).collect();
                    o.st.insert("entity.mesh".into(), clean_name(&s(2)));
                    o.mesh = Some(m);
                }
                o.skel = skel;
                if kind == Kind::ParticleSystem {
                    o.st.insert("particlesystem.template".into(), clean_name(&s(2)));
                }
                if kind == Kind::BillboardSet {
                    o.f.insert("billboardset.pool".into(), vec![n(2).floor()]);
                }
                if kind == Kind::ManualObject {
                    o.manual = Some(Manual { col: [1.0; 4], ..Default::default() });
                }
                h(nid)
            }
            "camera_destroy" | "light_destroy" | "entity_destroy" | "billboardset_destroy" | "manualobject_destroy" | "particlesystem_destroy"
            | "scenenode_destroy" | "movabletext_destroy" | "rendertexture_destroy" => {
                if name == "scenenode_destroy" && self.objs[&id].parent == 0 && self.root_node(self.objs[&id].scene) == id {
                    self.fail("корневой узел сцены нельзя удалить".into());
                    return fl(0.0);
                }
                if let Some(o) = self.objs.get(&id) {
                    if o.kind == Kind::MovableText {
                        let k = o.name.to_ascii_lowercase();
                        self.texts.remove(&k);
                    }
                }
                self.remove_obj(id);
                fl(0.0)
            }
            "movabletext_create" => {
                let mut nm = clean_name(&s(0));
                if nm.is_empty() {
                    let mut n = self.texts.len() + 1;
                    while self.texts.contains_key(&format!("movabletext{n}")) {
                        n += 1;
                    }
                    nm = format!("MovableText{n}");
                }
                if self.texts.contains_key(&nm.to_ascii_lowercase()) {
                    self.fail("MovableText_Create: имя занято".into());
                    return h(0);
                }
                let nid = self.add(Obj::new(Kind::MovableText, &nm, 0));
                if nid != 0 {
                    self.texts.insert(nm.to_ascii_lowercase(), nid);
                    let o = self.objs.get_mut(&nid).unwrap();
                    o.st.insert("movabletext.caption".into(), s(1).chars().take(MAX_TEXT).collect());
                    o.st.insert("movabletext.fontname".into(), clean_name(&s(2)));
                }
                h(nid)
            }
            // ---- узлы -------------------------------------------------------
            "node_addchild" => {
                let Some(c) = self.get_kind(&a[1], &[Kind::Node, Kind::Bone]) else {
                    self.fail("Node_AddChild: неверный потомок".into());
                    return fl(0.0);
                };
                if c == id {
                    self.fail("Node_AddChild: узел в самом себе".into());
                    return fl(0.0);
                }
                self.set_parent_node(c, id);
                fl(0.0)
            }
            "node_removechild" => {
                if let Some(c) = self.get_kind(&a[1], &[Kind::Node, Kind::Bone]) {
                    if self.objs[&c].parent == id {
                        self.detach(c);
                    }
                }
                fl(0.0)
            }
            "node_setparent" => {
                let p = if n(1) == 0.0 { 0 } else { self.get_kind(&a[1], &[Kind::Node, Kind::Bone]).unwrap_or(u32::MAX) };
                if p == u32::MAX || p == id {
                    self.fail("Node_SetParent: неверный родитель".into());
                } else {
                    self.set_parent_node(id, p);
                }
                fl(0.0)
            }
            "node_getparent" => h(self.objs[&id].parent),
            "node_getnumchildren" => fl(self.objs[&id].children.len() as f64),
            "node_getchild" => {
                let i = n(1);
                h(if i >= 0.0 { self.objs[&id].children.get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            "node_getname" => {
                outs.push((1, Value::Str(self.objs[&id].name.clone())));
                fl(0.0)
            }
            "node_getderivedposition" => {
                out_f(outs, 1, &self.derived(id).0);
                fl(0.0)
            }
            "node_getderivedscale" => {
                out_f(outs, 1, &self.derived(id).2);
                fl(0.0)
            }
            "node_getderivedrotationquaternion" => {
                out_f(outs, 1, &self.derived(id).1);
                fl(0.0)
            }
            "node_getrotationquaternion" => {
                out_f(outs, 1, &self.pf(id, "node.rot"));
                fl(0.0)
            }
            "node_setrotationquaternion" => {
                self.set_rot(id, [n(1), n(2), n(3), n(4)]);
                fl(0.0)
            }
            "node_setrotationaxis" => {
                self.set_rot(id, qaxis(n(4), [n(1), n(2), n(3)]));
                fl(0.0)
            }
            "node_setrotationeulerxyz" | "node_setrotationeulerxzy" | "node_setrotationeuleryxz" | "node_setrotationeuleryzx"
            | "node_setrotationeulerzxy" | "node_setrotationeulerzyx" => {
                let order = &name[name.len() - 3..];
                self.set_rot(id, euler(order, n(1), n(2), n(3)));
                fl(0.0)
            }
            "scenenode_attachobject" => {
                let Some(o) = self.get_kind(&a[1], &[Kind::Camera, Kind::Light, Kind::Entity, Kind::BillboardSet, Kind::ManualObject, Kind::MovableText, Kind::ParticleSystem]) else {
                    self.fail("SceneNode_AttachObject: неверный объект".into());
                    return fl(0.0);
                };
                self.attach_object(id, o);
                fl(0.0)
            }
            "scenenode_detachobject" => {
                if let Some(o) = self.get(&a[1]) {
                    if self.objs[&o].parent == id && self.objs[&o].kind.is_movable() {
                        self.detach(o);
                    }
                }
                fl(0.0)
            }
            "scenenode_getnumobjects" => fl(self.objs[&id].objects.len() as f64),
            "scenenode_getobject" => {
                let i = n(1);
                h(if i >= 0.0 { self.objs[&id].objects.get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            "scenenode_setautotracking" => {
                let t = if n(1) == 0.0 { 0 } else { self.get_kind(&a[1], &[Kind::Node]).unwrap_or(0) };
                let o = self.objs.get_mut(&id).unwrap();
                o.track = if t == id { 0 } else { t };
                o.track_off = [n(2), n(3), n(4)];
                fl(0.0)
            }
            "scenenode_setvisible" => {
                let v = bl(n(1) != 0.0).as_float();
                let cascade = n(2) != 0.0;
                let mut stack = vec![id];
                let mut guard = 0;
                while let Some(x) = stack.pop() {
                    guard += 1;
                    if guard > MAX_OBJECTS {
                        break;
                    }
                    let (objs, kids) = {
                        let o = &self.objs[&x];
                        (o.objects.clone(), o.children.clone())
                    };
                    for m in objs {
                        self.objs.get_mut(&m).unwrap().f.insert("movable.visible".into(), vec![v]);
                    }
                    if cascade {
                        stack.extend(kids);
                    }
                }
                fl(0.0)
            }
            // ---- перемещаемые объекты -----------------------------------------
            "movable_getname" | "movabletext_getname" | "overlay_getname" | "material_getname" | "particlesystem_getname" => {
                let idx = 1;
                outs.push((idx, Value::Str(self.objs[&id].name.clone())));
                fl(0.0)
            }
            "movable_getparent" | "particlesystem_getparent" => h(self.objs[&id].parent),
            "movable_setparent" | "particlesystem_setparent" => {
                if n(1) == 0.0 {
                    self.detach(id);
                } else if let Some(nd) = self.get_kind(&a[1], &[Kind::Node]) {
                    self.attach_object(nd, id);
                } else {
                    self.fail(format!("{name}: неверный узел"));
                }
                fl(0.0)
            }
            "movable_getboundingbox" => {
                let b = self.bounds(id);
                out_f(outs, 1, &b);
                fl(0.0)
            }
            // ---- сущность --------------------------------------------------------
            "entity_getsubentitycount" => fl(self.objs[&id].mesh.as_ref().map(|m| m.subs.len()).unwrap_or(0) as f64),
            "entity_getvertexcount" | "entity_getindexcount" => {
                let i = n(1);
                let r = self.objs[&id].mesh.as_ref().and_then(|m| if i >= 0.0 { m.subs.get(i as usize) } else { None });
                fl(r.map(|x| if name == "entity_getvertexcount" { x.verts.len() } else { x.idx.len() }).unwrap_or(0) as f64)
            }
            "entity_setmaterial" => {
                let i = n(1);
                let nm = s(2);
                let cnt = self.objs[&id].sub_mat.len();
                if i < 0.0 || i as usize >= cnt {
                    self.fail("Entity_SetMaterial: нет такой части".into());
                } else if !self.materials.contains_key(&nm.to_ascii_lowercase()) {
                    self.fail(format!("Entity_SetMaterial: материал '{nm}' не найден"));
                } else {
                    self.objs.get_mut(&id).unwrap().sub_mat[i as usize] = clean_name(&nm);
                }
                fl(0.0)
            }
            "entity_getanimationstate" | "entity_getbone" => {
                let Some(sk) = self.objs[&id].skel.clone() else { return h(0) };
                let want = s(1);
                let (kind, exists, len) = if name == "entity_getbone" {
                    (Kind::Bone, sk.bones.iter().any(|b| b.name == want), 0.0)
                } else {
                    let l = sk.anims.iter().find(|x| x.name == want).map(|x| x.length);
                    (Kind::AnimState, l.is_some(), l.unwrap_or(0.0))
                };
                if !exists {
                    return h(0);
                }
                if let Some((i, _)) = self.objs.iter().find(|(_, o)| o.kind == kind && o.parent == id && o.name == want) {
                    return h(*i);
                }
                let mut o = Obj::new(kind, &want, self.scene_of(id));
                o.parent = id;
                o.skel = Some(sk.clone());
                if kind == Kind::AnimState {
                    o.f.insert("animationstate.length".into(), vec![len]);
                }
                let nid = self.add(o);
                h(nid)
            }
            "bone_reset" => {
                let o = self.objs.get_mut(&id).unwrap();
                o.f.remove("node.position");
                o.f.remove("node.rot");
                o.f.remove("node.scale");
                fl(0.0)
            }
            // ---- анимация ------------------------------------------------------------
            "animationstate_addtime" => {
                let t = self.pf(id, "animationstate.timeposition").first().copied().unwrap_or(0.0) + n(1);
                self.set_anim_time(id, t);
                fl(0.0)
            }
            "animationstate_settimeposition" => {
                self.set_anim_time(id, n(1));
                fl(0.0)
            }
            // ---- ManualObject -------------------------------------------------------
            "manualobject_begin" => {
                let mat = s(1);
                let op = n(2);
                let m = self.objs.get_mut(&id).unwrap().manual.get_or_insert_with(Manual::default);
                if m.cur.is_some() || !(1.0..=6.0).contains(&op) {
                    self.fail("ManualObject_Begin: секция уже начата или неверный тип".into());
                } else {
                    m.cur = Some(Section { material: clean_name(&mat), op: op as u16, ..Default::default() });
                    m.normal = [0.0; 3];
                    m.uv = [0.0; 2];
                    m.col = [1.0; 4];
                }
                fl(0.0)
            }
            "manualobject_position" | "manualobject_normal" | "manualobject_texturecoord" | "manualobject_colour" | "manualobject_index" => {
                let (x, y, z, w) = (n(1) as f32, n(2) as f32, n(3) as f32, n(4) as f32);
                let m = self.objs.get_mut(&id).unwrap().manual.get_or_insert_with(Manual::default);
                let Some(cur) = m.cur.as_mut() else {
                    self.fail(format!("{name}: вне ManualObject_Begin/End"));
                    return fl(0.0);
                };
                match name {
                    "manualobject_position" => {
                        if cur.verts.len() >= MAX_VERTS {
                            self.fail("ManualObject: предел вершин".into());
                        } else {
                            cur.verts.push([x, y, z]);
                            cur.norms.push(m.normal);
                            cur.uvs.push(m.uv);
                            cur.cols.push(m.col);
                        }
                    }
                    "manualobject_normal" => m.normal = [x, y, z],
                    "manualobject_texturecoord" => m.uv = [x, y],
                    "manualobject_colour" => m.col = [x, y, z, w],
                    _ => {
                        let i = n(1);
                        if i < 0.0 || i > u32::MAX as f64 || cur.idx.len() >= MAX_INDICES {
                            self.fail("ManualObject_Index: неверный индекс или предел".into());
                        } else {
                            cur.idx.push(i as u32);
                        }
                    }
                }
                fl(0.0)
            }
            "manualobject_end" => {
                let m = self.objs.get_mut(&id).unwrap().manual.get_or_insert_with(Manual::default);
                let Some(mut cur) = m.cur.take() else {
                    self.fail("ManualObject_End без Begin".into());
                    return fl(0.0);
                };
                if cur.idx.is_empty() {
                    cur.idx = (0..cur.verts.len() as u32).collect();
                }
                if cur.idx.iter().any(|&i| i as usize >= cur.verts.len()) {
                    self.fail("ManualObject_End: индекс за пределами вершин, секция отброшена".into());
                } else {
                    m.sections.push(cur);
                }
                fl(0.0)
            }
            "manualobject_clear" => {
                let m = self.objs.get_mut(&id).unwrap().manual.get_or_insert_with(Manual::default);
                m.sections.clear();
                m.cur = None;
                fl(0.0)
            }
            "manualobject_estimatevertexcount" | "manualobject_estimateindexcount" => {
                let v = n(1);
                if !(0.0..=MAX_INDICES as f64).contains(&v) {
                    self.fail(format!("{name}: оценка вне 0..{MAX_INDICES}"));
                }
                fl(0.0)
            }
            "manualobject_converttomesh" => {
                let key = clean_name(&s(1)).to_ascii_lowercase();
                let m = self.objs[&id].manual.clone().unwrap_or_default();
                if key.is_empty() || self.meshes.contains_key(&key) || m.cur.is_some() || m.sections.is_empty() {
                    self.fail("ManualObject_ConvertToMesh: имя занято, секция не закончена или объект пуст".into());
                    return fl(0.0);
                }
                let mut mesh = Mesh { min: [f32::MAX; 3], max: [f32::MIN; 3], ..Default::default() };
                for sec in &m.sections {
                    for v in &sec.verts {
                        for k in 0..3 {
                            mesh.min[k] = mesh.min[k].min(v[k]);
                            mesh.max[k] = mesh.max[k].max(v[k]);
                        }
                    }
                    let nz = sec.norms.iter().any(|n| n.iter().any(|v| *v != 0.0));
                    mesh.subs.push(SubMesh {
                        verts: sec.verts.clone(),
                        idx: sec.idx.clone(),
                        material: sec.material.clone(),
                        op: sec.op,
                        nrm: if nz { sec.norms.clone() } else { Vec::new() },
                        uv: sec.uvs.clone(),
                        bones: Vec::new(),
                    });
                }
                self.meshes.insert(key, Arc::new(mesh));
                fl(0.0)
            }
            // ---- материалы ------------------------------------------------------------
            "material_create" => {
                let nm = clean_name(&s(0));
                if nm.is_empty() || self.materials.contains_key(&nm.to_ascii_lowercase()) {
                    self.fail("Material_Create: имя пустое или занято".into());
                    return h(0);
                }
                h(self.new_material(&nm))
            }
            "material_get" => h(self.materials.get(&s(0).to_ascii_lowercase()).copied().unwrap_or(0)),
            "material_getbesttechnique" => h(self.child_of(id, Kind::Technique, 0)),
            "material_gettechniquebyindex" => h(self.child_of(id, Kind::Technique, n(1) as i64)),
            "technique_getpassbyindex" => h(self.child_of(id, Kind::Pass, n(1) as i64)),
            "pass_gettextureunitstatebyindex" => h(self.child_of(id, Kind::TextureUnit, n(1) as i64)),
            "material_gettechniquebyname" | "technique_getpassbyname" | "pass_gettextureunitstatebyname" => {
                let want = s(1);
                h(self.objs[&id].children.iter().copied().find(|c| self.objs[c].name == want).unwrap_or(0))
            }
            "technique_create" | "pass_create" | "textureunitstate_create" => {
                let kind = match prefix {
                    "technique" => Kind::Technique,
                    "pass" => Kind::Pass,
                    _ => Kind::TextureUnit,
                };
                let parent_kind = match prefix {
                    "technique" => Kind::Material,
                    "pass" => Kind::Technique,
                    _ => Kind::Pass,
                };
                let Some(p) = self.get_kind(&a[0], &[parent_kind]) else {
                    self.fail(format!("{name}: неверный владелец"));
                    return h(0);
                };
                h(self.add_child(p, kind, &s(1)))
            }
            "textureunitstate_settexture" => {
                self.objs.get_mut(&id).unwrap().st.insert("textureunitstate.texture".into(), clean_name(&s(1)));
                self.want_texture(&s(1));
                self.objs.get_mut(&id).unwrap().f.insert("textureunitstate.texturetype".into(), vec![n(2)]);
                fl(0.0)
            }
            // ---- наложения ---------------------------------------------------------------
            "overlay_create" => {
                let nm = clean_name(&s(0));
                if nm.is_empty() || self.overlays.contains_key(&nm.to_ascii_lowercase()) {
                    self.fail("Overlay_Create: имя пустое или занято".into());
                    return h(0);
                }
                let nid = self.add(Obj::new(Kind::Overlay, &nm, 0));
                if nid != 0 {
                    self.overlays.insert(nm.to_ascii_lowercase(), nid);
                }
                h(nid)
            }
            "overlay_get" => h(self.overlays.get(&s(0).to_ascii_lowercase()).copied().unwrap_or(0)),
            "overlayelement_create" => {
                let ty = s(0).to_ascii_lowercase();
                let nm = clean_name(&s(1));
                if !matches!(ty.as_str(), "panel" | "borderpanel" | "textarea") || nm.is_empty() || self.elements.contains_key(&nm.to_ascii_lowercase()) {
                    self.fail("OverlayElement_Create: неизвестный тип или имя пустое/занято".into());
                    return h(0);
                }
                let nid = self.add(Obj::new(Kind::Element, &nm, 0));
                if nid != 0 {
                    self.elements.insert(nm.to_ascii_lowercase(), nid);
                    let o = self.objs.get_mut(&nid).unwrap();
                    o.container = ty != "textarea";
                    o.st.insert("overlayelement.type".into(), ty);
                }
                h(nid)
            }
            "overlayelement_get" => h(self.elements.get(&s(0).to_ascii_lowercase()).copied().unwrap_or(0)),
            "overlayelement_iscontainer" => bl(self.objs[&id].container),
            "overlayelement_getparent" => h(self.objs[&id].parent),
            "overlay_addchild" | "overlaycontainer_addchild" => {
                let Some(c) = self.get_kind(&a[1], &[Kind::Element]) else {
                    self.fail(format!("{name}: неверный элемент"));
                    return fl(0.0);
                };
                let ok_owner = prefix == "overlay" || self.objs[&id].container;
                if !ok_owner || c == id || (prefix == "overlaycontainer" && self.is_ancestor(c, id)) {
                    self.fail(format!("{name}: недопустимое добавление"));
                    return fl(0.0);
                }
                if prefix == "overlay" && !self.objs[&c].container {
                    self.fail("Overlay_AddChild: нужен контейнер".into());
                    return fl(0.0);
                }
                self.detach(c);
                self.objs.get_mut(&c).unwrap().parent = id;
                self.objs.get_mut(&id).unwrap().children.push(c);
                fl(0.0)
            }
            "overlay_removechild" | "overlaycontainer_removechild" => {
                if let Some(c) = self.get_kind(&a[1], &[Kind::Element]) {
                    if self.objs[&c].parent == id {
                        self.detach(c);
                    }
                }
                fl(0.0)
            }
            "overlaycontainer_getchildcount" => fl(self.objs[&id].children.len() as f64),
            "overlaycontainer_getchild" => {
                let i = n(1);
                h(if i >= 0.0 { self.objs[&id].children.get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            // ---- окна и области рендеринга ----------------------------------------------------
            "renderwindow_getcount" => fl(self.objs.values().filter(|o| o.kind == Kind::Window).count() as f64),
            "renderwindow_get" => {
                let i = n(0);
                h(if i >= 0.0 { self.objs.iter().filter(|(_, o)| o.kind == Kind::Window).nth(i as usize).map(|(k, _)| *k).unwrap_or(0) } else { 0 })
            }
            "renderwindow_getprimary" => h(self.objs.iter().find(|(_, o)| o.kind == Kind::Window).map(|(k, _)| *k).unwrap_or(0)),
            "renderwindow_getsize" => {
                out_f(outs, 1, &self.window_size(id));
                fl(0.0)
            }
            "renderwindow_setsize" => {
                if (1.0..=MAX_SURFACE as f64).contains(&n(1)) && (1.0..=MAX_SURFACE as f64).contains(&n(2)) {
                    self.objs.get_mut(&id).unwrap().f.insert("renderwindow.size".into(), vec![n(1).floor(), n(2).floor()]);
                } else {
                    self.fail("RenderWindow_SetSize: размер 1..16384".into());
                }
                fl(0.0)
            }
            "renderwindow_toggle_fullscreen" | "renderwindow_togglefullscreen" => {
                let cur = self.pf(id, "renderwindow.fullscreen").first().copied().unwrap_or(0.0);
                let nv = if cur != 0.0 { 0.0 } else { 1.0 };
                self.objs.get_mut(&id).unwrap().f.insert("renderwindow.fullscreen".into(), vec![nv]);
                fl(nv)
            }
            "renderwindow_getviewportcount" => fl(self.viewports_of(id).len() as f64),
            "renderwindow_getviewport" => {
                let i = n(1);
                h(if i >= 0.0 { self.viewports_of(id).get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            "renderwindow_getcursorposition" => {
                out_f(outs, 1, &self.cursor);
                fl(0.0)
            }
            "renderwindow_getkeyboardbuttonpressed" => bl(n(1) >= 0.0 && self.keys.contains(&(n(1) as u32))),
            "renderwindow_getmousebuttonpressed" => bl(n(1) >= 0.0 && (n(1) as usize) < 3 && self.mouse[n(1) as usize]),
            "renderwindow_getwheelposition" => fl(self.wheel),
            "rendertexture_create" => {
                let (w, hh) = (n(1).floor(), n(2).floor());
                if !self.root_created || !(1.0..=4096.0).contains(&w) || !(1.0..=4096.0).contains(&hh) {
                    self.fail("RenderTexture_Create: размер 1..4096 после Root_Create".into());
                    return h(0);
                }
                let nid = self.add(Obj::new(Kind::Texture, &clean_name(&s(0)), 0));
                if nid != 0 {
                    self.objs.get_mut(&nid).unwrap().f.insert("renderwindow.size".into(), vec![w, hh]);
                }
                h(nid)
            }
            "renderwindow_create" | "renderwindow_create2" | "renderwindow_createex" | "rendertexture_createex" => {
                if !self.root_created {
                    self.fail(format!("{name} до Root_Create"));
                    return h(0);
                }
                let (dw, dh) = self.config_video_mode(fx);
                // Create/CreateEx встраивают окно в пространство Stratum (его размер), Create2 берёт размер из аргументов
                let host = fx.gfx.spaces.get(&(n(0).max(0.0) as u32)).map(|sp| sp.client);
                let (mut w, mut hh) = match name {
                    "renderwindow_create2" => (n(2), n(3)),
                    _ => host.unwrap_or((dw, dh)),
                };
                if !((1.0..=MAX_SURFACE as f64).contains(&w) && (1.0..=MAX_SURFACE as f64).contains(&hh)) {
                    (w, hh) = (dw, dh);
                }
                let nid = if self.objs.values().filter(|o| o.kind == Kind::Window).count() >= 16 {
                    self.fail("слишком много окон рендеринга".into());
                    0
                } else {
                    self.add(Obj::new(Kind::Window, &clean_name(&if name == "renderwindow_create2" { s(1) } else { String::new() }), 0))
                };
                if nid != 0 {
                    let o = self.objs.get_mut(&nid).unwrap();
                    o.f.insert("renderwindow.size".into(), vec![w.floor(), hh.floor()]);
                    let fs = match name {
                        "renderwindow_create2" => n(4),
                        "renderwindow_createex" | "rendertexture_createex" => n(1),
                        _ => 0.0,
                    };
                    o.f.insert("renderwindow.fullscreen".into(), vec![bl(fs != 0.0).as_float()]);
                }
                h(nid)
            }
            "renderwindow_destroy" => {
                for v in self.viewports_of(id) {
                    self.remove_tree(v);
                }
                self.remove_obj(id);
                fl(0.0)
            }
            "renderwindow_getcursorhovered" => {
                let s = self.window_size(id);
                bl(self.cursor[0] >= 0.0 && self.cursor[1] >= 0.0 && self.cursor[0] < s[0] && self.cursor[1] < s[1])
            }
            "viewport_setdimensions" => {
                let d: Vec<f64> = (1..5).map(|i| n(i).clamp(0.0, 1.0)).collect();
                self.objs.get_mut(&id).unwrap().f.insert("viewport.dimensions".into(), d);
                fl(0.0)
            }
            "scenenode_getscene" => h(self.objs[&id].scene),
            "textureunitstate_gettexture" => {
                outs.push((1, Value::Str(self.objs[&id].st.get("textureunitstate.texture").cloned().unwrap_or_default())));
                outs.push((2, Value::Handle(self.pf(id, "textureunitstate.texturetype").first().copied().unwrap_or(0.0))));
                fl(0.0)
            }
            "entity_getmaterial" => {
                let i = n(1);
                outs.push((2, Value::Str(if i >= 0.0 { self.objs[&id].sub_mat.get(i as usize).cloned().unwrap_or_default() } else { String::new() })));
                fl(0.0)
            }
            "billboardset_createbillboard" => {
                if self.objs[&id].children.len() >= self.objs[&id].f.get("billboardset.pool").map(|p| p[0] as usize).unwrap_or(0) {
                    self.fail("BillboardSet_CreateBillboard: пул исчерпан".into());
                    return h(0);
                }
                let b = self.add_child(id, Kind::Billboard, "");
                if b != 0 {
                    self.objs.get_mut(&b).unwrap().f.insert("billboard.position".into(), vec![n(1), n(2), n(3)]);
                    self.objs.get_mut(&b).unwrap().scene = self.objs[&id].scene;
                }
                h(b)
            }
            "billboardset_getnumbillboards" => fl(self.objs[&id].children.len() as f64),
            "billboardset_getbillboard" => {
                let i = n(1);
                h(if i >= 0.0 { self.objs[&id].children.get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            "billboardset_removebillboard" => {
                if let Some(b) = self.get_kind(&a[1], &[Kind::Billboard]) {
                    if self.objs[&b].parent == id {
                        self.remove_obj(b);
                    }
                }
                fl(0.0)
            }
            "overlay_findelementat" => {
                let ws = self.window_size(self.objs.iter().find(|(_, o)| o.kind == Kind::Window).map(|(k, _)| *k).unwrap_or(0));
                let (px, py) = (n(1) * ws[0], n(2) * ws[1]);
                let sc = self.pf(id, "overlay.scale");
                let _ = sc;
                let mut hit = 0;
                for &e in &self.objs[&id].children {
                    if let Some(x) = self.find_element(e, (0.0, 0.0, ws[0], ws[1]), px, py, 0) {
                        hit = x;
                    }
                }
                h(hit)
            }
            "viewport_create" => {
                let Some(t) = self.get_kind(&a[0], &[Kind::Window, Kind::Texture]) else {
                    self.fail("Viewport_Create: неверная цель".into());
                    return h(0);
                };
                let Some(cam) = self.get_kind(&a[1], &[Kind::Camera]) else {
                    self.fail("Viewport_Create: неверная камера".into());
                    return h(0);
                };
                let z = n(2);
                if self.viewports_of(t).iter().any(|v| self.pf(*v, "viewport.zorder")[0] == z.floor()) {
                    self.fail("Viewport_Create: Z-порядок занят".into());
                    return h(0);
                }
                let nid = self.add(Obj::new(Kind::Viewport, "", 0));
                if nid != 0 {
                    let o = self.objs.get_mut(&nid).unwrap();
                    o.parent = t;
                    o.f.insert("camera".into(), vec![cam as f64]);
                    o.f.insert("viewport.zorder".into(), vec![z.floor()]);
                }
                h(nid)
            }
            "viewport_destroy" => {
                self.remove_tree(id);
                fl(0.0)
            }
            "viewport_getcamera" => h(self.objs[&id].f.get("camera").map(|c| c[0] as u32).unwrap_or(0)),
            "viewport_setcamera" => {
                if let Some(c) = self.get_kind(&a[1], &[Kind::Camera]) {
                    self.objs.get_mut(&id).unwrap().f.insert("camera".into(), vec![c as f64]);
                } else {
                    self.fail("Viewport_SetCamera: неверная камера".into());
                }
                fl(0.0)
            }
            "viewport_getdimensions" => {
                let d = self.pf(id, "viewport.dimensions");
                let d = if d.len() == 4 { d } else { vec![0.0, 0.0, 1.0, 1.0] };
                out_f(outs, 1, &d);
                fl(0.0)
            }
            "viewport_addcompositor" => {
                let nm = s(1);
                if !self.compositors.is_empty() && !self.compositors.contains(&nm.to_ascii_lowercase()) {
                    self.fail(format!("Viewport_AddCompositor: композитор '{nm}' не найден"));
                    return h(0);
                }
                let nid = self.add_child(id, Kind::Compositor, &nm);
                if nid != 0 {
                    let kids = &mut self.objs.get_mut(&id).unwrap().children;
                    kids.pop();
                    let pos = n(2);
                    let at = if pos < 0.0 || pos as usize >= kids.len() { kids.len() } else { pos as usize };
                    kids.insert(at, nid);
                }
                h(nid)
            }
            "viewport_getnumcompositors" => fl(self.objs[&id].children.len() as f64),
            "viewport_getcompositor" => {
                let i = n(1);
                h(if i >= 0.0 { self.objs[&id].children.get(i as usize).copied().unwrap_or(0) } else { 0 })
            }
            "viewport_removecompositor" => {
                if let Some(c) = self.get_kind(&a[1], &[Kind::Compositor]) {
                    if self.objs[&c].parent == id {
                        self.remove_obj(c);
                    }
                }
                fl(0.0)
            }
            "viewport_getray" => {
                if let Some((p, d)) = self.viewport_ray(id, n(1), n(2)) {
                    out_f(outs, 3, &p);
                    out_f(outs, 6, &d);
                } else {
                    self.fail("Viewport_GetRay: у области нет камеры".into());
                    out_f(outs, 3, &[0.0; 6]);
                }
                fl(0.0)
            }
            // ---- пересечения -----------------------------------------------------------------
            "collision_raycast" => {
                let Some(sc) = self.get_kind(&a[0], &[Kind::Scene]) else {
                    self.fail("Collision_RayCast: неверная сцена".into());
                    return fl(0.0);
                };
                self.hits = self.raycast(sc, [n(1), n(2), n(3)], [n(4), n(5), n(6)]);
                fl(0.0)
            }
            "collision_getresultcount" => fl(self.hits.len() as f64),
            "collision_sort" => {
                self.hits.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
                fl(0.0)
            }
            "collision_getobject" => {
                let i = n(0);
                h(if i >= 0.0 { self.hits.get(i as usize).map(|x| x.1).unwrap_or(0) } else { 0 })
            }
            "collision_getdistance" => {
                let i = n(0);
                fl(if i >= 0.0 { self.hits.get(i as usize).map(|x| x.0).unwrap_or(0.0) } else { 0.0 })
            }
            // ---- строковый интерфейс ----------------------------------------------------------------
            "stringinterface_setparameter" => {
                let (k, v) = (clean_name(&s(1)), s(2).chars().take(MAX_TEXT).collect::<String>());
                let o = self.objs.get_mut(&id).unwrap();
                if let Some(p) = o.params.iter_mut().find(|p| p.0.eq_ignore_ascii_case(&k)) {
                    p.1 = v;
                } else if o.params.len() < MAX_PARAMS && !k.is_empty() {
                    o.params.push((k, v));
                } else {
                    self.fail("StringInterface_SetParameter: предел параметров".into());
                }
                fl(0.0)
            }
            "stringinterface_getparameter" => {
                let k = s(1);
                let v = self.objs[&id].params.iter().find(|p| p.0.eq_ignore_ascii_case(&k)).map(|p| p.1.clone()).unwrap_or_default();
                outs.push((2, Value::Str(v)));
                fl(0.0)
            }
            "stringinterface_getparametercount" => fl(self.objs[&id].params.len() as f64),
            "stringinterface_getparametername" | "stringinterface_getparameterdescription" => {
                let i = n(1);
                let p = if i >= 0.0 { self.objs[&id].params.get(i as usize) } else { None };
                let v = p.map(|p| if name.ends_with("name") { p.0.clone() } else { p.1.clone() }).unwrap_or_default();
                outs.push((2, Value::Str(v)));
                fl(0.0)
            }
            "stringinterface_getparametertype" => fl(8.0),
            _ => self.generic(name, prefix, verb, id, sig, a, outs),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn generic(&mut self, name: &str, prefix: &str, verb: &str, id: u32, sig: &Sig, a: &[Value], outs: &mut Vec<(usize, Value)>) -> Value {
        if id == 0 {
            self.fail(format!("{name}: не поддерживается"));
            return zero(sig);
        }
        let (is_set, prop) = if let Some(p) = verb.strip_prefix("set") {
            (true, p)
        } else if let Some(p) = verb.strip_prefix("get") {
            (false, p)
        } else {
            self.fail(format!("{name}: не поддерживается"));
            return zero(sig);
        };
        let pre = match prefix {
            "scenenode" | "bone" => "node",
            "billboard" => "billboardset",
            "overlaycontainer" => "overlayelement",
            p => p,
        };
        let pre = if pre == "billboardset" && !matches!(prop, "billboardtype" | "commonupvector" | "commondirection" | "defaultdimensions" | "materialname") { "billboard" } else { pre };
        let key = format!("{pre}.{prop}");
        // свойства Movable_* общие для всех перемещаемых объектов
        if is_set {
            let mut floats = Vec::new();
            let mut text: Option<String> = None;
            for (i, (c, _)) in sig.params.iter().enumerate().skip(1) {
                match c {
                    'F' => floats.push(a[i].as_float()),
                    'S' => text = Some(a[i].as_string().chars().take(MAX_TEXT).collect()),
                    _ => {}
                }
            }
            if BOOL_PROPS.contains(&prop) {
                for v in floats.iter_mut() {
                    *v = if *v != 0.0 { 1.0 } else { 0.0 };
                }
            }
            if POSITIVE_PROPS.contains(&key.as_str()) && floats.first().is_some_and(|v| *v <= 0.0) {
                self.fail(format!("{name}: значение должно быть больше нуля"));
                return zero(sig);
            }
            if key == "viewport.backgroundcolour" || key == "light.diffusecolour" || key == "light.specularcolour" {
                // цвета Ogre - 0..1, отрицательные не имеют смысла
                for v in floats.iter_mut() {
                    *v = v.max(0.0);
                }
            }
            let o = self.objs.get_mut(&id).unwrap();
            if !floats.is_empty() {
                o.f.insert(key.clone(), floats);
            }
            if let Some(t) = text {
                o.st.insert(key, t);
            }
            return zero(sig);
        }
        let o = &self.objs[&id];
        let fv = o.f.get(&key).cloned().unwrap_or_else(|| default_of(&key));
        let sv = o.st.get(&key).cloned().unwrap_or_default();
        let (mut fi, mut any_out) = (0usize, false);
        for (i, (c, out)) in sig.params.iter().enumerate() {
            if *out {
                any_out = true;
                match c {
                    'F' => {
                        outs.push((i, Value::Float(fv.get(fi).copied().unwrap_or(0.0))));
                        fi += 1;
                    }
                    _ => outs.push((i, Value::Str(sv.clone()))),
                }
            }
        }
        if any_out {
            return zero(sig);
        }
        match sig.ret {
            'F' => fl(fv.first().copied().unwrap_or(0.0)),
            'H' => h(fv.first().copied().unwrap_or(0.0) as u32),
            'S' => Value::Str(sv),
            _ => zero(sig),
        }
    }

    fn child_of(&self, id: u32, kind: Kind, idx: i64) -> u32 {
        if idx < 0 {
            return 0;
        }
        self.objs[&id].children.iter().copied().filter(|c| self.objs[c].kind == kind).nth(idx as usize).unwrap_or(0)
    }

    fn set_anim_time(&mut self, id: u32, t: f64) {
        let len = self.pf(id, "animationstate.length").first().copied().unwrap_or(0.0);
        let looped = self.pf(id, "animationstate.loop").first().copied().unwrap_or(1.0) != 0.0;
        let t = if len > 0.0 {
            if looped {
                let r = t % len;
                if r < 0.0 { r + len } else { r }
            } else {
                t.clamp(0.0, len)
            }
        } else {
            t.max(0.0)
        };
        self.objs.get_mut(&id).unwrap().f.insert("animationstate.timeposition".into(), vec![t]);
    }

    fn config_video_mode(&mut self, fx: &mut Effects) -> (f64, f64) {
        let cfg = self.config.clone();
        if !cfg.is_empty() {
            if let Some(p) = super::extra::sandboxed(fx, &cfg, false) {
                if std::fs::metadata(&p).map(|m| m.len() < (1 << 20)).unwrap_or(false) {
                    if let Ok(t) = std::fs::read(&p) {
                        for line in String::from_utf8_lossy(&t).lines() {
                            if let Some((k, v)) = line.split_once('=') {
                                if k.trim().eq_ignore_ascii_case("Video Mode") {
                                    let mut it = v.split(|c: char| !c.is_ascii_digit()).filter(|x| !x.is_empty());
                                    if let (Some(w), Some(hh)) = (it.next(), it.next()) {
                                        if let (Ok(w), Ok(hh)) = (w.parse::<f64>(), hh.parse::<f64>()) {
                                            if (1.0..=MAX_SURFACE as f64).contains(&w) && (1.0..=MAX_SURFACE as f64).contains(&hh) {
                                                return (w, hh);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        (800.0, 600.0)
    }

    fn window_size(&self, id: u32) -> Vec<f64> {
        let d = self.pf(id, "renderwindow.size");
        if d.len() == 2 { d } else { vec![800.0, 600.0] }
    }

    fn viewports_of(&self, target: u32) -> Vec<u32> {
        let mut v: Vec<(f64, u32)> =
            self.objs.iter().filter(|(_, o)| o.kind == Kind::Viewport && o.parent == target).map(|(i, o)| (o.f.get("viewport.zorder").map(|z| z[0]).unwrap_or(0.0), *i)).collect();
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().map(|x| x.1).collect()
    }

    /// Ограничивающий параллелепипед объекта в локальных координатах.
    fn bounds(&self, id: u32) -> Vec<f64> {
        let o = &self.objs[&id];
        let mut mn = [f64::MAX; 3];
        let mut mx = [f64::MIN; 3];
        let mut any = false;
        let mut add = |v: &[f32; 3]| {
            any = true;
            for k in 0..3 {
                mn[k] = mn[k].min(v[k] as f64);
                mx[k] = mx[k].max(v[k] as f64);
            }
        };
        if let Some(m) = &o.mesh {
            for sm in &m.subs {
                sm.verts.iter().for_each(&mut add);
            }
        }
        if let Some(m) = &o.manual {
            for sec in &m.sections {
                sec.verts.iter().for_each(&mut add);
            }
        }
        if any { vec![mn[0], mn[1], mn[2], mx[0], mx[1], mx[2]] } else { vec![0.0; 6] }
    }

    fn camera_pose(&self, cam: u32) -> Option<(V3, Q)> {
        let node = self.objs.get(&cam)?.parent;
        if node == 0 {
            return Some(([0.0; 3], [1.0, 0.0, 0.0, 0.0]));
        }
        let (p, r, _) = self.derived(node);
        Some((p, r))
    }

    fn viewport_ray(&self, vp: u32, sx: f64, sy: f64) -> Option<(V3, V3)> {
        let cam = self.objs.get(&vp)?.f.get("camera")?[0] as u32;
        let (p, r) = self.camera_pose(cam)?;
        let fov = self.pf(cam, "camera.fov")[0];
        let target = self.objs[&vp].parent;
        let ws = self.window_size(target);
        let dim = { let d = self.pf(vp, "viewport.dimensions"); if d.len() == 4 { d } else { vec![0.0, 0.0, 1.0, 1.0] } };
        let aspect = (ws[0] * dim[2].max(1e-6)) / (ws[1] * dim[3].max(1e-6));
        let t = (fov / 2.0).tan();
        let (x, y) = ((2.0 * sx.clamp(0.0, 1.0) - 1.0) * t * aspect, (1.0 - 2.0 * sy.clamp(0.0, 1.0)) * t);
        if self.pf(cam, "camera.projectiontype")[0] == 0.0 {
            let near = self.pf(cam, "camera.nearclipdistance")[0];
            let o = qrot(r, [x * near, y * near, 0.0]);
            Some(([p[0] + o[0], p[1] + o[1], p[2] + o[2]], unit(qrot(r, [0.0, 0.0, -1.0]))))
        } else {
            Some((p, unit(qrot(r, [x, y, -1.0]))))
        }
    }

    fn raycast(&mut self, scene: u32, org: V3, dir: V3) -> Vec<(f64, u32)> {
        let dl = len(dir);
        let mut res = Vec::new();
        if dl < 1e-12 {
            return res;
        }
        let mut budget = MAX_TRIS_TESTED;
        let ids: Vec<u32> = self.objs.iter().filter(|(_, o)| matches!(o.kind, Kind::Entity | Kind::ManualObject) && o.scene == scene).map(|(i, _)| *i).collect();
        for id in ids {
            if self.pf(id, "movable.visible")[0] == 0.0 || !self.node_chain_visible(id) {
                continue;
            }
            let mut best: Option<f64> = None;
            for t in self.world_tris(id) {
                if budget == 0 || res.len() >= MAX_RESULTS {
                    return res;
                }
                budget -= 1;
                if let Some(d) = ray_tri(org, dir, t) {
                    best = Some(best.map_or(d, |b: f64| b.min(d)));
                }
            }
            if let Some(d) = best {
                res.push((d * dl, id));
            }
        }
        res
    }

    fn node_chain_visible(&self, id: u32) -> bool {
        self.objs[&id].parent != 0
    }

    /// Состояние ввода от окна приложения.
    pub fn set_input(&mut self, keys: &[u32], mouse: [bool; 3], cursor: [f64; 2], wheel: f64) {
        self.keys = keys.iter().copied().take(256).collect();
        self.mouse = mouse;
        self.cursor = if cursor[0].is_finite() && cursor[1].is_finite() { cursor } else { [0.0; 2] };
        self.wheel = if wheel.is_finite() { wheel } else { 0.0 };
    }

    /// Есть ли живая сцена (для плашки "нет движка").
    pub fn active(&self) -> bool {
        self.root_created
    }

}

fn is_creator(name: &str) -> bool {
    name.starts_with("renderwindow_create") || name.ends_with("_create") || name == "particlesystem_createex" || name == "rendertexture_createex" || name == "collision_raycast"
}

fn tri_list(op: u16, idx: &[u32]) -> Vec<[u32; 3]> {
    match op {
        4 => idx.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
        5 => idx.windows(3).enumerate().map(|(i, w)| if i % 2 == 0 { [w[0], w[1], w[2]] } else { [w[1], w[0], w[2]] }).collect(),
        6 => {
            if idx.len() < 3 {
                Vec::new()
            } else {
                idx[1..].windows(2).map(|w| [idx[0], w[0], w[1]]).collect()
            }
        }
        _ => Vec::new(),
    }
}

fn ray_tri(o: V3, d: V3, t: [V3; 3]) -> Option<f64> {
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let tv = sub(o, t[0]);
    let u = dot(tv, p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(tv, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let dist = dot(e2, q) * inv;
    (dist >= 0.0 && dist.is_finite()).then_some(dist)
}

fn walk_find(dir: &std::path::Path, want: &str, depth: usize, budget: &mut usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let rd = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for e in rd.flatten() {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            subdirs.push(p);
        } else if e.file_name().to_string_lossy().to_ascii_lowercase() == want {
            return Some(p);
        }
    }
    subdirs.into_iter().find_map(|d| walk_find(&d, want, depth - 1, budget))
}

fn collect_files(dir: &std::path::Path, rec: bool, depth: usize, out: &mut Vec<PathBuf>, count: &mut usize) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        if *count >= 4000 {
            return;
        }
        *count += 1;
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            if rec && depth > 0 {
                collect_files(&e.path(), rec, depth - 1, out, count);
            }
        } else {
            out.push(e.path());
        }
    }
}

// ---- скрипты материалов ---------------------------------------------------------------

fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("");
        let mut cur = String::new();
        let mut quoted = false;
        for c in line.chars() {
            if c == '"' {
                quoted = !quoted;
            } else if quoted {
                cur.push(c);
            } else if c.is_whitespace() || c == '{' || c == '}' {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                if c == '{' || c == '}' {
                    out.push(c.to_string());
                }
            } else {
                cur.push(c);
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        if out.len() > 2_000_000 {
            break;
        }
    }
    out
}

fn skip_block(t: &[String], open: usize) -> usize {
    let mut d = 0usize;
    let mut i = open;
    while i < t.len() {
        if t[i] == "{" {
            d += 1;
        } else if t[i] == "}" {
            d = d.saturating_sub(1);
            if d == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    t.len()
}

fn is_keyword(w: &str) -> bool {
    matches!(
        w,
        "ambient" | "diffuse" | "specular" | "emissive" | "lighting" | "depth_check" | "depth_write" | "cull_hardware" | "scene_blend" | "texture"
            | "technique" | "pass" | "texture_unit" | "material" | "cubic_texture"
    )
}

// ---- форматы .mesh и .skeleton ------------------------------------------------------------

struct Rd<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let e = self.p.checked_add(n)?;
        let s = self.d.get(self.p..e)?;
        self.p = e;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn f32(&mut self) -> Option<f32> {
        self.u32().map(f32::from_bits)
    }
    fn line(&mut self) -> Option<String> {
        let start = self.p;
        while self.p < self.d.len() && self.p - start < 1024 {
            if self.d[self.p] == b'\n' {
                let s = String::from_utf8_lossy(&self.d[start..self.p]).into_owned();
                self.p += 1;
                return Some(s);
            }
            self.p += 1;
        }
        None
    }
    fn peek16(&self) -> Option<u16> {
        Some(u16::from_le_bytes([*self.d.get(self.p)?, *self.d.get(self.p + 1)?]))
    }
    #[allow(dead_code)]
    fn chunk(&mut self) -> Option<(u16, usize)> {
        let id = self.u16()?;
        let l = self.u32()? as usize;
        if l < 6 {
            return None;
        }
        let end = (self.p - 6).checked_add(l)?;
        // длины в файлах некоторых экспортёров завышены: ограничиваем файлом
        Some((id, end.min(self.d.len())))
    }
}

#[derive(Default)]
struct Geom {
    pos: Vec<[f32; 3]>,
    nrm: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
}


fn spread_bones(n: usize, asg: &[(usize, u16, f32)]) -> Vec<Vec<(u16, f32)>> {
    if asg.is_empty() {
        return Vec::new();
    }
    let mut v = vec![Vec::new(); n];
    for &(i, b, w) in asg {
        if i < n && w.is_finite() && w > 0.0 && v[i].len() < 8 {
            v[i].push((b, w));
        }
    }
    v
}

/// Разбор идёт последовательно, как в Ogre: длины блоков у некоторых
/// экспортёров неверны (например, у Sinbad.mesh), поэтому на них не опираемся.
pub fn parse_mesh(data: &[u8]) -> Option<Mesh> {
    let mut r = Rd { d: data, p: 0 };
    if r.u16()? != 0x1000 {
        return None;
    }
    let ver = r.line()?;
    if !(ver.contains("v1.4") || ver.contains("v1.7") || ver.contains("v1.8")) {
        return None;
    }
    let mut mesh = Mesh { min: [f32::MAX; 3], max: [f32::MIN; 3], ..Default::default() };
    let mut shared = Geom::default();
    let mut shared_asg: Vec<(usize, u16, f32)> = Vec::new();
    if r.u16()? != 0x3000 {
        return None;
    }
    r.u32()?;
    r.u8()?;
    let mut subs: Vec<(SubMesh, bool, Vec<(usize, u16, f32)>)> = Vec::new();
    while let Some(id) = r.peek16() {
        match id {
            0x5000 => {
                r.p += 6;
                shared = parse_geometry(&mut r)?;
            }
            0x4000 => {
                r.p += 6;
                if subs.len() >= 1024 {
                    return None;
                }
                let mut sm = SubMesh { material: r.line()?, op: 4, ..Default::default() };
                let use_shared = r.u8()? != 0;
                let ic = r.u32()? as usize;
                let wide = r.u8()? != 0;
                if ic > MAX_INDICES {
                    return None;
                }
                for _ in 0..ic {
                    sm.idx.push(if wide { r.u32()? } else { r.u16()? as u32 });
                }
                if !use_shared {
                    if r.u16()? != 0x5000 {
                        return None;
                    }
                    r.u32()?;
                    let g = parse_geometry(&mut r)?;
                    sm.verts = g.pos;
                    sm.nrm = g.nrm;
                    sm.uv = g.uv;
                }
                let mut asg = Vec::new();
                while let Some(sid) = r.peek16() {
                    match sid {
                        0x4010 => {
                            r.p += 6;
                            sm.op = r.u16()?;
                        }
                        0x4100 => {
                            r.p += 6;
                            let v = r.u32()? as usize;
                            let b = r.u16()?;
                            let w = r.f32()?;
                            if asg.len() < MAX_VERTS * 4 {
                                asg.push((v, b, w));
                            }
                        }
                        0x4200 => {
                            r.p += 6;
                            r.line()?;
                            r.line()?;
                        }
                        _ => break,
                    }
                }
                subs.push((sm, use_shared, asg));
            }
            0x6000 => {
                r.p += 6;
                mesh.skeleton = Some(r.line()?);
            }
            0x7000 => {
                r.p += 6;
                let v = r.u32()? as usize;
                let b = r.u16()?;
                let w = r.f32()?;
                if shared_asg.len() < MAX_VERTS * 4 {
                    shared_asg.push((v, b, w));
                }
            }
            0x9000 => {
                r.p += 6;
                for k in 0..3 {
                    mesh.min[k] = r.f32()?;
                }
                for k in 0..3 {
                    mesh.max[k] = r.f32()?;
                }
                r.f32()?;
            }
            // LOD, таблицы имён, списки рёбер: нам не нужны, длины неточны
            _ => break,
        }
    }
    for (mut sm, use_shared, asg) in subs {
        if use_shared {
            sm.verts = shared.pos.clone();
            sm.nrm = shared.nrm.clone();
            sm.uv = shared.uv.clone();
            sm.bones = spread_bones(sm.verts.len(), &shared_asg);
        } else {
            sm.bones = spread_bones(sm.verts.len(), &asg);
        }
        if sm.idx.iter().any(|&i| i as usize >= sm.verts.len()) {
            return None;
        }
        mesh.subs.push(sm);
    }
    // границы считаем по вершинам (заголовочные могут быть неверны)
    mesh.min = [f32::MAX; 3];
    mesh.max = [f32::MIN; 3];
    for sm in &mesh.subs {
        for v in &sm.verts {
            for k in 0..3 {
                mesh.min[k] = mesh.min[k].min(v[k]);
                mesh.max[k] = mesh.max[k].max(v[k]);
            }
        }
    }
    if mesh.min[0] > mesh.max[0] {
        mesh.min = [0.0; 3];
        mesh.max = [0.0; 3];
    }
    if mesh.subs.is_empty() {
        return None;
    }
    Some(mesh)
}

/// Геометрия: вызывается после заголовка блока 0x5000.
fn parse_geometry(r: &mut Rd) -> Option<Geom> {
    let vc = r.u32()? as usize;
    if vc > MAX_VERTS {
        return None;
    }
    let (mut pos_e, mut nrm_e, mut uv_e): (Option<(u16, u16)>, Option<(u16, u16)>, Option<(u16, u16)>) = (None, None, None);
    let mut bufs: HashMap<u16, usize> = HashMap::new();
    let mut sizes: HashMap<u16, usize> = HashMap::new();
    while let Some(id) = r.peek16() {
        match id {
            0x5100 => {
                r.p += 6;
                while r.peek16() == Some(0x5110) {
                    r.p += 6;
                    let (src, ty, sem, off, idx) = (r.u16()?, r.u16()?, r.u16()?, r.u16()?, r.u16()?);
                    match (sem, ty) {
                        (1, 2) if pos_e.is_none() => pos_e = Some((src, off)),
                        (4, 2) if nrm_e.is_none() => nrm_e = Some((src, off)),
                        (7, 1) if uv_e.is_none() && idx == 0 => uv_e = Some((src, off)),
                        _ => {}
                    }
                }
            }
            0x5200 => {
                r.p += 6;
                let idx = r.u16()?;
                let vsize = r.u16()? as usize;
                if r.u16()? != 0x5210 {
                    return None;
                }
                r.u32()?;
                let start = r.p;
                let total = vc.checked_mul(vsize)?;
                r.take(total)?;
                sizes.insert(idx, vsize);
                bufs.insert(idx, start);
            }
            _ => break,
        }
    }
    let read = |e: (u16, u16), n: usize| -> Option<Vec<Vec<f32>>> {
        let start = *bufs.get(&e.0)?;
        let vsize = *sizes.get(&e.0)?;
        if vsize < e.1 as usize + n * 4 {
            return None;
        }
        let mut out = Vec::with_capacity(vc);
        for i in 0..vc {
            let mut rr = Rd { d: r.d, p: start + i * vsize + e.1 as usize };
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(rr.f32()?);
            }
            out.push(v);
        }
        Some(out)
    };
    let pos = read(pos_e?, 3)?.into_iter().map(|v| [v[0], v[1], v[2]]).collect();
    let nrm = nrm_e.and_then(|e| read(e, 3)).map(|a| a.into_iter().map(|v| [v[0], v[1], v[2]]).collect()).unwrap_or_default();
    let uv = uv_e.and_then(|e| read(e, 2)).map(|a| a.into_iter().map(|v| [v[0], v[1]]).collect()).unwrap_or_default();
    Some(Geom { pos, nrm, uv })
}

pub fn parse_skeleton(data: &[u8]) -> Option<Skeleton> {
    let mut r = Rd { d: data, p: 0 };
    if r.u16()? != 0x1000 {
        return None;
    }
    r.line()?;
    let mut sk = Skeleton::default();
    let mut parents: Vec<(u16, u16)> = Vec::new();
    // после поворота ключа/кости может идти необязательный масштаб (3 числа):
    // он есть, если дальше не начинается известный блок
    let known = |id: Option<u16>| matches!(id, None | Some(0x2000) | Some(0x3000) | Some(0x4000) | Some(0x4100) | Some(0x4110) | Some(0x5000));
    while let Some(id) = r.peek16() {
        match id {
            0x2000 => {
                r.p += 6;
                if sk.bones.len() >= 4096 {
                    return None;
                }
                let name = r.line()?;
                let handle = r.u16()?;
                let pos = [r.f32()? as f64, r.f32()? as f64, r.f32()? as f64];
                let (qx, qy, qz, qw) = (r.f32()? as f64, r.f32()? as f64, r.f32()? as f64, r.f32()? as f64);
                if !known(r.peek16()) {
                    r.take(12)?;
                }
                sk.bones.push(BoneDef { name, handle, parent: None, pos, rot: qnorm([qw, qx, qy, qz]) });
            }
            0x3000 => {
                r.p += 6;
                if parents.len() >= 4096 {
                    return None;
                }
                parents.push((r.u16()?, r.u16()?));
            }
            0x4000 => {
                r.p += 6;
                if sk.anims.len() >= 4096 {
                    return None;
                }
                let name = r.line()?;
                let l = r.f32()? as f64;
                let mut an = Anim { name, length: if l.is_finite() && l >= 0.0 { l } else { 0.0 }, tracks: Vec::new() };
                while r.peek16() == Some(0x4100) {
                    r.p += 6;
                    let bh = r.u16()?;
                    let mut keys = Vec::new();
                    while r.peek16() == Some(0x4110) {
                        r.p += 6;
                        let t = r.f32()? as f64;
                        let (qx, qy, qz, qw) = (r.f32()? as f64, r.f32()? as f64, r.f32()? as f64, r.f32()? as f64);
                        let pos = [r.f32()? as f64, r.f32()? as f64, r.f32()? as f64];
                        if !known(r.peek16()) {
                            r.take(12)?;
                        }
                        if keys.len() < 100_000 && t.is_finite() && pos.iter().all(|v| v.is_finite()) {
                            keys.push(Key { t, rot: qnorm([qw, qx, qy, qz]), pos });
                        }
                    }
                    if an.tracks.len() < 4096 {
                        if let Some(bi) = sk.bones.iter().position(|b| b.handle == bh) {
                            keys.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
                            an.tracks.push((bi, keys));
                        }
                    }
                }
                sk.anims.push(an);
            }
            _ => break,
        }
    }
    for (c, p) in parents {
        let pi = sk.bones.iter().position(|b| b.handle == p);
        if let Some(b) = sk.bones.iter_mut().find(|b| b.handle == c) {
            b.parent = pi;
        }
    }
    for i in 0..sk.bones.len() {
        let mut cur = Some(i);
        let mut n = 0;
        while let Some(c) = cur {
            n += 1;
            if n > sk.bones.len() + 1 {
                sk.bones[i].parent = None;
                break;
            }
            cur = sk.bones[c].parent;
        }
    }
    Some(sk)
}

mod fx;
mod image;
mod render;
mod scene;
mod zip;
pub use render::WINDOW_NAME;

#[cfg(test)]
mod tests;
