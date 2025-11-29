// helpers.rs - Utilities for C FFI and no_std environments

#![allow(dead_code)]

// ============================
// C String Helpers
// ============================
static mut TEXT_BUF: [u8; 256] = [0; 256];

/// Format a string for C FFI (null-terminated)
/// Usage: `cstr(format_args!("Hello {}", name))`
pub unsafe fn cstr(args: core::fmt::Arguments) -> *const i8 {
    use std::io::Write;
    let buf = &raw mut TEXT_BUF;
    let mut cursor = std::io::Cursor::new(&mut (*buf) as &mut [u8]);
    let _ = write!(cursor, "{}\0", args);
    (*buf).as_ptr() as *const i8
}

/// Create a static C string literal
/// Usage: `c_str!("Hello World")`
#[allow(unused_macros)]
#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

/// Convert Rust string slice to C string (stack-allocated)
pub fn str_to_cstr<const N: usize>(s: &str, buf: &mut [u8; N]) -> *const i8 {
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
    buf.as_ptr() as *const i8
}

// ============================
// Random Number Generation
// ============================

/// Simple LCG random number generator (no_std compatible)
pub fn rand_f32(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 32) as f32 / 4294967296.0
}

pub fn rand_range(seed: &mut u64, min: f32, max: f32) -> f32 {
    min + rand_f32(seed) * (max - min)
}

pub fn rand_int(seed: &mut u64, min: i32, max: i32) -> i32 {
    min + (rand_f32(seed) * (max - min) as f32) as i32
}

pub fn rand_bool(seed: &mut u64) -> bool {
    rand_f32(seed) > 0.5
}

// ============================
// Memory Utilities
// ============================

/// Zero-initialize memory (useful for C structs)
pub unsafe fn zero_memory<T>(ptr: *mut T) {
    core::ptr::write_bytes(ptr, 0, 1);
}

/// Copy memory (memcpy equivalent)
pub unsafe fn copy_memory<T>(src: *const T, dst: *mut T, count: usize) {
    core::ptr::copy_nonoverlapping(src, dst, count);
}

// ============================
// Array Helpers
// ============================

/// Fixed-size stack-allocated vector (no heap allocation)
pub struct StackVec<T, const N: usize> {
    data: [core::mem::MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    pub const fn new() -> Self {
        Self {
            data: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len].write(value);
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.data[self.len].assume_init_read() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

// ============================
// Math Utilities
// ============================

pub fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

pub fn clamp_i32(v: i32, min: i32, max: i32) -> i32 {
    v.max(min).min(max)
}

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}

// ============================
// Panic Handler for no_std
// ============================

/// Use this in no_std environments:
/// ```
/// #[panic_handler]
/// fn panic(_: &core::panic::PanicInfo) -> ! {
///     loop {}
/// }
/// ```

// ============================
// Allocator Stub for no_std
// ============================

/// Use this in no_std environments that need an allocator:
/// ```
/// #[global_allocator]
/// static ALLOCATOR: helpers::DummyAllocator = helpers::DummyAllocator;
/// ```
pub struct DummyAllocator;

unsafe impl core::alloc::GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}
