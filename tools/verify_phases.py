#!/usr/bin/env python3
"""Опыт: что видит имидж в связанной переменной без тильды, если другой
имидж уже изменил её в этом такте. Cnt (первый на схеме) каждый такт
увеличивает u; Rd (после него) читает u без тильды и с тильдой; Late
(перед Cnt) читает то же самое до изменения. Сообщением Snd вызывает Msg,
который читает u так же. Запуск ядром и оригиналом, как в verify_original.py.

    python3 tools/verify_phases.py
"""
import json, sys
import verify_original as vo


def project(dir, text, compiler=False):
    (dir / 'classes').mkdir(parents=True, exist_ok=True)
    names = ['Main', 'Cnt', 'Rd', 'Snd', 'Msg']
    (dir / 'project.json').write_text(json.dumps({'format': 'stratum-modern/1', 'root': 'Main', 'properties': [], 'variables': [], 'libraries': [],
                                                  'classes': [{'name': n, 'file': n} for n in names]}), encoding='utf-8')

    def cls(name, vars, text, kids=(), links=()):
        children = [{'class': c, 'handle': h, 'name': n, 'x': 0, 'y': 0, 'flags': 0} for c, n, h in kids]
        links = [{'flags': 0, 'handle': 100 + i, 'source': s, 'target': t, 'vars': [[a, b]]} for i, (s, t, a, b) in enumerate(links)]
        (dir / 'classes' / f'{name}.strat.json').write_text(json.dumps({'name': name, 'description': '', 'vars': vars, 'children': children, 'links': links}), encoding='utf-8')
        (dir / 'classes' / f'{name}.strat').write_text(text, encoding='utf-8')
    F = lambda n: vo.VAR(n, 'FLOAT')
    cls('Cnt', [F('u')], 'u := u + 1\n')
    cls('Rd', [F('u'), F('a'), F('b')], 'a := u\nb := ~u\n')
    cls('Msg', [F('u'), F('a'), F('b'), F('_disable')], '_disable := 1\na := u\nb := ~u\n')
    cls('Snd', [F('u')], 'SendMessage("", "Msg", "u", "u")\n')
    kids = [('Rd', 'Late', 1), ('Cnt', 'C', 2), ('Rd', 'R', 3), ('Msg', 'M', 4), ('Snd', 'S', 5)]
    # u всех пяти связан с u счётчика
    cls('Main', vo.PROBE_VARS, text, kids, [(2, h, 'u', 'u') for h in (1, 3, 4, 5)])


vo.native_project = project
vo.WAIT = 2
items = [(f'{p}.{v}', f'GetVarF("{p}", "{v}")') for p in ('Late', 'R', 'M') for v in ('a', 'b')] + [('C.u', 'GetVarF("C", "u")')]
work = vo.pathlib.Path(vo.os.environ.get('VERIFY_TMP', '/tmp')) / f'stratum-phase-{vo.os.getpid()}'
work.mkdir(parents=True, exist_ok=True)
ours = vo.run_core(items, work) or {}
theirs = vo.run_original(items, work) or {}
for name, _ in items:
    a, b = ours.get(name, '—'), theirs.get(name, '—')
    print(f'{"ok " if a == b else "РАЗНИЦА":8} {name:10} ядро={a!r:8} оригинал={b!r}')
sys.exit(0 if ours == theirs else 1)
