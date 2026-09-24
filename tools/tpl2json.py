#!/usr/bin/env python3
"""Parse the Stratum 2000 compiler tables (template/*.tpl) into JSON.

Usage:
    tpl2json.py TEMPLATE_DIR OUT_DIR

Produces:
    OUT_DIR/builtins.json    every `name "..." arg ... ret ... out N` entry
    OUT_DIR/operators.json   the OPERATORS section of COMPILER.TPL
    OUT_DIR/constants.json   CONSTANT.TPL name/value pairs
    OUT_DIR/tdl.json         DLL descriptors (library/*.tdl) if a library dir is given as 3rd arg

Entry grammar (one per line, `//` comments, `/* */` comments):
    name "Fn" [imp P[,P2]] [arg A,A,...] [ret T] [out N[,$index|$name]]
    arg items:  "TYPE"        by value
                &"TYPE"       by reference (out parameter)
                ["T","T"]     optional group (may be repeated)
"""
import json
import os
import re
import sys
from collections import OrderedDict

ENTRY_RE = re.compile(r'^\s*name\s+"([^"]+)"((?:\s*,\s*"[^"]+")*)(.*)$')
INCLUDE_RE = re.compile(r"^\s*include\s+(\S+)", re.I)
SECTION_RE = re.compile(r"^\s*([A-Z_]+)\s*$")


def strip_comments(text):
    """Убирает `//…` и `/*…*/` за один проход слева направо.

    Порядок важен: в шаблонах есть строки вида `… out 395 //**`, и если
    сначала искать `/*…*/`, «блок» откроется внутри строчного комментария и
    съест всё до ближайшего `*/` (раньше так пропадало полсотни функций
    Graph2d.tpl, например EnableControl2d и LBClearList)."""
    out, i, n = [], 0, len(text)
    in_str = False
    while i < n:
        c = text[i]
        if in_str:
            out.append(c)
            if c == '"' or c == "\n":
                in_str = False
            i += 1
        elif c == '"':
            in_str = True
            out.append(c)
            i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            skipped = text[i:n if j < 0 else j + 2]
            out.append("\n" * skipped.count("\n"))
            i = n if j < 0 else j + 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def parse_args(spec):
    """Turn `"A",&"B",["C","D"]` into a list of arg descriptors."""
    args = []
    for m in re.finditer(r'(\[)|(\])|(&?)"([A-Za-z0-9_]+)"(?::"([A-Za-z0-9_]+)")?', spec):
        if m.group(1):
            args.append({"group": "["})
        elif m.group(2):
            args.append({"group": "]"})
        else:
            a = {"type": m.group(4), "byref": m.group(3) == "&"}
            if m.group(5):
                a["ctype"] = m.group(5)
            args.append(a)
    # collapse optional groups into nested lists
    out = []
    stack = [out]
    for a in args:
        if a.get("group") == "[":
            g = {"optional": []}
            stack[-1].append(g)
            stack.append(g["optional"])
        elif a.get("group") == "]":
            stack.pop()
        else:
            stack[-1].append(a)
    return out


def parse_entry(name, rest, source):
    e = OrderedDict(name=name, source=source)
    m = re.search(r"\bimp\s+([\d,]+)", rest)
    if m:
        e["precedence"] = [int(x) for x in m.group(1).split(",")]
    m = re.search(r'\barg\s+((?:&?"[^"]*"|:|\[|\]|,|\s)+?)(?=\s+(?:ret|out|imp)\b|\s*$)', rest)
    e["args"] = parse_args(m.group(1)) if m else []
    m = re.search(r'\bret\s+"([^"]+)"(?::"([^"]+)")?', rest)
    e["ret"] = m.group(1) if m else None
    if m and m.group(2):
        e["ret_ctype"] = m.group(2)
    m = re.search(r"\bout\s+([\d$A-Za-z_,]+)", rest)
    if m:
        parts = m.group(1).split(",")
        e["opcode"] = int(parts[0]) if parts[0].isdigit() else parts[0]
        if len(parts) > 1:
            e["opcode_extra"] = parts[1:]
    return e


def find_file(dirpath, base):
    for f in os.listdir(dirpath):
        if f.lower() == base.lower() or f.lower() == base.lower() + ".tpl":
            return os.path.join(dirpath, f)
    return None


def parse_tpl(path, seen=None):
    seen = seen if seen is not None else set()
    if path in seen:
        return {}, []
    seen.add(path)
    text = open(path, "rb").read().decode("cp1251", errors="replace")
    text = strip_comments(text)
    sections = OrderedDict()
    current = "FUNCTIONS"
    entries = []
    source = os.path.basename(path)
    for line in text.splitlines():
        if not line.strip():
            continue
        m = INCLUDE_RE.match(line)
        if m:
            inc = find_file(os.path.dirname(path), m.group(1))
            if inc:
                _, sub = parse_tpl(inc, seen)
                entries.extend(sub)
            continue
        m = SECTION_RE.match(line)
        if m and m.group(1) in ("EPILOG", "OPERATORS", "FUNCTIONS"):
            current = m.group(1)
            continue
        m = ENTRY_RE.match(line)
        if m:
            e = parse_entry(m.group(1), m.group(3), source)
            e["section"] = current
            entries.append(e)
    return sections, entries


def parse_constants(path):
    text = open(path, "rb").read().decode("cp1251", errors="replace")
    consts = []
    group = ""
    for raw in text.splitlines():
        stripped = raw.strip()
        if stripped.startswith("//"):
            group = stripped[2:].strip()
            continue
        line = raw.split("//")[0].strip()
        if not line:
            continue
        m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s+(-?[\d.]+(?:[eE][-+]?\d+)?)\s*$", line)
        if m:
            v = m.group(2)
            consts.append({"name": m.group(1), "value": float(v) if "." in v or "e" in v.lower() else int(v), "group": group})
    return consts


def parse_tdl_dir(libdir):
    result = []
    for root, _, files in os.walk(libdir):
        for f in files:
            if f.lower().endswith(".tdl"):
                p = os.path.join(root, f)
                text = strip_comments(open(p, "rb").read().decode("cp1251", errors="replace"))
                dll = None
                for line in text.splitlines():
                    m = re.match(r'^\s*DLL\s+"([^"]+)"\s*(\d+)?', line, re.I)
                    if m:
                        dll = m.group(1)
                        continue
                    m = ENTRY_RE.match(line)
                    if m:
                        e = parse_entry(m.group(1), m.group(3), os.path.relpath(p, libdir))
                        e["dll"] = dll
                        aliases = re.findall(r'"([^"]+)"', m.group(2))
                        if aliases:
                            e["aliases"] = aliases
                        result.append(e)
    return result


def main():
    tpl_dir, out_dir = sys.argv[1:3]
    lib_dir = sys.argv[3] if len(sys.argv) > 3 else None
    os.makedirs(out_dir, exist_ok=True)
    _, entries = parse_tpl(find_file(tpl_dir, "COMPILER.TPL"))
    # _3D.TPL is an older copy of Graph3d.tpl and is not included by the
    # compiler table, so it is deliberately skipped.
    ops = [e for e in entries if e["section"] in ("OPERATORS", "EPILOG")]
    fns = [e for e in entries if e["section"] == "FUNCTIONS"]
    with open(os.path.join(out_dir, "builtins.json"), "w", encoding="utf-8") as f:
        json.dump(fns, f, ensure_ascii=False, indent=1)
    with open(os.path.join(out_dir, "operators.json"), "w", encoding="utf-8") as f:
        json.dump(ops, f, ensure_ascii=False, indent=1)
    consts = parse_constants(find_file(tpl_dir, "CONSTANT.TPL"))
    with open(os.path.join(out_dir, "constants.json"), "w", encoding="utf-8") as f:
        json.dump(consts, f, ensure_ascii=False, indent=1)
    if lib_dir:
        with open(os.path.join(out_dir, "tdl.json"), "w", encoding="utf-8") as f:
            json.dump(parse_tdl_dir(lib_dir), f, ensure_ascii=False, indent=1)
    names = {e["name"].lower() for e in fns}
    print("functions:", len(fns), "unique names:", len(names), "operators:", len(ops), "constants:", len(consts))


if __name__ == "__main__":
    main()
