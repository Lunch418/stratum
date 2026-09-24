#!/usr/bin/env bash
# Дымовая проверка собранной десктопной программы: запускает её, берёт из
# вывода ссылку с токеном и проверяет, что по ней отдаётся IDE (вшитые файлы
# фронтенда), а API без токена закрыт.
#
#   tools/smoke_desktop.sh путь/к/stratum-modern
#
# Без экрана запускать через xvfb-run. Пакет .deb для проверки не нужно
# ставить: `dpkg-deb -x пакет.deb папка` и путь к папка/usr/bin/stratum-modern.
set -u

bin=${1:?укажите путь к программе}
out=$(mktemp)
# своя группа процессов: AppImage запускает программу дочерним процессом
setsid "$bin" >"$out" 2>&1 &
pid=$!
trap 'kill -- -"$pid" 2>/dev/null; wait "$pid" 2>/dev/null; rm -f "$out"' EXIT

url=""
for _ in $(seq 1 120); do
    url=$(grep -o 'http://127\.0\.0\.1:[0-9]*/?token=[0-9a-f]*' "$out" | head -n 1)
    [ -n "$url" ] && break
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.25
done
fail() {
    echo "ОШИБКА: $*" >&2
    echo "--- вывод программы" >&2
    cat "$out" >&2
    exit 1
}
[ -n "$url" ] || fail "программа не напечатала ссылку на IDE"
base=${url%%/\?token=*}
token=${url##*token=}
echo "IDE: $base"

page=$(curl -fsS "$url") || fail "страница IDE не отдаётся"
echo "$page" | grep -q '<title>Stratum Modern</title>' || fail "по ссылке не страница IDE"
script=$(echo "$page" | grep -o '/assets/[^"]*\.js' | head -n 1)
[ -n "$script" ] || fail "в странице нет скрипта сборки"
code=$(curl -s -o /dev/null -w '%{http_code}' "$base$script")
[ "$code" = 200 ] || fail "скрипт IDE $script: $code"

code=$(curl -s -o /dev/null -w '%{http_code}' "$base/frame")
[ "$code" = 401 ] || fail "API без токена ответил $code, ждали 401"
code=$(curl -s -o /dev/null -w '%{http_code}' -H "X-Stratum-Token: $token" "$base/frame")
[ "$code" = 200 ] || fail "API с токеном ответил $code, ждали 200"

sleep 2
kill -0 "$pid" 2>/dev/null || fail "программа завершилась сама"
echo "ok: IDE отдаётся из программы, API закрыт токеном"
