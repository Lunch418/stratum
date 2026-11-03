#!/usr/bin/env python3
"""Опыт: вложенная рассылка из WRITEAVI. MakeFace3D (Mid) шлёт SendMessage("c", "") своему
ребёнку c (StratumClass_makeface, _enable = 0), а тот шлёт сообщение вычислителю R
верхнего уровня (SendMessage("..\\..\\R", "", "_x", "x", "_z", "_z")). Варианты:
0 - ребёнок без _enable (исполняется каждый такт сам), 1 - ребёнок с _enable без
значения (по умолчанию 0), 2 - то же, но Mid шлёт с пустым именем и классом Snd.
Печатает x получателя и _z у ребёнка.

    python3 tools/verify_msg_nested.py
"""
import json, sys
import verify_original as vo

VARIANTS = [(False, 'SendMessage("c", "")', 'R'), (True, 'SendMessage("c", "")', 'R'), (True, 'SendMessage("", "Snd")', 'R'),
            (True, 'SendMessage("c", "")', 'Rc'), (True, 'SendMessage("c", "")', '')]


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Rc']

    def cls(name, vars, text, kids=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F = lambda n: vo.VAR(n, 'FLOAT')
    cls('Rc', [F('x'), F('n'), F('_x'), F('_z')], 'n := ~n + 1\n_z := 7\n')
    kids = []
    for i, (gated, send, rname) in enumerate(VARIANTS):
        cls(f'Snd{i}', [F('t'), F('_x'), F('_z'), F('calls')] + ([F('_enable')] if gated else []),
            'calls := ~calls + 1\n_x := 3\nSendMessage("..\\..\\%s", "", "_x", "x", "_z", "_z")\n' % (rname + str(i) if rname == 'R' else rname))
        cls(f'Mid{i}', [F('t')], f't := ~t + 1\nif (~t == 2)\n {send.replace("Snd", f"Snd{i}")}\nendif\n', kids=[(f'Snd{i}', 'c', 1)])
        names += [f'Snd{i}', f'Mid{i}']
        kids += [('Rc', f'R{i}' if rname == 'R' else '', 2 * i + 1), (f'Mid{i}', f'M{i}', 2 * i + 2)]
    cls('Main', vo.PROBE_VARS, text, kids=kids)
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')


vo.native_project = project
vo.WAIT = 5
items = []
for i in range(len(VARIANTS)):
    rn = f'R{i}' if VARIANTS[i][2] == 'R' else f'#{2 * i + 1}'
    items += [(f'R{i}.x', f'GetVarF("{rn}", "x")'), (f'R{i}.n', f'GetVarF("{rn}", "n")'), (f'c{i}.calls', f'GetVarF("M{i}\\c", "calls")'), (f'c{i}._z', f'GetVarF("M{i}\\c", "_z")')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-msgnest-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
bad = 0
for name, _ in items:
    flag = '' if ours.get(name) == theirs.get(name) else '  <-- расходится'
    bad += bool(flag)
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}{flag}')
sys.exit(1 if bad else 0)
