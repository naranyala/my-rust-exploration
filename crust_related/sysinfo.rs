// build: rustc --edition 2021 -C panic=abort sysinfo.rs -o sysinfo
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
    let header = b"System Information Demo:\n";
    let _ = unsafe { write(1, header.as_ptr(), header.len()) };
    
    // Simulated system information
    display_info(b"Simulated PID: ", 12345);
    display_info(b"Timestamp: ", 1698765432);
    display_info(b"Memory Usage: ", 2048);
    display_info(b"Stack Size: ", 65536);
    display_info(b"Negative Test: ", -42);
    
    display_bool(b"System OK: ", true);
    display_bool(b"Debug Mode: ", false);
    
    let footer = b"Demo completed successfully!\n";
    let _ = unsafe { write(1, footer.as_ptr(), footer.len()) };
    
    0
}

fn display_info(prefix: &[u8], value: i64) {
    let mut buffer = [0u8; 50];
    let len = format_message(prefix, value, &mut buffer);
    
    unsafe {
        write(1, buffer.as_ptr(), len);
    }
}

fn display_bool(prefix: &[u8], value: bool) {
    let mut buffer = [0u8; 30];
    let mut pos = 0;
    
    // Copy prefix
    for &byte in prefix {
        if pos < buffer.len() {
            buffer[pos] = byte;
            pos += 1;
        }
    }
    
    // Add boolean value - FIXED: Use separate handling for true/false
    if value {
        let bool_str = b"true";
        for &byte in bool_str {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }
    } else {
        let bool_str = b"false";
        for &byte in bool_str {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }
    }
    
    // Add newline
    if pos < buffer.len() {
        buffer[pos] = b'\n';
        pos += 1;
    }
    
    unsafe {
        write(1, buffer.as_ptr(), pos);
    }
}

fn format_message(prefix: &[u8], value: i64, buffer: &mut [u8]) -> usize {
    let mut pos = 0;
    
    // Copy prefix
    for &byte in prefix {
        if pos < buffer.len() {
            buffer[pos] = byte;
            pos += 1;
        }
    }
    
    // Format the number
    pos = format_number(value, buffer, pos);
    
    // Add newline
    if pos < buffer.len() {
        buffer[pos] = b'\n';
        pos += 1;
    }
    
    pos
}

fn format_number(mut n: i64, buffer: &mut [u8], start_pos: usize) -> usize {
    let mut pos = start_pos;
    
    // Handle negative numbers
    let is_negative = n < 0;
    if is_negative {
        if pos < buffer.len() {
            buffer[pos] = b'-';
            pos += 1;
        }
        n = -n;
    }
    
    // Handle zero
    if n == 0 {
        if pos < buffer.len() {
            buffer[pos] = b'0';
            pos += 1;
        }
        return pos;
    }
    
    // Extract digits (in reverse order)
    let mut digits = [0u8; 20];
    let mut digit_count = 0;
    
    while n > 0 && digit_count < digits.len() {
        digits[digit_count] = b'0' + (n % 10) as u8;
        n /= 10;
        digit_count += 1;
    }
    
    // Copy digits in correct order
    for i in (0..digit_count).rev() {
        if pos < buffer.len() {
            buffer[pos] = digits[i];
            pos += 1;
        }
    }
    
    pos
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}
