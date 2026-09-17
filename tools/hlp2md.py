#!/usr/bin/env python3
"""Convert a helpdeco-produced RTF dump of SC3.HLP into Markdown.

Usage:
    hlp2md.py SC3.rtf sc3.cnt OUT_DIR

Produces:
    OUT_DIR/topics/<context>.md   one file per help topic
    OUT_DIR/README.md             table of contents built from sc3.cnt
    OUT_DIR/functions.json        machine-readable function reference
    OUT_DIR/functions.md          human-readable function table

Images (bmN.bmp / bmN.wmf next to the RTF) are referenced as
OUT_DIR/images/bmN.png; convert them separately (see tools/README.md).
"""
import json
import os
import re
import sys
from collections import OrderedDict

# markers used while rendering, replaced in post-processing
LINK_START = "\x01"
LINK_TGT_START = "\x02"
LINK_TGT_END = "\x03"
BOLD_ON = "\x04"
BOLD_OFF = "\x05"
CELL = "\x06"
ROW = "\x07"

SECTION_HEADERS = {
    "Синтаксис", "Описание", "Параметры", "Возвращаемое значение", "Пример",
    "Примеры", "См. также", "Замечания", "Примечание", "Примечания",
}

TOKEN_RE = re.compile(
    r"\\'([0-9a-fA-F]{2})"      # 1: hex-escaped byte
    r"|\\([a-zA-Z]+)(-?\d+)? ?"  # 2,3: control word with optional numeric param
    r"|\\(.)"                    # 4: control symbol
    r"|([{}])"                   # 5: group delimiters
    r"|([^\\{}]+)",              # 6: plain text
    re.S,
)


def decode_hex_run(buf):
    return bytes(buf).decode("cp1251", errors="replace")


def render_topic(page):
    """Render one RTF page into (meta, text-with-markers)."""
    meta = {"title": None, "context": None, "keywords": [], "browse": None}
    out = []
    hexbuf = []
    depth = 0
    # stack entries: dict(kind) ; kind in {"", "footnote:$", "skip", "target"}
    stack = []
    skip_depth = None
    fn_kind = None
    fn_buf = []
    tgt_buf = None
    tgt_depth = None
    pending_footnote = None  # the {\up X} tells the kind of the following footnote

    def flush_hex():
        if hexbuf:
            s = decode_hex_run(hexbuf)
            hexbuf.clear()
            return s
        return ""

    def emit(s):
        if not s:
            return
        if skip_depth is not None:
            return
        if fn_kind is not None:
            fn_buf.append(s)
        elif tgt_buf is not None:
            tgt_buf.append(s)
        else:
            out.append(s)

    for m in TOKEN_RE.finditer(page):
        hx, word, param, sym, grp, text = m.groups()
        if hx is not None:
            hexbuf.append(int(hx, 16))
            continue
        s = flush_hex()
        if s:
            emit(s)
        if grp == "{":
            depth += 1
            stack.append(depth)
            continue
        if grp == "}":
            if skip_depth == depth:
                skip_depth = None
            if fn_kind is not None and fn_depth == depth:
                val = "".join(fn_buf).strip()
                if fn_kind == "$":
                    meta["title"] = val
                elif fn_kind == "#":
                    meta["context"] = val
                elif fn_kind == "K":
                    meta["keywords"] = [k.strip() for k in val.split(";") if k.strip()]
                elif fn_kind == "+":
                    meta["browse"] = val
                fn_kind = None
                fn_buf = []
            if tgt_buf is not None and tgt_depth == depth:
                target = "".join(tgt_buf).strip()
                out.append(LINK_TGT_START + target + LINK_TGT_END)
                tgt_buf = None
            depth -= 1
            if stack:
                stack.pop()
            continue
        if text is not None:
            emit(text.replace("\r", "").replace("\n", ""))
            continue
        if sym is not None:
            if sym in "{}\\":
                emit(sym)
            elif sym == "~":
                emit(" ")
            elif sym == "-":
                pass
            elif sym == "_":
                emit("-")
            # \* and others: ignore
            continue
        # control word
        if word == "footnote":
            fn_kind = pending_footnote or "?"
            fn_depth = depth
            fn_buf = []
            pending_footnote = None
        elif word == "up":
            # {\up X} precedes a footnote; the X is emitted as text into the
            # current group, so capture it via a small trick: mark and remove.
            emit("\x08")
        elif word == "v":
            tgt_buf = []
            tgt_depth = depth
        elif word in ("pict", "fonttbl", "colortbl", "stylesheet", "info"):
            skip_depth = depth
        elif word == "par" or word == "line":
            emit("\n")
        elif word == "tab":
            emit("\t")
        elif word == "uldb" or word == "ul":
            emit(LINK_START)
        elif word == "b":
            emit(BOLD_OFF if param == "0" else BOLD_ON)
        elif word == "plain":
            emit(BOLD_OFF)
        elif word == "cell":
            emit(CELL)
        elif word == "row":
            emit(ROW)
        elif word == "u" and param is not None:
            cp = int(param)
            if cp < 0:
                cp += 65536
            emit(chr(cp))
        elif word == "bmc" or word == "bml" or word == "bmr":
            emit("\x09IMG:")
        # everything else (formatting) is ignored
    emit(flush_hex())
    raw = "".join(out)
    # footnote kind: "\x08$" etc. were emitted before footnotes; the footnote
    # group itself contains "\x08$ Title". Extract kind from fn_buf handled
    # above as fn_kind "?" — so re-derive from footnote text prefix instead.
    return meta, raw


def fix_meta_from_raw(page, meta):
    # helpdeco writes: {\up $}{\footnote\pard\plain{\up $} Title}
    for kind, key in (("$", "title"), ("#", "context")):
        m = re.search(r"\{\\footnote\\pard\\plain\{\\up \\" + re.escape(kind) + r"\}\s*([^}]*)\}", page)
        if not m:
            m = re.search(r"\{\\footnote\\pard\\plain\{\\up " + re.escape(kind) + r"\}\s*([^}]*)\}", page)
        if m:
            val = m.group(1)
            val = re.sub(r"\\'([0-9a-fA-F]{2})", lambda mm: bytes([int(mm.group(1), 16)]).decode("cp1251"), val)
            meta[key] = val.strip()
    m = re.search(r"\{\\footnote\\pard\\plain\{\\up K\}\s*([^}]*)\}", page)
    if m:
        val = re.sub(r"\\'([0-9a-fA-F]{2})", lambda mm: bytes([int(mm.group(1), 16)]).decode("cp1251"), m.group(1))
        meta["keywords"] = [k.strip() for k in val.split(";") if k.strip()]
    return meta


def postprocess(raw):
    # drop the {\up X} markers and footnote leftovers
    raw = re.sub(r"\x08.?", "", raw)
    # image references
    raw = re.sub(r"\x09IMG:\s*([\w.]+)", r"![\1](images/\1)", raw)
    raw = re.sub(r"\{bm[clr] ([\w]+)\.(bmp|wmf)\}", lambda m: "![%s](../images/%s.png)" % (m.group(1), m.group(1)), raw)
    # links: LINK_START text ... LINK_TGT_START target LINK_TGT_END
    raw = re.sub(
        LINK_START + r"([^\x01\x02\x03]*?)" + BOLD_OFF + r"?\s*" + LINK_TGT_START + r"([^\x03]*)" + LINK_TGT_END,
        lambda m: "[%s](%s)" % (m.group(1).strip(), link_target(m.group(2))),
        raw,
    )
    raw = raw.replace(LINK_START, "").replace(LINK_TGT_START, "").replace(LINK_TGT_END, "")
    raw = raw.replace(ROW, ROW + "\n")
    lines = raw.split("\n")
    result = []
    in_table = False
    header_done = False
    bold_active = False
    for line in lines:
        was_bold = bold_active
        for ch in line:
            if ch == BOLD_ON:
                bold_active = True
            elif ch == BOLD_OFF:
                bold_active = False
        if ROW in line or CELL in line:
            cells = [c.strip() for c in line.replace(ROW, "").split(CELL)]
            cells = [strip_bold(c) for c in cells]
            if cells and cells[-1] == "":
                cells.pop()
            if not any(cells):
                continue
            if not in_table:
                in_table = True
                header_done = False
                result.append("")
            result.append("| " + " | ".join(c.replace("|", "\\|") for c in cells) + " |")
            if not header_done:
                result.append("|" + "---|" * len(cells))
                header_done = True
            continue
        if in_table:
            in_table = False
            result.append("")
        line = line.rstrip()
        stripped = strip_bold(line).strip()
        if (BOLD_ON in line or was_bold) and stripped and stripped.rstrip(":") in SECTION_HEADERS:
            result.append("")
            result.append("### " + stripped.rstrip(":"))
            result.append("")
            continue
        if was_bold and BOLD_ON not in line:
            line = BOLD_ON + line
        if bold_active and BOLD_OFF not in line[line.rfind(BOLD_ON) + 1:]:
            line = line + BOLD_OFF
        line = bold_to_md(line)
        line = line.replace("\t", "    ")
        result.append(line)
    text = "\n".join(result)
    text = re.sub(r"[ \t]+\n", "\n", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip() + "\n"


def link_target(t):
    t = t.strip()
    if "@" in t:  # external file reference: ctx@file.hlp
        t = t.split("@")[0]
    if ">" in t:  # window spec
        t = t.split(">")[0]
    return t + ".md"


def strip_bold(s):
    return s.replace(BOLD_ON, "").replace(BOLD_OFF, "")


def bold_to_md(line):
    out = []
    bold = False
    buf = []
    for ch in line:
        if ch == BOLD_ON:
            if not bold:
                bold = True
                buf = []
            continue
        if ch == BOLD_OFF:
            if bold:
                bold = False
                txt = "".join(buf)
                if txt.strip():
                    lead = len(txt) - len(txt.lstrip())
                    trail = len(txt) - len(txt.rstrip())
                    out.append(txt[:lead] + "**" + txt.strip() + "**" + (txt[len(txt) - trail:] if trail else ""))
                else:
                    out.append(txt)
            continue
        (buf if bold else out).append(ch)
    if bold:
        txt = "".join(buf)
        if txt.strip():
            out.append("**" + txt.strip() + "**")
    return "".join(out)


SIG_RE = re.compile(r"^\s*(?:(FLOAT|STRING|HANDLE|COLORREF|INTEGER|WORD|BYTE|POINTER)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\((.*)\)\s*$")


def extract_function(md, context, title):
    """Pull structured data out of a function topic, or return None."""
    if "### Синтаксис" not in md:
        return None
    sections = split_sections(md)
    sigs = []
    for line in sections.get("Синтаксис", "").split("\n"):
        line = line.replace("**", "").strip()
        m = SIG_RE.match(line)
        if m:
            sigs.append({"ret": m.group(1), "name": m.group(2), "args": [a.strip() for a in m.group(3).split(",") if a.strip()], "text": line})
    if not sigs:
        return None
    params = []
    for line in sections.get("Параметры", "").split("\n"):
        if not line.strip() or line.startswith("|"):
            continue
        parts = re.split(r"\s{2,}|\t", line.strip(), maxsplit=1)
        if len(parts) == 2:
            params.append({"name": parts[0].strip("* "), "desc": parts[1].strip()})
    desc = sections.get("Описание", "").strip()
    return {
        "name": sigs[0]["name"],
        "context": context,
        "title": title,
        "signatures": sigs,
        "params": params,
        "description": desc,
        "returns": sections.get("Возвращаемое значение", "").strip(),
        "example": sections.get("Пример", sections.get("Примеры", "")).strip(),
        "see_also": re.findall(r"\[([^\]]+)\]\(([^)]+)\)", sections.get("См. также", "") + "\n" + tail_links(md)),
    }


def tail_links(md):
    # "см. [Графика 2D](Graphics2d.md)" lines at the bottom
    return "\n".join(l for l in md.split("\n") if l.strip().lower().startswith("см."))


def split_sections(md):
    sections = {}
    cur = "_"
    buf = []
    for line in md.split("\n"):
        if line.startswith("### "):
            sections[cur] = "\n".join(buf)
            cur = line[4:].strip()
            buf = []
        else:
            buf.append(line)
    sections[cur] = "\n".join(buf)
    return sections


def parse_cnt(path):
    entries = []
    with open(path, "rb") as f:
        for raw in f.read().decode("cp1251").splitlines():
            m = re.match(r"^(\d+)\s+(.*?)(?:=(\S+))?\s*$", raw)
            if m:
                entries.append((int(m.group(1)), m.group(2).strip(), m.group(3)))
    return entries


def main():
    rtf_path, cnt_path, out_dir = sys.argv[1:4]
    topics_dir = os.path.join(out_dir, "topics")
    os.makedirs(topics_dir, exist_ok=True)
    data = open(rtf_path, "rb").read().decode("cp1251")
    pages = data.split("\\page")
    topics = OrderedDict()
    functions = OrderedDict()
    seen = {}
    for page in pages:
        meta, raw = render_topic(page)
        meta = fix_meta_from_raw(page, meta)
        ctx = meta.get("context")
        if not ctx:
            continue
        md = postprocess(raw)
        title = meta.get("title") or ctx
        fname = ctx
        if fname in seen:
            seen[fname] += 1
            fname = "%s_%d" % (ctx, seen[fname])
        else:
            seen[fname] = 0
        first, _, rest = md.partition("\n")
        if first.replace("**", "").strip().rstrip(" !") == title.strip():
            md = rest.lstrip("\n")
        body = "# %s\n\n" % title
        if meta.get("keywords"):
            body += "_Ключевые слова: %s_\n\n" % ", ".join(meta["keywords"])
        body += md
        with open(os.path.join(topics_dir, fname + ".md"), "w", encoding="utf-8") as f:
            f.write(body)
        topics[ctx] = title
        fn = extract_function(md, ctx, title)
        if fn:
            functions[fn["name"]] = fn

    # group assignment: walk the link graph from the function-group topics
    # listed in sc3.cnt (two levels deep, so Ogre sub-pages are covered)
    cnt = parse_cnt(cnt_path)
    group_roots = [(ctx, title) for level, title, ctx in cnt
                   if ctx and ctx != "Function"
                   and (ctx.lower().endswith(("function", "functions")) or ctx == "Ogre3d")]
    seen_roots = set()
    group_roots = [r for r in group_roots if not (r[0] in seen_roots or seen_roots.add(r[0]))]
    def links_of(ctx):
        p = os.path.join(topics_dir, ctx + ".md")
        if not os.path.exists(p):
            return []
        return re.findall(r"\]\(([A-Za-z0-9_]+)\.md\)", open(p, encoding="utf-8").read())
    groups = {}
    for root, title in group_roots:
        for l1 in links_of(root):
            groups.setdefault(l1, title)
            if l1 not in functions and l1 != root:
                for l2 in links_of(l1):
                    groups.setdefault(l2, title)
    by_ctx = {fn["context"]: fn for fn in functions.values()}
    for name, fn in functions.items():
        fn["group"] = groups.get(fn["context"], "")
    # keywords that are not functions (operators) — drop them
    for kw in ("if", "while", "case", "until", "for", "switch", "do", "repeat", "function", "return"):
        functions.pop(kw, None)

    with open(os.path.join(out_dir, "functions.json"), "w", encoding="utf-8") as f:
        json.dump(list(functions.values()), f, ensure_ascii=False, indent=1)

    with open(os.path.join(out_dir, "functions.md"), "w", encoding="utf-8") as f:
        f.write("# Справочник функций Stratum 2000\n\n")
        f.write("Извлечено из `SC3.HLP`. Всего функций: %d.\n\n" % len(functions))
        by_group = OrderedDict()
        for fn in functions.values():
            by_group.setdefault(fn["group"] or "Прочие", []).append(fn)
        for g, fns in by_group.items():
            f.write("## %s\n\n| Функция | Сигнатура | Описание |\n|---|---|---|\n" % g)
            for fn in sorted(fns, key=lambda x: x["name"].lower()):
                sig = "<br>".join("`%s`" % s["text"] for s in fn["signatures"])
                d = fn["description"].split("\n")[0].replace("|", "\\|")
                f.write("| [%s](topics/%s.md) | %s | %s |\n" % (fn["name"], fn["context"], sig, d))
            f.write("\n")

    with open(os.path.join(out_dir, "README.md"), "w", encoding="utf-8") as f:
        f.write("# Справка Stratum 2000 (SC3.HLP)\n\n")
        f.write("Декомпилировано из WinHelp. Тем: %d. См. также [functions.md](functions.md).\n\n" % len(topics))
        for level, title, ctx in parse_cnt(cnt_path):
            indent = "  " * (level - 1)
            if ctx and ctx in topics:
                f.write("%s- [%s](topics/%s.md)\n" % (indent, title, ctx))
            else:
                f.write("%s- %s\n" % (indent, title))
    print("topics:", len(topics), "functions:", len(functions))


if __name__ == "__main__":
    main()
