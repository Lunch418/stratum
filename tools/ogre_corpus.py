#!/usr/bin/env python3
"""Прогон образцов Ogre_3D через ядро по HTTP: кадры в PNG, сообщения, ошибки.

Использование: tools/ogre_corpus.py ВЫХОДНАЯ_ПАПКА [ТАКТОВ] [ИМЯ_ОБРАЗЦА...]
Нужен собранный core/target/debug/stratum (cargo build). Без Wine и экрана.
"""
import base64, json, os, re, subprocess, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "core", "target", "debug", "stratum")
SAMPLES = os.path.join(ROOT, "fixtures", "PROJECTS", "samples", "Ogre_3D")


def find_spj(d):
    for f in sorted(os.listdir(d)):
        if f.lower().endswith(".spj"):
            return os.path.join(d, f)


def get(port, token, path):
    with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}{'&' if '?' in path else '?'}token={token}", timeout=60) as r:
        return r.read()


def post(port, token, query):
    req = urllib.request.Request(f"http://127.0.0.1:{port}/event?{query}&token={token}", method="POST")
    urllib.request.urlopen(req, timeout=60).read()


def run(name, out, ticks, port):
    spj = find_spj(os.path.join(SAMPLES, name))
    p = subprocess.Popen([BIN, "play", spj, "--port", str(port)], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        token = None
        for _ in range(100):
            line = p.stdout.readline()
            m = re.search(r"token=([0-9a-f]+)", line)
            if m:
                token = m.group(1)
                break
        if not token:
            return {"name": name, "error": "сервер не запустился"}
        time.sleep(0.3)
        for _ in range(ticks):
            post(port, token, "type=step")
        fr = json.loads(get(port, token, "/frame"))
        deadline = time.time() + 90
        while fr["tick"] < ticks and time.time() < deadline and not fr.get("halt"):
            time.sleep(1)
            fr = json.loads(get(port, token, "/frame"))
        res = {"name": name, "tick": fr["tick"], "windows": [w["name"] for w in fr["windows"]], "unsupported": fr["unsupported"], "log": fr["log"][:6], "halt": fr.get("halt")}
        for w in fr["windows"]:
            m = re.search(r'base64,([^"]+)"', w["svg"])
            if w["name"] == "Ogre3D" and m:
                bmp = base64.b64decode(m.group(1))
                path = os.path.join(out, name.replace(" ", "_") + ".bmp")
                open(path, "wb").write(bmp)
                try:
                    from PIL import Image
                    im = Image.open(path)
                    im.save(path[:-4] + ".png")
                    os.remove(path)
                    px = im.convert("RGB").resize((64, 48))
                    res["colors"] = len(set(px.getdata()))
                except ImportError:
                    pass
        return res
    finally:
        p.terminate()


if __name__ == "__main__":
    out = sys.argv[1]
    ticks = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    names = sys.argv[3:] or sorted(os.listdir(SAMPLES))
    os.makedirs(out, exist_ok=True)
    for i, n in enumerate(names):
        print(json.dumps(run(n, out, ticks, 8800 + i), ensure_ascii=False))
