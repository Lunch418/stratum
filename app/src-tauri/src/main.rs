//! Десктопная оболочка: поднимает ядро с IDE на локальном порту и
//! открывает его в окне Tauri. Проект — первым аргументом командной строки
//! или из диалога выбора папки.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::mpsc;
use tauri::{WebviewUrl, WebviewWindowBuilder};

fn static_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut candidates = vec![PathBuf::from("dist"), PathBuf::from("app/dist"), PathBuf::from("../dist")];
    if let Some(dir) = exe.parent() {
        for rel in ["dist", "../dist", "../../dist", "../../../app/dist", "../../../../app/dist"] {
            candidates.push(dir.join(rel));
        }
    }
    candidates.into_iter().find(|p| p.join("index.html").exists())
}

fn main() {
    // без аргумента открывается пустая IDE с диалогом «Открыть проект»
    let project = std::env::args().nth(1).map(PathBuf::from).unwrap_or_default();
    let (tx, rx) = mpsc::channel::<Result<u16, String>>();
    let libraries = stratum_core::formats::default_library_dirs();
    let dist = static_dir();
    std::thread::spawn(move || {
        let result = stratum_core::player::serve_with(
            stratum_core::player::Options { project, libraries, port: 0, fps: 30, static_dir: dist },
            |port| {
                let _ = tx.send(Ok(port));
            },
        );
        if let Err(e) = result {
            let _ = tx.send(Err(e));
        }
    });
    let port = match rx.recv() {
        Ok(Ok(port)) => port,
        Ok(Err(e)) => {
            eprintln!("ядро не запустилось: {e}");
            std::process::exit(1)
        }
        Err(_) => {
            eprintln!("ядро не запустилось");
            std::process::exit(1)
        }
    };

    tauri::Builder::default()
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}/").parse().unwrap();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Stratum Modern")
                .inner_size(1440.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("окно Tauri не запустилось");
}
