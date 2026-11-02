"""Пробный файл ширин символов Arial: python3 tools/measure_arial_probe.py РАЗМЕРЫ ФАЙЛ (env FL=0|8 - флаги шрифта)."""
import sys, os
sizes=[int(x) for x in sys.argv[1].split(',')]
face=os.environ.get("FACE","Arial"); import os; FL=int(os.environ.get("FL","0"))
L=["# Ширина символов растрового текста (%s) по размерам шрифта: GetObjectWidth2d, advance для text_extent"%face,
"","% Wait = 1",
'! hs := CreateWindowEx("W1", "", "", 10, 10, 300, 300, "")']
for h in sizes:
    L.append(f'! f{h} := CreateFont2d(~hs, "{face}", {h}, {FL})')
    L.append(f'! c := 32')
    L.append(f'! while (~c < 256)')
    L.append(f'!  sx := CreateString2d(~hs, Chr(~c))')
    L.append(f'!  tx := CreateText2d(~hs, ~f{h}, ~sx, Rgb(0,0,0), Rgb(255,255,255))')
    L.append(f'!  rx := CreateRasterText2d(~hs, ~tx, 0, 0, 0)')
    L.append(f'!  if (~c == 32); r{h} := ~rx; endif')
    L.append(f'!  c := ~c + 1')
    L.append(f'! endwhile')
L.append('$ STRING out')
L.append('$ FLOAT c, wd, rez, rf, k')
for h in sizes:
    L.append('> k := 0')
    L.append(f'> rf := Float(~r{h})')
    for part in range(4):
        L.append('> out := ""')
        L.append('> k := %d' % (part * 56))
        L.append('> while (~k < %d)' % (part * 56 + 56))
        L.append('>  wd := GetObjectWidth2d(~hs, Handle(~rf + ~k))')
        L.append('>  out := ~out + String(~wd) + ","')
        L.append('>  k := ~k + 1')
        L.append('> endwhile')
        L.append(f'out{h}{"abcd"[part]} | s:~out')
open(sys.argv[2],'w',encoding='utf-8').write('\n'.join(L)+'\n')
