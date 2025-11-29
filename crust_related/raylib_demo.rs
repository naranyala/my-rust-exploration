// rustc --edition 2021 -C panic=abort raylib_demo.rs -o raylib_demo

#![no_std]
#![no_main]

// Link against raylib and required system libraries
#[link(name = "raylib")]
#[link(name = "m")]
#[link(name = "pthread")]
#[link(name = "dl")]
#[link(name = "rt")]
#[link(name = "GL")]
#[link(name = "c")]
extern "C" {
    // Core functions
    pub fn InitWindow(width: i32, height: i32, title: *const u8);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn SetTargetFPS(fps: i32);
    
    // Drawing functions
    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);
    pub fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawCircle(centerX: i32, centerY: i32, radius: f32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    
    // Timing
    pub fn GetFrameTime() -> f32;
}

// Color struct - must match raylib's Color layout
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// Common colors
pub const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };
pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
pub const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
pub const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };
pub const GREEN: Color = Color { r: 0, g: 228, b: 48, a: 255 };
pub const YELLOW: Color = Color { r: 253, g: 249, b: 0, a: 255 };

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Window setup
    let width = 800;
    let height = 600;
    let title = b"Crust Raylib Demo\0";
    
    InitWindow(width, height, title.as_ptr());
    SetTargetFPS(60);
    
    // Animation state
    let mut circle_x: f32 = 100.0;
    let mut circle_y: f32 = 300.0;
    let mut velocity_x: f32 = 200.0;
    let mut velocity_y: f32 = 150.0;
    let radius: f32 = 30.0;
    
    // Game loop
    while !WindowShouldClose() {
        let delta = GetFrameTime();
        
        // Update circle position
        circle_x += velocity_x * delta;
        circle_y += velocity_y * delta;
        
        // Bounce off walls
        if circle_x - radius <= 0.0 || circle_x + radius >= width as f32 {
            velocity_x = -velocity_x;
        }
        if circle_y - radius <= 0.0 || circle_y + radius >= height as f32 {
            velocity_y = -velocity_y;
        }
        
        // Draw
        BeginDrawing();
        ClearBackground(RAYWHITE);
        
        DrawText(b"Crust + Raylib!\0".as_ptr(), 20, 20, 40, BLACK);
        DrawText(b"Bouncing ball demo\0".as_ptr(), 20, 70, 20, BLACK);
        
        DrawCircle(circle_x as i32, circle_y as i32, radius, RED);
        DrawRectangle(10, height - 60, 100, 50, BLUE);
        DrawRectangle(width - 110, height - 60, 100, 50, GREEN);
        
        EndDrawing();
    }
    
    CloseWindow();
    0
}

#[panic_handler]
pub unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}


#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
