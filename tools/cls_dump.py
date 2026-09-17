#!/usr/bin/env python3
"""Read Stratum 2000 `.cls` (image/class) files and dump them as JSON.

Usage:
    cls_dump.py PATH [PATH ...]        # dump one file as JSON to stdout
    cls_dump.py --scan DIR [DIR ...]   # parse everything, report statistics

The format is undocumented; what is implemented here was reconstructed from
the files in `fixtures/` (see docs/formats/cls.md). Everything is
little-endian; strings are `u16 length` + CP1251 bytes.

    "SB" u32 version u32 body_size u32 flags  str name
    then a sequence of `u16 section_id` + section body, ending with a 0x0000
    marker, followed by the section index: `u32 offset, u16 id` records up to
    end of file. Index offsets are relative to byte 6 of the file (right after
    the "SB" signature and the version word).
"""
import json
import os
import struct
import sys
from collections import Counter, OrderedDict

# body offsets in the index are counted from here (after "SB" + version)
INDEX_BASE = 6

TYPE_NAMES = ("FLOAT", "STRING", "HANDLE", "COLORREF", "INTEGER", "WORD", "BYTE", "POINTER")

# variable flag bits seen in the corpus
VAR_FLAGS = OrderedDict((
    (0x00000002, "local"),      # not exported through the contact place
    (0x00000080, "computed"),   # written by the image's own text
    (0x00000100, "in"),
    (0x00000200, "out"),
    (0x00000400, "inout"),
    (0x00020000, "present"),    # set on every variable of a v3 file
))


class ClsError(Exception):
    pass


class Reader:
    def __init__(self, data, path=""):
        self.b = data
        self.o = 0
        self.path = path

    def need(self, n):
        if self.o + n > len(self.b):
            raise ClsError("%s: truncated at 0x%x (want %d bytes)" % (self.path, self.o, n))

    def u8(self):
        self.need(1)
        v = self.b[self.o]
        self.o += 1
        return v

    def u16(self):
        self.need(2)
        v = struct.unpack_from("<H", self.b, self.o)[0]
        self.o += 2
        return v

    def u32(self):
        self.need(4)
        v = struct.unpack_from("<I", self.b, self.o)[0]
        self.o += 4
        return v

    def f64(self):
        self.need(8)
        v = struct.unpack_from("<d", self.b, self.o)[0]
        self.o += 8
        return v

    def string(self):
        n = self.u16()
        self.need(n)
        s = self.b[self.o:self.o + n].decode("cp1251", "replace")
        self.o += n
        return s

    def blob(self, n):
        self.need(n)
        v = self.b[self.o:self.o + n]
        self.o += n
        return v


def decode_var_flags(fl):
    names = [name for bit, name in VAR_FLAGS.items() if fl & bit]
    rest = fl & ~sum(VAR_FLAGS)
    if rest:
        names.append("0x%x" % rest)
    return names


def read_vars(r):
    out = []
    for _ in range(r.u16()):
        name = r.string()
        desc = r.string()
        default = r.string()
        vtype = r.string()
        flags = r.u32()
        out.append(OrderedDict(name=name, type=vtype, description=desc,
                               default=default, flags=decode_var_flags(flags),
                               raw_flags=flags))
    return out


def read_links(r, version):
    out = []
    for _ in range(r.u16()):
        src, dst, handle, flags = struct.unpack_from("<HHHI", r.b, r.o)
        r.o += 10
        npairs = r.u16()
        reserved = r.u16()
        pairs = [OrderedDict(source=r.string(), target=r.string()) for _ in range(npairs)]
        out.append(OrderedDict(source_handle=src, target_handle=dst, handle=handle,
                               flags=flags, reserved=reserved, vars=pairs))
    return out


def read_children(r, version):
    out = []
    for _ in range(r.u16()):
        cls = r.string()
        handle = r.u16()
        instance = r.string()   # empty when the instance keeps the class name
        # scheme coordinates were 16-bit pixels in file version 3.001
        if version >= 0x3002:
            x = r.f64()
            y = r.f64()
        else:
            x, y = struct.unpack_from("<hh", r.b, r.o)
            r.o += 4
        flags = r.u8()
        out.append(OrderedDict(klass=cls, handle=handle, name=instance,
                               x=x, y=y, flags=flags))
    return out


def read_sized_blob(r, kind):
    size = r.u32()
    data = r.blob(size - 4 if size >= 4 else 0)
    return OrderedDict(kind=kind, size=size, head=data[:16].hex())


# section id -> (name, reader)
def read_section(r, sid, version, index_at=None):
    if sid == 0x0F:
        return "vars", read_vars(r)
    if sid == 0x0A:
        return "text", r.string()
    if sid == 0x0E:
        return "description", r.string()
    if sid == 0x16:
        return "icon_file", r.string()
    if sid == 0x07:
        return "links", read_links(r, version)
    if sid == 0x11:
        return "children", read_children(r, version)
    if sid == 0x08:
        return "icon", read_sized_blob(r, "icon")
    if sid == 0x0C:
        return "scheme", read_sized_blob(r, "scheme")
    if sid == 0x09:
        return "image", read_sized_blob(r, "image")
    if sid == 0x04:
        return "flags", r.u32()
    if sid == 0x14:
        return "icon_index", r.u16()
    if sid == 0x0B:
        return "unknown_0b", r.u16()
    if sid == 0x0D:
        count = r.u32()
        return "bytecode", OrderedDict(words=count, data=r.blob(count * 2).hex())
    if sid == 0x15:
        return "timestamp", r.u32()
    if sid == 0x1E:
        # equations, used by the CHAINS/TOE circuit libraries. The word layout
        # is not decoded yet; the section always runs to the end of the body,
        # optionally followed by the 8-byte timestamp section.
        end = index_at
        if index_at is not None and index_at >= 8 and \
                struct.unpack_from("<H", r.b, index_at - 8)[0] == 0x15:
            end = index_at - 8
        return "equations", r.blob(max(0, end - r.o)).hex()
    raise ClsError("unknown section 0x%02x at 0x%x" % (sid, r.o - 2))


def parse(data, path=""):
    if data[:2] != b"SB":
        raise ClsError("%s: not a .cls file" % path)
    r = Reader(data, path)
    r.o = 2
    version = r.u32()
    body_size = r.u32()
    header_flags = r.u32()
    name = r.string()
    cls = OrderedDict(name=name, version="%x" % version, header_flags=header_flags,
                      sections=OrderedDict())
    index_at = body_size + INDEX_BASE
    while True:
        if r.o >= index_at:
            break
        sid = r.u16()
        if sid == 0:
            break
        key, value = read_section(r, sid, version, index_at)
        if key in cls["sections"]:
            key = "%s_%d" % (key, sum(1 for k in cls["sections"] if k.startswith(key)))
        cls["sections"][key] = value
    # section index at the tail
    r.o = index_at
    index = []
    while r.o + 6 <= len(data):
        off = r.u32()
        sid = r.u16()
        index.append(OrderedDict(id=sid, offset=off, absolute=off + INDEX_BASE))
    cls["index"] = index
    return cls


def flatten(cls):
    """A friendlier view: the parts a reimplementation actually needs."""
    s = cls["sections"]
    return OrderedDict(
        name=cls["name"],
        version=cls["version"],
        vars=s.get("vars", []),
        text=s.get("text", ""),
        description=s.get("description", ""),
        children=s.get("children", []),
        links=s.get("links", []),
        icon_file=s.get("icon_file"),
        icon_index=s.get("icon_index"),
        has_icon="icon" in s,
        has_scheme="scheme" in s,
        has_image="image" in s,
    )


def scan(paths):
    files = []
    for a in paths:
        if os.path.isdir(a):
            for root, _, names in os.walk(a):
                files += [os.path.join(root, n) for n in names if n.lower().endswith((".cls", "._cl", ".bak"))]
        else:
            files.append(a)
    ok = 0
    failures = []
    sections = Counter()
    versions = Counter()
    index_ok = index_bad = 0
    for p in sorted(files):
        try:
            data = open(p, "rb").read()
            if data[:2] != b"SB":
                continue
            cls = parse(data, p)
            ok += 1
            versions[cls["version"]] += 1
            for k in cls["sections"]:
                sections[k.rstrip("_0123456789")] += 1
            for e in cls["index"]:
                if e["absolute"] + 2 <= len(data) and struct.unpack_from("<H", data, e["absolute"])[0] == e["id"]:
                    index_ok += 1
                else:
                    index_bad += 1
        except Exception as exc:
            failures.append((p, str(exc)))
    print("parsed %d/%d files" % (ok, ok + len(failures)))
    print("versions:", dict(versions))
    print("sections:", dict(sections))
    print("index entries verified: %d ok, %d mismatched" % (index_ok, index_bad))
    if failures:
        print("failures:")
        for p, e in failures[:40]:
            print("   ", p, "--", e)
    return 1 if failures else 0


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 2
    if args[0] == "--scan":
        return scan(args[1:])
    for p in args:
        cls = parse(open(p, "rb").read(), p)
        print(json.dumps(flatten(cls), ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
