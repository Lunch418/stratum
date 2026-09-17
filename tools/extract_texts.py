#!/usr/bin/env python3
"""Extract the model text of every image in a tree of `.cls` files.

Usage:
    extract_texts.py SRC_DIR OUT_DIR

Writes one UTF-8 `.strat` file per image, mirroring the source tree, plus
`index.json` listing every image with its variables. This is the corpus the
language front end is tested against.
"""
import json
import os
import sys
from collections import Counter, OrderedDict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cls_dump


def main():
    src, out = sys.argv[1], sys.argv[2]
    index = []
    lines = 0
    types = Counter()
    for root, _, names in os.walk(src):
        for name in sorted(names):
            if not name.lower().endswith(".cls"):
                continue
            path = os.path.join(root, name)
            data = open(path, "rb").read()
            if data[:2] != b"SB":
                continue
            cls = cls_dump.flatten(cls_dump.parse(data, path))
            rel = os.path.relpath(path, src)
            text = cls["text"].replace("\r\n", "\n")
            if text.strip():
                dest = os.path.join(out, os.path.splitext(rel)[0] + ".strat")
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                with open(dest, "w", encoding="utf-8") as f:
                    f.write(text if text.endswith("\n") else text + "\n")
                lines += text.count("\n") + 1
            for v in cls["vars"]:
                types[v["type"]] += 1
            index.append(OrderedDict(
                file=rel, name=cls["name"], version=cls["version"],
                text_lines=text.count("\n") + 1 if text.strip() else 0,
                vars=[OrderedDict(name=v["name"], type=v["type"],
                                  default=v["default"], flags=v["flags"])
                      for v in cls["vars"]],
                children=[c["klass"] for c in cls["children"]],
                links=len(cls["links"]),
            ))
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "index.json"), "w", encoding="utf-8") as f:
        json.dump(index, f, ensure_ascii=False, indent=1)
    print("images: %d, with text: %d, lines: %d" %
          (len(index), sum(1 for i in index if i["text_lines"]), lines))
    print("variables by type:", dict(types))


if __name__ == "__main__":
    main()
