# Stratum Modern

Современная реимплементация среды визуального моделирования Stratum 2000
(ПРОФИ, v3.01). Техническое задание — [`stratum-modern-tz.md`](stratum-modern-tz.md).

Выполнены **этапы 0–1** (справка, язык, форматы, ядро на Rust) и основная
часть **этапа 2**: графическое пространство, 2D-функции, окна модели,
сообщения мыши и клавиатуры, веб-плеер с транспортом. «Солнечная система»,
Pinball и другие 2D-примеры выглядят как в оригинале и крутятся живьём.

## Что уже есть

| Результат | Где |
| --- | --- |
| Спецификация формата имиджа `.cls` | [`docs/formats/cls.md`](docs/formats/cls.md) |
| Спецификация `project.spj` и `_preload.stt` | [`docs/formats/spj.md`](docs/formats/spj.md) |
| Спецификация векторной графики `.vdr` | [`docs/formats/vdr.md`](docs/formats/vdr.md) |
| Родной текстовый формат проекта (`project.json`, `*.strat`) и обмен со Stratum 2000 | [`docs/formats/native.md`](docs/formats/native.md) |
| Справочник встроенных функций (960 имён: типы аргументов, опкоды, описания) | `docs/lang/functions.md`, `functions.json` |
| Таблицы компилятора: функции, операторы с приоритетами, 414 констант, описатели DLL | `docs/lang/{builtins,operators,constants,tdl}.json` |
| Справка `SC3.HLP` целиком в markdown — 1329 тем с оглавлением и рисунками | `docs/help/` (собирается локально) |
| Тексты моделей всех 656 имиджей корпуса + индекс переменных | `docs/corpus/` |
| Инструменты разбора и конвертации | [`tools/`](tools/) |
| Ядро на Rust: форматы, язык, симуляция, CLI `stratum` | [`core/`](core/) |
| Уточнённая семантика языка (поправки к ТЗ: две фазы и `~`) | [`docs/lang/semantics.md`](docs/lang/semantics.md) |

Проверка на корпусе (`make check`):

- `.cls` — **660 из 660** файлов разбираются полностью, все 1376 записей
  оглавления сходятся с фактическим положением секций;
- `.spj` / `.stt` — 127 из 142; оставшиеся 15 — это `DEFAULT.STT` и снимки
  состояния в старой редакции, рабочие `_preload.stt` читаются все;
- в 582 текстах моделей не опознано лишь 11 имён функций (39 вызовов) — судя по
  всему, переименованные в поздних версиях вызовы в старых примерах.

## Ядро

```sh
cd core
cargo build --release
./target/release/stratum run ../fixtures/user/solar_system --ticks 100 \
    --watch StratumClass_0e3148_ce.xE --watch день.day
./target/release/stratum info ../fixtures/PROJECTS/samples/BALLS
./target/release/stratum check ../fixtures          # все .cls через оба парсера
./target/release/stratum render ../fixtures/PROJECTS/samples/PINBALL_ --ticks 50 --out /tmp/pinball
./target/release/stratum play ../fixtures/user/solar_system     # плеер: http://127.0.0.1:8765/
./target/release/stratum play                                    # IDE без проекта: диалог «Открыть»
./target/release/stratum convert ../fixtures/user/solar_system ~/solar   # импорт в project.json
./target/release/stratum convert ~/solar /tmp/solar-2000 --to stratum2000  # экспорт обратно
cargo test --release                                 # юнит-тесты + корпус
```

`play` поднимает локальный сервер и страницу с транспортом (Пуск/Пауза F5,
Шаг F10, Сброс Shift+F5, скорость), живым окном модели, мышью и
клавиатурой, которые доходят до имиджей через `RegisterObject`.

Библиотечные имиджи (`LGSpace`, `NumberView`…) ищутся в `--lib`, затем в
`$STRATUM_LIBRARY`, затем в `fixtures/library` и в установленном Stratum.

Модули: `formats` (`.cls`, `.spj`, `.stt`, `.vdr`, CP1251), `lang` (лексер,
парсер, AST), `runtime` (значения, 414 констант, встроенные функции), `gfx`
(графическое пространство, ~120 функций 2D и окон, SVG-рендер; `space3d` —
трёхмерные пространства, камеры, тела `3DTOOLS`, проекция в окно), `sim`
(дерево экземпляров, связи как общие ячейки, две фазы переменных, такты,
сообщения), `player` (веб-плеер). Внешних зависимостей нет.

## IDE

```sh
cd app && npm install && npm run build     # сборка в app/dist
cd ../core && ./target/release/stratum play ../fixtures/user/solar_system
```

Если рядом есть `app/dist`, `stratum play` отдаёт IDE вместо простого
плеера: иерархия проекта с иконками, холст схемы (панорама, зум, перетаскивание
блоков, вход в подсхему по двойному клику; имиджи добавляются перетаскиванием
из иерархии, связи — тягой от порта к блоку, правка пар по двойному клику,
удаление Delete, Undo/Redo Ctrl+Z / Ctrl+Shift+Z), командная палитра
(Ctrl+Shift+P), графики наблюдаемых переменных (∿ в инспекторе), отладка
(шаг назад по истории 60 тактов, условные точки останова, выражения-наблюдения,
пауза при ошибке с подсветкой строки, профиль такта), справка F1 по темам
`SC3.HLP` (нужна локальная сборка `make help`), фильтр сообщений,
автосохранение проектов в родном формате, живое редактирование (текст имиджа
подменяется в работающей модели без сброса), свойства графического объекта
(Alt+щелчок в окне модели: положение, линия, заливка, точки), вкладка «Граф»
— граф зависимостей переменных схемы (связи и присваивания в текстах), редактор кода (Monaco, грамматика
языка, подсказки по 960 функциям и константам, ошибки разбора в строке),
инспектор переменных с живыми значениями, окно модели, транспорт, тёмная тема,
сохранение проекта в текстовом формате и экспорт в Stratum 2000 (см.
[`docs/formats/native.md`](docs/formats/native.md)).
Для разработки фронтенда — `npm run dev` (проксирует API в ядро на 8765).

Стек по ТЗ: React 19 + TypeScript + Vite, Zustand, Monaco.

Десктопное окно (Tauri 2, нужны `libwebkit2gtk-4.1-dev` и `libgtk-3-dev`):

```sh
cd app && npm run build
cd src-tauri && cargo build --release
./target/release/stratum-modern                      # диалог «Открыть проект»
./target/release/stratum-modern ../../fixtures/user/solar_system
```

Оболочка поднимает ядро на свободном порту и открывает IDE в своём окне;
установщик `.deb` собирается командой `npx tauri build --bundles deb`
(появляется в `app/src-tauri/target/release/bundle/deb/`).

## Сборка данных

Ничего из оригинального Stratum в репозитории не лежит: `fixtures/` и всё
производное собирается локально из вашей установки (см. §10 ТЗ про лицензию).

```sh
make corpus                     # скопировать library, samples, template, help
make help HELPDECO=path/to/helpdeco   # декомпилировать SC3.HLP -> docs/help
make lang texts                 # таблицы языка и тексты моделей
make check                      # прогнать парсеры по всему корпусу
```

Ссылки вида `../help/topics/...` в `docs/lang/functions.md` начинают работать
после `make help`: сама справка в репозитории не лежит.

`make corpus` по умолчанию берёт `~/.wine32/drive_c/Program Files/Stratum`;
другой путь — `make corpus STRATUM=...`.

`helpdeco` в Ubuntu не пакетируется, собирается один раз:

```sh
git clone --depth 1 https://github.com/pmachapman/helpdeco
make -C helpdeco/gcc
```

Ещё нужны `python3` с Pillow (иконки и рисунки справки) и ImageMagick
(`convert`) для векторных рисунков WMF.

## Инструменты

| Инструмент | Назначение |
| --- | --- |
| `tools/cls_dump.py` | читает `.cls`; `--scan` прогоняет весь корпус и отчитывается |
| `tools/spj_dump.py` | читает `project.spj` и `_preload.stt` |
| `tools/tpl2json.py` | таблицы компилятора `template/*.tpl` и описатели `*.tdl` → JSON |
| `tools/hlp2md.py` | RTF от helpdeco → markdown-темы + `functions.json` |
| `tools/merge_functions.py` | сводит сигнатуры из таблиц с описаниями из справки |
| `tools/extract_texts.py` | тексты моделей всех имиджей → `docs/corpus/*.strat` |
| `tools/dbm2png.py` | наборы иконок `.dbm` → PNG (2203 иконки) |
| `tools/check_corpus.py` | ищет в текстах имена, которых нет в таблицах |

## Дальше по ТЗ

Этап 5 начат: 3D-пространства, камеры (`CreateDefCamera3d`, `CreateCamera3dEx`,
`TransformCamera3d`, `FitToCamera3d`), объекты из точек и примитивов, тела
`MakeBar/Cylinder/Tube/Sphere/Tore/Grid3d`, `CreateSurface3d`, локальные системы
координат, материалы как цвет, попадание `GetObject3dFromPoint2d`; рендер —
проекция в SVG окна модели (алгоритм художника, плоская подсветка). Не сделано:
текстуры, `CreateObjectFromFile3d` (v3d), `SweepAndExtrude3d`, Ogre3D. Из этапа 2 остались настоящие
контролы (Button/Edit/ListBox) и звук.

Не закрыто из этапов 0–1: сверка значений на такте 100 с оригиналом через
Wine; явный порядок вычислений; формат `.vdr`;
байт-код секций `0x0d`/`0x1e`; старая редакция `.stt`.

## Лицензия

Личный некоммерческий проект. Stratum 2000 — Пермский ГТУ, О. И. Мухин,
В. А. Носков, А. А. Шелемехов; рег. №980665 от 18.09.1998. Файлы оригинала не
распространяются вместе с этим репозиторием.
