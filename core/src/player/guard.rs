//! Защита локального сервера IDE от чужих страниц в браузере.
//!
//! Ядро слушает 127.0.0.1, но любая открытая веб-страница может слать туда
//! запросы (`<img src>`, `fetch` без CORS, DNS rebinding). Поэтому:
//!
//! - у сессии есть случайный токен; IDE получает его один раз в ссылке
//!   `/?token=…`, сервер кладёт его в cookie `SameSite=Strict`, и дальше
//!   страница ходит с ним автоматически. Запросы с чужих сайтов cookie не
//!   несут;
//! - заголовок `Host` должен быть нашим адресом (защита от DNS rebinding);
//! - `Origin`, если есть, — только наш;
//! - всё, что меняет состояние, принимается только методом POST.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

pub const COOKIE: &str = "stratum_token";

/// 128 бит случайности без внешних зависимостей: `RandomState` берёт зерно
/// у ОС, к нему примешиваются время и номер процесса.
pub fn new_token() -> String {
    let mut out = String::new();
    for i in 0..2u64 {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(i);
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
        h.write_u32(std::process::id());
        out.push_str(&format!("{:016x}", h.finish()));
    }
    out
}

/// Заголовки запроса, нужные для проверки.
#[derive(Default, Debug)]
pub struct Headers {
    pub host: Option<String>,
    pub origin: Option<String>,
    pub cookie: Option<String>,
    pub token: Option<String>,
}

impl Headers {
    pub fn read(line: &str, into: &mut Headers) {
        let Some((name, value)) = line.split_once(':') else { return };
        let value = value.trim().to_string();
        match name.trim().to_ascii_lowercase().as_str() {
            "host" => into.host = Some(value),
            "origin" => into.origin = Some(value),
            "cookie" => into.cookie = Some(value),
            "x-stratum-token" => into.token = Some(value),
            _ => {}
        }
    }

    fn cookie_token(&self) -> Option<&str> {
        self.cookie.as_deref()?.split(';').find_map(|kv| kv.trim().strip_prefix(COOKIE).and_then(|v| v.strip_prefix('=')))
    }
}

/// Сравнение без раннего выхода — чтобы по времени ответа нельзя было
/// подбирать токен по символу.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn host_ok(h: &Headers, port: u16) -> bool {
    match h.host.as_deref() {
        Some(host) => host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}"),
        // HTTP/1.0 без Host: так ходят только локальные утилиты
        None => true,
    }
}

pub fn origin_ok(h: &Headers, port: u16) -> bool {
    match h.origin.as_deref() {
        None | Some("null") => h.origin.is_none(),
        Some(o) => o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}") || o == "tauri://localhost",
    }
}

/// Токен из cookie, заголовка `X-Stratum-Token` или параметра `token`.
pub fn token_ok(h: &Headers, query_token: Option<&str>, token: &str) -> bool {
    [h.cookie_token(), h.token.as_deref(), query_token].into_iter().flatten().any(|t| same(t, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_and_different() {
        let (a, b) = (new_token(), new_token());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }

    #[test]
    fn foreign_origin_and_host_are_rejected() {
        let mut h = Headers::default();
        Headers::read("Host: 127.0.0.1:8765", &mut h);
        assert!(host_ok(&h, 8765));
        Headers::read("Origin: http://evil.example", &mut h);
        assert!(!origin_ok(&h, 8765));
        let mut r = Headers::default();
        Headers::read("Host: evil.example:8765", &mut r);
        assert!(!host_ok(&r, 8765));
    }

    #[test]
    fn token_from_cookie() {
        let mut h = Headers::default();
        Headers::read("Cookie: a=1; stratum_token=abc", &mut h);
        assert!(token_ok(&h, None, "abc"));
        assert!(!token_ok(&h, None, "abd"));
        assert!(!token_ok(&Headers::default(), None, "abc"));
    }
}
