//! Среда исполнения: значения, константы и встроенные функции.

pub mod builtins;
pub mod clock;
pub mod constants;
pub mod data;
pub mod extra;
pub mod value;

pub use builtins::Effects;
pub use value::{Value, ValueType};
