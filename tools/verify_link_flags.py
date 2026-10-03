#!/usr/bin/env python3
"""Опыт: какие значения поля flags связи отключают связь. Для каждого flags пара
детей (x = 1 и x = 2) со связью; читаем x у первого: 2 - связь работает, 1 - нет.

    python3 tools/verify_link_flags.py
"""
import json, sys
import verify_original as vo

FLAGS = [0, 1, 2, 3, 4, 16, 513, 528, 32768, 295620, 196026275]
SELF = [0, 1, 513]  # связи между переменной главного имиджа p и ребёнком E


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'V1', 'V2']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')
    for n, d in (('V1', '1'), ('V2', '2')):
        var = dict(vo.VAR('x', 'FLOAT'), default=d)
        (dir / 'classes' / f'{n}.strat.json').write_text(json.dumps({'name': n, 'description': '', 'vars': [var], 'children': [], 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{n}.strat').write_text('x := x\n', encoding='utf-8')
    kids, links = [], []
    for i, fl in enumerate(FLAGS):
        kids += [{'class': 'V1', 'handle': 2 * i + 1, 'name': f'C{i}', 'x': 0, 'y': 0, 'flags': 0}, {'class': 'V2', 'handle': 2 * i + 2, 'name': f'D{i}', 'x': 0, 'y': 0, 'flags': 0}]
        links.append({'flags': fl, 'handle': 100 + i, 'source': 2 * i + 2, 'target': 2 * i + 1, 'vars': [['x', 'x']]})
    for j, fl in enumerate(SELF):
        h = 1000 + j
        kids.append({'class': 'V1', 'handle': h, 'name': f'E{j}', 'x': 0, 'y': 0, 'flags': 0})
        links.append({'flags': fl, 'handle': 200 + j, 'source': 0, 'target': h, 'vars': [['p', 'x']]})
        links.append({'flags': fl, 'handle': 300 + j, 'source': h, 'target': 0, 'vars': [['x', 'q']]})
    (dir / 'classes' / 'Main.strat.json').write_text(json.dumps({'name': 'Main', 'description': '', 'vars': vo.PROBE_VARS + [dict(vo.VAR('p', 'FLOAT'), default='7'), vo.VAR('q', 'FLOAT')], 'children': kids, 'links': links}), encoding='utf-8')
    (dir / 'classes' / 'Main.strat').write_text(text, encoding='utf-8')


vo.native_project = project
items = [(f'flags_{fl}', f'GetVarF("C{i}", "x")') for i, fl in enumerate(FLAGS)] + [(f'self_{fl}', f'GetVarF("E{j}", "x")') for j, fl in enumerate(SELF)]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-linkflags-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}')
sys.exit(0 if ours == theirs else 1)
