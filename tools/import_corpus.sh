#!/usr/bin/env bash
# Copy the test corpus out of an installed Stratum 2000 into fixtures/.
#
#   tools/import_corpus.sh [PATH_TO_STRATUM]
#
# Default path is the Wine installation this project was developed against.
# Nothing from the original program is redistributed with this repository;
# fixtures/ is built locally from your own copy.
set -euo pipefail

SRC="${1:-$HOME/.wine32/drive_c/Program Files/Stratum}"
DEST="$(cd "$(dirname "$0")/.." && pwd)/fixtures"

if [ ! -d "$SRC" ]; then
    echo "Stratum not found at: $SRC" >&2
    echo "Pass the installation directory as the first argument." >&2
    exit 1
fi

mkdir -p "$DEST"/{help,config,PROJECTS}

cp -r "$SRC/library"            "$DEST/library"
cp -r "$SRC/add.lib"            "$DEST/add.lib"
cp -r "$SRC/PROJECTS/samples"   "$DEST/PROJECTS/samples"
cp -r "$SRC/data/ICONS"         "$DEST/ICONS"
cp -r "$SRC/template"           "$DEST/template"
cp    "$SRC/help/"*             "$DEST/help/"
for f in control.dsk SC3.INI GRAPHICS.INI readme.txt License.txt; do
    [ -f "$SRC/$f" ] && cp "$SRC/$f" "$DEST/config/"
done

echo "corpus imported into $DEST"
find "$DEST" -type f | wc -l | xargs echo "files:"
