#!/usr/bin/env python3
"""Split a Stratum 2000 icon sheet (`.dbm`) into transparent PNG icons.

Usage:
    dbm2png.py SHEET.dbm OUT_DIR [--size 32] [--mask MASK.bmp]
    dbm2png.py --all ICONS_DIR OUT_DIR

A `.dbm` file is an ordinary Windows BMP holding a grid of square icons
(32x32 in every sheet shipped with Stratum); blank cells are skipped. The
icons have no alpha channel - the background is plain white. `mask.bmp` in
`data/ICONS` is 64x64 and is NOT a per-sheet mask, so pass `--mask` yourself
if you have a matching one.
"""
import os
import sys

from PIL import Image


def split(sheet_path, out_dir, size=32, mask_path=None):
    sheet = Image.open(sheet_path).convert("RGBA")
    mask = None
    if mask_path and os.path.exists(mask_path):
        mask = Image.open(mask_path).convert("L")
        if mask.size != sheet.size:
            mask = None
    cols = sheet.width // size
    rows = sheet.height // size
    os.makedirs(out_dir, exist_ok=True)
    stem = os.path.splitext(os.path.basename(sheet_path))[0].lower()
    written = 0
    for row in range(rows):
        for col in range(cols):
            box = (col * size, row * size, (col + 1) * size, (row + 1) * size)
            icon = sheet.crop(box)
            if icon.convert("RGB").getextrema() == ((255, 255), (255, 255), (255, 255)):
                continue    # blank cell
            if mask:
                m = mask.crop(box).point(lambda v: 0 if v > 127 else 255)
                icon.putalpha(m)
            icon.save(os.path.join(out_dir, "%s_%02d_%02d.png" % (stem, row, col)))
            written += 1
    return written, cols, rows


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 2
    if args[0] == "--all":
        icons_dir, out_dir = args[1], args[2]
        total = 0
        for f in sorted(os.listdir(icons_dir)):
            if not f.lower().endswith(".dbm"):
                continue
            n, cols, rows = split(os.path.join(icons_dir, f),
                                  os.path.join(out_dir, os.path.splitext(f)[0].lower()))
            print("%-16s %2dx%-2d  %3d icons" % (f, cols, rows, n))
            total += n
        print("total:", total)
        return 0
    sheet, out_dir = args[0], args[1]
    size = 32
    mask = None
    if "--size" in args:
        size = int(args[args.index("--size") + 1])
    if "--mask" in args:
        mask = args[args.index("--mask") + 1]
    n, cols, rows = split(sheet, out_dir, size, mask)
    print("%d icons (%dx%d grid)" % (n, cols, rows))
    return 0


if __name__ == "__main__":
    sys.exit(main())
