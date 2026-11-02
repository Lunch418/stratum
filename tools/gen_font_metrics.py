#!/usr/bin/env python3
"""tools/verify/arial_widths.json -> core/src/gfx/arial.rs (ширины символов Arial и Arial CYR,
измеренные в оригинале; см. tools/measure_arial.py)."""
import json, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
d = json.loads((ROOT / 'tools/verify/arial_widths.json').read_text())
out = ['// Сгенерировано tools/gen_font_metrics.py из tools/verify/arial_widths.json - не править руками.',
       '// Ширины символов Arial в пикселях по размеру шрифта (4..=72) и коду cp1251 (32..=255),',
       '// как их возвращает оригинал для растрового текста (Wine).', '',
       'pub const MIN_SIZE: i32 = 4;', 'pub const MAX_SIZE: i32 = 72;', '']
for name, key in (('REGULAR', 'regular'), ('BOLD', 'bold'), ('CYR_REGULAR', 'cyr_regular'), ('CYR_BOLD', 'cyr_bold')):
    out.append(f'pub static {name}: [[u8; 224]; 69] = [')
    for size in range(4, 73):
        w = d[key][str(size)]
        assert len(w) == 224 and max(w) < 256
        out.append('    [' + ','.join(map(str, w)) + '],')
    out.append('];')
    out.append('')
(ROOT / 'core/src/gfx/arial.rs').write_text('\n'.join(out), encoding='utf-8')
