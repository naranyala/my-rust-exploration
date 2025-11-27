// build: rustc --edition 2021 -C panic=abort cat.rs -o cat
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn open(path: *const u8, flags: i32, mode: u32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn close(fd: i32) -> i32;
    fn exit(status: i32) -> !;
}

const O_RDONLY: i32 = 0;
const BUF_SZ: usize = 4096;

#[no_mangle]
pub unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        exit(0); // no files → nothing to do
    }

    let mut buf = [0u8; BUF_SZ];

    // argv[1] .. argv[argc-1] are file names
    for i in 1..argc as isize {
        let path = *argv.offset(i);
        let fd = open(path, O_RDONLY, 0);
        if fd < 0 {
            exit(1);
        }

        loop {
            let n = read(fd, buf.as_mut_ptr(), BUF_SZ);
            if n <= 0 {
                break;
            }
            write(1, buf.as_ptr(), n as usize);
        }

        close(fd);
    }

    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1) }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() -> ! {
    loop {}
}
