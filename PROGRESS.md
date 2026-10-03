# Прогресс Stratum Modern

ТЗ: stratum-modern-tz.md. Подробные таблицы: docs/audit.md (функции среды),
docs/verification.md (сверка с оригиналом в Wine), docs/review.md (ревью
безопасности и корректности).

## Сделано
- Ядро: язык, компилятор в байт-код 0x0d, интерпретатор, форматы .spj/.cls/.vdr/.stt/.dbm,
  2D-графика, 3D-пространства в рисунках, окна (MDI-модель оригинала), сообщения, гипербаза.
- Сверка с оригиналом (20 тактов, все значения совпали): BALLS, BALLS2, DIFF, GIST, L1, L2, L4,
  MOUSE, PHISICS, PixelAccess, linear_brush, VIDEO, Trigger, solar_system.
  Почти: ROBOT 2614/2615, EDS_IND 915/916, L3 771/774, T80, IRONCLAD.
- Среда (app/): все пункты docs/audit.md, кроме видео, просмотра БД и Ogre3D.
- Дизайн: система токенов "чертежный лист", светлая/темная тема, AA-контраст,
  клавиатурная навигация (app/src/a11y.ts), узкие окна.
- Безопасность: токен сессии, Host/Origin, песочница файлов модели, фаззинг
  разборщиков (core/tests/fuzz_parsers.rs), 21 находка исправлена (docs/review.md).
- Сборка: CI (Linux/Windows/macOS), release.yml -> .deb/.AppImage/.msi/.exe/.dmg.

## В работе
- Сверка: ENGINE; начат обратный разборщик байт-кода (--decompile в `stratum bytecode`,
  незакоммичен, модуля lang/decompile.rs еще нет).

## Дальше
- Сверка: ENGINE, ANATOMY, Surf3d, ROBOT2, SMO2, AudioPlayer, HIST3D, WindowRegion,
  VIDEO2, Osc3d, Example.33, WRITEAVI, MENU; хвосты ROBOT, EDS_IND, L3, T80, IRONCLAD.
- 3D: источники света и материалы формата 2.x.
- Гипербаза: номера полей окна/объекта/эффекта не сверены (в корпусе нет примеров).
- Решение владельца: лицензия (Cargo.toml MIT против "личный некоммерческий"),
  подпись установщиков, тег релиза.

## Известные баги
- Без снимка в Wine: sclogo (падает оригинал), NUI и TextAnalyser (внешние библиотеки),
  2d/api/Net (нет файла проекта).
- Нет движка: видеокадры, просмотр БД (DBF), Ogre3D - вызовы считаются и показываются
  плашкой "функции, которых здесь нет".
- Данные листа из рисунков 2.x теряются при записи в 3.x; старый DEFAULT.STT не читается.
