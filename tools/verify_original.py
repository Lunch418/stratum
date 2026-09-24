#!/usr/bin/env python3
"""Сверка ядра с настоящим Stratum 2000 через Wine.

Из файла проб (`tools/verify/*.txt`, строки `имя | выражение`) собирается
проект из одного имиджа. Его текст в первом такте вычисляет все выражения,
пишет строки `имя=значение` в файл и вызывает Quit(1). Тот же проект
выполняется ядром (`stratum run`) и оригиналом (`SC200032.EXE проект /run`),
ответы сравниваются.

Префикс `s:` у выражения — результат уже строка (без String()).

    python3 tools/verify_original.py tools/verify/strings.txt

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
        name, expr = [p.strip() for p in line.split('|', 1)]
        out.append((name, expr))
    return out


def model_text(items, out_path):
    lines = [
        'if (done == 0)',
        f' h := CreateStream("FILE", "{out_path}", "CREATE")',
    ]
    for name, expr in items:
        value = expr[2:].strip() if expr.startswith('s:') else f'String({expr})'
        lines.append(f' r := WriteLn(~h, "{name}=" + {value})')
    lines += [' r := CloseStream(~h)', ' done := 1', ' Quit(1)', 'endif']
    return '\n'.join(lines) + '\n'


def native_project(dir, text):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    var = lambda n, t: {'name': n, 'type': t, 'default': '', 'description': '', 'flags': 0}
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [], 'classes': [{'name': 'Main', 'file': 'Main'}]}), encoding='utf-8')
    (dir / 'classes' / 'Main.strat.json').write_text(json.dumps({'name': 'Main', 'description': '', 'vars': [var('h', 'HANDLE'), var('r', 'FLOAT'), var('done', 'FLOAT')], 'children': [], 'links': []}), encoding='utf-8')
    (dir / 'classes' / 'Main.strat').write_text(text, encoding='utf-8')


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


def run_original(items, work, timeout=60):
    src = work / 'orig_native'
    native_project(src, model_text(items, r'C:\verify\out.txt'))
    if WORK.exists():
        shutil.rmtree(WORK)
    r = subprocess.run([str(CORE), 'convert', str(src), str(WORK), '--to', 'stratum2000'], capture_output=True, text=True)
    if r.returncode:
        sys.exit('экспорт не удался: ' + r.stderr)
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
    return parse(out)


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    items = probes(sys.argv[1])
    work = pathlib.Path(os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-verify-{os.getpid()}'
    work.mkdir(parents=True, exist_ok=True)
    ours = run_core(items, work) or {}
    theirs = run_original(items, work)
    if theirs is None:
        sys.exit('оригинал не записал ответ: проверьте Wine и путь к SC200032.EXE')
    bad = 0
    for name, expr in items:
        a, b = ours.get(name, '—'), theirs.get(name, '—')
        mark = 'ok ' if a == b else 'РАЗНИЦА'
        bad += a != b
        print(f'{mark:8} {name:18} ядро={a!r:22} оригинал={b!r:22} {expr}')
    print(f'\nсовпало {len(items) - bad} из {len(items)}')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
