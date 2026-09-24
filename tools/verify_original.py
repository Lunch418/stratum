#!/usr/bin/env python3
"""Сверка ядра с настоящим Stratum 2000 через Wine.

Из файла проб (`tools/verify/*.txt`, строки `имя | выражение`) собирается
проект из одного имиджа. Его текст в первом такте вычисляет все выражения,
пишет строки `имя=значение` в файл и вызывает Quit(1). Тот же проект
выполняется ядром (`stratum run`) и оригиналом (`SC200032.EXE проект /run`),
ответы сравниваются.

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
        if line.startswith('>'):
            out.append((None, line[1:].strip()))
            continue
        name, expr = [p.strip() for p in line.split('|', 1)]
        out.append((name, expr))
    return out


def model_text(items, out_path):
    lines = [
        'if (done == 0)',
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
PROBE_VARS = [VAR('h', 'HANDLE'), VAR('r', 'FLOAT'), VAR('done', 'FLOAT')]


def write_class(dir, name, vars, text, children=()):
    kids = [{'class': c, 'handle': i + 1, 'name': '', 'x': 0, 'y': 0, 'flags': 0} for i, c in enumerate(children)]
    (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': kids, 'links': []}), encoding='utf-8')
    (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')


def native_project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    classes = [{'name': 'Main', 'file': 'Main'}] + ([{'name': 'Probe', 'file': 'Probe'}] if compiler else [])
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [], 'classes': classes}), encoding='utf-8')
    if not compiler:
        write_class(dir, 'Main', PROBE_VARS, text)
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
    subprocess.run([str(CORE), 'run', str(proj), '--ticks', '2'], capture_output=True, text=True)
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
        sys.exit('модель не скомпилирована нашим компилятором (оригинал её тоже не примет): проверьте пробы')
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
