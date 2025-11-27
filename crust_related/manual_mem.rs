// build: rustc --edition 2021 -C panic=abort manual_mem.rs -o memory

#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut array = [0u8; 64];
    let array_len = array.len();  // Immutable borrow ends here
    
    // Now mutable borrow is safe
    memset(&mut array, b'A', array_len);
    print_buffer("After memset:", &array);

    let mut dest = [0u8; 64];
    let copy_len = 32;
    memcpy(&array, &mut dest, copy_len);
    print_buffer("After memcpy (first 32 bytes):", &dest);

    let result = memcmp(&array, &dest, copy_len);
    let msg: &[u8] = if result == 0 {
        b"Memory regions are equal\n"
    } else {
        b"Memory regions differ\n"
    };
    let _ = unsafe { write(1, msg.as_ptr(), msg.len()) };
    0
}

fn memset(buffer: &mut [u8], value: u8, count: usize) {
    for i in 0..count {
        if i < buffer.len() {
            buffer[i] = value;
        }
    }
}

fn memcpy(src: &[u8], dest: &mut [u8], count: usize) {
    for i in 0..count {
        if i < src.len() && i < dest.len() {
            dest[i] = src[i];
        }
    }
}

fn memcmp(a: &[u8], b: &[u8], count: usize) -> i32 {
    for i in 0..count {
        if i >= a.len() || i >= b.len() {
            return 1;
        }
        if a[i] != b[i] {
            return (a[i] as i32) - (b[i] as i32);
        }
    }
    0
}

fn print_buffer(prefix: &str, buffer: &[u8]) {
    let prefix_bytes = prefix.as_bytes();
    unsafe {
        let _ = write(1, prefix_bytes.as_ptr(), prefix_bytes.len());
        let _ = write(1, b" ".as_ptr(), 1);
        for &byte in buffer.iter().take(16) {
            let mut hex_buf = [0u8; 2];
            let len = byte_to_hex(byte, &mut hex_buf);
            let _ = write(1, hex_buf.as_ptr(), len);
            let _ = write(1, b" ".as_ptr(), 1);
        }
        let _ = write(1, b"\n".as_ptr(), 1);
    }
}

fn byte_to_hex(byte: u8, buffer: &mut [u8]) -> usize {
    let high = (byte >> 4) & 0x0F;
    let low = byte & 0x0F;
    buffer[0] = if high < 10 { b'0' + high } else { b'A' + (high - 10) };
    buffer[1] = if low < 10 { b'0' + low } else { b'A' + (low - 10) };
    2
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }

