#!/usr/bin/env bash
# Copy the locally built help (docs/help/topics and images) into the per-user
# data directory where the installed app looks for it.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
case "$(uname -s)" in
    Darwin) dest="$HOME/Library/Application Support/stratum-modern/help" ;;
    MINGW*|MSYS*|CYGWIN*) dest="$APPDATA/stratum-modern/help" ;;
    *) dest="${XDG_DATA_HOME:-$HOME/.local/share}/stratum-modern/help" ;;
esac
mkdir -p "$dest"
cp -r "$ROOT/docs/help/topics" "$ROOT/docs/help/images" "$dest/"
echo "help installed to $dest"
