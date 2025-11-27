// build: rustc --edition 2021 -C panic=abort string.rs -o string
#![no_std]
#![no_main]


#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let original = b"Hello, Low-Level World!";
    let mut reversed = *original;
    
    reverse_bytes(&mut reversed);
    
    let msg1 = b"Original: ";
    let msg2 = b"\nReversed: ";
    let newline = b"\n";
    
    let _ = unsafe { write(1, msg1.as_ptr(), msg1.len()) };
    let _ = unsafe { write(1, original.as_ptr(), original.len()) };
    let _ = unsafe { write(1, msg2.as_ptr(), msg2.len()) };
    let _ = unsafe { write(1, reversed.as_ptr(), reversed.len()) };
    let _ = unsafe { write(1, newline.as_ptr(), newline.len()) };
    
    0
}

fn reverse_bytes(bytes: &mut [u8]) {
    let len = bytes.len();
    for i in 0..len/2 {
        bytes.swap(i, len - 1 - i);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }

