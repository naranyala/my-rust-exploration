// build: rustc --edition 2021 -C panic=abort ls.rs -o ls
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn open(path: *const u8, flags: i32, mode: u32) -> i32;
    fn getdents64(fd: i32, dirp: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

const O_RDONLY: i32 = 0;
const BUF_SIZE: usize = 4096;

#[repr(C)]
struct LinuxDirent64 {
    inode: u64,
    offset: u64,
    reclen: u16,
    typ: u8,
    name: [u8; 0], // flexible array
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let path = b".\0";
    let fd = open(path.as_ptr(), O_RDONLY, 0);
    if fd < 0 {
        exit(1);
    }

    let mut buf = [0u8; BUF_SIZE];
    loop {
        let n = getdents64(fd, buf.as_mut_ptr(), BUF_SIZE);
        if n <= 0 {
            break;
        }

        let mut offset = 0;
        while offset < n as usize {
            let entry = &*(buf.as_ptr().add(offset) as *const LinuxDirent64);
            let name_ptr = entry.name.as_ptr();
            let mut name_len = 0;
            while *name_ptr.add(name_len) != 0 {
                name_len += 1;
            }

            // Skip "." and ".."
            if name_len == 1 && *name_ptr == b'.' {
                // .
            } else if name_len == 2 && *name_ptr == b'.' && *name_ptr.add(1) == b'.' {
                // ..
            } else {
                write(1, name_ptr, name_len);
                write(1, b"\n".as_ptr(), 1);
            }

            offset += entry.reclen as usize;
        }
    }

    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
