# Third-party components

Stratum Modern itself is MIT-licensed (see `LICENSE`). The distributed
programs include the following third-party components under their own licenses.

## Frontend (bundled into the application)

| Component | Version | License |
|---|---|---|
| React, React DOM | 19.x | MIT |
| Zustand | 5.x | MIT |
| Monaco Editor, @monaco-editor/react | 0.57 / 4.7 | MIT (Monaco ships its own third-party notices inside the package) |
| Golos Text (variable font, via Fontsource) | 5.x | SIL Open Font License 1.1 |
| JetBrains Mono (variable font, via Fontsource) | 5.x | SIL Open Font License 1.1 |

The fonts are redistributed unmodified under the SIL OFL 1.1; the license texts
are in `app/node_modules/@fontsource-variable/*/LICENSE` of a build checkout.

## Desktop shell

| Component | License |
|---|---|
| Tauri 2 and its Rust dependencies | MIT or Apache-2.0 (each crate states its own terms; `cargo tree` in `app/src-tauri` lists them) |
| WebKitGTK (Linux), WebView2 (Windows), WKWebView (macOS) | provided by the operating system, not redistributed |

## Core

The `core` crate (`stratum-core`) has no third-party Rust dependencies: the
parsers, inflate, image and audio codecs are implemented in the repository.

## Not included

Files of the original Stratum 2000 (program, libraries, help, samples, fonts,
images) are not part of this repository or of the installers.
