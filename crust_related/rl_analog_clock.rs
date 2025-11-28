// rustc --edition 2021 -C panic=abort analog_clock.rs -o analog_clock
// (raylib must be installed on your system)

#![no_std]
#![no_main]

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
    fn InitWindow(width: i32, height: i32, title: *const u8);
    fn CloseWindow();
    fn WindowShouldClose() -> bool;
    fn SetTargetFPS(fps: i32);
    fn BeginDrawing();
    fn EndDrawing();
    fn ClearBackground(color: Color);
    fn DrawCircle(centerX: i32, centerY: i32, radius: f32, color: Color);
    fn DrawCircleLines(centerX: i32, centerY: i32, radius: f32, color: Color);
    fn DrawLineEx(startPos: Vector2, endPos: Vector2, thick: f32, color: Color);
    fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    // fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    // fn GetTime() -> f64;
    
    // C time functions
    fn time(tloc: *mut i64) -> i64;
    fn localtime(timer: *const i64) -> *mut Tm;
}

// C tm struct for localtime
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Tm {
    pub tm_sec: i32,    // seconds after the minute [0-60]
    pub tm_min: i32,    // minutes after the hour [0-59]
    pub tm_hour: i32,   // hours since midnight [0-23]
    pub tm_mday: i32,   // day of the month [1-31]
    pub tm_mon: i32,    // months since January [0-11]
    pub tm_year: i32,   // years since 1900
    pub tm_wday: i32,   // days since Sunday [0-6]
    pub tm_yday: i32,   // days since January 1 [0-365]
    pub tm_isdst: i32,  // Daylight Saving Time flag
}

// ------------------------------------------------------------------
// Math functions from libm
// ------------------------------------------------------------------
extern "C" {
    fn sinf(x: f32) -> f32;
    fn cosf(x: f32) -> f32;
}

// ------------------------------------------------------------------
// Types
// ------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector2 { 
    pub x: f32, 
    pub y: f32 
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color { 
    pub r: u8, 
    pub g: u8, 
    pub b: u8, 
    pub a: u8 
}

// ------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------
const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };
const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
const DARKGRAY: Color = Color { r: 80, g: 80, b: 80, a: 255 };
const GRAY: Color = Color { r: 130, g: 130, b: 130, a: 255 };
// const LIGHTGRAY: Color = Color { r: 200, g: 200, b: 200, a: 255 };
const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };
// const GREEN: Color = Color { r: 0, g: 228, b: 48, a: 255 };

const PI: f32 = 3.14159265358979323846;
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const CENTER_X: i32 = SCREEN_WIDTH / 2;
const CENTER_Y: i32 = SCREEN_HEIGHT / 2;
const CLOCK_RADIUS: f32 = 200.0;

// ------------------------------------------------------------------
// Helper Functions
// ------------------------------------------------------------------

// Convert degrees to radians (clock starts at 12, so subtract 90°)
fn deg_to_rad(degrees: f32) -> f32 {
    (degrees - 90.0) * PI / 180.0
}

// Calculate hand endpoint
fn get_hand_endpoint(center_x: f32, center_y: f32, angle_deg: f32, length: f32) -> Vector2 {
    unsafe {
        let rad = deg_to_rad(angle_deg);
        Vector2 {
            x: center_x + cosf(rad) * length,
            y: center_y + sinf(rad) * length,
        }
    }
}

// Simple integer to string converter
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
    
    let _start_pos = pos;
    let mut temp = value;
    
    if temp == 0 {
        if pos >= buffer.len() - 1 {
            buffer[0] = 0;
            return 0;
        }
        buffer[pos] = b'0';
        pos += 1;
    } else {
        // let mut digit_start = pos;
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
    
    buffer[pos] = 0;
    pos
}

fn copy_str(buffer: &mut [u8], s: &[u8], offset: usize) -> usize {
    if offset >= buffer.len() - 1 {
        return offset;
    }
    
    let available = buffer.len() - offset - 1;
    let to_copy = core::cmp::min(s.len(), available);
    
    buffer[offset..offset + to_copy].copy_from_slice(&s[..to_copy]);
    offset + to_copy
}

// Add leading zero to single digits
fn write_time_component(buffer: &mut [u8], value: i32, offset: usize) -> usize {
    if value < 10 {
        buffer[offset] = b'0';
        write_i32(&mut buffer[offset + 1..], value) + offset + 1
    } else {
        write_i32(&mut buffer[offset..], value) + offset
    }
}

// ------------------------------------------------------------------
// Drawing Functions
// ------------------------------------------------------------------

unsafe fn draw_clock_face() {
    // Outer circle
    DrawCircleLines(CENTER_X, CENTER_Y, CLOCK_RADIUS, BLACK);
    DrawCircleLines(CENTER_X, CENTER_Y, CLOCK_RADIUS - 2.0, BLACK);
    
    // Center dot
    DrawCircle(CENTER_X, CENTER_Y, 8.0, BLACK);
    
    // Draw hour markers
    for i in 0..12 {
        let angle = (i as f32) * 30.0; // 360° / 12 = 30° per hour
        let rad = deg_to_rad(angle);
        
        let outer_x = CENTER_X as f32 + cosf(rad) * (CLOCK_RADIUS - 15.0);
        let outer_y = CENTER_Y as f32 + sinf(rad) * (CLOCK_RADIUS - 15.0);
        let inner_x = CENTER_X as f32 + cosf(rad) * (CLOCK_RADIUS - 25.0);
        let inner_y = CENTER_Y as f32 + sinf(rad) * (CLOCK_RADIUS - 25.0);
        
        DrawLineEx(
            Vector2 { x: outer_x, y: outer_y },
            Vector2 { x: inner_x, y: inner_y },
            3.0,
            BLACK
        );
    }
    
    // Draw minute markers (smaller)
    for i in 0..60 {
        if i % 5 == 0 {
            continue; // Skip hour markers
        }
        
        let angle = (i as f32) * 6.0; // 360° / 60 = 6° per minute
        let rad = deg_to_rad(angle);
        
        let outer_x = CENTER_X as f32 + cosf(rad) * (CLOCK_RADIUS - 10.0);
        let outer_y = CENTER_Y as f32 + sinf(rad) * (CLOCK_RADIUS - 10.0);
        let inner_x = CENTER_X as f32 + cosf(rad) * (CLOCK_RADIUS - 15.0);
        let inner_y = CENTER_Y as f32 + sinf(rad) * (CLOCK_RADIUS - 15.0);
        
        DrawLineEx(
            Vector2 { x: outer_x, y: outer_y },
            Vector2 { x: inner_x, y: inner_y },
            1.0,
            GRAY
        );
    }
}

unsafe fn draw_hands(hours: i32, minutes: i32, seconds: i32) {
    let center = Vector2 { 
        x: CENTER_X as f32, 
        y: CENTER_Y as f32 
    };
    
    // Second hand (thin, red)
    let second_angle = (seconds as f32) * 6.0; // 360° / 60 = 6° per second
    let second_end = get_hand_endpoint(
        center.x, 
        center.y, 
        second_angle, 
        CLOCK_RADIUS - 30.0
    );
    DrawLineEx(center, second_end, 2.0, RED);
    
    // Minute hand (medium, blue)
    let minute_angle = (minutes as f32) * 6.0 + (seconds as f32) * 0.1; // Smooth movement
    let minute_end = get_hand_endpoint(
        center.x, 
        center.y, 
        minute_angle, 
        CLOCK_RADIUS - 50.0
    );
    DrawLineEx(center, minute_end, 6.0, BLUE);
    
    // Hour hand (thick, dark)
    let hour_angle = (hours as f32) * 30.0 + (minutes as f32) * 0.5; // Smooth movement
    let hour_end = get_hand_endpoint(
        center.x, 
        center.y, 
        hour_angle, 
        CLOCK_RADIUS - 90.0
    );
    DrawLineEx(center, hour_end, 8.0, DARKGRAY);
}


unsafe fn draw_date_time_info(tm: *const Tm) {
    if tm.is_null() {
        return;
    }
    
    let time_data = *tm;
    let mut buffer = [0u8; 128];
    
    // Format: "YYYY-MM-DD HH:MM:SS"
    let mut pos = 0;
    
    // Year
    let year = time_data.tm_year + 1900;
    pos = write_i32(&mut buffer[pos..], year) + pos;
    pos = copy_str(&mut buffer, b"-", pos);
    
    // Month (0-11, so add 1)
    pos = write_time_component(&mut buffer, time_data.tm_mon + 1, pos);
    pos = copy_str(&mut buffer, b"-", pos);
    
    // Day
    pos = write_time_component(&mut buffer, time_data.tm_mday, pos);
    pos = copy_str(&mut buffer, b"  ", pos);
    
    // Hour
    pos = write_time_component(&mut buffer, time_data.tm_hour, pos);
    pos = copy_str(&mut buffer, b":", pos);
    
    // Minute
    pos = write_time_component(&mut buffer, time_data.tm_min, pos);
    pos = copy_str(&mut buffer, b":", pos);
    
    // Second
    pos = write_time_component(&mut buffer, time_data.tm_sec, pos);
    buffer[pos] = 0;
    
    DrawText(buffer.as_ptr(), CENTER_X-120, SCREEN_HEIGHT - 80, 24, DARKGRAY);
}

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(SCREEN_WIDTH, SCREEN_HEIGHT, b"Analog Clock - no_std Rust\0".as_ptr());
    SetTargetFPS(60);

    while !WindowShouldClose() {
        // Get current system time using localtime
        let timestamp = time(core::ptr::null_mut());
        let tm_ptr = localtime(&timestamp);
        
        if tm_ptr.is_null() {
            BeginDrawing();
            ClearBackground(RAYWHITE);
            DrawText(b"Error: Could not get system time\0".as_ptr(), 100, 100, 20, RED);
            EndDrawing();
            continue;
        }
        
        let tm = *tm_ptr;
        
        // Extract time for clock hands (12-hour format for analog clock)
        let hours = tm.tm_hour % 12;
        let minutes = tm.tm_min;
        let seconds = tm.tm_sec;

        BeginDrawing();
        ClearBackground(RAYWHITE);
        
        // Title
        DrawText(
            b"Analog Clock\0".as_ptr(), 
            CENTER_X - 100, 
            50, 
            32, 
            BLACK
        );
        
        // Draw clock
        draw_clock_face();
        draw_hands(hours, minutes, seconds);
        
        // Draw full date and time from system
        draw_date_time_info(tm_ptr);
        

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
pub extern "C" fn rust_eh_personality() {
    loop {}
}
