// rustc --edition 2021 -C panic=abort event_bus_counter.rs -o event_bus_counter
// (raylib must be installed on your system)

#![no_std]
#![no_main]

use core::ffi::c_char;

// ------------------------------------------------------------------
// Raylib FFI
// ------------------------------------------------------------------
#[link(name = "raylib")]
#[link(name = "m")]
#[link(name = "pthread")]
#[link(name = "dl")]
#[link(name = "rt")]
#[link(name = "GL")]
#[link(name = "c")]
extern "C" {
    pub fn InitWindow(width: i32, height: i32, title: *const c_char);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn SetTargetFPS(fps: i32);

    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);
    pub fn DrawText(text: *const c_char, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);

    pub fn GetMousePosition() -> Vector2;
    pub fn IsMouseButtonPressed(button: i32) -> bool;

    pub fn GetFrameTime() -> f32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector2 { pub x: f32, pub y: f32 }

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

pub const RAYWHITE:   Color = Color { r: 245, g: 245, b: 245, a: 255 };
pub const BLACK:      Color = Color { r: 0,   g: 0,   b: 0,   a: 255 };
pub const RED:        Color = Color { r: 230, g: 41,  b: 55,  a: 255 };
pub const BLUE:       Color = Color { r: 0,   g: 121, b: 241, a: 255 };
pub const GREEN:      Color = Color { r: 0,   g: 228, b: 48,  a: 255 };
pub const YELLOW:     Color = Color { r: 253, g: 249, b: 0,   a: 255 };
pub const GRAY:       Color = Color { r: 130, g: 130, b: 130, a: 255 };
pub const LIGHTGRAY:  Color = Color { r: 200, g: 200, b: 200, a: 255 };

const MOUSE_LEFT_BUTTON: i32 = 0;

// ------------------------------------------------------------------
// no-std integer formatting helpers
// ------------------------------------------------------------------
fn write_i32(buffer: &mut [u8], mut value: i32) -> usize {
    if buffer.is_empty() {
        return 0;
    }

    let mut pos = 0;
    let is_negative = value < 0;

    if is_negative {
        if pos >= buffer.len() - 1 {
            buffer[0] = 0;
            return 0;
        }
        buffer[pos] = b'-';
        pos += 1;
        value = -value;
    }

    let mut temp = value;

    if temp == 0 {
        if pos >= buffer.len() - 1 {
            buffer[0] = 0;
            return 0;
        }
        buffer[pos] = b'0';
        pos += 1;
    } else {
        let digit_start = pos;
        while temp > 0 && pos < buffer.len() - 1 {
            buffer[pos] = b'0' + (temp % 10) as u8;
            pos += 1;
            temp /= 10;
        }

        let digit_end = pos - 1;
        let mut i = digit_start;
        let mut j = digit_end;
        while i < j {
            let tmp = buffer[i];
            buffer[i] = buffer[j];
            buffer[j] = tmp;
            i += 1;
            j -= 1;
        }
    }

    buffer[pos] = 0; // Null terminate
    pos
}

fn copy_str_to_buffer(buffer: &mut [u8], s: &[u8], offset: usize) -> usize {
    if offset >= buffer.len() - 1 {
        return offset;
    }

    let available = buffer.len() - offset - 1; // Reserve space for null
    let to_copy = core::cmp::min(s.len(), available);

    buffer[offset..offset + to_copy].copy_from_slice(&s[..to_copy]);
    offset + to_copy
}

// ------------------------------------------------------------------
// Event Bus
// ------------------------------------------------------------------
const MAX_LISTENERS: usize = 64;
type Callback = unsafe extern "C" fn(event: *const c_char, data: *mut u8);
type ListenerId = u32;

#[repr(C)]
#[derive(Copy, Clone)]
struct Entry {
    event: *const c_char,
    cb: Callback,
    id: ListenerId,
}

unsafe extern "C" fn invalid_cb(_: *const c_char, _: *mut u8) {}

impl Entry {
    const EMPTY: Self = Entry {
        event: core::ptr::null(),
        cb: invalid_cb,
        id: 0,
    };
}

static mut EVENT_TABLE: [Entry; MAX_LISTENERS] = [Entry::EMPTY; MAX_LISTENERS];
static mut EVENT_COUNT: usize = 0;
static mut NEXT_LISTENER_ID: ListenerId = 1;

unsafe fn eventbus_on(event: *const c_char, cb: Callback) -> Option<ListenerId> {
    if EVENT_COUNT >= MAX_LISTENERS { return None; }
    let id = NEXT_LISTENER_ID;
    NEXT_LISTENER_ID = NEXT_LISTENER_ID.wrapping_add(1);
    if id == 0 { NEXT_LISTENER_ID = 1; }

    EVENT_TABLE[EVENT_COUNT] = Entry { event, cb, id };
    EVENT_COUNT += 1;
    Some(id)
}

#[allow(dead_code)]
unsafe fn eventbus_off(id: ListenerId) -> i32 {
    for i in 0..EVENT_COUNT {
        if EVENT_TABLE[i].id == id {
            EVENT_TABLE[i] = EVENT_TABLE[EVENT_COUNT - 1];
            EVENT_COUNT -= 1;
            return 0;
        }
    }
    -1
}

unsafe fn eventbus_emit(event: *const c_char, data: *mut u8) {
    let count = EVENT_COUNT;
    for i in 0..count {
        let entry = EVENT_TABLE[i];
        if !entry.event.is_null() && entry.event == event {
            (entry.cb)(event, data);
        }
    }
}

// ------------------------------------------------------------------
// Application Events / Data
// ------------------------------------------------------------------
static EVENT_COUNTER_UPDATED: &[u8] = b"counter_updated\0";
static EVENT_BUTTON_PRESSED:  &[u8] = b"button_pressed\0";
static EVENT_LOG_MESSAGE:     &[u8] = b"log_message\0";

#[repr(C)]
#[derive(Copy, Clone)]
struct CounterData { value: i32, doubled_value: i32 }

#[repr(C)]
#[derive(Copy, Clone)]
struct LogMessage {
    message: [u8; 128],
    length: usize,
}

// ------------------------------------------------------------------
// Event Handlers
// ------------------------------------------------------------------
unsafe extern "C" fn on_counter_updated(_: *const c_char, data: *mut u8) {
    let counter_data = &*(data as *const CounterData);
    let mut log_msg = LogMessage { message: [0; 128], length: 0 };

    let mut pos = 0;
    pos = copy_str_to_buffer(&mut log_msg.message, b"Counter updated: ", pos);
    pos = write_i32(&mut log_msg.message[pos..], counter_data.value) + pos;
    pos = copy_str_to_buffer(&mut log_msg.message, b" (doubled: ", pos);
    pos = write_i32(&mut log_msg.message[pos..], counter_data.doubled_value) + pos;
    pos = copy_str_to_buffer(&mut log_msg.message, b")", pos);

    log_msg.message[pos] = 0; // Null terminate
    log_msg.length = pos;

    eventbus_emit(EVENT_LOG_MESSAGE.as_ptr() as *const c_char, &mut log_msg as *mut _ as *mut u8);
}

unsafe extern "C" fn on_button_pressed(_: *const c_char, _: *mut u8) {
    let mut log_msg = LogMessage { message: [0; 128], length: 0 };

    let msg = b"Button pressed!";
    let len = copy_str_to_buffer(&mut log_msg.message, msg, 0);
    log_msg.message[len] = 0; // Null terminate
    log_msg.length = len;

    eventbus_emit(EVENT_LOG_MESSAGE.as_ptr() as *const c_char, &mut log_msg as *mut _ as *mut u8);
}

unsafe extern "C" fn on_log_message(_: *const c_char, data: *mut u8) {
    let log_msg = &*(data as *const LogMessage);
    let app_state = get_app_state();
    (*app_state).last_log_message = *log_msg;
}

// ------------------------------------------------------------------
// Application State
// ------------------------------------------------------------------
struct AppState {
    counter: CounterData,
    last_log_message: LogMessage,
    button_rect: (i32, i32, i32, i32),
}

static mut APP_STATE: AppState = AppState {
    counter: CounterData { value: 0, doubled_value: 0 },
    last_log_message: LogMessage { message: [0; 128], length: 0 },
    button_rect: (300, 200, 200, 60),
};

unsafe fn get_app_state() -> *mut AppState {
    &raw mut APP_STATE
}

// ------------------------------------------------------------------
// Button Logic
// ------------------------------------------------------------------
unsafe fn is_point_in_rect(p: Vector2, r: (i32, i32, i32, i32)) -> bool {
    let (x, y, w, h) = r;
    p.x >= x as f32 && p.x <= (x + w) as f32 &&
    p.y >= y as f32 && p.y <= (y + h) as f32
}

unsafe fn handle_button_click() {
    let mouse = GetMousePosition();
    let app = get_app_state();
    if IsMouseButtonPressed(MOUSE_LEFT_BUTTON) &&
       is_point_in_rect(mouse, (*app).button_rect) {
        eventbus_emit(EVENT_BUTTON_PRESSED.as_ptr() as *const c_char, core::ptr::null_mut());

        (*app).counter.value += 1;
        (*app).counter.doubled_value = (*app).counter.value * 2;

        eventbus_emit(
            EVENT_COUNTER_UPDATED.as_ptr() as *const c_char,
            &mut (*app).counter as *mut _ as *mut u8
        );
    }
}

// ------------------------------------------------------------------
// Drawing
// ------------------------------------------------------------------
unsafe fn draw_ui() {
    let app = get_app_state();
    let (x, y, w, h) = (*app).button_rect;

    // Button
    DrawRectangle(x, y, w, h, BLUE);
    DrawRectangle(x + 4, y + 4, w - 8, h - 8, LIGHTGRAY);
    DrawText(b"Click Me!\0".as_ptr() as *const c_char, x + 50, y + 18, 28, BLACK);

    // Counter display
    let mut buf1 = [0u8; 64];
    let mut pos1 = copy_str_to_buffer(&mut buf1, b"Counter: ", 0);
    pos1 = write_i32(&mut buf1[pos1..], (*app).counter.value) + pos1;
    buf1[pos1] = 0;
    DrawText(buf1.as_ptr() as *const c_char, 50, 100, 32, BLACK);

    let mut buf2 = [0u8; 64];
    let mut pos2 = copy_str_to_buffer(&mut buf2, b"Doubled: ", 0);
    pos2 = write_i32(&mut buf2[pos2..], (*app).counter.doubled_value) + pos2;
    buf2[pos2] = 0;
    DrawText(buf2.as_ptr() as *const c_char, 50, 150, 32, GREEN);

    // Log message
    if (*app).last_log_message.length > 0 {
        DrawText((*app).last_log_message.message.as_ptr() as *const c_char, 50, 420, 20, GRAY);
    }

    // Title
    DrawText(b"Event Bus Demo - no_std Rust + Raylib\0".as_ptr() as *const c_char, 50, 360, 24, BLACK);
    DrawText(b"Click the button -> events -> logging\0".as_ptr() as *const c_char, 50, 390, 18, GRAY);
}

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const c_char) -> i32 {
    InitWindow(800, 600, b"Raylib + no_std Rust Event Bus Counter\0".as_ptr() as *const c_char);
    SetTargetFPS(60);

    // Subscribe to events
    let _ = eventbus_on(EVENT_COUNTER_UPDATED.as_ptr() as *const c_char, on_counter_updated);
    let _ = eventbus_on(EVENT_BUTTON_PRESSED.as_ptr() as *const c_char,  on_button_pressed);
    let _ = eventbus_on(EVENT_LOG_MESSAGE.as_ptr() as *const c_char,     on_log_message);

    // Initial log
    let mut initial_log = LogMessage { message: [0; 128], length: 0 };
    let msg = b"Application started! Counter initialized to 0";
    let len = copy_str_to_buffer(&mut initial_log.message, msg, 0);
    initial_log.message[len] = 0;
    initial_log.length = len;

    eventbus_emit(EVENT_LOG_MESSAGE.as_ptr() as *const c_char, &mut initial_log as *mut _ as *mut u8);

    while !WindowShouldClose() {
        handle_button_click();

        BeginDrawing();
        ClearBackground(RAYWHITE);
        draw_ui();
        EndDrawing();
    }

    CloseWindow();
    0
}

// ------------------------------------------------------------------
// no-std stubs
// ------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }

