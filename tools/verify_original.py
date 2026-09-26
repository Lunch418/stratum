#!/usr/bin/env python3
"""Сверка ядра с настоящим Stratum 2000 через Wine.

Из файла проб (`tools/verify/*.txt`, строки `имя | выражение`) собирается
проект из одного имиджа. Его текст в первом такте вычисляет все выражения,
пишет строки `имя=значение` в файл и вызывает Quit(1). Тот же проект
выполняется ядром (`stratum run`) и оригиналом (`SC200032.EXE проект /run`),
ответы сравниваются.

Строка `! оператор` выполняется один раз на первом такте, а пробы — через
N тактов после него (`% Wait = N`): так проверяется то, что оригинал делает
не сразу (окна получают размеры сообщением WM_SIZE уже после открытия).

Строка `@ папка Имя` добавляет в проект готовый имидж из родного проекта
(`stratum convert`) вместе с рисунками — для проб окон и графики. Папка
берётся от корня репозитория или абсолютная.

Префикс `s:` у выражения — результат уже строка (без String()).
Строка `> оператор` вставляется в текст как есть (объявления, присваивания)
перед следующими пробами — так проверяется семантика, а не только функции.

    python3 tools/verify_original.py tools/verify/strings.txt

С ключом --compiler текст пробы компилирует сам оригинал: главный имидж
читает его из файла и передаёт в SetModelText имиджу Probe на своей схеме.
Так проверяются и решения компилятора (регистр имён, приоритеты), а не
только исполнение нашего байт-кода.

Нужны: собранное ядро (core/target/debug/stratum), Wine с установленным
Stratum 2000 (по умолчанию ~/.wine32, STRATUM_EXE/WINEPREFIX — переопределить).
"""
import json, os, pathlib, shutil, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / 'core' / 'target' / 'debug' / 'stratum'
PREFIX = pathlib.Path(os.environ.get('WINEPREFIX', pathlib.Path.home() / '.wine32'))
EXE = os.environ.get('STRATUM_EXE', r'C:\Program Files\Stratum\SC200032.EXE')
WORK = PREFIX / 'drive_c' / 'verify'


def probes(path):
    out = []
    for line in pathlib.Path(path).read_text(encoding='utf-8').splitlines():
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        if line.startswith('!'):
            SETUP.append(line[1:].strip())
            continue
        if line.startswith('%'):
            key, value = [p.strip() for p in line[1:].split('=', 1)]
            if key == 'Wait':
                global WAIT
                WAIT = int(value)
                continue
            PROPERTIES.append({'key': key, 'int': int(value)})
            continue
        if line.startswith('>'):
            out.append((None, line[1:].strip()))
            continue
        if line.startswith('+'):
            FILES.append(ROOT / line[1:].strip())
            continue
        if line.startswith('@'):
            folder, name = line[1:].split()
            EXTRA_CLASSES.append((ROOT / folder, name))
            continue
        name, expr = [p.strip() for p in line.split('|', 1)]
        out.append((name, expr))
    return out


def model_text(items, out_path):
    lines = ['if (tick == 0)'] + [' ' + st for st in SETUP] + ['endif', 'tick := tick + 1'] if SETUP or WAIT else []
    lines += [
        f'if (done == 0 && tick > {WAIT})' if SETUP or WAIT else 'if (done == 0)',
        f' h := CreateStream("FILE", "{out_path}", "CREATE")',
    ]
    for name, expr in items:
        if name is None:
            lines.append(' ' + expr)
            continue
        value = expr[2:].strip() if expr.startswith('s:') else f'String({expr})'
        lines.append(f' r := WriteLn(~h, "{name}=" + {value})')
    lines += [' r := CloseStream(~h)', ' done := 1', ' Quit(1)', 'endif']
    return '\n'.join(lines) + '\n'


VAR = lambda n, t: {'name': n, 'type': t, 'default': '', 'description': '', 'flags': 0}
PROBE_VARS = [VAR('h', 'HANDLE'), VAR('r', 'FLOAT'), VAR('done', 'FLOAT'), VAR('tick', 'FLOAT')]


def write_class(dir, name, vars, text, children=()):
    kids = [{'class': c, 'handle': i + 1, 'name': '', 'x': 0, 'y': 0, 'flags': 0} for i, c in enumerate(children)]
    (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': kids, 'links': []}), encoding='utf-8')
    (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')


# Свойства проекта для проб: строка `% MathMode = 3` в файле проб
PROPERTIES = []
# Файлы для модели (`+ путь`): кладутся в C:\verify\ — рядом с out.txt
FILES = []
# Готовые имиджи из родных проектов: строка `@ папка Имя`
EXTRA_CLASSES = []
# Операторы первого такта (`! оператор`) и задержка проб в тактах (`% Wait = N`)
SETUP = []
WAIT = 0


def native_project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    classes = [{'name': 'Main', 'file': 'Main'}] + ([{'name': 'Probe', 'file': 'Probe'}] if compiler else [])
    for folder, name in EXTRA_CLASSES:
        if not (folder / 'project.json').exists():
            native = dir.parent / f'import-{folder.name}'
            if not native.exists():
                subprocess.run([str(CORE), 'convert', str(folder), str(native)], capture_output=True, check=True)
            folder = native
        # имидж вместе с детьми его схемы из той же папки (библиотечных
        # детей оригинал найдёт в своих библиотеках)
        todo, seen = [name], set()
        while todo:
            n = todo.pop()
            meta = folder / 'classes' / f'{n}.strat.json'
            if n in seen or not meta.exists():
                continue
            seen.add(n)
            for f in (folder / 'classes').glob(f'{n}.*'):
                shutil.copy(f, dir / 'classes' / f.name)
            classes.append({'name': n, 'file': n})
            todo += [c['class'] for c in json.loads(meta.read_text(encoding='utf-8'))['children']]
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': PROPERTIES, 'variables': [], 'libraries': [], 'classes': classes}), encoding='utf-8')
    if not compiler:
        # имидж, которого нет в дереве, оригинал не загружает — ставим
        # импортированные имиджи на схему главного
        write_class(dir, 'Main', PROBE_VARS, text, children=[n for _, n in EXTRA_CLASSES])
        return
    # Main передаёт текст пробы компилятору оригинала; Probe его исполняет
    driver = '\n'.join([
        'if (done == 0)',
        ' hs := CreateStream("FILE", "C:\\verify\\probe.mdl", "READONLY")',
        ' ok := SetModelText("Probe", ~hs, 0)',
        ' r := CloseStream(~hs)',
        ' hl := CreateStream("FILE", "C:\\verify\\compile.txt", "CREATE")',
        ' r := WriteLn(~hl, "compiled=" + String(~ok))',
        ' r := CloseStream(~hl)',
        ' done := 1',
        'endif', ''])
    write_class(dir, 'Main', [VAR('hs', 'HANDLE'), VAR('hl', 'HANDLE'), VAR('ok', 'FLOAT'), VAR('r', 'FLOAT'), VAR('done', 'FLOAT')], driver, children=['Probe'])
    write_class(dir, 'Probe', PROBE_VARS, '')


def parse(path):
    if not path.exists():
        return None
    raw = path.read_bytes()
    text = raw.decode('cp1251', errors='replace')
    return dict(l.split('=', 1) for l in text.replace('\r', '').splitlines() if '=' in l)


def run_core(items, work):
    proj = work / 'core_project'
    native_project(proj, model_text(items, r'C:\verify\out.txt'))
    # диск C: модели ядро отображает в папку проекта — папка должна быть
    (proj / 'verify').mkdir(exist_ok=True)
    for f in FILES:
        shutil.copy(f, proj / 'verify' / f.name)
    subprocess.run([str(CORE), 'run', str(proj), '--ticks', str(WAIT + 2)], capture_output=True, text=True)
    # диск C: модели ядро отображает в папку проекта
    return parse(proj / 'verify' / 'out.txt')


def run_original(items, work, timeout=60, compiler=False):
    src = work / 'orig_native'
    text = model_text(items, r'C:\verify\out.txt')
    native_project(src, text, compiler)
    if WORK.exists():
        shutil.rmtree(WORK)
    r = subprocess.run([str(CORE), 'convert', str(src), str(WORK), '--to', 'stratum2000'], capture_output=True, text=True)
    if r.returncode:
        sys.exit('экспорт не удался: ' + r.stderr)
    # без байт-кода оригинал исполнит пустоту — значит, текст не скомпилирован
    d = subprocess.run([sys.executable, str(ROOT / 'tools' / 'disasm.py'), str(WORK / 'Main.cls')], capture_output=True, text=True)
    if d.returncode:
        sys.exit('модель не скомпилирована нашим компилятором (оригинал её тоже не примет): проверьте пробы\n' + r.stderr)
    for f in FILES:
        shutil.copy(f, WORK / f.name)
    if compiler:
        (WORK / 'probe.mdl').write_bytes(text.replace('\n', '\r\n').encode('cp1251'))
    out = WORK / 'out.txt'
    env = dict(os.environ, WINEPREFIX=str(PREFIX), WINEDEBUG='-all')
    proc = subprocess.Popen(['wine', EXE, r'C:\verify\project.spj', '/run'], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    start = time.time()
    while time.time() - start < timeout:
        if out.exists() and proc.poll() is not None:
            break
        time.sleep(1)
    if proc.poll() is None:
        proc.kill()
        subprocess.run(['wineserver', '-k'], env=env)
    if compiler:
        log = parse(WORK / 'compile.txt') or {}
        # справка обещает 1 при успехе, на деле бывает и 2; 0 — ошибка
        code = log.get('compiled', '0')
        print('компилятор оригинала:', f'текст принят (SetModelText = {code})' if code not in ('0', '') else 'ошибка компиляции')
    return parse(out)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    compiler = '--compiler' in sys.argv
    if not args:
        sys.exit(__doc__)
    items = probes(args[0])
    work = pathlib.Path(os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-verify-{os.getpid()}'
    work.mkdir(parents=True, exist_ok=True)
    ours = run_core(items, work) or {}
    theirs = run_original(items, work, compiler=compiler)
    if theirs is None:
        sys.exit('оригинал не записал ответ: проверьте Wine и путь к SC200032.EXE')
    bad = 0
    items = [(n, e) for n, e in items if n is not None]
    for name, expr in items:
        a, b = ours.get(name, '—'), theirs.get(name, '—')
        mark = 'ok ' if a == b else 'РАЗНИЦА'
        bad += a != b
        print(f'{mark:8} {name:18} ядро={a!r:22} оригинал={b!r:22} {expr}')
    print(f'\nсовпало {len(items) - bad} из {len(items)}')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
