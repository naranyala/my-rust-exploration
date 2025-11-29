// raylib/mod.rs - Raylib FFI bindings and constants

#![allow(dead_code)]

// ============================
// Types
// ============================
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8, pub g: u8, pub b: u8, pub a: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

// ============================
// Color Constants
// ============================
pub const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };
pub const BLACK:    Color = Color { r:   0, g:   0, b:   0, a: 255 };
pub const RED:      Color = Color { r: 230, g:  41, b:  55, a: 255 };
pub const BLUE:     Color = Color { r:   0, g: 121, b: 241, a: 255 };
pub const GREEN:    Color = Color { r:   0, g: 228, b:  48, a: 255 };
pub const YELLOW:   Color = Color { r: 253, g: 249, b:   0, a: 255 };
pub const PURPLE:   Color = Color { r: 163, g:  73, b: 164, a: 255 };
pub const ORANGE:   Color = Color { r: 255, g: 161, b:   0, a: 255 };
pub const DARKGRAY: Color = Color { r:  80, g:  80, b:  80, a: 255 };
pub const LIGHTGRAY: Color = Color { r: 200, g: 200, b: 200, a: 255 };

// ============================
// Input Constants
// ============================
pub const MOUSE_LEFT: i32 = 0;
pub const MOUSE_RIGHT: i32 = 1;
pub const KEY_R: i32 = 82;
pub const KEY_SPACE: i32 = 32;
pub const KEY_C: i32 = 67;
pub const KEY_UP: i32 = 265;
pub const KEY_DOWN: i32 = 264;
pub const KEY_LEFT: i32 = 263;
pub const KEY_RIGHT: i32 = 262;

// ============================
// FFI Bindings
// ============================
extern "C" {
    pub fn InitWindow(width: i32, height: i32, title: *const i8);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn SetTargetFPS(fps: i32);

    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);

    pub fn DrawText(text: *const i8, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawCircle(posX: i32, posY: i32, radius: f32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    pub fn DrawRectangleLines(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    pub fn DrawLine(x1: i32, y1: i32, x2: i32, y2: i32, color: Color);
    pub fn DrawTriangle(v1: Vector2, v2: Vector2, v3: Vector2, color: Color);
    pub fn DrawPoly(center: Vector2, sides: i32, radius: f32, rotation: f32, color: Color);

    pub fn IsMouseButtonPressed(button: i32) -> bool;
    pub fn IsMouseButtonDown(button: i32) -> bool;
    pub fn IsMouseButtonReleased(button: i32) -> bool;
    pub fn GetMousePosition() -> Vector2;
    pub fn IsKeyPressed(key: i32) -> bool;
    pub fn GetFrameTime() -> f32;
}
