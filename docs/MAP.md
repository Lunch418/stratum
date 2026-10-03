# Карта кода Stratum Modern

Только справка для навигации; имена - без сигнатур. Обновляй при добавлении модулей.
Состав: core/ (Rust-ядро), app/ (React/TS IDE), app/src-tauri (оболочка), tools/ (Python), docs/.

## core/src (крейт stratum core)
- lib.rs - корень: pub mod formats, gfx, lang, player, runtime, sim
- bin/stratum.rs - CLI ядра (запуск, проверка, экспорт, декомпиляция)

### formats/ - чтение и запись файлов Stratum 2000 и родного формата
- mod.rs - реэкспорт подмодулей
- reader.rs / writer.rs - little-endian поля и строки с префиксом длины: Reader, Writer, FormatError
- cp1251.rs - перекодировка CP1251: decode, encode
- json.rs - свой JSON без крейтов: Json, parse, write_string, object
- cls.rs - имидж .cls (секции, переменные, связи): Variable, Child, Link, section
- project.rs - project.spj и _preload.stt: Project, parse_project, write_project, write_state
- vdr.rs - векторная графика .vdr и секции иконки/рисунка/схемы, 3D-данные: Picture, Space3dData, Object3dData, Hyper, parse_hyper
- native.rs - родной текстовый формат проекта: load, save, export_stratum2000, check_text, image_functions

### lang/ - язык моделирования
- lexer.rs - Tok, Token, tokenize
- parser.rs - parse, ParseError
- ast.rs - Expr, Stmt, Declaration, Model, BinOp
- compile.rs - компилятор текста в байт-код (секция 0x0d .cls): compile, image_function, Compiled
- opcodes.rs - таблица опкодов (генерируется tools/gen_opcodes.py)
- decompile.rs - байт-код -> текст
- mod.rs - fold, same_name (регистр имён), dependencies

### runtime/ - среда исполнения
- value.rs - Value, ValueType, format_g, format_number
- builtins.rs - встроенные функции: call, call_stub, Effects, DialogRequest
- extra.rs - строки, файлы, папки, потоки: call, Stream, Streams
- data.rs - матрицы и динамические массивы: Matrix, Matrices, multiply, transpose, determinant
- constants.rs - константы (PI, WM_*, стили окон): lookup (генерируется tools/gen_constants.py)
- clock.rs - часы: local_now, tick_count, hundredths

### sim/ - симуляция
- mod.rs - планировщик: дерево экземпляров, связи, такты, окна и сообщения (mod wm): Simulation, Instance, Registration, BuildError
- interp.rs - цикл исполнения текста имиджа в такте: Interpreter, Phase, Vars, Flow, RuntimeError
- equations.rs - решатель уравнений a = b и ? x, y: ClassEquations, collect, solve_step

### gfx/ - графика
- mod.rs - графическое пространство, объекты, инструменты, окна: Pen, Brush, Font
- api.rs - 2D-функции языка: call, insert_picture, bitmap_src
- api3d.rs - 3D-функции языка: call, find_by_name, set_visible
- space3d.rs - 3D-пространства, камеры, группы: Space3d, Object3d, Camera, Group3d, Prim
- svg.rs - рендер в SVG для снимков и проверки: render, render_in

### player/ - локальный HTTP-сервер IDE
- mod.rs - сервер, транспорт, живой кадр, остановки: Shared, Trace, Breakpoint, Halt
- api.rs - JSON-API проекта (тексты, переменные, схемы): handle, Response
- editor.rs - редактор графики имиджа (рисунок, схема, иконка): open, store, apply, handle
- guard.rs - защита сервера от чужих страниц: new_token, host_ok, origin_ok, token_ok

## core/tests
- corpus.rs - корпус из fixtures/ (локально) | equations.rs - решатель на проекте в памяти
- fuzz_parsers.rs - fuzz: битые .cls/.vdr/.spj не должны паниковать
- player_guard.rs - защита сервера на живом сервере | semantics.rs - семантика языка (регистр, фазы)

## app/src (React/TS IDE)
- main.tsx, App.tsx - вход и корневой макет | store.ts - состояние (useStore, classByName)
- api.ts - клиент JSON-API ядра (api) | hyper.ts - обработка гиперсобытий (handleHyper)
- menus.tsx - меню и команды | print.ts - печать | a11y.ts - доступность | autolink.ts - автоссылки | hooks.ts
- lang-data.json - данные языка для редактора
- components/: ModelView, SchemeCanvas (схема), Hierarchy, Inspector, ObjectProps/ObjectDialog, CodeEditor (Monaco), Debug, Messages, Graph/Graphs, PictureEditor, BitmapEditor, Palette, Help, Search, Options, MenuBar, Open/Link/Path/Print/ChooseClass/ProjectDialogs, Icon, ZoomSelect
## app/src-tauri/src
- main.rs - оболочка Tauri: поднимает ядро и WebView (WebviewWindowBuilder)

## tools/ (Python/sh)
- Форматы: cls_dump.py, spj_dump.py, vdr_dump.py, dbm2png.py, dump_resources.py, disasm.py, tpl2json.py
- Генерация: gen_opcodes.py, gen_constants.py, merge_functions.py, hlp2md.py, extract_texts.py, decompile_help.sh
- Проверки: check_corpus.py, check_functions.py, verify_links.py, verify_defaults.py, verify_messages.py
- Сверка с оригиналом (Wine): verify_original.py (пробы verify/*.txt), verify_native.py, verify_phases.py, verify_trajectory.py, wine_dialogs.py
- import_corpus.sh, smoke_desktop.sh

## docs/
- formats/{cls,spj,vdr,native}.md - форматы | lang/ - semantics.md, functions.md, *.json (справочники)
- original/ - тексты диалогов и меню оригинала | verification.md, audit.md, review.md | PROGRESS.md в корне

## Где искать
- Компилятор в байт-код: core/src/lang/compile.rs, опкоды opcodes.rs
- Цикл интерпретатора и фазы такта: sim/interp.rs, планировщик sim/mod.rs
- 2D графика: gfx/mod.rs, gfx/api.rs; SVG: gfx/svg.rs
- 3D пространства: gfx/space3d.rs, gfx/api3d.rs; данные: formats/vdr.rs
- Окна/MDI и сообщения: sim/mod.rs (mod wm), gfx/mod.rs (окна); UI: app/src/components/ModelView.tsx
- Гиперсвязи (hyperbase): formats/vdr.rs (Hyper), app/src/hyper.ts
- Форматы .spj/.stt: formats/project.rs; .cls: formats/cls.rs; .vdr: formats/vdr.rs; .dbm: tools/dbm2png.py, vdr.rs; родной: formats/native.rs
- Безопасность, HTTP-сервер: player/mod.rs, player/guard.rs, тест player_guard.rs; файлы из языка: runtime/extra.rs
- Fuzz: core/tests/fuzz_parsers.rs
- Сверка с Wine: tools/verify_original.py + tools/verify/*.txt, docs/verification.md
- CI: .github/workflows/ci.yml (cargo test, clippy, сборка IDE, oxlint, cargo check оболочки), release.yml
