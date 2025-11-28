// reactive_counter.rs
// rustc --edition 2021 -C panic=abort reactive_counter.rs -o reactive_counter -lraylib -lc

#![no_std]
#![no_main]

mod reactive;
use reactive::*;

use core::str;

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

    fn DrawText(text: *const u8, x: i32, y: i32, size: i32, color: Color);
    fn DrawRectangle(x: i32, y: i32, w: i32, h: i32, color: Color);

    fn GetFrameTime() -> f32;
    fn IsMouseButtonPressed(button: i32) -> bool;
    fn GetMouseX() -> i32;
    fn GetMouseY() -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color { 
    pub r: u8, 
    pub g: u8, 
    pub b: u8, 
    pub a: u8 
}

const WHITE: Color = Color{r:255, g:255, b:255, a:255};
const BLACK: Color = Color{r:0, g:0, b:0, a:255};
// const RED: Color = Color{r:230, g:41, b:55, a:255};
const GREEN: Color = Color{r:0, g:228, b:48, a:255};
const BLUE: Color = Color{r:0, g:121, b:241, a:255};
const GRAY: Color = Color{r:130, g:130, b:130, a:255};
const LIGHTGRAY: Color = Color{r:200, g:200, b:200, a:255};
const DARKGRAY: Color = Color{r:80, g:80, b:80, a:255};

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const u8
    };
}

// Helper to format numbers without allocation
fn format_int_to_buf(buf: &mut [u8; 64], value: i32) -> &str {
    let mut pos = 0;
    let negative = value < 0;
    let mut v: i64 = if negative { -(value as i64) } else { value as i64 };
    
    if v == 0 {
        buf[0] = b'0';
        return unsafe { str::from_utf8_unchecked(&buf[..1]) };
    }

    let mut temp = [0u8; 32];
    let mut temp_pos = 0;
    while v > 0 {
        temp[temp_pos] = (v % 10) as u8 + b'0';
        v /= 10;
        temp_pos += 1;
    }

    if negative {
        buf[pos] = b'-';
        pos += 1;
    }

    for i in (0..temp_pos).rev() {
        buf[pos] = temp[i];
        pos += 1;
    }

    unsafe { str::from_utf8_unchecked(&buf[..pos]) }
}

fn copy_str_to_buf(buf: &mut [u8; 256], prefix: &str, value: &str) {
    let mut pos = 0usize;

    let prefix_bytes = prefix.as_bytes();
    let prefix_len = prefix_bytes.len().min(128);
    
    // Use our own copy instead of relying on memcpy
    for i in 0..prefix_len {
        buf[pos + i] = prefix_bytes[i];
    }
    pos += prefix_len;

    let value_bytes = value.as_bytes();
    let value_len = value_bytes.len().min(128);
    
    // Use our own copy instead of relying on memcpy
    for i in 0..value_len {
        buf[pos + i] = value_bytes[i];
    }
    pos += value_len;

    if pos < buf.len() {
        buf[pos] = 0;
    } else {
        buf[buf.len()-1] = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(800, 600, cstr!("Reactive Counter Demo"));
    SetTargetFPS(60);

    // ---- Reactive signals -------------------------------------------------
    let counter = signal_int(0);
    if counter.is_null() {
        return 1;
    }

    let doubled = signal_computed(compute_double, &[counter]);
    if doubled.is_null() {
        return 1;
    }

    let log = signal_string("Click the button to start");
    if log.is_null() {
        return 1;
    }

    // ---- UI state ---------------------------------------------------------
    let button_x = 300;
    let button_y = 400;
    let button_w = 200;
    let button_h = 80;

    // Buffers for text rendering
    let mut counter_buf = [0u8; 256];
    let mut doubled_buf = [0u8; 256];
    let mut log_buf = [0u8; 256];
    let mut temp_num_buf = [0u8; 64];

    // ---- Main loop --------------------------------------------------------
    while !WindowShouldClose() {
        let _dt = GetFrameTime();

        // ----- Input: button click -----------------------------------------
        if IsMouseButtonPressed(0) { // 0 = MOUSE_BUTTON_LEFT
            let mx = GetMouseX();
            let my = GetMouseY();

            if mx >= button_x && mx <= button_x + button_w &&
               my >= button_y && my <= button_y + button_h {
                let new_val = get_int(counter) + 1;
                set_int(counter, new_val);

                // Update log
                let num_str = format_int_to_buf(&mut temp_num_buf, new_val);
                let mut msg_buf = [0u8; 256];
                copy_str_to_buf(&mut msg_buf, "Counter incremented to: ", num_str);

                let len = msg_buf.iter().position(|&b| b == 0).unwrap_or(255);
                let msg_str = unsafe { str::from_utf8_unchecked(&msg_buf[..len]) };
                set_string(log, msg_str);
            }
        }

        // ----- Rendering ----------------------------------------------------
        BeginDrawing();
        ClearBackground(LIGHTGRAY);

        // Title
        DrawText(cstr!("Reactive Counter Demo"), 20, 20, 40, BLACK);

        // Format current values
        let counter_val = get_int(counter);
        let doubled_val = get_int(doubled);

        let counter_str = format_int_to_buf(&mut temp_num_buf, counter_val);
        copy_str_to_buf(&mut counter_buf, "Counter: ", counter_str);

        let doubled_str = format_int_to_buf(&mut temp_num_buf, doubled_val);
        copy_str_to_buf(&mut doubled_buf, "Doubled: ", doubled_str);

        // Get log string
        get_string(log, &mut log_buf);

        // Draw text
        DrawText(counter_buf.as_ptr(), 20, 120, 32, BLACK);
        DrawText(doubled_buf.as_ptr(), 20, 170, 32, BLACK);
        DrawText(log_buf.as_ptr(), 20, 250, 24, GRAY);

        // Button with color based on counter parity
        let btn_color = if counter_val % 2 == 0 { GREEN } else { BLUE };
        DrawRectangle(button_x, button_y, button_w, button_h, btn_color);

        // Button border
        DrawRectangle(button_x, button_y, button_w, 3, DARKGRAY);
        DrawRectangle(button_x, button_y + button_h - 3, button_w, 3, DARKGRAY);
        DrawRectangle(button_x, button_y, 3, button_h, DARKGRAY);
        DrawRectangle(button_x + button_w - 3, button_y, 3, button_h, DARKGRAY);

        DrawText(cstr!("+1"), button_x + 75, button_y + 25, 40, WHITE);

        // Instructions
        DrawText(cstr!("Click the button to increment"), 20, 320, 20, DARKGRAY);

        EndDrawing();
    }

    // Clean up
    signals_reset();
    CloseWindow();

    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Provide our own simple implementations to avoid libc dependency
#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u8;
    for i in 0..n {
        *dest.add(i) = c;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

