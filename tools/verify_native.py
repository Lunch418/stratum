#!/usr/bin/env python3
"""Опыт на готовом примере: в корневой имидж дописывается журнал выражений на
такте N, затем пример исполняют оригинал (Wine) и ядро, строки сравниваются.

    python3 tools/verify_native.py fixtures/PROJECTS/samples/L3 --tick 4 \\
        'String(GetWindowOrgX("OSC"))' 'String(GetClientWidth("OSC"))'

Выражения должны давать строку. Пример переводится в родной формат
(`stratum convert`), корневой имидж получает переменные qq, lh, r.
"""
import json, os, pathlib, shutil, subprocess, sys, tempfile, time

import verify_original as vo


def main():
    args = sys.argv[1:]
    tick = 3
    if '--tick' in args:
        i = args.index('--tick')
        tick = int(args[i + 1])
        del args[i:i + 2]
    if len(args) < 2:
        sys.exit(__doc__)
    src, exprs = pathlib.Path(args[0]), args[1:]
    tmp = pathlib.Path(os.environ.get('VERIFY_TMP', tempfile.gettempdir())) / f'stratum-native-{os.getpid()}'
    proj = tmp / 'project'
    subprocess.run([str(vo.CORE), 'convert', str(src), str(proj)], capture_output=True, check=True)
    root = json.loads((proj / 'project.json').read_text(encoding='utf-8'))['root']
    cls = proj / 'classes'
    text = cls / f'{root}.strat'
    lines = ['', 'qq := ~qq + 1', f'if (~qq == {tick})', ' lh := CreateStream("FILE", "C:\\\\verify\\\\log.txt", "CREATE")']
    lines += [f' r := WriteLn(~lh, "{i}=" + {e})' for i, e in enumerate(exprs)]
    lines += [' r := CloseStream(~lh)', ' Quit(1)', 'endif', '']
    text.write_text((text.read_text(encoding='utf-8') if text.exists() else '') + '\n'.join(lines), encoding='utf-8')
    meta = cls / f'{root}.strat.json'
    j = json.loads(meta.read_text(encoding='utf-8'))
    names = {v['name'].lower() for v in j['vars']}
    j['vars'] += [vo.VAR(n, t) for n, t in [('qq', 'FLOAT'), ('lh', 'HANDLE'), ('r', 'FLOAT')] if n not in names]
    meta.write_text(json.dumps(j), encoding='utf-8')
    # оригинал
    if vo.WORK.exists():
        shutil.rmtree(vo.WORK)
    r = subprocess.run([str(vo.CORE), 'convert', str(proj), str(vo.WORK), '--to', 'stratum2000'], capture_output=True, text=True)
    if r.returncode:
        sys.exit('экспорт не удался: ' + r.stderr)
    env = dict(os.environ, WINEPREFIX=str(vo.PREFIX), WINEDEBUG='-all')
    proc = subprocess.Popen(['wine', vo.EXE, r'C:\verify\project.spj', '/run'], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    start = time.time()
    while time.time() - start < 60 and proc.poll() is None:
        time.sleep(1)
    if proc.poll() is None:
        proc.kill()
        subprocess.run(['wineserver', '-k'], env=env)
    theirs = vo.parse(vo.WORK / 'log.txt') or {}
    # ядро
    (proj / 'verify').mkdir(exist_ok=True)
    subprocess.run([str(vo.CORE), 'run', str(proj), '--ticks', str(tick + 2)], capture_output=True)
    ours = vo.parse(proj / 'verify' / 'log.txt') or {}
    same = 0
    for i, e in enumerate(exprs):
        a, b = ours.get(str(i), '—'), theirs.get(str(i), '—')
        same += a == b
        print(f'{"ok " if a == b else "РАЗНИЦА":8} ядро={a!r:24} оригинал={b!r:24} {e}')
    print(f'\nсовпало {same} из {len(exprs)}')
    sys.exit(0 if same == len(exprs) else 1)


if __name__ == '__main__':
    main()
