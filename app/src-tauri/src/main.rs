//! Десктопная оболочка: поднимает ядро с IDE на локальном порту и
//! открывает его в окне Tauri. Проект — первым аргументом командной строки
//! или из диалога выбора папки.
//!
//! Файлы IDE вшиты в бинарник (`frontendDist`) и отдаются ядром из памяти,
//! поэтому установленной программе (`/usr/bin/stratum-modern`) папка `dist`
//! рядом не нужна. Доступ к API — только с токеном сессии из ссылки окна.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    // без аргумента открывается пустая IDE с диалогом «Открыть проект»
    let project = std::env::args().nth(1).map(PathBuf::from).unwrap_or_default();
    let libraries = stratum_core::formats::default_library_dirs();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let assets: stratum_core::player::Assets = Arc::new(move |path: &str| {
                let path = if path == "/" { "/index.html" } else { path };
                handle.asset_resolver().get(path.to_string()).map(|a| (a.mime_type, a.bytes))
            });
            let (tx, rx) = mpsc::channel::<Result<(u16, String), String>>();
            let (project, libraries) = (project.clone(), libraries.clone());
            std::thread::spawn(move || {
                let ready = tx.clone();
                let result = stratum_core::player::serve_with(
                    stratum_core::player::Options { project, libraries, port: 0, fps: 30, static_dir: None, assets: Some(assets), token: None },
                    move |port, token| {
                        let _ = ready.send(Ok((port, token.to_string())));
                    },
                );
                if let Err(e) = result {
                    let _ = tx.send(Err(e));
                }
            });
            let (port, token) = match rx.recv() {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("ядро не запустилось: {e}").into()),
                Err(_) => return Err("ядро не запустилось".into()),
            };
            let url = format!("http://127.0.0.1:{port}/?token={token}").parse()?;
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
