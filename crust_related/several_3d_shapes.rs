// rustc --edition 2021 -C panic=abort linalg_demo.rs -o linalg_demo
// (raylib must be installed on your system)

#![no_std]
#![no_main]

mod linalg;
use linalg::*;

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
    fn DrawSphere(center: Vector3, radius: f32, color: Color);
    fn DrawCube(position: Vector3, width: f32, height: f32, length: f32, color: Color);
    fn DrawCubeWires(position: Vector3, width: f32, height: f32, length: f32, color: Color);
    fn DrawCylinder(position: Vector3, radiusTop: f32, radiusBottom: f32, height: f32, slices: i32, color: Color);
    fn DrawCylinderWires(position: Vector3, radiusTop: f32, radiusBottom: f32, height: f32, slices: i32, color: Color);
    fn DrawLine3D(start: Vector3, end: Vector3, color: Color);
    fn DrawGrid(slices: i32, spacing: f32);
    fn UpdateCamera(camera: *mut Camera3D, mode: i32);
    fn GetTime() -> f64;
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
const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };
const GREEN: Color = Color { r: 0, g: 228, b: 48, a: 255 };
const YELLOW: Color = Color { r: 253, g: 249, b: 0, a: 255 };
// const PURPLE: Color = Color { r: 200, g: 122, b: 255, a: 255 };
// const ORANGE: Color = Color { r: 255, g: 161, b: 0, a: 255 };
const MAROON: Color = Color { r: 190, g: 33, b: 55, a: 255 };
const DARKGREEN: Color = Color { r: 0, g: 117, b: 44, a: 255 };
const DARKBLUE: Color = Color { r: 0, g: 82, b: 172, a: 255 };
// const GOLD: Color = Color { r: 255, g: 203, b: 0, a: 255 };

const CAMERA_ORBITAL: i32 = 2;
const CAMERA_PERSPECTIVE: i32 = 0;

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
fn vec3_to_vector3(v: Vec3) -> Vector3 {
    Vector3 { x: v.x, y: v.y, z: v.z }
}

// ------------------------------------------------------------------
// Shape Definitions
// ------------------------------------------------------------------
struct Shape {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    color: Color,
    shape_type: ShapeType,
}

enum ShapeType {
    Cube,
    Sphere,
    Cylinder,
}

impl Shape {
    fn new(position: Vec3, rotation: Quat, scale: Vec3, color: Color, shape_type: ShapeType) -> Self {
        Self {
            position,
            rotation,
            scale,
            color,
            shape_type,
        }
    }

    fn draw(&self) {
        let pos = vec3_to_vector3(self.position);
        
        match self.shape_type {
            ShapeType::Cube => {
                unsafe {
                    DrawCube(pos, self.scale.x, self.scale.y, self.scale.z, self.color);
                    DrawCubeWires(pos, self.scale.x, self.scale.y, self.scale.z, DARKBLUE);
                }
            }
            ShapeType::Sphere => {
                unsafe {
                    DrawSphere(pos, self.scale.x, self.color);
                }
            }
            ShapeType::Cylinder => {
                unsafe {
                    DrawCylinder(pos, self.scale.x, self.scale.x, self.scale.y, 16, self.color);
                    DrawCylinderWires(pos, self.scale.x, self.scale.x, self.scale.y, 16, DARKBLUE);
                }
            }
        }
    }
}

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(1200, 900, b"3D Shapes Demo\0".as_ptr());
    SetTargetFPS(60);

    let mut camera = Camera3D {
        position: Vector3 { x: 15.0, y: 10.0, z: 15.0 },
        target: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        up: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
        fovy: 45.0,
        projection: CAMERA_PERSPECTIVE,
    };

    // Initialize shapes with better scales for visibility
    let mut shapes = [
        // Central sphere
        Shape::new(vec3(0.0, 1.0, 0.0), quat_identity(), vec3(1.5, 1.5, 1.5), RED, ShapeType::Sphere),
        
        // Surrounding shapes with better scales
        Shape::new(vec3(6.0, 1.0, 0.0), quat_identity(), vec3(1.5, 1.5, 1.5), BLUE, ShapeType::Cube),
        Shape::new(vec3(-6.0, 1.0, 0.0), quat_identity(), vec3(1.2, 1.2, 1.2), GREEN, ShapeType::Sphere),
        Shape::new(vec3(0.0, 1.0, 6.0), quat_identity(), vec3(1.0, 2.5, 1.0), YELLOW, ShapeType::Cylinder),
        Shape::new(vec3(-5.0, 1.0, -5.0), quat_identity(), vec3(1.2, 1.2, 1.2), MAROON, ShapeType::Cube),
        Shape::new(vec3(5.0, 1.0, -5.0), quat_identity(), vec3(0.8, 2.0, 0.8), DARKGREEN, ShapeType::Cylinder),
    ];

    while !WindowShouldClose() {
        UpdateCamera(&mut camera, CAMERA_ORBITAL);

        let time = GetTime() as f32;

        // Update shape rotations
        for (i, shape) in shapes.iter_mut().enumerate() {
            let speed = 0.5 + (i as f32 * 0.1);
            let axis = match i % 3 {
                0 => vec3(1.0, 0.0, 0.0),
                1 => vec3(0.0, 1.0, 0.0),
                _ => vec3(0.0, 0.0, 1.0),
            };
            shape.rotation = quat_angle_axis(time * speed, axis);
        }

        BeginDrawing();
        ClearBackground(RAYWHITE);

        BeginMode3D(camera);
        
        // Draw all shapes
        for shape in &shapes {
            shape.draw();
        }

        // Draw coordinate axes
        let axis_len = 5.0;
        unsafe {
            DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, Vector3{x:axis_len,y:0.0,z:0.0}, RED);
            DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, Vector3{x:0.0,y:axis_len,z:0.0}, GREEN);
            DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, Vector3{x:0.0,y:0.0,z:axis_len}, BLUE);
        }
        
        DrawGrid(20, 1.0);
        
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
