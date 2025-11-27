// rustc --edition 2021 -C panic=abort sample.rs -o sample

#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let message = b"Hello, World!\n";
    let res = unsafe { write(1, message.as_ptr(), message.len()) };
    if res == -1 {
        unsafe { exit(1); }
    }
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}
