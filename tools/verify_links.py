#!/usr/bin/env python3
"""Опыт: какое начальное значение получают связанные переменные с разными
значениями по умолчанию. Дети V1 (x = 1), V2 (x = 2), V3 (x = 3) на схеме
главного имиджа, связи в разных направлениях и порядке; главный имидж читает
x каждого ребёнка через GetVarF. Запуск ядром и оригиналом, как в
verify_original.py.

    python3 tools/verify_links.py
"""
import json, sys
import verify_original as vo

# (имя ребёнка, класс, handle) в порядке схемы
CHILDREN = [('A', 'V1', 1), ('B', 'V2', 2),            # A → B
            ('C', 'V1', 3), ('D', 'V2', 4),            # D → C
            ('E', 'V1', 5), ('F', 'V2', 6), ('G', 'V3', 7),  # G → E, F → E (как в BALLS)
            ('H', 'V2', 8), ('I', 'V1', 9),            # H → I
            ('J', 'V1', 10), ('K', 'V3', 11), ('L', 'V2', 12),  # J → K, K → L
            ('N', 'V1', 13), ('O', 'V0', 14),          # N → O, у O значение не задано
            ('P', 'V2', 15)]                           # P → переменная p самого Main (p = 7)
LINKS = [(1, 2), (4, 3), (7, 5), (6, 5), (8, 9), (10, 11), (11, 12), (13, 14), (15, 0)]


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'V0', 'V1', 'V2', 'V3']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')
    for i, n in enumerate(names[1:]):
        var = dict(vo.VAR('x', 'FLOAT'), default=str(i) if i else '')
        (dir / 'classes' / f'{n}.strat.json').write_text(json.dumps({'name': n, 'description': '', 'vars': [var], 'children': [], 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{n}.strat').write_text('x := x\n', encoding='utf-8')
    kids = [{'class': c, 'handle': h, 'name': name, 'x': 0, 'y': 0, 'flags': 0} for name, c, h in CHILDREN]
    links = [{'flags': 0, 'handle': 100 + i, 'source': s, 'target': t, 'vars': [['x', 'p' if t == 0 else 'x']]} for i, (s, t) in enumerate(LINKS)]
    (dir / 'classes' / 'Main.strat.json').write_text(json.dumps({'name': 'Main', 'description': '', 'vars': vo.PROBE_VARS + [dict(vo.VAR('p', 'FLOAT'), default='7')], 'children': kids, 'links': links}), encoding='utf-8')
    (dir / 'classes' / 'Main.strat').write_text(text, encoding='utf-8')


vo.native_project = project
items = [(name, f'GetVarF("{name}", "x")') for name, _, _ in CHILDREN] + [('p', 'p')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-links-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name in [c[0] for c in CHILDREN] + ['p']:
    print(f'{name}: ядро {ours.get(name, "—")}, оригинал {theirs.get(name, "—")}')
sys.exit(0 if ours == theirs else 1)
