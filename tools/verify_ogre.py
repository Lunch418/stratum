#!/usr/bin/env python3
"""verify_original.py для проб с Ogre: окно "OGRE Engine Rendering Setup" и
окна ошибок закрываются нажатием OK (кнопка в правом нижнем углу).

    python3 tools/verify_ogre.py tools/verify/ogre_euler.txt
"""
import runpy, sys, pathlib, threading, time
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from wine_dialogs import Answerer
from Xlib import display, X
from Xlib.ext import xtest

stop = False


def click_ok():
    d = display.Display()
    while not stop:
        time.sleep(1)
        try:
            for w in d.screen().root.query_tree().children:
                for c in w.query_tree().children:
                    n = c.get_wm_name()
                    if n and n.startswith('OGRE Engine Rendering Setup'):
                        t = c.translate_coords(d.screen().root, 0, 0)
                        xtest.fake_input(d, X.MotionNotify, x=-t.x + 352, y=-t.y + 423)
                        d.sync(); time.sleep(0.2)
                        xtest.fake_input(d, X.ButtonPress, 1); xtest.fake_input(d, X.ButtonRelease, 1); d.sync()
                        time.sleep(3)
        except Exception:
            pass


threading.Thread(target=click_ok, daemon=True).start()
with Answerer():
    sys.argv[0] = str(pathlib.Path(__file__).with_name('verify_original.py'))
    runpy.run_path(sys.argv[0], run_name='__main__')
