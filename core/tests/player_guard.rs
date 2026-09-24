//! Защита локального сервера IDE на живом сервере: запросы идут по
//! настоящему TCP-соединению, как их шлёт браузер или чужая страница.
//! Проверки заголовков по отдельности — в `player/guard.rs`; здесь —
//! что сервер действительно применяет их к каждому запросу.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use stratum_core::player::{serve_with, Options};

/// Сервер без проекта на свободном порту; возвращает порт и токен сессии.
fn start() -> (u16, String) {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let opts = Options { project: PathBuf::new(), libraries: Vec::new(), port: 0, fps: 30, static_dir: None, assets: None, token: None };
        let result = serve_with(opts, move |port, token| {
            let _ = tx.send((port, token.to_string()));
        });
        panic!("сервер остановился: {result:?}");
    });
    rx.recv_timeout(Duration::from_secs(20)).expect("сервер не запустился")
}

/// Ответ сервера: код статуса и заголовки с телом одной строкой.
struct Reply {
    status: u16,
    text: String,
}

fn request(port: u16, method: &str, target: &str, headers: &[(&str, String)]) -> Reply {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("нет соединения");
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut req = format!("{method} {target} HTTP/1.1\r\n");
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host")) {
        req.push_str(&format!("Host: 127.0.0.1:{port}\r\n"));
    }
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("Content-Length: 0\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Reply { status, text }
}

fn token_header(token: &str) -> (&'static str, String) {
    ("X-Stratum-Token", token.to_string())
}

#[test]
fn api_without_token_is_401() {
    let (port, _) = start();
    assert_eq!(request(port, "GET", "/frame", &[]).status, 401);
    assert_eq!(request(port, "GET", "/api/project", &[]).status, 401);
    assert_eq!(request(port, "POST", "/event?type=key", &[]).status, 401);
}

#[test]
fn wrong_token_is_401() {
    let (port, token) = start();
    // тот же токен с другим первым символом
    let wrong = format!("{}{}", if token.starts_with('0') { '1' } else { '0' }, &token[1..]);
    assert_eq!(request(port, "GET", "/frame", &[token_header(&wrong)]).status, 401);
    assert_eq!(request(port, "GET", &format!("/frame?token={wrong}"), &[]).status, 401);
    assert_eq!(request(port, "GET", "/frame", &[("Cookie", format!("stratum_token={wrong}"))]).status, 401);
}

#[test]
fn valid_token_is_200() {
    let (port, token) = start();
    let r = request(port, "GET", "/frame", &[token_header(&token)]);
    assert_eq!(r.status, 200, "{}", r.text);
    assert_eq!(request(port, "GET", &format!("/frame?token={token}"), &[]).status, 200);
    assert_eq!(request(port, "GET", "/frame", &[("Cookie", format!("a=1; stratum_token={token}"))]).status, 200);
}

#[test]
fn page_link_sets_strict_http_only_cookie() {
    let (port, token) = start();
    let r = request(port, "GET", &format!("/?token={token}"), &[]);
    assert_eq!(r.status, 200);
    let cookie = r.text.lines().find(|l| l.to_ascii_lowercase().starts_with("set-cookie:")).expect("нет Set-Cookie");
    assert!(cookie.contains(&format!("stratum_token={token}")), "{cookie}");
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"), "{cookie}");
    // без токена в ссылке cookie не выдаётся
    assert!(!request(port, "GET", "/", &[]).text.to_ascii_lowercase().contains("set-cookie"));
}

#[test]
fn foreign_host_is_rejected_even_with_token() {
    let (port, token) = start();
    for host in [format!("evil.example:{port}"), "127.0.0.1:1".to_string(), format!("127.0.0.1.evil.example:{port}")] {
        let r = request(port, "GET", "/frame", &[("Host", host.clone()), token_header(&token)]);
        assert_eq!(r.status, 403, "Host: {host}");
    }
    // страница тоже не отдаётся чужому имени (DNS rebinding)
    assert_eq!(request(port, "GET", "/", &[("Host", format!("evil.example:{port}"))]).status, 403);
    assert_eq!(request(port, "GET", "/frame", &[("Host", format!("localhost:{port}")), token_header(&token)]).status, 200);
}

#[test]
fn cross_origin_post_is_rejected_even_with_token() {
    let (port, token) = start();
    let cookie = ("Cookie", format!("stratum_token={token}"));
    for origin in ["http://evil.example", "null", "http://127.0.0.1:1"] {
        let r = request(port, "POST", "/event?type=key", &[("Origin", origin.to_string()), cookie.clone(), token_header(&token)]);
        assert_eq!(r.status, 403, "Origin: {origin}");
        let r = request(port, "POST", "/api/run", &[("Origin", origin.to_string()), cookie.clone()]);
        assert_eq!(r.status, 403, "Origin: {origin}");
    }
    let r = request(port, "POST", "/event?type=key", &[("Origin", format!("http://127.0.0.1:{port}")), cookie]);
    assert_eq!(r.status, 200, "{}", r.text);
}
