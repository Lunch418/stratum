//! Ядро Stratum Modern.
//!
//! - [`formats`] — чтение файлов Stratum 2000 (`.cls`, `.spj`, `.stt`);
//! - [`lang`] — язык моделирования: лексер, парсер, дерево разбора;
//! - [`runtime`] — значения и встроенные функции;
//! - [`sim`] — планировщик тактов, состояние и связи.

pub mod formats;
pub mod lang;
pub mod runtime;
pub mod sim;
