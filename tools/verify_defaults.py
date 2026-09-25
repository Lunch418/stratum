#!/usr/bin/env python3
"""Опыт: как оригинал читает значения по умолчанию переменных (цвета
`Transparent`, `rgbex(…)`, «255, 255, 255», числа с `#`). Переменные главного
имиджа получают такие умолчания; проба печатает их числом. Запуск ядром и
оригиналом, как в verify_original.py.

    python3 tools/verify_defaults.py
"""
import sys
import verify_original as vo

CASES = [
    ('c_transp', 'COLORREF', 'Transparent'),
    ('c_transp_l', 'COLORREF', 'transparent'),
    ('c_rgb', 'COLORREF', 'rgb(1,2,3)'),
    ('c_rgbex', 'COLORREF', 'rgbex(0,128,0,1)'),
    ('c_rgbex0', 'COLORREF', 'rgbex(1,2,3,0)'),
    ('c_rgbex2', 'COLORREF', 'rgbex(1,2,3,2)'),
    ('c_list', 'COLORREF', '255, 255, 255'),
    ('c_num', 'COLORREF', '16777215'),
    ('c_255', 'COLORREF', '255'),
    ('c_1', 'COLORREF', '1'),
    ('c_hash', 'COLORREF', '#255'),
    ('c_rgbex3', 'COLORREF', 'rgbex(255,255,255,1)'),
    ('c_rgbex4', 'COLORREF', 'rgbex(1,2,3,255)'),
    ('c_Rgb', 'COLORREF', 'RGB(1,2,3)'),
    ('f_transp', 'FLOAT', 'Transparent'),
    ('h_hash', 'HANDLE', '#5'),
    ('f_list', 'FLOAT', '1, 2'),
    ('f_exp', 'FLOAT', '1e3'),
]


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    import json
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': 'Main', 'file': 'Main'}]}), encoding='utf-8')
    vars = vo.PROBE_VARS + [dict(vo.VAR(n, t), default=d) for n, t, d in CASES]
    vo.write_class(dir, 'Main', vars, text)


vo.native_project = project
# число целиком: старшие и младшие 16 бит (String даёт только 6 цифр)
F = lambda n, t: n if t == 'FLOAT' else f'Float({n})'
items = [(n, f's:String(trunc({F(n, t)}/65536)) + ":" + String({F(n, t)} - trunc({F(n, t)}/65536)*65536)') for n, t, _ in CASES]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-defaults-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for (name, _), (_, t, d) in zip(items, CASES):
    a, b = ours.get(name, '—'), theirs.get(name, '—')
    print(f'{"ok " if a == b else "РАЗНИЦА":8} {name:12} {t:9} {d!r:22} ядро={a!r:14} оригинал={b!r}')
sys.exit(0 if ours == theirs else 1)
