
# my-rust-exploration

> TODO

## reusable reactivity primitives

```rust 
// reactive/mod.rs

use core::{cmp, mem::MaybeUninit, ptr, str};

const MAX_DEPS: usize = 8;
const MAX_STR: usize = 256;
const MAX_SIGNALS: usize = 256;


#[allow(unused)]
#[derive(Copy, Clone, PartialEq)]
pub enum SignalType {
    Int,
    Double,
    String,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Signal {
    pub(crate) ty: SignalType,
    pub(crate) dirty: bool,

    // "Union" fields
    pub(crate) val_i: i32,
    pub(crate) val_d: f64,
    pub(crate) val_s: [u8; MAX_STR],

    pub(crate) deps: [*mut Signal; MAX_DEPS],
    pub(crate) dep_count: usize,

    pub(crate) compute: Option<fn(&mut Signal)>,
}

// Static pool and registry (no dynamic allocation)
static mut POOL: [MaybeUninit<Signal>; MAX_SIGNALS] =
    [MaybeUninit::<Signal>::uninit(); MAX_SIGNALS];
static mut POOL_USED: usize = 0;

static mut SIGNALS: [*mut Signal; MAX_SIGNALS] = [ptr::null_mut(); MAX_SIGNALS];
static mut SIGNAL_COUNT: usize = 0;

fn register_signal(s: *mut Signal) {
    unsafe {
        if SIGNAL_COUNT < MAX_SIGNALS {
            SIGNALS[SIGNAL_COUNT] = s;
            SIGNAL_COUNT += 1;
        }
    }
}

fn alloc_signal(initial: Signal) -> *mut Signal {
    unsafe {
        if POOL_USED >= MAX_SIGNALS {
            return ptr::null_mut();
        }
        let slot = &mut POOL[POOL_USED];
        let p = slot.as_mut_ptr();
        ptr::write(p, initial);
        POOL_USED += 1;
        register_signal(p);
        p
    }
}

// ====================== Creation ======================
pub fn signal_int(value: i32) -> *mut Signal {
    let s = Signal {
        ty: SignalType::Int,
        dirty: false,
        val_i: value,
        val_d: 0.0,
        val_s: [0; MAX_STR],
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

#[allow(unused)]
pub fn signal_double(value: f64) -> *mut Signal {
    let s = Signal {
        ty: SignalType::Double,
        dirty: false,
        val_i: 0,
        val_d: value,
        val_s: [0; MAX_STR],
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

pub fn signal_string(value: &str) -> *mut Signal {
    let mut val_s = [0u8; MAX_STR];
    let bytes = value.as_bytes();
    let len = cmp::min(bytes.len(), MAX_STR - 1);
    val_s[..len].copy_from_slice(&bytes[..len]);
    val_s[len] = 0;

    let s = Signal {
        ty: SignalType::String,
        dirty: false,
        val_i: 0,
        val_d: 0.0,
        val_s,
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

// ====================== Getters ======================
#[inline]
pub fn get_int(s: *mut Signal) -> i32 {
    unsafe {
        if s.is_null() {
            return 0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        (*s).val_i
    }
}

#[inline]
#[allow(unused)]
pub fn get_double(s: *mut Signal) -> f64 {
    unsafe {
        if s.is_null() {
            return 0.0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        (*s).val_d
    }
}

#[inline]
pub fn get_string(s: *mut Signal, out: &mut [u8]) -> usize {
    unsafe {
        if s.is_null() {
            if !out.is_empty() {
                out[0] = 0;
            }
            return 0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        let sig = &*s;
        let len = sig
            .val_s
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_STR);
        let copy_len = cmp::min(len, if out.len() > 0 { out.len() - 1 } else { 0 });
        if copy_len > 0 {
            out[..copy_len].copy_from_slice(&sig.val_s[..copy_len]);
        }
        if !out.is_empty() {
            out[copy_len] = 0;
        }
        copy_len
    }
}

// ====================== Setters ======================
#[inline]
pub fn set_int(s: *mut Signal, value: i32) {
    unsafe {
        if s.is_null() {
            return;
        }
        if (*s).val_i != value || (*s).ty != SignalType::Int {
            (*s).ty = SignalType::Int;
            (*s).val_i = value;
            (*s).dirty = true;
            propagate(s);
        }
    }
}

#[inline]
#[allow(unused)]
pub fn set_double(s: *mut Signal, value: f64) {
    unsafe {
        if s.is_null() {
            return;
        }
        if (*s).val_d != value || (*s).ty != SignalType::Double {
            (*s).ty = SignalType::Double;
            (*s).val_d = value;
            (*s).dirty = true;
            propagate(s);
        }
    }
}

#[inline]
pub fn set_string(s: *mut Signal, value: &str) {
    unsafe {
        if s.is_null() {
            return;
        }
        let sig = &mut *s;
        let bytes = value.as_bytes();
        let len = cmp::min(bytes.len(), MAX_STR - 1);

        let curr_len = sig
            .val_s
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_STR);

        let mut changed = curr_len != len;
        if !changed {
            for i in 0..len {
                if sig.val_s[i] != bytes[i] {
                    changed = true;
                    break;
                }
            }
        }

        if changed || sig.ty != SignalType::String {
            sig.ty = SignalType::String;
            sig.val_s[..len].copy_from_slice(&bytes[..len]);
            sig.val_s[len] = 0;
            sig.dirty = true;
            propagate(s);
        }
    }
}

// ====================== Computed Signals ======================
pub fn signal_computed(compute: fn(&mut Signal), deps: &[*mut Signal]) -> *mut Signal {
    let dep_count = deps.len().min(MAX_DEPS);
    let mut deps_arr = [ptr::null_mut(); MAX_DEPS];
    deps_arr[..dep_count].copy_from_slice(&deps[..dep_count]);

    let s = Signal {
        ty: SignalType::Int, // Default type, compute function should set this
        dirty: true, // Start as dirty to force first computation
        val_i: 0,
        val_d: 0.0,
        val_s: [0; MAX_STR],
        deps: deps_arr,
        dep_count,
        compute: Some(compute),
    };

    alloc_signal(s)
}

// Safe compute function for doubling
pub fn compute_double(signal: &mut Signal) {
    if signal.dep_count == 0 {
        return;
    }
    
    let counter = signal.deps[0];
    if counter.is_null() {
        return;
    }
    
    // Get the current counter value and compute doubled
    let v = get_int(counter);
    signal.ty = SignalType::Int;
    signal.val_i = v * 2;
}

// ====================== Propagation ======================
fn propagate(changed: *mut Signal) {
    unsafe {
        if changed.is_null() {
            return;
        }
        for i in 0..SIGNAL_COUNT {
            let candidate = SIGNALS[i];
            if candidate.is_null() || candidate == changed {
                continue;
            }

            let mut is_dependent = false;
            for j in 0..(*candidate).dep_count {
                if (*candidate).deps[j] == changed {
                    is_dependent = true;
                    break;
                }
            }

            if is_dependent {
                (*candidate).dirty = true;
            }
        }
    }
}

// ====================== Utilities ======================
pub fn signals_reset() {
    unsafe {
        // Just reset the counters - Signal doesn't need destructors
        for i in 0..POOL_USED {
            SIGNALS[i] = ptr::null_mut();
        }
        POOL_USED = 0;
        SIGNAL_COUNT = 0;
    }
}

```

## raylib counter with event bus (not use previous primitives yet)

```rs

#![no_std]
#![no_main]

// ---------- C bindings (only what raylib needs) ----------
#[link(name = "raylib")]
#[link(name = "m")]
#[link(name = "pthread")]
#[link(name = "dl")]
#[link(name = "rt")]
#[link(name = "GL")]
#[link(name = "c")]
extern "C" {
    fn InitWindow(width: i32, height: i32, title: *const u8);
    fn CloseWindow();
    fn WindowShouldClose() -> bool;
    fn SetTargetFPS(fps: i32);

    fn BeginDrawing();
    fn EndDrawing();
    fn ClearBackground(color: Color);
    fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);

    fn IsMouseButtonPressed(button: i32) -> bool;
    fn GetMouseX() -> i32;
    fn GetMouseY() -> i32;
}
const MOUSE_LEFT: i32 = 0;

// ---------- re-use the event-bus implementation ----------
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
unsafe extern "C" fn invalid_cb(_: *const u8, _: *mut u8) {}
impl Entry {
    const EMPTY: Self = Entry {
        event: core::ptr::null(),
        cb:    invalid_cb,
        id:    0,
    };
}
static mut TABLE: [Entry; MAX_LISTENERS] = [Entry::EMPTY; MAX_LISTENERS];
static mut COUNT: usize = 0;
static mut NEXT_ID: ListenerId = 1;

unsafe fn eventbus_on(event: *const u8, cb: Callback) -> Option<ListenerId> {
    if COUNT >= MAX_LISTENERS { return None; }
    let id = NEXT_ID;
    NEXT_ID = NEXT_ID.wrapping_add(1);
    if id == 0 { NEXT_ID = 1; }
    TABLE[COUNT] = Entry { event, cb, id };
    COUNT += 1;
    Some(id)
}
unsafe fn eventbus_emit(event: *const u8, data: *mut u8) {
    let c = COUNT;
    for i in 0..c {
        let e = TABLE[i];
        if e.event == event { (e.cb)(event, data); }
    }
}

// ---------- event names ----------
static EVT_INC: &[u8] = b"inc\0";
static EVT_MUL: &[u8] = b"mul\0";

// ---------- global counter ----------
static mut COUNTER: u32 = 0;

unsafe extern "C" fn handle_inc(_: *const u8, _: *mut u8) {
    COUNTER = COUNTER.wrapping_add(1);
}
unsafe extern "C" fn handle_mul(_: *const u8, _: *mut u8) {
    COUNTER = COUNTER.wrapping_mul(2);
}

// ---------- colors ----------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color { r: u8, g: u8, b: u8, a: u8 }
const RAYWHITE: Color = Color{r:245,g:245,b:245,a:255};
const BLACK:    Color = Color{r:0,g:0,b:0,a:255};
const RED:      Color = Color{r:230,g:41,b:55,a:255};
const GREEN:    Color = Color{r:0,g:228,b:48,a:255};
const BLUE:     Color = Color{r:0,g:121,b:241,a:255};

// ---------- u32 -> &str ----------
fn u32_to_str(mut n: u32) -> &'static str {
    static mut BUF: [u8; 11] = [0; 11];
    unsafe {
        let mut i = 9;
        if n == 0 { BUF[i] = b'0'; } else {
            while n > 0 { BUF[i] = (n % 10) as u8 + b'0'; n /= 10; i -= 1; }
            i += 1;
        }
        BUF[10] = 0;
        core::str::from_utf8_unchecked(&BUF[i..11])
    }
}

// ---------- entry ----------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(800, 450, b"Counter + EventBus\0".as_ptr());
    SetTargetFPS(60);

    // subscribe once
    eventbus_on(EVT_INC.as_ptr(), handle_inc);
    eventbus_on(EVT_MUL.as_ptr(), handle_mul);

    let btn_plus = (80, 200, 140, 60);
    let btn_mul  = (250, 200, 140, 60);

    while !WindowShouldClose() {
        // ---- input ----
        if IsMouseButtonPressed(MOUSE_LEFT) {
            let mx = GetMouseX();
            let my = GetMouseY();
            if mx >= btn_plus.0 && mx <= btn_plus.0 + btn_plus.2 &&
               my >= btn_plus.1 && my <= btn_plus.1 + btn_plus.3 {
                eventbus_emit(EVT_INC.as_ptr(), core::ptr::null_mut());
            }
            if mx >= btn_mul.0 && mx <= btn_mul.0 + btn_mul.2 &&
               my >= btn_mul.1 && my <= btn_mul.1 + btn_mul.3 {
                eventbus_emit(EVT_MUL.as_ptr(), core::ptr::null_mut());
            }
        }

        // ---- draw ----
        BeginDrawing();
        ClearBackground(RAYWHITE);
        DrawText(b"Counter:\0".as_ptr(), 80, 80, 40, BLACK);
        DrawText(u32_to_str(COUNTER).as_ptr(), 280, 82, 40, RED);
        DrawRectangle(btn_plus.0, btn_plus.1, btn_plus.2, btn_plus.3, BLUE);
        DrawText(b"+1\0".as_ptr(), btn_plus.0 + 50, btn_plus.1 + 20, 30, RAYWHITE);
        DrawRectangle(btn_mul.0, btn_mul.1, btn_mul.2, btn_mul.3, GREEN);
        DrawText(b"x2\0".as_ptr(), btn_mul.0 + 50, btn_mul.1 + 20, 30, RAYWHITE);
        EndDrawing();
    }

    CloseWindow();
    0
}

// ---------- no-std stubs ----------
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
```
