#!/usr/bin/env python3
"""Generate core/src/runtime/constants.rs from docs/lang/constants.json.

Usage:
    gen_constants.py docs/lang/constants.json core/src/runtime/constants.rs

The core has no dependencies, so the constant table is compiled in as a sorted
array looked up with a binary search on the lower-cased name.
"""
import json
import sys


def main():
    src, dest = sys.argv[1], sys.argv[2]
    consts = json.load(open(src, encoding="utf-8"))
    seen = {}
    for c in consts:
        seen.setdefault(c["name"].lower(), (c["name"], float(c["value"]), c["group"]))
    rows = sorted(seen.items())
    with open(dest, "w", encoding="utf-8") as f:
        f.write("//! Константы языка: PI, WM_XXX, стили окон, флаги графики.\n")
        f.write("//!\n//! Файл сгенерирован `tools/gen_constants.py` из `docs/lang/constants.json`\n")
        f.write("//! (исходник — `template/CONSTANT.TPL` установленного Stratum). Не править вручную.\n\n")
        f.write("/// Имя в нижнем регистре и значение; отсортировано для двоичного поиска.\n")
        f.write("pub const CONSTANTS: [(&str, f64); %d] = [\n" % len(rows))
        for key, (name, value, group) in rows:
            comment = (" // %s" % group) if group else ""
            f.write('    ("%s", %r),%s\n' % (key, value, comment))
        f.write("];\n\n")
        f.write("""/// Значение константы по имени без учёта регистра.
pub fn lookup(name: &str) -> Option<f64> {
    let key = name.to_ascii_lowercase();
    CONSTANTS
        .binary_search_by(|(n, _)| (*n).cmp(key.as_str()))
        .ok()
        .map(|i| CONSTANTS[i].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_constants() {
        assert_eq!(lookup("WM_LBUTTONDOWN"), Some(513.0));
        assert_eq!(lookup("wm_allmousemessage"), Some(1536.0));
        assert_eq!(lookup("PFC_MOVEOBJECT"), Some(32768.0));
        assert!((lookup("pi").unwrap() - std::f64::consts::PI).abs() < 1e-9);
        assert_eq!(lookup("нет такой"), None);
    }
}
""")
    print("constants written:", len(rows))


if __name__ == "__main__":
    main()
