#!/usr/bin/env python3
"""Ответ на модальные окна оригинала: пока идёт сверка, окна диалогов
Stratum 2000 (ввод значения, сообщение) получают Enter — «OK» с значением по
умолчанию, как их обрабатывает ядро без пользователя.

    python3 tools/wine_dialogs.py СЕКУНД      # отвечать заданное время
    from wine_dialogs import Answerer          # в других скриптах

Нужен X-сервер (DISPLAY) и python-xlib; без них ничего не делает.
"""
import sys, threading, time

MAIN_MARKS = ('Stratum (Professional)', 'Default IME')


class Answerer:
    """Фоновый поток: раз в полсекунды ищет окна диалогов оригинала и жмёт Enter."""

    def __init__(self):
        self.stop = threading.Event()
        self.answered = []
        self.thread = threading.Thread(target=self.run, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *exc):
        self.stop.set()
        self.thread.join(timeout=2)

    def run(self):
        try:
            from Xlib import X, XK, display
            from Xlib.ext import xtest
            d = display.Display()
        except Exception:
            return
        enter = d.keysym_to_keycode(XK.string_to_keysym('Return'))
        escape = d.keysym_to_keycode(XK.string_to_keysym('Escape'))
        tries = {}
        while not self.stop.is_set():
            try:
                for w in self.dialogs(d):
                    name = w.get_wm_name() or ''
                    # окно, которое Enter не закрыл (выбор файла с пустым
                    # именем), закрывается Escape — «Отмена»
                    n = tries[w.id] = tries.get(w.id, 0) + 1
                    key = enter if n <= 2 else escape
                    w.set_input_focus(X.RevertToParent, X.CurrentTime)
                    d.sync()
                    time.sleep(0.2)
                    xtest.fake_input(d, X.KeyPress, key)
                    xtest.fake_input(d, X.KeyRelease, key)
                    d.sync()
                    self.answered.append(f'{name or hex(w.id)} ({"Enter" if key == enter else "Escape"})')
            except Exception:
                pass
            self.stop.wait(0.5)

    @staticmethod
    def dialogs(d):
        out = []

        def walk(w):
            try:
                children = w.query_tree().children
            except Exception:
                return
            for c in children:
                try:
                    cls = c.get_wm_class() or ()
                    name = c.get_wm_name()
                    attrs = c.get_attributes()
                except Exception:
                    continue
                # заголовок с кириллицей приходит пустыми байтами (он в
                # _NET_WM_NAME), поэтому годится любое видимое окно, кроме главного
                if 'sc200032.exe' in cls and attrs.map_state == 2:
                    text = name if isinstance(name, str) else (name or b'').decode('latin-1')
                    if not any(m in text for m in MAIN_MARKS):
                        out.append(c)
                walk(c)

        walk(d.screen().root)
        return out


if __name__ == '__main__':
    with Answerer() as a:
        time.sleep(float(sys.argv[1]) if len(sys.argv) > 1 else 30)
    print('\n'.join(a.answered))
