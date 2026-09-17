#!/usr/bin/env python3
"""Merge the two sources of truth about built-in functions into one reference.

Usage:
    merge_functions.py docs/lang/builtins.json docs/help/functions.json OUT_DIR

`builtins.json` comes from the compiler tables shipped with Stratum
(`template/*.tpl`): exact argument types, by-reference flags, overloads and the
opcode each overload compiles to — but no prose.

`functions.json` comes from the decompiled help: description, parameter
meanings, return value, example, and the group the function belongs to.

The result is `functions.json` + `functions.md` in OUT_DIR, keyed by the
lower-cased name (the language is case-insensitive).
"""
import json
import os
import sys
from collections import OrderedDict


def signature(entry):
    def one(a):
        if "optional" in a:
            return "[" + ", ".join(one(x) for x in a["optional"]) + "]"
        return ("&" if a.get("byref") else "") + a["type"]
    args = ", ".join(one(a) for a in entry["args"])
    ret = (entry.get("ret") + " ") if entry.get("ret") else ""
    return "%s%s(%s)" % (ret, entry["name"], args)


def main():
    builtins_path, help_path, out_dir = sys.argv[1:4]
    builtins = json.load(open(builtins_path, encoding="utf-8"))
    helps = {f["name"].lower(): f for f in json.load(open(help_path, encoding="utf-8"))}

    merged = OrderedDict()
    for e in builtins:
        key = e["name"].lower()
        fn = merged.setdefault(key, OrderedDict(
            name=e["name"], overloads=[], group="", description="",
            params=[], returns="", example="", help_topic=None, source="tpl"))
        fn["overloads"].append(OrderedDict(
            signature=signature(e), args=e["args"], ret=e.get("ret"),
            opcode=e.get("opcode"), table=e["source"]))

    for key, h in helps.items():
        fn = merged.get(key)
        if fn is None:
            fn = merged[key] = OrderedDict(
                name=h["name"], overloads=[], group="", description="",
                params=[], returns="", example="", help_topic=None, source="help")
            for s in h["signatures"]:
                fn["overloads"].append(OrderedDict(
                    signature=s["text"], args=s["args"], ret=s["ret"],
                    opcode=None, table=None))
        else:
            fn["source"] = "tpl+help"
        fn["group"] = h.get("group", "")
        fn["description"] = h.get("description", "")
        fn["params"] = h.get("params", [])
        fn["returns"] = h.get("returns", "")
        fn["example"] = h.get("example", "")
        fn["help_topic"] = h["context"]

    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "functions.json"), "w", encoding="utf-8") as f:
        json.dump(list(merged.values()), f, ensure_ascii=False, indent=1)

    by_group = OrderedDict()
    for fn in merged.values():
        by_group.setdefault(fn["group"] or "Без группы (только в таблицах компилятора)", []).append(fn)

    with open(os.path.join(out_dir, "functions.md"), "w", encoding="utf-8") as f:
        f.write("# Встроенные функции Stratum 2000\n\n")
        f.write("Сведено из таблиц компилятора (`template/*.tpl`: типы аргументов и "
                "опкоды) и декомпилированной справки `SC3.HLP` (описания).\n\n")
        f.write("Всего имён: %d. Из них в обоих источниках: %d, только в таблицах: %d, "
                "только в справке: %d.\n\n" % (
                    len(merged),
                    sum(1 for x in merged.values() if x["source"] == "tpl+help"),
                    sum(1 for x in merged.values() if x["source"] == "tpl"),
                    sum(1 for x in merged.values() if x["source"] == "help")))
        for group, fns in sorted(by_group.items(), key=lambda kv: -len(kv[1])):
            f.write("## %s (%d)\n\n" % (group, len(fns)))
            f.write("| Функция | Сигнатуры | Опкод | Описание |\n|---|---|---|---|\n")
            for fn in sorted(fns, key=lambda x: x["name"].lower()):
                sigs = "<br>".join("`%s`" % o["signature"] for o in fn["overloads"])
                ops = ", ".join(str(o["opcode"]) for o in fn["overloads"] if o["opcode"] is not None)
                desc = fn["description"].split("\n")[0].replace("|", "\\|")
                name = fn["name"]
                if fn["help_topic"]:
                    name = "[%s](../help/topics/%s.md)" % (name, fn["help_topic"])
                f.write("| %s | %s | %s | %s |\n" % (name, sigs, ops, desc))
            f.write("\n")
    print("functions: %d (tpl+help %d, tpl only %d, help only %d)" % (
        len(merged),
        sum(1 for x in merged.values() if x["source"] == "tpl+help"),
        sum(1 for x in merged.values() if x["source"] == "tpl"),
        sum(1 for x in merged.values() if x["source"] == "help")))


if __name__ == "__main__":
    main()
