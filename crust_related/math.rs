// build: rustc --edition 2021 -C panic=abort math.rs -o math
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}


#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let header = b"Fibonacci Sequence (first 15):\n";
    let _ = write(1, header.as_ptr(), header.len());

    let fib_seq = generate_fibonacci();

    for (i, &num) in fib_seq.iter().enumerate() {
        let mut buffer = [0u8; 20];
        let len = format_number(num, &mut buffer);
        let _ = write(1, buffer.as_ptr(), len);

        let separator: &[u8] = if i == fib_seq.len() - 1 { b"\n" } else { b", " };
        let _ = write(1, separator.as_ptr(), separator.len());
    }

    0
}

fn generate_fibonacci() -> [u64; 15] {
    let mut fib = [0u64; 15];
    if fib.len() > 0 {
        fib[0] = 0;
    }
    if fib.len() > 1 {
        fib[1] = 1;
    }
    for i in 2..fib.len() {
        fib[i] = fib[i - 1] + fib[i - 2];
    }
    fib
}

fn format_number(mut n: u64, buffer: &mut [u8]) -> usize {
    if n == 0 {
        buffer[0] = b'0';
        return 1;
    }

    let mut i = 0;
    while n > 0 {
        if i >= buffer.len() {
            unsafe { exit(1); }
        }
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    let len = i;
    for j in 0..len / 2 {
        buffer.swap(j, len - 1 - j);
    }

    len
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1); }
}


#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
