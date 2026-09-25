//! CLI ядра Stratum Modern.
//!
//!     stratum run PROJECT [--ticks N] [--dump] [--watch имидж.переменная]
//!     stratum parse PATH...        разобрать тексты имиджей
//!     stratum info PROJECT         состав проекта: имиджи, схема, связи
//!     stratum check DIR            прогнать все .cls в дереве через парсеры

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use stratum_core::formats::{self, cls};
use stratum_core::lang;
use stratum_core::sim::Simulation;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("parse") => cmd_parse(&args[1..]),
        Some("info") => cmd_info(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
        Some("render") => cmd_render(&args[1..]),
        Some("play") => cmd_play(&args[1..]),
        Some("convert") => cmd_convert(&args[1..]),
        Some("bytecode") => cmd_bytecode(&args[1..]),
        Some("instrument") => cmd_instrument(&args[1..]),
        Some("sttdiff") => cmd_sttdiff(&args[1..]),
        Some("--help") | Some("-h") | None => {
            usage();
            Ok(())
        }
        Some(other) => Err(format!("неизвестная команда {other:?}; см. stratum --help")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ошибка: {message}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    println!(
        "stratum — ядро Stratum Modern\n\n\
         Команды:\n  \
         run PROJECT [--ticks N] [--dump] [--watch ИМИДЖ.ПЕРЕМЕННАЯ] [--lib ПАПКА]\n      \
         открыть проект (.spj или папку) и просчитать N тактов;\n      \
         библиотеки: --lib, иначе $STRATUM_LIBRARY, иначе fixtures/ или Stratum в Wine\n  \
         parse ФАЙЛ...\n      разобрать текст имиджа и показать дерево\n  \
         info PROJECT\n      состав проекта: имиджи, экземпляры, связи\n  \
         check КАТАЛОГ\n      прогнать все .cls каталога через парсеры форматов и языка\n  \
         render PROJECT [--ticks N] [--out ПАПКА] [--lib ПАПКА]\n      \
         просчитать N тактов и записать окна модели в SVG\n  \
         play PROJECT [--port 8765] [--lib ПАПКА]\n      \
         открыть плеер в браузере: транспорт, окно модели, мышь и клавиатура\n  \
         convert ИСТОЧНИК ПАПКА [--to stratum2000] [--lib ПАПКА]\n      \
         импорт проекта Stratum 2000 в родной текстовый формат (project.json)\n      \
         или экспорт обратно в .spj/.cls с ключом --to stratum2000"
    );
}

fn cmd_convert(args: &[String]) -> Result<(), String> {
    let mut paths = Vec::new();
    let mut to_stratum2000 = false;
    let mut libraries = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--to" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some("stratum2000") => to_stratum2000 = true,
                    Some("native") => {}
                    other => return Err(format!("--to: ожидалось native или stratum2000, а не {other:?}")),
                }
            }
            "--lib" => {
                i += 1;
                libraries.push(PathBuf::from(args.get(i).ok_or("--lib без папки")?));
            }
            other if other.starts_with("--") => return Err(format!("неизвестный ключ {other}")),
            other => paths.push(PathBuf::from(other)),
        }
        i += 1;
    }
    let [src, dest] = paths.as_slice() else { return Err("нужно два пути: источник и папка назначения".into()) };
    if libraries.is_empty() {
        libraries = formats::default_library_dirs();
    }
    let loaded = formats::load_project(src, &libraries).map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    if to_stratum2000 {
        formats::native::export_stratum2000(dest, &loaded).map_err(|e| e.to_string())?;
        // имидж, который компилятор не принял, уходит без байт-кода —
        // оригинал его исполнять не станет; об этом надо сказать
        for cls in &loaded.classes[..loaded.own_classes] {
            let failure = match lang::parse(&cls.text) {
                Ok(model) => formats::native::check_text(&loaded, cls, &model).err().map(|e| format!("строка {}: {}", e.line, e.message)),
                Err(e) => Some(format!("{e:?}")),
            };
            if let Some(msg) = failure {
                eprintln!("предупреждение: {} не скомпилирован, {msg}", cls.name);
            }
        }
        println!("экспортировано в Stratum 2000: {} имиджей → {}", loaded.own_classes, dest.display());
    } else {
        std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        formats::native::save(dest, &loaded).map_err(|e| e.to_string())?;
        println!("сохранено: {} имиджей → {}", loaded.own_classes, dest.join("project.json").display());
    }
    Ok(())
}

struct RunOptions {
    path: PathBuf,
    ticks: u64,
    dump: bool,
    watch: Vec<String>,
    libraries: Vec<PathBuf>,
    out: PathBuf,
    port: u16,
}

fn parse_run_options(args: &[String]) -> Result<RunOptions, String> {
    let mut opts = RunOptions {
        path: PathBuf::new(),
        ticks: 1,
        dump: false,
        watch: Vec::new(),
        libraries: Vec::new(),
        out: PathBuf::from("."),
        port: 8765,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ticks" => {
                i += 1;
                opts.ticks = args
                    .get(i)
                    .ok_or("после --ticks нужно число")?
                    .parse()
                    .map_err(|_| "после --ticks нужно число")?;
            }
            "--dump" => opts.dump = true,
            "--port" => {
                i += 1;
                opts.port = args.get(i).ok_or("после --port нужно число")?.parse().map_err(|_| "после --port нужно число")?;
            }
            "--out" => {
                i += 1;
                opts.out = PathBuf::from(args.get(i).ok_or("после --out нужна папка")?);
            }
            "--lib" => {
                i += 1;
                opts.libraries.push(PathBuf::from(args.get(i).ok_or("после --lib нужна папка")?));
            }
            "--watch" => {
                i += 1;
                opts.watch.push(args.get(i).ok_or("после --watch нужно имя переменной")?.clone());
            }
            other if other.starts_with("--") => return Err(format!("неизвестный ключ {other}")),
            other => opts.path = PathBuf::from(other),
        }
        i += 1;
    }
    if opts.path.as_os_str().is_empty() && !args.iter().any(|a| a.is_empty()) {
        return Err("не указан проект".into());
    }
    if opts.libraries.is_empty() {
        opts.libraries = formats::default_library_dirs();
    }
    Ok(opts)
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let opts = parse_run_options(args)?;
    let loaded = formats::load_project(&opts.path, &opts.libraries)
        .map_err(|e| format!("{}: {e}", opts.path.display()))?
        .map_err(|e| e.to_string())?;

    println!(
        "проект {}: корневой имидж {}, имиджей {} (+{} из библиотек)",
        opts.path.display(),
        loaded.project.root,
        loaded.own_classes,
        loaded.classes.len() - loaded.own_classes
    );

    let mut sim = Simulation::build(&loaded).map_err(|e| e.to_string())?;
    println!("экземпляров на схеме: {}", sim.instances().len());

    // за чем следим: `имидж.переменная`
    let watches: Vec<(usize, String)> = opts
        .watch
        .iter()
        .map(|w| {
            let (image, var) = w
                .rsplit_once('.')
                .ok_or_else(|| format!("{w:?}: нужно имя вида имидж.переменная"))?;
            let index = sim
                .find(image)
                .ok_or_else(|| format!("{image:?}: такого имиджа нет в проекте"))?;
            Ok((index, var.to_string()))
        })
        .collect::<Result<_, String>>()?;

    for _ in 0..opts.ticks {
        sim.step().map_err(|e| e.message)?;
        if !watches.is_empty() {
            let values: Vec<String> = watches
                .iter()
                .map(|(i, var)| match sim.value(*i, var) {
                    Some(v) => format!("{}.{}={}", sim.instances()[*i].name, var, v),
                    None => format!("{}.{}=—", sim.instances()[*i].name, var),
                })
                .collect();
            println!("такт {:>6}  {}", sim.tick_number(), values.join("  "));
        }
        if sim.stopped {
            println!("модель остановлена на такте {}", sim.tick_number());
            break;
        }
    }

    if opts.dump {
        println!("\nзначения после такта {}:", sim.tick_number());
        for (i, instance) in sim.instances().iter().enumerate() {
            let vars: Vec<String> = instance
                .var_names()
                .iter()
                .filter_map(|name| sim.value(i, name).map(|v| format!("{name}={v}")))
                .collect();
            if vars.is_empty() {
                continue;
            }
            println!("  {} [{}]", instance.path, instance.class_name);
            for chunk in vars.chunks(6) {
                println!("      {}", chunk.join("  "));
            }
        }
    }

    if !sim.effects.log.is_empty() {
        println!("\nLogMessage:");
        for line in &sim.effects.log {
            println!("  {line}");
        }
    }
    if !sim.effects.missing.is_empty() {
        println!("\nещё не реализованные функции (вызовов):");
        let mut rows: Vec<_> = sim.effects.missing.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in rows.iter().take(20) {
            println!("  {name:<28} {count}");
        }
    }
    Ok(())
}

fn cmd_render(args: &[String]) -> Result<(), String> {
    let opts = parse_run_options(args)?;
    let loaded = formats::load_project(&opts.path, &opts.libraries)
        .map_err(|e| format!("{}: {e}", opts.path.display()))?
        .map_err(|e| e.to_string())?;
    let mut sim = Simulation::build(&loaded).map_err(|e| e.to_string())?;
    sim.run(opts.ticks).map_err(|e| e.message)?;
    std::fs::create_dir_all(&opts.out).map_err(|e| format!("{}: {e}", opts.out.display()))?;
    let gfx = &sim.effects.gfx;
    if gfx.window_order.is_empty() {
        println!("модель не открыла ни одного окна за {} тактов", sim.tick_number());
    }
    for name in &gfx.window_order {
        let Some(space) = gfx.window_space(name).and_then(|h| gfx.space(h)) else { continue };
        let file = opts.out.join(format!("{}.svg", safe_name(name)));
        std::fs::write(&file, stratum_core::gfx::svg::render_in(space, Some(&sim.effects.gfx)))
            .map_err(|e| format!("{}: {e}", file.display()))?;
        if opts.dump {
            for h in &space.top_order() {
                dump_object(space, *h, 0);
            }
            let mut pens: Vec<_> = space.pens.iter().collect();
            pens.sort_by_key(|(h, _)| **h);
            for (h, p) in pens {
                println!("pen #{h}: color={:06x} style={} width={} rop={}", p.color, p.style, p.width, p.rop);
            }
            let mut dibs: Vec<_> = space.dibs.iter().collect();
            dibs.sort_by_key(|(h, _)| **h);
            for (h, d) in dibs {
                println!("dib #{h}: file={:?} bmp={} байт {}×{}", d.file, d.bmp.len(), d.width, d.height);
            }
        }
        println!(
            "{}: окно «{}» — объектов {}, {}×{} → {}",
            sim.tick_number(),
            name,
            space.objects.len(),
            space.client.0,
            space.client.1,
            file.display()
        );
    }
    if !sim.effects.missing.is_empty() {
        let mut rows: Vec<_> = sim.effects.missing.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        println!("не реализовано: {}", rows.iter().map(|(n, c)| format!("{n}×{c}")).collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

fn dump_object(space: &stratum_core::gfx::Space, h: stratum_core::gfx::Handle, depth: usize) {
    use stratum_core::gfx::Shape;
    let Some(o) = space.objects.get(&h) else { return };
    let kind = match &o.shape {
        Shape::Polyline { pen, brush, points } => format!("линия pen={pen} brush={brush} точек={}", points.len()),
        Shape::Bitmap { dib, src, .. } => format!("растр dib={dib} src={:?} есть={}", src, space.dibs.get(dib).map(|d| !d.bmp.is_empty()).unwrap_or(false)),
        Shape::Text { text } => format!("текст {text}"),
        Shape::Control { class, .. } => format!("контрол {class}"),
        Shape::View3d { space, camera } => format!("проекция 3D пространства #{space}, камера #{camera}"),
        Shape::Group { children } => format!("группа из {}", children.len()),
        Shape::Unknown => "?".into(),
    };
    println!(
        "{}#{} {:?} {} ({:.0},{:.0}) {:.0}×{:.0}{}",
        "  ".repeat(depth), h, o.name, kind, o.x, o.y, o.w, o.h, if o.visible { "" } else { " скрыт" }
    );
    if let Shape::Group { children } = &o.shape {
        for c in children {
            dump_object(space, *c, depth + 1);
        }
    }
}

fn cmd_play(args: &[String]) -> Result<(), String> {
    // `play` без проекта открывает пустую IDE с диалогом «Открыть»
    let opts = match parse_run_options(args) {
        Ok(o) => o,
        Err(e) if e == "не указан проект" => parse_run_options(&[args.to_vec(), vec!["".into()]].concat())?,
        Err(e) => return Err(e),
    };
    // если рядом есть сборка IDE (app/dist), отдаётся она
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let static_dir = ["app/dist", "../app/dist", "../../app/dist", "../../../app/dist"]
        .iter()
        .flat_map(|rel| {
            let mut v = vec![PathBuf::from(rel)];
            if let Some(d) = &exe_dir {
                v.push(d.join(rel));
            }
            v
        })
        .find(|p| p.join("index.html").exists());
    stratum_core::player::serve(stratum_core::player::Options {
        project: opts.path,
        libraries: opts.libraries,
        port: opts.port,
        fps: 30,
        static_dir,
        assets: None,
        token: None,
    })
}

fn safe_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect()
}

fn cmd_parse(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("не указан файл".into());
    }
    for arg in args {
        let path = Path::new(arg);
        let data = std::fs::read(path).map_err(|e| format!("{arg}: {e}"))?;
        let text = if data.starts_with(b"SB") {
            cls::parse(&data, arg).map_err(|e| e.to_string())?.text
        } else {
            String::from_utf8_lossy(&data).into_owned()
        };
        let model = lang::parse(&text).map_err(|e| format!("{arg}: {e}"))?;
        println!("{arg}: операторов {}, объявлений {}", model.body.len(), model.declarations.len());
        for stmt in &model.body {
            println!("  {stmt:?}");
        }
    }
    Ok(())
}

fn cmd_info(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(args.first().ok_or("не указан проект")?);
    let loaded = formats::load_project(&path, &formats::default_library_dirs())
        .map_err(|e| format!("{}: {e}", path.display()))?
        .map_err(|e| e.to_string())?;
    println!("корневой имидж: {}", loaded.project.root);
    for p in &loaded.project.properties {
        println!("свойство {}: {:?}", p.key, p.value);
    }
    println!("\nимиджи проекта:");
    for c in &loaded.classes[..loaded.own_classes] {
        println!(
            "  {:<28} переменных {:>3}, детей {:>3}, связей {:>3}, строк текста {:>4}",
            c.name,
            c.vars.len(),
            c.children.len(),
            c.links.len(),
            c.text.lines().count()
        );
    }
    let sim = Simulation::build(&loaded).map_err(|e| e.to_string())?;
    println!("\nэкземпляры ({}):", sim.instances().len());
    for instance in sim.instances() {
        println!("  {} [{}]", instance.path, instance.class_name);
    }
    Ok(())
}

fn cmd_check(args: &[String]) -> Result<(), String> {
    let dir = PathBuf::from(args.first().ok_or("не указан каталог")?);
    let mut files = Vec::new();
    collect(&dir, &mut files).map_err(|e| e.to_string())?;
    files.sort();

    let (mut read_ok, mut parsed_ok) = (0u32, 0u32);
    let mut read_failed = Vec::new();
    let mut parse_failed = Vec::new();
    for file in &files {
        let data = match std::fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                read_failed.push(format!("{}: {e}", file.display()));
                continue;
            }
        };
        if !data.starts_with(b"SB") {
            continue;
        }
        match cls::parse(&data, &file.display().to_string()) {
            Ok(c) => {
                read_ok += 1;
                if c.text.trim().is_empty() {
                    parsed_ok += 1;
                    continue;
                }
                match lang::parse(&c.text) {
                    Ok(_) => parsed_ok += 1,
                    Err(e) => parse_failed.push(format!("{} [{}]: {e}", file.display(), c.name)),
                }
            }
            Err(e) => read_failed.push(e.to_string()),
        }
    }
    println!("прочитано имиджей: {read_ok}, тексты разобраны: {parsed_ok}");
    if !read_failed.is_empty() {
        println!("\nне прочитаны ({}):", read_failed.len());
        for e in read_failed.iter().take(20) {
            println!("  {e}");
        }
    }
    if !parse_failed.is_empty() {
        println!("\nне разобран текст ({}):", parse_failed.len());
        for e in parse_failed.iter().take(40) {
            println!("  {e}");
        }
    }
    if read_failed.is_empty() && parse_failed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} файлов не прочитано, {} текстов не разобрано",
            read_failed.len(),
            parse_failed.len()
        ))
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cls")) {
            out.push(path);
        }
    }
    Ok(())
}

/// Сверка нашего компилятора с компилятором оригинала: каждый `.cls` в
/// папке компилируется заново и сравнивается с его секцией 0x0d слово в
/// слово. `--show N` — вывести первые N расхождений подробно.
fn cmd_bytecode(args: &[String]) -> Result<(), String> {
    use stratum_core::lang::compile::{compile, Env, ImageFunction, Ty};
    let mut dir = None;
    let mut show = 0usize;
    let mut filter = String::new();
    let mut aliases = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--show" => {
                i += 1;
                show = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(5);
            }
            "--aliases" => aliases = true,
            "--only" => {
                i += 1;
                filter = args.get(i).cloned().unwrap_or_default();
            }
            other => dir = Some(PathBuf::from(other)),
        }
        i += 1;
    }
    let dir = dir.ok_or("нужна папка с .cls")?;
    let mut files = Vec::new();
    fn walk(d: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("cls")) {
                out.push(p);
            }
        }
    }
    walk(&dir, &mut files);
    files.sort();
    // имиджи-функции всех найденных .cls: параметры (флаг 0x200) и результат
    let mut image_functions: std::collections::HashMap<String, ImageFunction> = Default::default();
    for f in &files {
        let Ok(data) = std::fs::read(f) else { continue };
        let Ok(cls) = formats::cls::parse(&data, "") else { continue };
        let Ok(model) = lang::parse(&cls.text) else { continue };
        if model.is_function {
            image_functions.entry(lang::fold(&cls.name)).or_insert_with(|| lang::compile::image_function(&cls, &model));
        }
    }
    let (mut same, mut differ, mut failed, mut skipped, mut shown) = (0, 0, 0, 0, 0);
    // старое имя → найденные для него коды операций
    let mut votes: std::collections::BTreeMap<String, std::collections::BTreeMap<u16, usize>> = Default::default();
    let mut reasons: std::collections::BTreeMap<String, usize> = Default::default();
    let mut seen = std::collections::HashSet::new();
    for f in &files {
        let Ok(data) = std::fs::read(f) else { continue };
        let Ok(cls) = formats::cls::parse(&data, &f.display().to_string()) else { continue };
        let Some(bc) = &cls.bytecode else {
            skipped += 1;
            continue;
        };
        // одинаковые имиджи из разных примеров считаем один раз
        if !seen.insert((cls.text.clone(), bc.clone())) {
            continue;
        }
        if !filter.is_empty() && !f.display().to_string().contains(&filter) {
            continue;
        }
        let original: Vec<u16> = bc.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let known: Vec<(String, Ty)> = cls.vars.iter().map(|v| (v.name.clone(), Ty::from_name(&v.var_type))).collect();
        let model = match lang::parse(&cls.text) {
            Ok(m) => m,
            Err(e) => {
                failed += 1;
                *reasons.entry(format!("разбор: {}", e.message)).or_default() += 1;
                continue;
            }
        };
        let constants = |n: &str| stratum_core::runtime::constants::lookup(n);
        let functions = |n: &str| image_functions.get(&lang::fold(n)).cloned();
        let env = |fold_minus| Env { constant: &constants, function: &functions, fold_minus, placeholders: false };
        // сворачивание `-число` в корпусе встречается в обоих вариантах
        let result = compile(&model, &known, &env(true)).and_then(|a| {
            if a.code == original {
                Ok(a)
            } else {
                compile(&model, &known, &env(false)).map(|b| if b.code == original { b } else { a })
            }
        });
        match result {
            Ok(c) if c.code == original => same += 1,
            Ok(c) => {
                differ += 1;
                let at = c.code.iter().zip(&original).position(|(a, b)| a != b).unwrap_or(c.code.len().min(original.len()));
                *reasons.entry(format!("расхождение, оригинал: {:?}", &original[at.saturating_sub(0)..(at + 2).min(original.len())])).or_default() += 1;
                if shown < show {
                    shown += 1;
                    println!("== {}\n{}\nпеременные: {:?}", f.display(), cls.text.trim(), known.iter().map(|(n, t)| format!("{n}:{}", t.name())).collect::<Vec<_>>());
                    println!("ядро:     {:?}", c.code);
                    println!("оригинал: {:?}", original);
                    println!("первое расхождение в слове {at}\n");
                }
            }
            Err(e) => {
                failed += 1;
                if aliases {
                    let probe = Env { constant: &constants, function: &functions, fold_minus: true, placeholders: true };
                    for fold in [true, false] {
                        let Ok(c) = compile(&model, &known, &Env { fold_minus: fold, ..probe }) else { break };
                        if c.code.len() != original.len() {
                            continue;
                        }
                        let slots: Vec<usize> = c.code.iter().enumerate().filter(|(_, w)| **w == 0xFFFF).map(|(i, _)| i).collect();
                        let rest_same = c.code.iter().zip(&original).enumerate().all(|(i, (a, b))| a == b || slots.contains(&i));
                        if rest_same && slots.len() == c.unknown.len() {
                            for (at, name) in slots.iter().zip(&c.unknown) {
                                *votes.entry(lang::fold(name)).or_default().entry(original[*at]).or_default() += 1;
                            }
                            break;
                        }
                    }
                }
                *reasons.entry(e.message.split(':').next().unwrap_or("").to_string() + ": " + e.message.split(':').nth(1).unwrap_or("").trim()).or_default() += 1;
                if shown < show && filter.len() > 0 {
                    shown += 1;
                    println!("== {} — ошибка: {} (строка {})", f.display(), e.message, e.line);
                }
            }
        }
    }
    if aliases {
        // код операции → нынешнее имя функции
        let names = stratum_core::lang::compile::function_names();
        let mut found = Vec::new();
        for (old, codes) in &votes {
            let (code, _) = codes.iter().max_by_key(|(_, n)| **n).unwrap();
            match names.get(code) {
                Some(new) => found.push(format!("  {:?}: {:?}", old, new)),
                None => println!("старое имя {old}: код {code} не найден в таблице"),
            }
        }
        println!("{{\n{}\n}}", found.join(",\n"));
    }
    let total = same + differ + failed;
    println!("совпало слово в слово: {same} из {total}; расходится: {differ}; не скомпилировано: {failed}; без байт-кода: {skipped}");
    let mut top: Vec<_> = reasons.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    for (r, n) in top.iter().take(25) {
        println!("  {n:4}  {r}");
    }
    Ok(())
}

/// Подготовка примера к сверке траектории с оригиналом: в начало текста
/// корневого имиджа добавляется счётчик тактов; в начале такта N+1 модель
/// сохраняет своё состояние (`SaveObjectState`) и завершается (`Quit`).
/// Остальные имиджи экспортируются с родным байт-кодом оригинала.
///   stratum instrument проект папка --ticks 100 --state C:\\verify\\state.stt
fn cmd_instrument(args: &[String]) -> Result<(), String> {
    let mut paths = Vec::new();
    let mut ticks = 100u32;
    let mut state = String::from("C:\\verify\\state.stt");
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ticks" => {
                i += 1;
                ticks = args.get(i).and_then(|v| v.parse().ok()).ok_or("--ticks: нужно число")?;
            }
            "--state" => {
                i += 1;
                state = args.get(i).cloned().ok_or("--state: нужен путь")?;
            }
            other => paths.push(PathBuf::from(other)),
        }
        i += 1;
    }
    let [src, dest] = paths.as_slice() else { return Err("нужно два пути: проект и папка назначения".into()) };
    let mut loaded = formats::load_project(src, &formats::default_library_dirs()).map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    let root = loaded.project.root.clone();
    let cls = loaded.classes[..loaded.own_classes].iter_mut().find(|c| lang::same_name(&c.name, &root)).ok_or("нет корневого имиджа среди имиджей проекта")?;
    let probe = format!(
        "__tick := __tick + 1\r\nif (__tick > {ticks})\r\n __r := SaveObjectState(\"\", \"{state}\")\r\n Quit(1)\r\nendif\r\n"
    );
    cls.text = probe + &cls.text;
    for (name, ty) in [("__tick", "FLOAT"), ("__r", "FLOAT")] {
        if !cls.vars.iter().any(|v| v.name == name) {
            cls.vars.push(formats::Variable { name: name.into(), description: String::new(), default: String::new(), var_type: ty.into(), flags: 0x20000 });
        }
    }
    formats::native::export_stratum2000(dest, &loaded).map_err(|e| e.to_string())?;
    // файлы модели рядом с проектом (меню .mnu, матрицы, тексты, картинки,
    // вложенные папки) нужны оригиналу по относительным путям
    let src_dir = if src.is_dir() { src.clone() } else { src.parent().map(PathBuf::from).unwrap_or_default() };
    copy_resources(&src_dir, dest).map_err(|e| e.to_string())?;
    // без байт-кода корня оригинал не выполнит счётчик — проверяем
    let data = std::fs::read(std::fs::read_dir(dest).map_err(|e| e.to_string())?.flatten().map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("cls")) && formats::cls::parse(&std::fs::read(p).unwrap_or_default(), "").is_ok_and(|c| lang::same_name(&c.name, &root)))
        .ok_or("корневой имидж не записан")?).map_err(|e| e.to_string())?;
    let written = formats::cls::parse(&data, "").map_err(|e| e.to_string())?;
    if written.bytecode.is_none() {
        return Err(format!("текст корневого имиджа {root} не компилируется — сверка невозможна"));
    }
    println!("подготовлено: {} имиджей, сохранение состояния на такте {} → {state}", loaded.own_classes, ticks);
    Ok(())
}

/// Копирует в `dest` всё, кроме имиджей и файлов проекта, чего там ещё нет
/// (имена сравниваются без регистра, как в Windows).
fn copy_resources(src: &Path, dest: &Path) -> std::io::Result<()> {
    let existing: Vec<String> = std::fs::read_dir(dest)?.flatten().map(|e| e.file_name().to_string_lossy().to_lowercase()).collect();
    for entry in std::fs::read_dir(src)?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if existing.contains(&name) {
            continue;
        }
        if path.is_dir() {
            std::fs::create_dir_all(dest.join(entry.file_name()))?;
            copy_resources(&path, &dest.join(entry.file_name()))?;
        } else if !(name.ends_with(".cls") || name.ends_with(".spj") || name.ends_with(".prj")) {
            std::fs::copy(&path, dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Сравнение двух снимков .stt (оригинал и ядро): переменные по имиджам,
/// числа — с относительным допуском.
///   stratum sttdiff оригинал.stt ядро.stt [--tol 1e-9]
fn cmd_sttdiff(args: &[String]) -> Result<(), String> {
    let mut paths = Vec::new();
    let mut tol = 1e-9f64;
    let mut all = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => all = true,
            "--tol" => {
                i += 1;
                tol = args.get(i).and_then(|v| v.parse().ok()).ok_or("--tol: нужно число")?;
            }
            other => paths.push(PathBuf::from(other)),
        }
        i += 1;
    }
    let [a, b] = paths.as_slice() else { return Err("нужно два файла .stt".into()) };
    let read = |p: &PathBuf| -> Result<formats::State, String> {
        let d = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        formats::project::parse_state(&d, &p.display().to_string()).map_err(|e| e.to_string())
    };
    let (orig, ours) = (read(a)?, read(b)?);
    // экземпляры одного имиджа с тем же handle встречаются в разных ветвях
    // (у каждого шара свой Engine#3): сопоставляем k-е вхождение с k-м —
    // оба снимка пишутся в одном обходе дерева
    let keyed = |images: &[formats::project::StateImage]| -> Vec<((String, u16), usize)> {
        let mut seen: std::collections::HashMap<(String, u16), usize> = std::collections::HashMap::new();
        images.iter().map(|img| {
            let k = (lang::fold(&img.class_name), img.handle);
            let n = seen.entry(k.clone()).or_insert(0);
            *n += 1;
            (k, *n)
        }).collect()
    };
    let (orig_keys, our_keys) = (keyed(&orig.images), keyed(&ours.images));
    let (mut same, mut differ, mut missing) = (0, 0, 0);
    // типы переменных: дескрипторы считаем отдельно — их нумерация своя
    let only_numbers = std::env::var_os("STT_SKIP").map(|v| v.to_string_lossy().to_string()).unwrap_or_default();
    let mut worst: Vec<(f64, String)> = Vec::new();
    for (img, k) in orig.images.iter().zip(&orig_keys) {
        let Some(other) = our_keys.iter().position(|o| o == k).map(|j| &ours.images[j]) else {
            missing += 1;
            continue;
        };
        for (name, value) in &img.vars {
            let Some((_, v2)) = other.vars.iter().find(|(n, _)| lang::same_name(n, name)) else { continue };
            if !only_numbers.is_empty() && only_numbers.split(',').any(|p| name.to_lowercase().starts_with(p) || name.to_lowercase().starts_with(&format!("_{p}"))) {
                continue;
            }
            // дескрипторы ядро пишет как «#n», оригинал — числом
            let (value, v2) = (&value.trim_start_matches('#').to_string(), &v2.trim_start_matches('#').to_string());
            let eq = match (value.parse::<f64>(), v2.parse::<f64>()) {
                (Ok(x), Ok(y)) => (x - y).abs() <= tol * x.abs().max(y.abs()).max(1.0),
                _ => value == v2,
            };
            if eq {
                same += 1;
            } else {
                differ += 1;
                let d = match (value.parse::<f64>(), v2.parse::<f64>()) {
                    (Ok(x), Ok(y)) => (x - y).abs() / x.abs().max(y.abs()).max(1e-300),
                    _ => f64::INFINITY,
                };
                let nth = if k.1 > 1 { format!("/{}", k.1) } else { String::new() };
                worst.push((d, format!("{}#{}{nth} {name}: оригинал {value}, ядро {v2}", img.class_name, img.handle)));
            }
        }
    }
    worst.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
    println!("переменных совпало: {same}; расходится: {differ}; имиджей оригинала без пары в ядре: {missing}");
    for (_, w) in worst.iter().take(if all { usize::MAX } else { 15 }) {
        println!("  {w}");
    }
    if differ > 0 { Err("снимки расходятся".into()) } else { Ok(()) }
}
