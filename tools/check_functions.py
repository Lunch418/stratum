#!/usr/bin/env python3
"""Сверка таблиц компилятора с ядром: какие функции языка не имеют ветки
в core/src (builtins, extra, gfx/api, gfx/api3d, sim). Ogre3D и Db*
считаются отдельно — это плагины.

    python3 tools/check_functions.py
"""
import json
import re
import collections
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCES = [
    "core/src/runtime/builtins.rs",
    "core/src/runtime/extra.rs",
    "core/src/gfx/api.rs",
    "core/src/gfx/api3d.rs",
    "core/src/sim/mod.rs",
    "core/src/sim/interp.rs",
]
OGRE = ("root_", "entity_", "scenenode_", "renderwindow_", "billboard", "overlay", "material", "camera_",
        "light_", "animationstate_", "particlesystem_", "textureunitstate_", "viewport_", "scenemanager_",
        "mesh_", "node_", "compositor", "skeleton_")


def implemented() -> set[str]:
    names = set()
    for rel in SOURCES:
        text = (ROOT / rel).read_text(encoding="utf-8")
        names.update(re.findall(r'"([a-z_0-9]+)"\s*(?:\||=>)', text))
        names.update(re.findall(r'\|\s*"([a-z_0-9]+)"', text))
    return names


def main() -> int:
    table = json.load(open(ROOT / "docs/lang/functions.json", encoding="utf-8"))
    have = implemented()
    groups = collections.defaultdict(list)
    ogre = db = 0
    for f in table:
        n = f["name"].lower()
        if n in have:
            continue
        if n.startswith(OGRE) or "Ogre" in (f.get("group") or ""):
            ogre += 1
        elif n.startswith("db"):
            db += 1
        else:
            groups[f.get("group") or "?"].append(f["name"])
    other = sum(len(v) for v in groups.values())
    print(f"функций в таблицах: {len(table)}; нет ветки: Ogre3D {ogre}, Db* {db}, прочих {other}")
    for g, names in sorted(groups.items(), key=lambda kv: -len(kv[1])):
        print(f"  {g} ({len(names)}): {', '.join(sorted(names))}")
    return 1 if other else 0


if __name__ == "__main__":
    raise SystemExit(main())
