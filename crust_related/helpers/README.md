# helpers.rs - C FFI & no_std Utilities

Comprehensive utility library for working with C code in Rust, especially for `no_std` environments.

## C String Helpers

### `cstr()`
Format dynamic strings for C FFI (null-terminated):
```rust
unsafe {
    DrawText(cstr(format_args!("Score: {}", score)), 10, 10, 20, BLACK);
}
```

### `c_str!()` macro
Create static C string literals:
```rust
InitWindow(800, 600, c_str!("My Game"));
```

### `str_to_cstr()`
Convert Rust string to C string (stack-allocated):
```rust
let mut buf = [0u8; 64];
let c_string = str_to_cstr("Hello", &mut buf);
```

## Random Number Generation

All functions use a simple LCG (no dependencies):

```rust
let mut seed = 12345u64;
let f = rand_f32(&mut seed);           // 0.0..1.0
let r = rand_range(&mut seed, 10.0, 20.0);  // 10.0..20.0
let i = rand_int(&mut seed, 1, 100);   // 1..100
let b = rand_bool(&mut seed);          // true/false
```

## Memory Utilities

### `zero_memory()`
Zero-initialize C structs:
```rust
unsafe {
    let mut my_struct: MyStruct = std::mem::uninitialized();
    zero_memory(&mut my_struct);
}
```

### `copy_memory()`
Fast memory copy (memcpy equivalent):
```rust
unsafe {
    copy_memory(src_ptr, dst_ptr, count);
}
```

## StackVec - No-Heap Vector

Fixed-size vector for `no_std` environments (no heap allocation):

```rust
let mut vec: StackVec<i32, 10> = StackVec::new();
vec.push(42)?;
vec.push(100)?;
assert_eq!(vec.len(), 2);
assert_eq!(vec.pop(), Some(100));

// Use as slice
for item in vec.as_slice() {
    println!("{}", item);
}
```

## Math Utilities

```rust
let clamped = clamp_f32(value, 0.0, 1.0);
let clamped_int = clamp_i32(value, 0, 100);
let interpolated = lerp_f32(start, end, 0.5);
let mapped = map_range(value, 0.0, 100.0, 0.0, 1.0);
```

## no_std Support

### Panic Handler
```rust
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### Dummy Allocator
For environments that require a global allocator but don't use heap:
```rust
#[global_allocator]
static ALLOCATOR: helpers::DummyAllocator = helpers::DummyAllocator;
```

## Usage Examples

### Basic C FFI Demo
```rust
mod helpers;
use helpers::*;

extern "C" {
    fn some_c_function(msg: *const i8);
}

fn main() {
    unsafe {
        // Static string
        some_c_function(c_str!("Hello"));
        
        // Dynamic string
        let name = "World";
        some_c_function(cstr(format_args!("Hello {}", name)));
    }
}
```

### no_std Game Loop
```rust
#![no_std]
#![no_main]

mod helpers;
use helpers::*;

static mut ENTITIES: StackVec<Entity, 100> = StackVec::new();

#[no_mangle]
pub extern "C" fn game_update() {
    unsafe {
        let mut rng = 12345;
        
        // Spawn random entity
        if rand_bool(&mut rng) {
            let _ = ENTITIES.push(Entity {
                x: rand_range(&mut rng, 0.0, 800.0),
                y: rand_range(&mut rng, 0.0, 600.0),
            });
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
```
