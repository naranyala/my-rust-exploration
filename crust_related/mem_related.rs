// build: rustc --edition 2021 -C panic=abort memory.rs -o memory
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Stack-allocated arrays
    let mut array1 = [0u8; 16];
    let array2 = [b'A', b'B', b'C', b'D', b'E', b'F', 0];

    // Demonstrate memory operations
    my_memset(&mut array1, b'X');
    let _ = write(1, b"After memset: ".as_ptr(), 14);
    let _ = write(1, array1.as_ptr(), array1.len());
    let _ = write(1, b"\n".as_ptr(), 1);

    my_memcpy(&mut array1, &array2);
    let _ = write(1, b"After memcpy: ".as_ptr(), 14);
    let _ = write(1, array1.as_ptr(), array1.len());
    let _ = write(1, b"\n".as_ptr(), 1);

    // Find character
    let search_result = my_memchr(&array1, b'C');
    let mut msg = [0u8; 64]; // Buffer lives long enough
    let position_msg: &[u8] = if let Some(pos) = search_result {
        let len = format_usize(pos, &mut msg, b"Found 'C' at index: ");
        &msg[..len]
    } else {
        b"Character not found\n"
    };

    let _ = write(1, position_msg.as_ptr(), position_msg.len());

    0
}

fn my_memset(dest: &mut [u8], value: u8) {
    for byte in dest.iter_mut() {
        *byte = value;
    }
}

fn my_memcpy(dest: &mut [u8], src: &[u8]) {
    let len = core::cmp::min(dest.len(), src.len());
    for i in 0..len {
        dest[i] = src[i];
    }
}

fn my_memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    for (i, &byte) in haystack.iter().enumerate() {
        if byte == needle {
            return Some(i);
        }
    }
    None
}

fn format_usize(n: usize, buffer: &mut [u8], prefix: &[u8]) -> usize {
    let mut i = 0;

    // Copy prefix
    for &byte in prefix {
        if i < buffer.len() {
            buffer[i] = byte;
            i += 1;
        } else {
            return i;
        }
    }

    // Format number
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
    for j in (0..digit_count).rev() {
        if i < buffer.len() {
            buffer[i] = digits[j];
            i += 1;
        } else {
            return i;
        }
    }

    // Add newline
    if i < buffer.len() {
        buffer[i] = b'\n';
        i += 1;
    }

    i
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    loop {}
}
