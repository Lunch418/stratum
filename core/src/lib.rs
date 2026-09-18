//! Ядро Stratum Modern.
//!
//! - [`formats`] — чтение файлов Stratum 2000 (`.cls`, `.spj`, `.stt`);
//! - [`lang`] — язык моделирования: лексер, парсер, дерево разбора;
//! - [`gfx`] — графическое пространство и 2D-функции;
//! - [`player`] — локальный веб-плеер с транспортом;
//! - [`runtime`] — значения и встроенные функции;
//! - [`sim`] — планировщик тактов, состояние и связи.

pub mod formats;
pub mod gfx;
pub mod lang;
pub mod player;
pub mod runtime;
pub mod sim;
