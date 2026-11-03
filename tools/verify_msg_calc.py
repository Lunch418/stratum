#!/usr/bin/env python3
"""Опыт: Calcer3d из WRITEAVI. Отправитель S лежит в Box (как StratumClass_makeface
в MakeFace3D) и шлёт SendMessage("..\\..\\" + имя, "", "_x", "x", "_z", "z") получателю
верхнего уровня. Варианты получателя: A - без _enable, B - _enable без значения
(по умолчанию 0, как у Calcer3d), C - _enable = 1, D - _disable = 1 после работы.
Печатает x получателя (дошло ли сообщение и исполнился ли он) и _z у отправителя.

    python3 tools/verify_msg_calc.py
"""
import json, sys
import verify_original as vo

# {r} - имя получателя; отправитель лежит в Box (Main\\B<n>\\S) и в Main (S<n>)
PATHS = [('..\\..\\{r}', '', ''), ('..\\..\\{r}', '', ''), ('..\\..\\{r}', '', ''), ('..\\..\\{r}', '', '')]
# дополнительная пара: (переменная отправителя, переменная получателя)
EXTRA = [('Color', 'Color'), ('Color', 'nope'), ('_y', 'y'), ('_x', 'y')]
NAMES = [str(i) for i in range(len(PATHS))]


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Rc']

    def cls(name, vars, text, kids=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F = lambda n: vo.VAR(n, 'FLOAT')
    body = 'n := ~n + 1\n_x := x * 2\n_z := 7\n'
    cls('Rc', [F('x'), F('y'), F('_x'), F('_z'), F('n')], body)
    base = [F('x'), F('y'), F('_x'), F('_z'), F('n')]
    cls('RcB', base + [F('_enable')], body)
    cls('RcC', base + [dict(F('_enable'), default='1')], body)
    cls('RcD', base + [F('_disable')], body + '_disable := 1\n')
    names += ['RcB', 'RcC', 'RcD']
    kids = []
    h = 1
    for i, p in enumerate(PATHS):
        path, klass, rk = p[0].format(r=f'R{i}'), p[1], p[2]
        extra = f', "{EXTRA[i][0]}", "{EXTRA[i][1]}"' if EXTRA[i] else ''
        for tag, dst in (('S', 'x'), ('D', 'y')):
            snd = f't := ~t + 1\nif (~t == 2)\n _x := 3\n SendMessage("{path}", "{klass}", "_x", "{dst}", "_z", "_z"{extra})\nendif\n'
            cls(f'Sn{tag}{i}', [F('t'), F('_x'), F('_y'), F('_z'), vo.VAR('Color', 'COLORREF')], snd)
            names.append(f'Sn{tag}{i}')
        cls(f'Bx{i}', [], '', kids=[(f'SnD{i}', 'S', 1)])
        names.append(f'Bx{i}')
        kids += [(f'Rc{rk}', f'R{i}', h), (f'Bx{i}', f'B{i}', h + 1), (f'SnS{i}', f'S{i}', h + 2)]
        h += 3
    cls('Main', vo.PROBE_VARS, text, kids=kids)
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')


vo.native_project = project
vo.WAIT = 5
items = []
for n in NAMES:
    items += [(f'R{n}.x', f'GetVarF("R{n}", "x")'), (f'R{n}.y', f'GetVarF("R{n}", "y")'), (f'R{n}.n', f'GetVarF("R{n}", "n")'), (f'S{n}.t', f'GetVarF("S{n}", "t")'), (f'S{n}._z', f'GetVarF("S{n}", "_z")'), (f'BS{n}._z', f'GetVarF("B{n}\\S", "_z")')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-msgcalc-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
bad = 0
for name, _ in items:
    flag = '' if ours.get(name) == theirs.get(name) else '  <-- расходится'
    bad += bool(flag)
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}{flag}')
sys.exit(1 if bad else 0)
