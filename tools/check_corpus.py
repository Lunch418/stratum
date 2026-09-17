#!/usr/bin/env python3
"""Check the extracted model texts against the function and constant tables.

Usage:
    check_corpus.py CORPUS_DIR docs/lang

Reports every identifier that is called like a function but is not in
`functions.json`, and every all-caps identifier that looks like a constant but
is not in `constants.json`. This is the smoke test for the reference tables:
anything listed here is something the new interpreter would not know about.
"""
import json
import os
import re
import sys
from collections import Counter

CALL = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(")
WORD = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\b")

KEYWORDS = {
    "if", "else", "endif", "while", "endwhile", "for", "next", "do", "until",
    "repeat", "switch", "case", "default", "endswitch", "break", "continue",
    "function", "return", "var", "local", "and", "or", "not",
    "float", "string", "handle", "colorref", "integer", "word", "byte", "pointer",
}


def strip_comments_and_strings(text):
    text = re.sub(r"//[^\n]*", "", text)
    return re.sub(r'"(?:[^"\\]|\\.)*"', '""', text)


def main():
    corpus, lang = sys.argv[1], sys.argv[2]
    functions = {f["name"].lower() for f in json.load(open(os.path.join(lang, "functions.json"), encoding="utf-8"))}
    # functions living in DLLs, declared by the .tdl descriptors
    tdl_path = os.path.join(lang, "tdl.json")
    if os.path.exists(tdl_path):
        for e in json.load(open(tdl_path, encoding="utf-8")):
            functions.add(e["name"].lower())
            functions.update(a.lower() for a in e.get("aliases", []))
    # user-written functions: every image in the corpus is callable by name
    index_path = os.path.join(corpus, "index.json")
    if os.path.exists(index_path):
        functions.update(i["name"].lower() for i in json.load(open(index_path, encoding="utf-8")))
    constants = {c["name"].lower() for c in json.load(open(os.path.join(lang, "constants.json"), encoding="utf-8"))}
    unknown_calls = Counter()
    unknown_consts = Counter()
    files = 0
    for root, _, names in os.walk(corpus):
        for n in names:
            if not n.endswith(".strat"):
                continue
            files += 1
            text = strip_comments_and_strings(open(os.path.join(root, n), encoding="utf-8").read())
            declared = set()
            for line in text.split("\n"):
                m = re.match(r"\s*(FLOAT|STRING|HANDLE|COLORREF|INTEGER|WORD|BYTE|POINTER)\s+(.*)",
                             line, re.I)
                if m:
                    declared |= {w.lower() for w in WORD.findall(m.group(2))}
            called = {c.lower() for c in CALL.findall(text)}
            for c in called - functions - KEYWORDS - declared:
                unknown_calls[c] += 1
            for w in WORD.findall(text):
                if w.isupper() and len(w) > 2 and w.lower() not in functions \
                        and w.lower() not in constants and w.lower() not in declared \
                        and w.lower() not in KEYWORDS:
                    unknown_consts[w] += 1
    print("files: %d" % files)
    print("unknown calls: %d distinct, %d uses" % (len(unknown_calls), sum(unknown_calls.values())))
    for name, n in unknown_calls.most_common(30):
        print("   %-28s %d" % (name, n))
    print("unknown CONSTANT-looking words: %d distinct" % len(unknown_consts))
    for name, n in unknown_consts.most_common(20):
        print("   %-28s %d" % (name, n))


if __name__ == "__main__":
    main()
