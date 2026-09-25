#!/usr/bin/env python3
"""Сверка траектории примера с Stratum 2000 через Wine.

    python3 tools/verify_trajectory.py fixtures/user/solar_system --ticks 100

Шаги: stratum instrument (счётчик тактов, SaveObjectState и Quit в корневом
имидже), запуск оригинала с /run, запуск ядра на том же проекте,
stratum sttdiff снимков. Нужны Wine и Stratum 2000 (~/.wine32).
"""
import os, pathlib, shutil, subprocess, sys, time

from wine_dialogs import Answerer

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / 'core' / 'target' / 'debug' / 'stratum'
PREFIX = pathlib.Path(os.environ.get('WINEPREFIX', pathlib.Path.home() / '.wine32'))
EXE = os.environ.get('STRATUM_EXE', r'C:\Program Files\Stratum\SC200032.EXE')
WORK = PREFIX / 'drive_c' / 'verify'


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    ticks = int(sys.argv[sys.argv.index('--ticks') + 1]) if '--ticks' in sys.argv else 100
    if '--ticks' in sys.argv:
        args = [a for a in args if a != str(ticks)]
    if not args:
        sys.exit(__doc__)
    project = pathlib.Path(args[0])
    env = dict(os.environ, WINEPREFIX=str(PREFIX), WINEDEBUG='-all')
    subprocess.run(['wineserver', '-k'], env=env, capture_output=True)
    if WORK.exists():
        shutil.rmtree(WORK)
    r = subprocess.run([str(CORE), 'instrument', str(project), str(WORK), '--ticks', str(ticks)], capture_output=True, text=True)
    if r.returncode:
        sys.exit(f'{project.name}: подготовка не удалась: {r.stderr.strip() or r.stdout.strip()}')
    proc = subprocess.Popen(['wine', EXE, r'C:\verify\project.spj', '/run'], env=env, cwd='/tmp', stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    deadline = time.time() + max(60, ticks)
    # модальные окна (ввод значения, сообщение) получают Enter: ядро без
    # пользователя берёт значение по умолчанию
    with Answerer() as answerer:
        while time.time() < deadline and proc.poll() is None:
            time.sleep(1)
    if answerer.answered:
        print('окна оригинала, закрытые Enter:', ', '.join(dict.fromkeys(answerer.answered)))
    if proc.poll() is None:
        proc.kill()
        subprocess.run(['wineserver', '-k'], env=env, capture_output=True)
    if not (WORK / 'state.stt').exists():
        sys.exit(f'{project.name}: оригинал не сохранил состояние (окно ошибки или модель не дошла до такта {ticks})')
    (WORK / 'verify').mkdir(exist_ok=True)
    subprocess.run([str(CORE), 'run', str(WORK), '--ticks', str(ticks + 5)], capture_output=True)
    if not (WORK / 'verify' / 'state.stt').exists():
        sys.exit(f'{project.name}: ядро не сохранило состояние')
    d = subprocess.run([str(CORE), 'sttdiff', str(WORK / 'state.stt'), str(WORK / 'verify' / 'state.stt')], capture_output=True, text=True)
    print(f'== {project.name}, {ticks} тактов')
    print(d.stdout.rstrip())
    sys.exit(d.returncode)


if __name__ == '__main__':
    main()
