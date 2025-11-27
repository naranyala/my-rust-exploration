// build: rustc --edition 2021 -C panic=abort hello.rs -o hello
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let message = b"Hello, no_std World!\n";
    let _ = unsafe { write(1, message.as_ptr(), message.len()) };
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}
