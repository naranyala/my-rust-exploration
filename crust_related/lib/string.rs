// crust_string/lib.rs
#![no_std]

use crate::vec::Vec;

pub struct String<const CAP: usize> {
    data: Vec<u8, CAP>,
}

impl<const CAP: usize> String<CAP> {
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let mut string = Self::new();
        string.push_str(s)?;
        Ok(string)
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), &'static str> {
        for byte in s.bytes() {
            self.data.push(byte)?;
        }
        Ok(())
    }

    pub fn push(&mut self, c: char) -> Result<(), &'static str> {
        let mut buf = [0; 4];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        for &byte in bytes {
            self.data.push(byte)?;
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.data.as_slice()).unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

impl<const CAP: usize> core::fmt::Display for String<CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Advanced formatting system
pub struct Formatter<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> Formatter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    pub fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.pos + bytes.len() > self.buffer.len() {
            return Err(core::fmt::Error);
        }
        self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }

    pub fn write_char(&mut self, c: char) -> core::fmt::Result {
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        self.write_str(s)
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.pos]).unwrap_or("")
    }
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        use $crate::string::Formatter;
        let mut buf = [0u8; 256];
        let mut formatter = Formatter::new(&mut buf);
        core::write!(formatter, $($arg)*).unwrap();
        formatter.as_str()
    }};
}
