#!/usr/bin/env python3
"""Дизассемблер байт-кода Stratum 2000 (секция 0x0d в .cls).

    python3 tools/disasm.py файл.cls

Печатает текст имиджа и команды байт-кода с именами переменных, функций и
операций. Коды — из docs/lang/builtins.json и operators.json.
"""
import importlib.util, json, pathlib, struct, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('cd', ROOT / 'tools' / 'cls_dump.py')
cd = importlib.util.module_from_spec(spec); spec.loader.exec_module(cd)

VAR1 = {1: 'push', 2: 'push_new', 3: 'push.h', 4: 'push_new.h', 120: 'push.s', 121: 'push_new.s',
        11: ':=', 10: ':=_old', 13: ':=.h', 12: ':=_old.h', 123: ':=.s', 157: ':=_old.s', 477: '&', 800: '++'}
JUMP = {51: 'jmp', 52: 'jnz', 53: 'jz', 110: 'jnz.h', 111: 'jz.h'}
OPS = {18: '+', 19: '-', 21: '*', 20: '/', 22: '%', 32: '^', 54: '==', 55: '!=', 56: '>', 57: '>=', 58: '<', 59: '<=',
       144: '==.s', 145: '!=.s', 146: '>.s', 147: '>=.s', 148: '<.s', 149: '<=.s', 82: '==.h', 83: '!=.h',
       105: '&', 106: '|', 43: '&&', 44: '||', 85: '&&.h', 86: '||.h', 116: '<<', 117: '>>', 113: 'neg', 45: '!',
       124: '+.s', 770: '+.sf', 771: '+.fs', 0: 'end'}
FUNCS = {}
VARIADIC = set()
for x in json.loads((ROOT / 'docs/lang/builtins.json').read_text(encoding='utf-8')):
    if isinstance(x.get('opcode'), int):
        FUNCS.setdefault(x['opcode'], x['name'])
        if any('optional' in a for a in x['args']):
            VARIADIC.add(x['opcode'])


def disasm(words, names):
    out, i = [], 0
    while i < len(words):
        o = words[i]
        name = lambda k: names[k] if k < len(names) else f'#{k}'
        if o in VAR1:
            out.append(f'{i:5} {VAR1[o]} {name(words[i + 1])}'); i += 2
        elif o in JUMP:
            out.append(f'{i:5} {JUMP[o]} {words[i + 1]}'); i += 2
        elif o == 6:
            out.append(f'{i:5} const {struct.unpack("<d", struct.pack("<4H", *words[i + 1:i + 5]))[0]!r}'); i += 5
        elif o == 5:
            out.append(f'{i:5} const.h {words[i + 1] | words[i + 2] << 16}'); i += 3
        elif o == 122:
            n = words[i + 1]
            raw = b''.join(struct.pack('<H', w) for w in words[i + 2:i + 2 + n]).split(b'\0')[0]
            out.append(f'{i:5} const.s {raw.decode("cp1251")!r}'); i += 2 + n
        elif o in OPS:
            out.append(f'{i:5} {OPS[o]}'); i += 1
        elif o in FUNCS:
            if o in VARIADIC:
                out.append(f'{i:5} {FUNCS[o]} (+{words[i + 1]} необяз.)'); i += 2
            else:
                out.append(f'{i:5} {FUNCS[o]}'); i += 1
        else:
            out.append(f'{i:5} ?{o}'); i += 1
    return out


def main():
    d = cd.parse(open(sys.argv[1], 'rb').read(), sys.argv[1])
    s = d['sections']
    names = [v['name'] for v in s.get('vars', [])]
    raw = bytes.fromhex(s['bytecode']['data'])
    words = struct.unpack('<%dH' % (len(raw) // 2), raw)
    print(s.get('text', '').strip(), '\n---')
    print('\n'.join(disasm(words, names)))


if __name__ == '__main__':
    main()
