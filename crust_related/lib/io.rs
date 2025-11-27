// crust_io/lib.rs
#![no_std]

pub mod fs {
    use crate::result::Result;
    use crate::string::String;
    use crate::vec::Vec;

    pub struct File {
        fd: i32,
    }

    impl File {
        pub unsafe fn open(path: &str, flags: i32) -> Result<Self, i32> {
            extern "C" {
                fn open(path: *const u8, flags: i32, ...) -> i32;
            }
            
            let fd = unsafe { open(path.as_ptr(), flags) };
            if fd < 0 {
                Result::Err(-fd)
            } else {
                Result::Ok(File { fd })
            }
        }

        pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
            extern "C" {
                fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
            }
            
            let result = unsafe { read(self.fd, buf.as_mut_ptr(), buf.len()) };
            if result < 0 {
                Result::Err(-(result as i32))
            } else {
                Result::Ok(result as usize)
            }
        }

        pub fn write(&self, buf: &[u8]) -> Result<usize, i32> {
            extern "C" {
                fn write(fd: i32, buf: *const u8, count: usize) -> isize;
            }
            
            let result = unsafe { write(self.fd, buf.as_ptr(), buf.len()) };
            if result < 0 {
                Result::Err(-(result as i32))
            } else {
                Result::Ok(result as usize)
            }
        }

        pub fn close(self) -> Result<(), i32> {
            extern "C" {
                fn close(fd: i32) -> i32;
            }
            
            let result = unsafe { close(self.fd) };
            if result < 0 {
                Result::Err(-result)
            } else {
                Result::Ok(())
            }
        }
    }
}

pub mod net {
    use crate::result::Result;

    pub struct TcpStream {
        fd: i32,
    }

    impl TcpStream {
        pub fn connect(addr: &str, port: u16) -> Result<Self, i32> {
            extern "C" {
                fn socket(domain: i32, type_: i32, protocol: i32) -> i32;
                fn connect(fd: i32, addr: *const u8, addrlen: u32) -> i32;
            }
            
            // Simplified - in real implementation you'd need proper socket address setup
            let fd = unsafe { socket(2, 1, 0) }; // AF_INET, SOCK_STREAM
            if fd < 0 {
                return Result::Err(-fd);
            }

            // Connect logic would go here
            Result::Ok(TcpStream { fd })
        }

        pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
            fs::File { fd: self.fd }.read(buf)
        }

        pub fn write(&self, buf: &[u8]) -> Result<usize, i32> {
            fs::File { fd: self.fd }.write(buf)
        }
    }
}

pub mod time {
    pub fn sleep_ms(ms: u32) {
        extern "C" {
            fn usleep(usec: u32) -> i32;
        }
        unsafe { usleep(ms * 1000) };
    }

    pub fn current_time() -> u64 {
        extern "C" {
            fn time(tloc: *mut u64) -> u64;
        }
        unsafe { time(core::ptr::null_mut()) }
    }
}
