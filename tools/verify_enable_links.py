#!/usr/bin/env python3
"""Опыт: связанная переменная _enable с разными умолчаниями (у C 1, у D 0,
связь D -> C). Сколько раз успевает выполниться C и что в её _enable.

    python3 tools/verify_enable_links.py
"""
import json, sys
import verify_original as vo

CHILDREN = [('C', 'E1', 1), ('D', 'E0', 2), ('F', 'E0', 3), ('G', 'E1', 4), ('P', 'E0K', 5), ('Q', 'E1K', 6)]
LINKS = [(2, 1), (4, 3)]  # D -> C (D после C); G -> F (G после F: у F 0, у G 1)


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'E0', 'E1', 'E0K', 'E1K']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')
    for n, d in (('E0', '0'), ('E1', '1')):
        vs = [dict(vo.VAR('_enable', 'FLOAT'), default=d), dict(vo.VAR('n', 'FLOAT'), default='0')]
        (dir / 'classes' / f'{n}.strat.json').write_text(json.dumps({'name': n, 'description': '', 'vars': vs, 'children': [], 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{n}.strat').write_text('n := n + 1\n', encoding='utf-8')
    for n, d, t in (('E0K', '0', 'n := n + 1\n'), ('E1K', '1', 'n := n + 1\n_enable := 0\n')):
        vs = [dict(vo.VAR('_enable', 'FLOAT'), default=d), dict(vo.VAR('n', 'FLOAT'), default='0')]
        (dir / 'classes' / f'{n}.strat.json').write_text(json.dumps({'name': n, 'description': '', 'vars': vs, 'children': [{'class': 'E1', 'handle': 1, 'name': 'K', 'x': 0, 'y': 0, 'flags': 0}], 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{n}.strat').write_text(t, encoding='utf-8')
    kids = [{'class': c, 'handle': h, 'name': name, 'x': 0, 'y': 0, 'flags': 0} for name, c, h in CHILDREN]
    links = [{'flags': 0, 'handle': 100 + i, 'source': s, 'target': t, 'vars': [['_enable', '_enable']]} for i, (s, t) in enumerate(LINKS)]
    (dir / 'classes' / 'Main.strat.json').write_text(json.dumps({'name': 'Main', 'description': '', 'vars': vo.PROBE_VARS, 'children': kids, 'links': links}), encoding='utf-8')
    (dir / 'classes' / 'Main.strat').write_text(text, encoding='utf-8')


vo.native_project = project
vo.WAIT = 4
items = [(f'{c}_{v}', f'GetVarF("{c}", "{v}")') for c, _, _ in CHILDREN for v in ('n', '_enable')] + [(f'{c}K_n', f'GetVarF("{c}\\K", "n")') for c in 'PQ']
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-enable-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}')
sys.exit(0 if ours == theirs else 1)
