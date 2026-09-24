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
| Сверка с оригиналом: меню, панели, диалоги, 960 функций — что есть, чего нет | [`docs/audit.md`](docs/audit.md) |
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
(Alt+щелчок в окне модели: положение, линия, заливка, точки), редактор
графики имиджа и иконки (вкладки «Рисунок», «Иконка»: линии, фигуры, текст,
группы, порядок, точки — с записью в `.vdr`), вкладка «Граф»
— граф зависимостей переменных схемы (связи и присваивания в текстах), редактор кода (Monaco, грамматика
языка, подсказки по 960 функциям и константам, ошибки разбора в строке),
инспектор переменных с живыми значениями, окно модели, транспорт, тёмная тема,
сохранение проекта в текстовом формате и экспорт в Stratum 2000 (см.
[`docs/formats/native.md`](docs/formats/native.md)).
Для разработки фронтенда — `npm run dev` (проксирует API в ядро на 8765).

Стек по ТЗ: React 19 + TypeScript + Vite, Zustand, Monaco.

## Установка и запуск

Десктопная программа `stratum-modern` — одна программа: ядро и IDE вшиты
в неё, отдельно ставить Rust, Node или папку `app/dist` не нужно. При запуске
она поднимает ядро на свободном порту `127.0.0.1` и открывает IDE в своём
окне (Tauri 2).

### Готовые установщики

Собираются в GitHub Actions (workflow `release`): при запуске вручную —
артефактами запуска, по метке `v*` — ещё и черновиком релиза.

| Система | Файл | Установка |
| --- | --- | --- |
| Debian, Ubuntu | `Stratum Modern_…_amd64.deb` | `sudo apt install ./Stratum*.deb` |
| Другие Linux | `Stratum Modern_…_amd64.AppImage` | `chmod +x Stratum*.AppImage` и запустить |
| Windows 10/11 | `…_x64_en-US.msi` или `…_x64-setup.exe` | запустить установщик |
| macOS 10.15+ (Intel и Apple Silicon) | `…_universal.dmg` | перетащить в «Программы» |

Установщики не подписаны: Windows SmartScreen предложит «Подробнее →
Выполнить в любом случае», macOS при первом запуске — открыть программу
через контекстное меню «Открыть». На Windows нужен WebView2 (в Windows 11 уже
есть, установщик докачает его сам).

```sh
stratum-modern                                  # пустая IDE, диалог «Открыть проект»
stratum-modern ~/stratum/solar_system           # сразу открыть проект
```

Библиотека и примеры оригинала в пакет не входят. Библиотечные имиджи
(`LGSpace`, `NumberView`…) ищутся в `$STRATUM_LIBRARY` (несколько папок через
`:`), затем в `fixtures/library` рядом с текущей папкой и в
`~/.wine32/drive_c/Program Files/Stratum`.

### Сборка из исходников

Нужны Rust (stable), Node.js 22 и npm; на Linux ещё
`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev` (Debian/Ubuntu), на
Windows — Microsoft C++ Build Tools, на macOS — Xcode Command Line Tools.

```sh
cd app
npm ci
npx tauri build                 # Linux: .deb и .AppImage
npx tauri build --bundles msi   # Windows
npx tauri build --bundles dmg   # macOS
```

Установщики появляются в `app/src-tauri/target/release/bundle/<тип>/`, сама
программа — `app/src-tauri/target/release/stratum-modern`. `tauri build`
сначала собирает IDE (`npm run build`) и вшивает её в программу.

Проверить собранный `.deb`, ничего не ставя в систему:

```sh
dpkg-deb -x "app/src-tauri/target/release/bundle/deb/Stratum Modern_0.1.0_amd64.deb" /tmp/stratum-deb
tools/smoke_desktop.sh /tmp/stratum-deb/usr/bin/stratum-modern   # без экрана — через xvfb-run
```

Скрипт запускает программу, берёт из её вывода ссылку и проверяет, что по
ней отдаётся IDE, а API без токена закрыт.

### Токен сессии локального сервера

Ядро (и `stratum play`, и десктопная программа) слушает только `127.0.0.1`,
но открытые в браузере чужие страницы тоже могут слать туда запросы. Поэтому
при каждом запуске создаётся случайный токен сессии (128 бит), и ядро
печатает ссылку с ним:

```
IDE: http://127.0.0.1:8765/?token=3f9c…  (Ctrl+C — выход)
```

- IDE открывается только по этой ссылке; сервер запоминает токен в cookie
  `HttpOnly; SameSite=Strict`, и дальше страница ходит с ним сама. Без токена
  API (`/frame`, `/event`, `/api/…`) отвечает 401;
- заголовок `Host` должен быть `127.0.0.1:порт` или `localhost:порт` (защита
  от DNS rebinding), `Origin`, если есть, — только свой; иначе 403;
- всё, что меняет состояние, принимается только методом POST.

Десктопная программа сама открывает окно по ссылке с токеном; на Linux и
macOS, запущенная из терминала, она печатает ту же ссылку, и её можно открыть
в обычном браузере. Для своих скриптов
токен передаётся заголовком `X-Stratum-Token` или параметром `?token=`.
Токен живёт до выхода из программы. Проверки — `core/src/player/guard.rs`,
тест на живом сервере — `core/tests/player_guard.rs`.

## Сверка поведения с оригиналом

Что сверено с настоящим Stratum 2000, а что проверено ядром само на себе, и
как повторить сверку (байт-код корпуса, траектории переменных через Wine) —
[`docs/verification.md`](docs/verification.md). Тесты ядра без корпуса
оригинала: `cd core && STRATUM_SKIP_CORPUS=1 cargo test` (так они идут в CI).

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
| `tools/dump_resources.py` | меню, диалоги и строки из ресурсов `r1251s32.dll` → `docs/original/` |
| `tools/check_functions.py` | функции таблиц, для которых в ядре нет ветки |

## Сверка с оригиналом

Эталон — ресурсы `r1251s32.dll` (меню и диалоги), снятые
`tools/dump_resources.py` в `docs/original/`, и справка `SC3.HLP`. Таблица
«что есть, что иначе, чего нет» — `docs/audit.md`. Главное меню среды
повторяет оригинал (Файл, Правка, Вид, Вставка, Формат, Моделирование,
Параметры, Окна, Помощь); пункты, которым в новой среде нет места, показаны
выключенными с объяснением.

## Дальше по ТЗ

Из этапа 5 не сделано: текстуры, `CreateObjectFromFile3d` (v3d),
`SweepAndExtrude3d`, Ogre3D (плагин). Не закрыто из ранних этапов: сверка
значений на такте 100 с оригиналом через Wine (нужен человек за оригиналом);
байт-код секций `0x0d`/`0x1e`; старая редакция `.stt`; базы данных DBF.
Из интерфейса оригинала: контактные площадки, параметры печати, чтение
гиперссылок из `.vdr`.

## Лицензия

Личный некоммерческий проект. Stratum 2000 — Пермский ГТУ, О. И. Мухин,
В. А. Носков, А. А. Шелемехов; рег. №980665 от 18.09.1998. Файлы оригинала не
распространяются вместе с этим репозиторием.
