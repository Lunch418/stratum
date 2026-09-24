#!/usr/bin/env python3
"""Опыт: SendMessage по имени класса. Отправитель S на втором такте шлёт
сообщение всем имиджам класса Rcv (три на схеме Main, один внутри Box) с
парами acc→acc и val→v. Журнал acc показывает порядок доставки; seen и seen2
у получателя — что он видит в переданной переменной без тильды и с ней;
val у отправителя — что возвращается обратно. Запуск ядром и оригиналом,
как в verify_original.py.

    python3 tools/verify_messages.py
"""
import json, sys
import verify_original as vo

RCV = '_disable := 1\nif (~id == 0) ; exit() ; endif\nacc := ~acc + String(id)\nseen := v\nseen2 := ~v\nv := ~v + 1\n'
SND = 't := ~t + 1\nif (~t == 2)\n acc := ""\n val := 10\n SendMessage("", "Rcv", "acc", "acc", "val", "v")\nendif\n'


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Snd', 'Rcv', 'Box']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')

    def cls(name, vars, text, kids=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': []}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F, S = (lambda n: vo.VAR(n, 'FLOAT')), (lambda n: vo.VAR(n, 'STRING'))
    cls('Snd', [F('t'), S('acc'), F('val')], SND)
    cls('Rcv', [F('id'), S('acc'), F('v'), F('seen'), F('seen2'), F('_disable')], RCV)
    cls('Box', [], '', [('Rcv', 'R4', 1)])
    cls('Main', vo.PROBE_VARS, text, [('Rcv', 'R1', 1), ('Snd', 'S', 2), ('Rcv', 'R2', 3), ('Box', 'B', 4), ('Rcv', 'R3', 5)])


vo.native_project = project
vo.SETUP += [f'SetVar("{p}", "id", {i})' for i, p in [(1, 'R1'), (2, 'R2'), (3, 'R3'), (4, 'B\\R4')]]
vo.WAIT = 3
items = [('order', 's:GetVarS("S", "acc")'), ('val', 'GetVarF("S", "val")')]
for p in ['R1', 'R2', 'R3', 'B\\R4']:
    items += [(f'{p}.{v}', f'GetVarF("{p}", "{v}")') for v in ('seen', 'seen2', 'v')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-msg-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    a, b = ours.get(name, '—'), theirs.get(name, '—')
    print(f'{"ok " if a == b else "РАЗНИЦА":8} {name:12} ядро={a!r:12} оригинал={b!r}')
sys.exit(0 if ours == theirs else 1)
