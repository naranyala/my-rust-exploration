// io_utils.rs
#![no_std]

pub mod io {
    use core::fmt;
    
    pub struct Writer;
    
    impl Writer {
        pub const fn new() -> Self {
            Self
        }
        
        pub unsafe fn write_bytes(&self, bytes: &[u8]) -> isize {
            #[link(name = "c")]
            extern "C" {
                fn write(fd: i32, buf: *const u8, count: usize) -> isize;
            }
            
            unsafe { write(1, bytes.as_ptr(), bytes.len()) }
        }
        
        pub fn write_str(&self, s: &str) -> isize {
            unsafe { self.write_bytes(s.as_bytes()) }
        }
    }
    
    /// Basic formatter for no_std environment
    pub struct Formatter<'a> {
        buffer: &'a mut [u8],
        position: usize,
    }
    
    impl<'a> Formatter<'a> {
        pub fn new(buffer: &'a mut [u8]) -> Self {
            Self { buffer, position: 0 }
        }
        
        pub fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
            let bytes = s.as_bytes();
            if self.position + bytes.len() > self.buffer.len() {
                return Err(fmt::Error);
            }
            
            for (i, &byte) in bytes.iter().enumerate() {
                self.buffer[self.position + i] = byte;
            }
            self.position += bytes.len();
            Ok(())
        }
        
        pub fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
            let mut buf = [0u8; 4];
            let bytes = c.encode_utf8(&mut buf).as_bytes();
            self.write_str(core::str::from_utf8(bytes).unwrap())
        }
        
        pub fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buffer[..self.position]).unwrap_or("")
        }
        
        pub fn clear(&mut self) {
            self.position = 0;
        }
    }
}
