#!/usr/bin/env python3
"""Таблица функций компилятора оригинала → core/src/lang/opcodes.rs.

Источник — docs/lang/builtins.json (из template/*.tpl, см. tpl2json.py).
Для каждой перегрузки: имя, типы аргументов (& — по ссылке), тип
необязательных повторяемых аргументов, тип результата и код операции.
"""
import json, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
TY = {'FLOAT': 'F', 'STRING': 'S', 'HANDLE': 'H', 'COLORREF': 'C', 'INTEGER': 'I', None: '-'}

items = json.loads((ROOT / 'docs/lang/builtins.json').read_text(encoding='utf-8'))
rows = []
for x in items:
    if not isinstance(x.get('opcode'), int):
        continue
    args, opt = '', ''
    for a in x['args']:
        if 'type' in a:
            args += ('&' if a['byref'] else '') + TY[a['type']]
        else:
            opt += ''.join(('&' if o['byref'] else '') + TY[o['type']] for o in a['optional'])
    rows.append((x['name'].lower(), args, opt, TY[x.get('ret')], x['opcode']))

# функции внешних библиотек (.TDL): вызов — DLLFunction 479 и имя строкой
tdl = json.loads((ROOT / 'docs/lang/tdl.json').read_text(encoding='utf-8'))
seen = {r[0] for r in rows}
for x in tdl:
    for name in [x['name'], *x.get('aliases', [])]:
        if name.lower() in seen:
            continue
        seen.add(name.lower())
        args = ''.join(('&' if a['byref'] else '') + TY[a['type']] for a in x['args'] if 'type' in a)
        rows.append((name.lower(), args, '', TY[x.get('ret')], 479))

out = ['// Сгенерировано tools/gen_opcodes.py из docs/lang/builtins.json — не править руками.',
       '// (имя в нижнем регистре, аргументы, повторяемые необязательные, результат, код)',
       '// Типы: F — FLOAT, S — STRING, H — HANDLE, C — COLORREF, I — INTEGER, - — нет;',
       '// & перед типом — аргумент по ссылке (выходной).',
       '',
       'pub static FUNCTIONS: &[(&str, &str, &str, char, u16)] = &[']
for name, args, opt, ret, op in rows:
    out.append(f'    ({json.dumps(name, ensure_ascii=False)}, "{args}", "{opt}", \'{ret}\', {op}),')
out.append('];')
(ROOT / 'core/src/lang/opcodes.rs').write_text('\n'.join(out) + '\n', encoding='utf-8')
print(len(rows), 'перегрузок')
