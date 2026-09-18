//! Запись двоичных структур Stratum 2000: зеркало `Reader`.

use super::cp1251;

#[derive(Default)]
pub struct Writer {
    pub data: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u8(&mut self, v: u8) {
        self.data.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    /// Строка: u16 длина + байты CP1251; длиннее 65535 байт обрезается.
    pub fn string(&mut self, s: &str) {
        let mut bytes = cp1251::encode(s);
        bytes.truncate(u16::MAX as usize);
        self.u16(bytes.len() as u16);
        self.data.extend_from_slice(&bytes);
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.data.extend_from_slice(b);
    }

    pub fn pos(&self) -> usize {
        self.data.len()
    }

    pub fn patch_u32(&mut self, at: usize, v: u32) {
        self.data[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
}
