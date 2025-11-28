// rustc --edition 2021 -C panic=abort cube_demo.rs -o cube_demo
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
    fn BeginMode3D(camera: Camera3D);
    fn EndMode3D();
    fn DrawCube(position: Vector3, width: f32, height: f32, length: f32, color: Color);
    fn DrawCubeWires(position: Vector3, width: f32, height: f32, length: f32, color: Color);
    fn DrawGrid(slices: i32, spacing: f32);
    fn UpdateCamera(camera: *mut Camera3D, mode: i32);
}

// ------------------------------------------------------------------
// Types
// ------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector3 { 
    pub x: f32, 
    pub y: f32, 
    pub z: f32 
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color { 
    pub r: u8, 
    pub g: u8, 
    pub b: u8, 
    pub a: u8 
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Camera3D {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub fovy: f32,
    pub projection: i32,
}

// ------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------
const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };
const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
const MAROON: Color = Color { r: 190, g: 33, b: 55, a: 255 };
// const DARKGRAY: Color = Color { r: 80, g: 80, b: 80, a: 255 };

// Camera modes
// const CAMERA_FREE: i32 = 1;
const CAMERA_ORBITAL: i32 = 2;

// Camera projection
const CAMERA_PERSPECTIVE: i32 = 0;

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(800, 600, b"3D Cube Demo - Use Mouse to Rotate\0".as_ptr());
    SetTargetFPS(60);

    let mut camera = Camera3D {
        position: Vector3 { x: 10.0, y: 10.0, z: 10.0 },
        target: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        up: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
        fovy: 45.0,
        projection: CAMERA_PERSPECTIVE,
    };

    let cube_pos = Vector3 { x: 0.0, y: 1.0, z: 0.0 };

    while !WindowShouldClose() {
        // Update camera with orbital mode (mouse drag to rotate)
        UpdateCamera(&mut camera, CAMERA_ORBITAL);

        BeginDrawing();
        ClearBackground(RAYWHITE);

        BeginMode3D(camera);
        
        // Draw the cube (solid)
        DrawCube(cube_pos, 2.0, 2.0, 2.0, RED);
        
        // Draw cube wireframe on top
        DrawCubeWires(cube_pos, 2.0, 2.0, 2.0, MAROON);
        
        // Draw ground grid
        DrawGrid(10, 1.0);
        
        EndMode3D();

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
