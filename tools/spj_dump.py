#!/usr/bin/env python3
"""Read Stratum 2000 project files: `project.spj` and `_preload.stt`.

Usage:
    spj_dump.py PATH [PATH ...]      # dump as JSON
    spj_dump.py --scan DIR           # parse every .spj/.stt found, report

project.spj
    "Ih" + kind ('d' or 'f') + u8 0
    kind 'f': u16 count, then that many property records
    kind 'd': str root_class, then optional blocks introduced by a marker
              byte: 'f' = property records, 'g' = project variables
    Property record: u16 size, u8 type, u8 key_length, NUL-terminated key,
              value filling the rest of the record (u32 for type 0, text for 2)
    Project variable: u16 kind, u16 flags, [u16 handle when kind == 2],
              str name, str description

_preload.stt  — the values every image had when the project was last saved
    str "SC Scheme Variables", str root_class, u16 size, u16 count, ...
    then per image: str class_name, u32 handle, u16 var_count,
              var_count * (str name, str value)
"""
import json
import os
import struct
import sys
from collections import OrderedDict

PROP_TYPES = {0: "int", 2: "string"}


class SpjError(Exception):
    pass


class Reader:
    def __init__(self, data, path=""):
        self.b = data
        self.o = 0
        self.path = path

    def u8(self):
        v = self.b[self.o]
        self.o += 1
        return v

    def u16(self):
        v = struct.unpack_from("<H", self.b, self.o)[0]
        self.o += 2
        return v

    def u32(self):
        v = struct.unpack_from("<I", self.b, self.o)[0]
        self.o += 4
        return v

    def string(self):
        n = self.u16()
        if self.o + n > len(self.b):
            raise SpjError("%s: string of %d bytes at 0x%x runs past the end" % (self.path, n, self.o))
        s = self.b[self.o:self.o + n].decode("cp1251", "replace")
        self.o += n
        return s

    def eof(self):
        return self.o >= len(self.b)


def read_properties(r, count):
    props = []
    for _ in range(count):
        size = r.u16()
        end = r.o + size
        ptype = r.u8()
        keylen = r.u8()
        key = r.b[r.o:r.o + keylen].split(b"\0")[0].decode("cp1251", "replace")
        r.o += keylen
        raw = r.b[r.o:end]
        r.o = end
        if ptype == 0 and len(raw) >= 4:
            value = struct.unpack_from("<I", raw, 0)[0]
        else:
            value = raw.split(b"\0")[0].decode("cp1251", "replace")
        props.append(OrderedDict(key=key, type=PROP_TYPES.get(ptype, ptype), value=value))
    return props


def read_project_vars(r, count):
    out = []
    for _ in range(count):
        kind = r.u16()
        flags = r.u16()
        handle = r.u16() if kind == 2 else None
        name = r.string()
        desc = r.string()
        out.append(OrderedDict(kind=kind, flags=flags, handle=handle,
                               name=name, description=desc))
    return out


def parse_spj(data, path=""):
    if data[:2] != b"Ih":
        raise SpjError("%s: not a .spj file" % path)
    kind = chr(data[2])
    r = Reader(data, path)
    r.o = 4
    out = OrderedDict(kind=kind, properties=[], variables=[], root=None)
    if kind == "f":
        out["properties"] = read_properties(r, r.u16())
        r.u16()  # block terminator / size word
        out["root"] = r.string()
    elif kind == "d":
        out["root"] = r.string()
    else:
        raise SpjError("%s: unknown project kind %r" % (path, kind))
    while not r.eof():
        marker = r.u8()
        if marker == 0:
            break
        if marker == ord("f"):
            r.u8()
            out["properties"] += read_properties(r, r.u16())
        elif marker == ord("g"):
            r.u8()
            out["variables"] += read_project_vars(r, r.u16())
        else:
            raise SpjError("%s: unknown block marker %r at 0x%x" % (path, chr(marker), r.o - 1))
    return out


def parse_stt(data, path=""):
    r = Reader(data, path)
    magic = r.string()
    if not magic.startswith("SC "):
        raise SpjError("%s: not a state file (%r)" % (path, magic))
    root = r.string()
    out = OrderedDict(magic=magic, root=root, images=[])
    out["record_size"] = r.u16()   # 0x28 in the current revision
    out["format"] = r.u16()
    # the current revision has a third header word; the older one (DEFAULT.STT
    # and the numbered snapshots) repeats the root name here and then stores
    # variables by index instead of by name
    if struct.unpack_from("<H", r.b, r.o)[0] == len(root.encode("cp1251")):
        raise SpjError("%s: older .stt revision (variables stored by index) "
                       "is not decoded yet" % path)
    out["id"] = r.u16()
    while r.o + 6 <= len(r.b):   # the list ends with a 2-byte terminator
        try:
            ref = r.u32()          # handle of the image this record belongs to
            flags = r.u16()
            name = r.string()
            stamp = r.u32()
            count = r.u16()
            vars_ = [OrderedDict(name=r.string(), value=r.string()) for _ in range(count)]
        except (IndexError, struct.error) as exc:
            raise SpjError("%s: truncated state record at 0x%x (%s)" % (path, r.o, exc))
        out["images"].append(OrderedDict(klass=name, ref=ref, flags=flags,
                                         stamp=stamp, vars=vars_))
    return out


def parse(path):
    data = open(path, "rb").read()
    if path.lower().endswith(".stt"):
        return parse_stt(data, path)
    return parse_spj(data, path)


def scan(dirs):
    files = []
    for a in dirs:
        for root, _, names in os.walk(a):
            files += [os.path.join(root, n) for n in names
                      if n.lower().endswith((".spj", ".stt"))]
    ok, bad, old, skipped = 0, [], [], []
    for p in sorted(files):
        head = open(p, "rb").read(4)
        if p.lower().endswith(".spj") and head[:2] != b"Ih":
            skipped.append(p)   # some samples ship a RAR archive under a .spj name
            continue
        try:
            parse(p)
            ok += 1
        except SpjError as exc:
            (old if "older .stt revision" in str(exc) else bad).append((p, str(exc)))
        except Exception as exc:
            bad.append((p, str(exc)))
    print("parsed %d/%d project files (%d in the older .stt revision, "
          "%d not project files)" % (ok, len(files), len(old), len(skipped)))
    for p, e in bad:
        print("   ", p, "--", e)
    return 1 if bad else 0


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 2
    if args[0] == "--scan":
        return scan(args[1:])
    for p in args:
        print(json.dumps(parse(p), ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
