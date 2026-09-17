#!/usr/bin/env bash
# Decompile fixtures/help/SC3.HLP into docs/help/ (markdown topics, images and
# the function reference).
#
#   tools/decompile_help.sh [PATH_TO_HELPDECO]
#
# helpdeco is not packaged on Ubuntu; build it once:
#   git clone --depth 1 https://github.com/pmachapman/helpdeco
#   make -C helpdeco/gcc
# and pass helpdeco/gcc/helpdeco here (or put it on PATH).
#
# Needs ImageMagick (`convert`) for the WMF figures and Pillow for the bitmaps.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HELPDECO="${1:-$(command -v helpdeco || true)}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

if [ -z "$HELPDECO" ] || [ ! -x "$HELPDECO" ]; then
    echo "helpdeco not found; see the comment at the top of this script" >&2
    exit 1
fi
if [ ! -f "$ROOT/fixtures/help/SC3.HLP" ]; then
    echo "fixtures/help/SC3.HLP is missing - run tools/import_corpus.sh first" >&2
    exit 1
fi

cp "$ROOT/fixtures/help/SC3.HLP" "$WORK/"
( cd "$WORK" && "$HELPDECO" -y SC3.HLP >/dev/null )
# helpdeco always complains about the phrase table of this file; the RTF it
# produces is complete regardless.

rm -rf "$ROOT/docs/help"
python3 "$ROOT/tools/hlp2md.py" "$WORK/SC3.rtf" "$ROOT/fixtures/help/sc3.cnt" "$ROOT/docs/help"

mkdir -p "$ROOT/docs/help/images"
python3 - "$WORK" "$ROOT/docs/help" <<'PY'
import glob, os, re, subprocess, sys
work, out = sys.argv[1], sys.argv[2]
wanted = set()
for p in glob.glob(os.path.join(out, "topics", "*.md")):
    wanted |= set(re.findall(r"\.\./images/(\w+)\.png", open(p, encoding="utf-8").read()))
from PIL import Image
for name in sorted(wanted):
    bmp, wmf = os.path.join(work, name + ".bmp"), os.path.join(work, name + ".wmf")
    dest = os.path.join(out, "images", name + ".png")
    if os.path.exists(bmp):
        Image.open(bmp).save(dest)
    elif os.path.exists(wmf):
        subprocess.run(["convert", wmf, dest], check=False)
print("images:", len(wanted))
PY

python3 "$ROOT/tools/merge_functions.py" \
    "$ROOT/docs/lang/builtins.json" "$ROOT/docs/help/functions.json" "$ROOT/docs/lang"
