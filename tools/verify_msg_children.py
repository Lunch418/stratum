#!/usr/bin/env python3
"""Опыт: сообщение отключенному имиджу с детьми. R отключает себя (_disable := 1) на
первом такте, на втором такте S шлёт ему сообщение. Выполняется ли при этом
подсхема R (счётчик n у ребёнка K) и сколько раз сам R.

    python3 tools/verify_msg_children.py
"""
import json, sys
import verify_original as vo

CHILDREN = [('S', 'Snd', 1), ('R', 'Rcv', 2)]


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Snd', 'Rcv', 'Kid', 'Off', 'Gk']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')

    def cls(name, vars, text, kids=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F = lambda n: vo.VAR(n, 'FLOAT')
    cls('Snd', [F('t')], 't := ~t + 1\nif (~t == 2)\n SendMessage("", "Rcv")\nendif\n')
    cls('Rcv', [F('n'), F('_disable')], 'n := ~n + 1\n_disable := 1\n', kids=[('Kid', 'K', 1), ('Off', 'L', 2)])
    cls('Kid', [F('n')], 'n := ~n + 1\n')
    cls('Off', [F('n'), dict(F('_enable'), default='0')], 'n := ~n + 1\n', kids=[('Gk', 'G', 1)])
    cls('Gk', [F('n')], 'n := ~n + 1\n')
    cls('Main', vo.PROBE_VARS, text, kids=[(c, n, h) for n, c, h in CHILDREN])


vo.native_project = project
vo.WAIT = 5
items = [('R_n', 'GetVarF("R", "n")'), ('K_n', 'GetVarF("R\\K", "n")'), ('L_n', 'GetVarF("R\\L", "n")'), ('G_n', 'GetVarF("R\\L\\G", "n")')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-msgkids-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    print(f'{name}: ядро {ours.get(name, "-")}, оригинал {theirs.get(name, "-")}')
sys.exit(0 if ours == theirs else 1)
