// event_bus.rs
// Compile: rustc --edition=2021 -C panic=abort event_bus.rs -o event_bus_demo

#![no_std]
#![no_main]

#[link(name = "c")]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(status: i32) -> !;
}

// ──────────────────────────────────────────────────────────────
// Configuration & types
// ──────────────────────────────────────────────────────────────
const MAX_LISTENERS: usize = 64;

type Callback = unsafe extern "C" fn(event: *const u8, data: *mut u8);
type ListenerId = u32;

#[derive(Copy, Clone)]
#[repr(C)]
struct Entry {
    event: *const u8,
    cb:    Callback,
    id:    ListenerId,
}

// Placeholder callback (never called)
unsafe extern "C" fn invalid_cb(_event: *const u8, _data: *mut u8) {
    // Intentionally empty
}

impl Entry {
    const EMPTY: Self = Entry {
        event: core::ptr::null(),
        cb:    invalid_cb,
        id:    0,
    };
}

// ──────────────────────────────────────────────────────────────
// Global storage
// ──────────────────────────────────────────────────────────────
static mut TABLE: [Entry; MAX_LISTENERS] = [Entry::EMPTY; MAX_LISTENERS];
static mut COUNT: usize = 0;
static mut NEXT_ID: ListenerId = 1; // Start at 1; 0 = invalid

// ──────────────────────────────────────────────────────────────
// Public API (unsafe due to raw pointers, but ID-based removal is reliable)
// ──────────────────────────────────────────────────────────────
unsafe fn eventbus_on(event: *const u8, cb: Callback) -> Option<ListenerId> {
    if COUNT >= MAX_LISTENERS {
        return None;
    }
    let id = NEXT_ID;
    NEXT_ID = NEXT_ID.wrapping_add(1);
    // Avoid 0 (reserved for invalid)
    if id == 0 {
        NEXT_ID = 1;
        return Some(1); // This case is extremely unlikely but safe
    }
    TABLE[COUNT] = Entry { event, cb, id };
    COUNT += 1;
    Some(id)
}

unsafe fn eventbus_off(id: ListenerId) -> i32 {
    if id == 0 {
        return -1; // 0 is never a valid ID
    }
    for i in 0..COUNT {
        if TABLE[i].id == id {
            TABLE[i] = TABLE[COUNT - 1];
            COUNT -= 1;
            return 0;
        }
    }
    -1
}

unsafe fn eventbus_emit(event: *const u8, data: *mut u8) {
    let count = COUNT;
    for i in 0..count {
        let entry = TABLE[i];
        if entry.event == event {
            (entry.cb)(event, data);
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Tiny printing helpers (no std, no alloc)
// ──────────────────────────────────────────────────────────────
unsafe fn write_str(s: &str) {
    let _ = write(1, s.as_ptr(), s.len());
}

unsafe fn print_u32(mut n: u32) {
    let mut buf = [0u8; 20];
    let mut i = 0;

    if n == 0 {
        write_str("0\n");
        return;
    }

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        i += 1;
        n /= 10;
    }

    while i > 0 {
        i -= 1;
        let _ = write(1, &buf[i] as *const u8, 1);
    }
    write_str("\n");
}

// ──────────────────────────────────────────────────────────────
// Demo callback
// ──────────────────────────────────────────────────────────────
unsafe extern "C" fn on_damage(_event: *const u8, data: *mut u8) {
    let dmg = *(data as *const i32);
    write_str("[EVENT] Damage: ");
    // Reuse print_u32 logic for positive i32
    if dmg >= 0 {
        print_u32(dmg as u32);
    } else {
        write_str("-");
        print_u32((-dmg) as u32);
    }
}

// ──────────────────────────────────────────────────────────────
// Entry point
// ──────────────────────────────────────────────────────────────
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    static EVENT_NAME: &[u8] = b"damage\0";

    let id = eventbus_on(EVENT_NAME.as_ptr(), on_damage);
    if id.is_none() {
        write_str("Failed to register listener!\n");
        return 1;
    }
    let id = id.unwrap();

    let damage = 15;
    eventbus_emit(EVENT_NAME.as_ptr(), &damage as *const i32 as *mut u8);

    if eventbus_off(id) != 0 {
        write_str("Failed to unregister listener!\n");
        return 1;
    }

    // Emit again — should produce no output
    let damage2 = 42;
    eventbus_emit(EVENT_NAME.as_ptr(), &damage2 as *const i32 as *mut u8);

    0
}

// ──────────────────────────────────────────────────────────────
// Required no-std stubs
// ──────────────────────────────────────────────────────────────
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { exit(1) }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    loop {}
}
