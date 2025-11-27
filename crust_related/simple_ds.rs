// build: rustc --edition 2021 -C panic=abort datastruct.rs -o datastruct
#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

// A simple fixed-size stack
struct Stack<T, const N: usize> {
    data: [Option<T>; N],
    top: usize,
}

impl<T: Copy, const N: usize> Stack<T, N> {
    const fn new() -> Self {
        Self {
            data: [None; N],
            top: 0,
        }
    }
    
    fn push(&mut self, value: T) -> Result<(), &'static str> {
        if self.top >= N {
            return Err("Stack overflow");
        }
        self.data[self.top] = Some(value);
        self.top += 1;
        Ok(())
    }
    
    fn pop(&mut self) -> Option<T> {
        if self.top == 0 {
            return None;
        }
        self.top -= 1;
        self.data[self.top]
    }
    
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut stack = Stack::<i32, 5>::new();
    
    // Push values
    for i in 0..5 {
        if stack.push(i * 10).is_err() {
            let msg = b"Failed to push to stack\n";
            unsafe { write(1, msg.as_ptr(), msg.len()) };
        }
    }
    
    // Try to overflow
    if stack.push(100).is_err() {
        let msg = b"Stack overflow correctly detected\n";
        unsafe { write(1, msg.as_ptr(), msg.len()) };
    }
    
    // Pop and print values
    while let Some(value) = stack.pop() {
        let mut buffer = [0u8; 20];
        let len = format_signed_number(value, &mut buffer);
        unsafe {
            write(1, buffer.as_ptr(), len);
            write(1, b"\n".as_ptr(), 1);
        }
    }
    
    0
}

fn format_signed_number(n: i32, buffer: &mut [u8]) -> usize {
    let mut num = n.abs() as u32;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;
    let mut buffer_index = 0;
    
    if n < 0 {
        buffer[buffer_index] = b'-';
        buffer_index += 1;
    }
    
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

