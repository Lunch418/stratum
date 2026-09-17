//! Чтение little-endian полей и строк с префиксом длины.

use super::cp1251;
use std::fmt;

#[derive(Debug)]
pub struct FormatError {
    pub path: String,
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: 0x{:x}: {}", self.path, self.offset, self.message)
    }
}

impl std::error::Error for FormatError {}

pub type Result<T> = std::result::Result<T, FormatError>;

pub struct Reader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
    path: String,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8], path: impl Into<String>) -> Self {
        Reader { data, pos: 0, path: path.into() }
    }

    pub fn err<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(FormatError { path: self.path.clone(), offset: self.pos, message: message.into() })
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            return self.err(format!(
                "нужно {} байт, а до конца файла осталось {}",
                n,
                self.data.len().saturating_sub(self.pos)
            ));
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn i16(&mut self) -> Result<i16> {
        Ok(self.u16()? as i16)
    }

    pub fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn f64(&mut self) -> Result<f64> {
        let b = self.take(8)?;
        Ok(f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }

    /// Строка: u16 длина + байты CP1251.
    pub fn string(&mut self) -> Result<String> {
        let n = self.u16()? as usize;
        Ok(cp1251::decode(self.take(n)?))
    }

    pub fn bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        Ok(self.take(n)?.to_vec())
    }

    pub fn peek_u16(&self, at: usize) -> Option<u16> {
        if at + 2 <= self.data.len() {
            Some(u16::from_le_bytes([self.data[at], self.data[at + 1]]))
        } else {
            None
        }
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
