// build: rustc --edition 2021 -C panic=abort args.rs -o args
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let header = b"Command Line Arguments:\n";
    let _ = unsafe { write(1, header.as_ptr(), header.len()) };

    // Convert C args to something we can work with
    for i in 0..argc {
        let arg_ptr = unsafe { *argv.offset(i as isize) };
        if !arg_ptr.is_null() {
            let mut len = 0;
            // Find string length
            while unsafe { *arg_ptr.offset(len) } != 0 {
                len += 1;
            }

            // Write argument number
            let mut num_buf = [0u8; 20];
            let num_len = format_number(i, &mut num_buf);
            let _ = unsafe { write(1, num_buf.as_ptr(), num_len) };
            let _ = unsafe { write(1, b": ".as_ptr(), 2) };

            // Write argument
            let _ = unsafe { write(1, arg_ptr, len as usize) };
            let _ = unsafe { write(1, b"\n".as_ptr(), 1) };
        }
    }

    // Demonstrate different exit codes based on argument count
    match argc {
        1 => {
            let msg = b"No arguments provided\n";
            let _ = unsafe { write(1, msg.as_ptr(), msg.len()) };
        }
        2 => {
            let msg = b"One argument provided\n";
            let _ = unsafe { write(1, msg.as_ptr(), msg.len()) };
        }
        _ => {
            let msg = b"Multiple arguments provided\n";
            let _ = unsafe { write(1, msg.as_ptr(), msg.len()) };
        }
    }

    0
}

fn format_number(n: i32, buffer: &mut [u8]) -> usize {
    let mut num = n;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;

    if num == 0 {
        digits[0] = b'0';
        digit_count = 1;
    } else {
        while num > 0 && digit_count < digits.len() {
            digits[digit_count] = b'0' + (num % 10) as u8;
            num /= 10;
            digit_count += 1;
        }
    }

    // Copy digits in reverse
    for i in (0..digit_count).rev() {
        buffer[digit_count - 1 - i] = digits[i];
    }

    digit_count
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        exit(1);
    }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    loop {}
}
