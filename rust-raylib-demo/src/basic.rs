use raylib_ffi::*;
use raylib_ffi::colors::{ BLACK, RAYWHITE };

fn main() {
    unsafe {
        // Initialize window
        InitWindow(800, 600, b"Raylib FFI Demo\0".as_ptr() as *const i8);

        // Set target FPS
        SetTargetFPS(60);

        // Main loop
        while !WindowShouldClose() {
            BeginDrawing();
            ClearBackground(RAYWHITE);
            DrawText(b"Hello from raylib-ffi!\0".as_ptr() as *const i8, 190, 200, 20, BLACK);
            EndDrawing();
        }

        // Close window
        CloseWindow();
    }
}

