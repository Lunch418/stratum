#!/usr/bin/env python3
"""Read Stratum 2000 vector graphics (`.vdr` files and the icon/image/scheme
sections of `.cls`) and dump them as JSON.

Usage:
    vdr_dump.py PATH [PATH ...]        # dump one blob as JSON
    vdr_dump.py --scan DIR [DIR ...]   # parse every blob found, report

Reconstructed from the corpus; the layout is written up in
docs/formats/vdr.md. Two format generations matter: 2.x (no record sizes,
every record must be decoded exactly) and 3.x (every chunk and record carries
its size, unknown records can be skipped).
"""
import json
import os
import struct
import sys
from collections import Counter, OrderedDict

OBJECT_TYPES = {
    3: "group", 4: "rgroup", 5: "group3d", 10: "object3d", 20: "polyline",
    21: "bitmap", 22: "doublebitmap", 23: "text", 24: "view3d", 26: "control",
    50: "editframe", 51: "rotatecenter", 52: "frame3d", 53: "axis3d",
}
TOOL_TYPES = {101: "pen", 102: "brush", 103: "dib", 104: "doubledib",
              105: "font", 106: "string", 107: "text", 110: "dibref",
              111: "doubledibref"}
CHUNKS = {1004: "pens", 1005: "brushes", 1006: "dibs", 1007: "doubledibs",
          1008: "fonts", 1009: "strings", 1010: "texts", 1011: "textparts",
          1020: "objects", 1021: "zorder", 1022: "page"}


class VdrError(Exception):
    pass


class R:
    def __init__(self, b, path):
        self.b, self.o, self.path = b, 0, path
        self.wide = True

    def need(self, n):
        if self.o + n > len(self.b):
            raise VdrError("truncated at 0x%x (want %d)" % (self.o, n))

    def u8(self):
        self.need(1); v = self.b[self.o]; self.o += 1; return v

    def u16(self):
        self.need(2); v = struct.unpack_from("<H", self.b, self.o)[0]; self.o += 2; return v

    def i16(self):
        self.need(2); v = struct.unpack_from("<h", self.b, self.o)[0]; self.o += 2; return v

    def u32(self):
        self.need(4); v = struct.unpack_from("<I", self.b, self.o)[0]; self.o += 4; return v

    def i32(self):
        self.need(4); v = struct.unpack_from("<i", self.b, self.o)[0]; self.o += 4; return v

    def f64(self):
        self.need(8); v = struct.unpack_from("<d", self.b, self.o)[0]; self.o += 8; return v

    def s(self):
        n = self.u16(); self.need(n)
        v = self.b[self.o:self.o + n].decode("cp1251", "replace"); self.o += n; return v

    def bytes(self, n):
        self.need(n); v = self.b[self.o:self.o + n]; self.o += n; return v

    def num(self):
        """Coordinate: f64 from format 2.2 on, i16 before."""
        if self.wide:
            return self.f64()
        self.need(2); v = struct.unpack_from("<h", self.b, self.o)[0]; self.o += 2; return float(v)

    def peek16(self, off=0):
        if self.o + off + 2 <= len(self.b):
            return struct.unpack_from("<H", self.b, self.o + off)[0]
        return None


def parse(data, path=""):
    start = data.find(b"2D")
    if start < 0 or start > 16:
        raise VdrError("no 2D signature")
    r = R(data, path)
    r.o = start + 2
    major = r.u16()
    minor = r.u16()
    v3 = major >= 0x0300
    r.wide = major >= 0x0200
    doc = OrderedDict(version="%04x" % major, version2="%04x" % minor)
    if v3:
        doc["unknown"] = r.u16()
        doc["size"] = r.u32()
    else:
        doc["header_size"] = r.u32()
        doc["tools_at"] = r.u32()
    doc["origin"] = [r.num(), r.num()]
    doc["scale"] = [r.num(), r.num()]
    doc["window"] = [r.num(), r.num()]
    if v3:
        doc["flags"] = r.u16()
        doc["reserved"] = r.bytes(8).hex()
    else:
        doc["icons_path"] = r.s()
        doc["reserved"] = r.bytes(8).hex()
        doc["unknown_ffff"] = r.u32()
        doc["unknown_0"] = r.u16()
    doc["chunks"] = []
    try:
        while True:
            cid = r.peek16()
            if cid is not None and 1000 <= cid <= 1030:
                doc["chunks"].append(read_chunk(r, cid, v3))
                continue
            if not v3 and doc["tools_at"] and r.o <= start + doc["tools_at"] + 2:
                # 2.x: the objects end two bytes before `tools_at` (counted
                # from the 2D signature); then a u16 and the tool chunks
                r.o = start + doc["tools_at"] + 4
                continue
            break
    except VdrError as e:
        raise VdrError("%s (%s)" % (e, path))
    doc["tail_at"] = r.o
    doc["tail"] = r.b[r.o:r.o + 24].hex()
    return doc


def read_chunk(r, cid, v3):
    at = r.o
    r.u16()
    size = r.u32() if v3 else None
    count = r.u16()
    capacity = r.u16()
    delta = r.u16()
    chunk = OrderedDict(id=cid, name=CHUNKS.get(cid, "chunk_%d" % cid), at=at,
                        size=size, count=count, capacity=capacity, delta=delta, items=[])
    if cid == 1021:
        chunk["items"] = [r.u16() for _ in range(count)]
    elif cid == 1011:
        for _ in range(count):
            chunk["items"].append(OrderedDict(fg=r.u32(), bg=r.u32(), font=r.u16(), string=r.u16()))
    elif count:
        chunk["lead"] = r.u8()          # 1
        chunk["root"] = r.u16()         # 0xffff at top level
        for _ in range(count):
            chunk["items"].append(read_item(r, cid, v3))
    if size is not None:
        end = at + size
        if r.o != end:
            chunk["size_mismatch"] = r.o - end
            r.o = end
    return chunk


def read_item(r, cid, v3):
    at = r.o
    t = r.u16()
    size = r.u16() if v3 else None
    end = at + size if v3 else None
    if cid == 1020:
        item = OrderedDict(type=t, kind=OBJECT_TYPES.get(t, "object_%d" % t), size=size)
        if v3:
            item["reserved"] = r.u16()
            item["handle"] = r.u16()
            item["flags"] = r.u16()
            item["name"] = ""
        else:
            item["handle"] = r.u16()
            item["flags"] = r.u16()
            item["name"] = r.s()
        try:
            read_object(r, t, item, v3, end)
            # 3.x: the name travels in a tagged block at the end of the record
            if v3 and r.o + 6 <= end and r.peek16() == 0xCD:
                r.u16()
                n = r.u32()
                item["name"] = r.bytes(n - 6).decode("cp1251", "replace")
        except VdrError:
            if end is None:
                raise
            item["undecoded"] = True
    else:
        item = OrderedDict(type=t, kind=TOOL_TYPES.get(t, "tool_%d" % t), size=size)
        if v3 and t not in (103, 104):
            item["reserved"] = r.u16()
        item["refs"] = r.u16()
        item["handle"] = r.u16()
        try:
            read_tool(r, t, item, v3, end)
        except VdrError:
            if end is None:
                raise
            item["undecoded"] = True
    if end is not None:
        if r.o != end:
            item["size_mismatch"] = r.o - end
        r.o = end
    return item


def read_object(r, t, item, v3, end):
    if t in (3, 4, 5):                    # groups: only a list of children
        cid = r.peek16()
        if cid == 1021:
            item["children"] = read_chunk(r, cid, v3)["items"]
        else:
            n = r.u16()
            item["children"] = [r.u16() for _ in range(n)]
        return
    item["origin"] = [r.num(), r.num()]
    item["size_xy"] = [r.num(), r.num()]
    if t == 20:                           # polyline
        item["pen"] = r.u16()
        item["brush"] = r.u16()
        if not r.wide:                    # 1.x: no attachment byte
            n = r.u16()
            item["points"] = [[r.num(), r.num()] for _ in range(n)]
            return
        n = r.u16()
        item["points"] = [[r.num(), r.num()] for _ in range(n)]
        extra = r.u8()                    # optional attachment, e.g. arrow data
        if extra:
            item["extra"] = r.bytes(extra).hex()
    elif t == 23:                         # text
        item["text_tool"] = r.u16()
        item["unknown"] = r.bytes(18).hex()
    elif t == 26:                         # control (window with a class)
        item["class"] = r.s()
        item["caption"] = r.s()
        item["unknown_a"] = r.u16()
        item["style"] = r.u32()
        item["unknown_b"] = r.u16()
        item["unknown_c"] = r.u16()
        item["width"] = r.u16()
        item["height"] = r.u16()
        item["unknown_d"] = r.u16()
    elif t in (21, 22):                   # bitmap, double bitmap
        item["src"] = [r.num(), r.num(), r.num(), r.num()]
        item["unknown"] = r.u16()
        item["dib"] = r.u16()
    else:
        raise VdrError("object type %d not decoded" % t)


def read_tool(r, t, item, v3, end):
    if t == 101:                          # pen
        item["color"] = r.u32()
        item["style"] = r.u16()
        item["width"] = r.u16()
        item["rop"] = r.u16()
    elif t == 102:                        # brush
        item["color"] = r.u32()
        item["style"] = r.u16()
        item["hatch"] = r.u16()
        item["rop"] = r.u16()
        item["dib"] = r.u16()
    elif t == 105:                        # font: 16-bit LOGFONT
        item["height"] = r.i16()
        item["width"] = r.i16()
        item["escapement"] = r.i16()
        item["orientation"] = r.i16()
        item["weight"] = r.i16()
        item["attrs"] = r.bytes(8).hex()
        face = r.bytes(32)
        item["face"] = face.split(b"\0")[0].decode("cp1251", "replace")
        if v3:
            item["extra"] = r.bytes(8).hex()
    elif t == 106:                        # string
        item["text"] = r.s()
    elif t == 107:                        # text = list of (font, string, colors)
        item["parts"] = read_chunk(r, r.peek16(), v3)["items"]
    elif t in (110, 111):                 # reference to an icon sheet file
        item["file"] = r.s()
    elif t in (103, 104):                 # embedded BMP (+ 1-bit mask for 104)
        item["bmp_size"] = read_bmp(r, item, "bmp")
        if t == 104:
            item["mask_size"] = read_bmp(r, item, "mask")
    elif t == 34:                         # page settings (chunk 1022)
        item["raw"] = r.bytes(34).hex()
    else:
        raise VdrError("tool type %d not decoded" % t)


def read_bmp(r, item, key):
    if r.peek16() != 0x4D42:              # "BM"
        raise VdrError("dib without BM header")
    size = struct.unpack_from("<I", r.b, r.o + 2)[0]
    r.need(size)
    item[key] = r.b[r.o:r.o + 16].hex() + "..."
    r.o += size
    return size


def scan(paths):
    files = []
    for a in paths:
        if os.path.isdir(a):
            for root, _, names in os.walk(a):
                files += [os.path.join(root, n) for n in names]
        else:
            files.append(a)
    ok, bad = 0, Counter()
    examples = {}
    versions = Counter()
    mism = Counter()
    undecoded = Counter()
    for p in sorted(files):
        d = open(p, "rb").read()
        if d.find(b"2D") not in range(0, 17):
            continue
        try:
            doc = parse(d, p)
            versions[doc["version"]] += 1
            ok += 1
            for c in doc["chunks"]:
                if "size_mismatch" in c:
                    mism[(c["name"], c["size_mismatch"])] += 1
                for it in c["items"]:
                    if isinstance(it, dict):
                        if "size_mismatch" in it:
                            mism[(it["kind"], it["size_mismatch"])] += 1
                        if it.get("undecoded"):
                            undecoded[it["kind"]] += 1
        except VdrError as e:
            key = str(e).split(" (")[0]
            key = key.split(" at 0x")[0]
            bad[key] += 1
            examples.setdefault(key, p)
    print("parsed %d, failed %d" % (ok, sum(bad.values())))
    print("versions:", dict(versions))
    for k, v in bad.most_common(15):
        print("  %4d  %s   e.g. %s" % (v, k, os.path.basename(examples[k])))
    if mism:
        print("size mismatches (kind, bytes):", mism.most_common(12))
    if undecoded:
        print("undecoded (skipped by size):", dict(undecoded))


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__); return 2
    if args[0] == "--scan":
        return scan(args[1:])
    for p in args:
        print(json.dumps(parse(open(p, "rb").read(), p), ensure_ascii=False, indent=1))


if __name__ == "__main__":
    sys.exit(main())
