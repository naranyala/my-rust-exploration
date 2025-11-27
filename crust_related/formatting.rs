// build: rustc --edition 2021 -C panic=abort format.rs -o format
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Test different data types
    format_and_print(42);
    format_and_print(-123);
    format_and_print(0);
    format_and_print(9999);
    
    0
}

fn format_and_print(n: i32) {
    let mut buffer = [0u8; 32];
    let len = format_signed_number(n, &mut buffer);
    
    unsafe {
        write(1, buffer.as_ptr(), len);
        write(1, b"\n".as_ptr(), 1);
    }
}

fn format_signed_number(n: i32, buffer: &mut [u8]) -> usize {
    let mut num = n.abs() as u32;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;
    let mut buffer_index = 0;
    
    // Handle negative numbers
    if n < 0 {
        buffer[buffer_index] = b'-';
        buffer_index += 1;
    }
    
    // Handle zero
    if num == 0 {
        digits[0] = b'0';
        digit_count = 1;
    } else {
        // Extract digits
        while num > 0 && digit_count < digits.len() {
            digits[digit_count] = b'0' + (num % 10) as u8;
            num /= 10;
            digit_count += 1;
        }
    }
    
    // Copy digits in reverse order
    for i in (0..digit_count).rev() {
        buffer[buffer_index] = digits[i];
        buffer_index += 1;
    }
    
    buffer_index
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}


#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
