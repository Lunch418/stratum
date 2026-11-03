#!/usr/bin/env python3
"""Опыт: какие значения _enable включают имидж (связь MakeFace3D.Mat -> Transformer._enable
даёт -1, а в оригинале Transformer при этом не исполняется).

    python3 tools/verify_enable_values.py
"""
import json, sys
import verify_original as vo

VALUES = ['-1', '-2', '0.5', '1', '2', '0', 'd-1', 'd0.5', 'd1', 'd2', 'd0']


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Rcv']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')

    def cls(name, vars, text, kids=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F = lambda n: vo.VAR(n, 'FLOAT')
    cls('Rcv', [F('n'), F('_enable')], 'n := ~n + 1\n')
    kids = []
    for i, v in enumerate(VALUES):
        key = 'dis' if v.startswith('d') else 'en'
        cls(f'R{i}', [F('n'), dict(F('_disable' if key == 'dis' else '_enable'), default=v.lstrip('d'))], 'n := ~n + 1\n')
        kids.append((f'R{i}', f'R{i}', i + 1))
    cls('Main', vo.PROBE_VARS, text, kids=kids)
    names += [f'R{i}' for i in range(len(VALUES))]
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')


vo.native_project = project
vo.WAIT = 5
items = [(f'v{v}', f'GetVarF("R{i}", "n")') for i, v in enumerate(VALUES)]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-enval-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}')
sys.exit(0 if ours == theirs else 1)
