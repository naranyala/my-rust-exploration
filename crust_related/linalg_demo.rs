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
const PURPLE: Color = Color { r: 200, g: 122, b: 255, a: 255 };
const ORANGE: Color = Color { r: 255, g: 161, b: 0, a: 255 };

const CAMERA_ORBITAL: i32 = 2;
const CAMERA_PERSPECTIVE: i32 = 0;

const NUM_PARTICLES: usize = 50;
const SPIRAL_RADIUS: f32 = 5.0;
const SPIRAL_HEIGHT: f32 = 8.0;

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
fn vec3_to_vector3(v: Vec3) -> Vector3 {
    Vector3 { x: v.x, y: v.y, z: v.z }
}

extern "C" {
    fn sinf(x: f32) -> f32;
    fn cosf(x: f32) -> f32;
}

fn sin_f32(x: f32) -> f32 {
    unsafe { sinf(x) }
}

fn cos_f32(x: f32) -> f32 {
    unsafe { cosf(x) }
}

// ------------------------------------------------------------------
// Particle System
// ------------------------------------------------------------------
struct Particle {
    base_pos: Vec3,
    color: Color,
}

impl Particle {
    fn new(index: usize, total: usize) -> Self {
        let t = index as f32 / total as f32;
        let angle = t * 6.28318530718 * 3.0; // 3 full rotations
        let height = t * SPIRAL_HEIGHT - SPIRAL_HEIGHT * 0.5;
        
        let base_pos = vec3(
            cos_f32(angle) * SPIRAL_RADIUS,
            height,
            sin_f32(angle) * SPIRAL_RADIUS,
        );

        let colors = [RED, BLUE, GREEN, YELLOW, PURPLE, ORANGE];
        let color = colors[index % colors.len()];

        Self { base_pos, color }
    }

    fn get_position(&self, time: f32, rot_mat: Mat4) -> Vec3 {
        // Apply rotation matrix transformation
        let rotated = mat4_mul_vec3(rot_mat, self.base_pos);
        
        // Add wave motion
        let wave = vec3(
            0.0,
            sin_f32(time * 2.0 + self.base_pos.y) * 0.3,
            0.0
        );
        
        vec3_add(rotated, wave)
    }
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(1000, 800, b"Linear Algebra Demo - Rotating Spiral\0".as_ptr());
    SetTargetFPS(60);

    let mut camera = Camera3D {
        position: Vector3 { x: 15.0, y: 12.0, z: 15.0 },
        target: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        up: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
        fovy: 45.0,
        projection: CAMERA_PERSPECTIVE,
    };

    // Initialize particles - FIXED VERSION
    let particles: [Particle; NUM_PARTICLES] = core::array::from_fn(|i| {
        Particle::new(i, NUM_PARTICLES)
    });

    while !WindowShouldClose() {
        UpdateCamera(&mut camera, CAMERA_ORBITAL);

        let time = GetTime() as f32;

        // Create combined rotation matrix using quaternions
        let rot_y = quat_angle_axis(time * 0.5, vec3(0.0, 1.0, 0.0));
        let rot_x = quat_angle_axis(sin_f32(time * 0.3) * 0.5, vec3(1.0, 0.0, 0.0));
        let rot_combined = quat_mul(rot_y, rot_x);
        let rot_mat = quat_to_mat4(rot_combined);

        BeginDrawing();
        ClearBackground(RAYWHITE);

        BeginMode3D(camera);
        
        // Draw particles and connections
        for i in 0..NUM_PARTICLES {
            let pos = particles[i].get_position(time, rot_mat);
            DrawSphere(vec3_to_vector3(pos), 0.2, particles[i].color);

            // Connect to next particle
            if i < NUM_PARTICLES - 1 {
                let next_pos = particles[i + 1].get_position(time, rot_mat);
                DrawLine3D(
                    vec3_to_vector3(pos),
                    vec3_to_vector3(next_pos),
                    particles[i].color
                );
            }
        }

        // Draw axis lines showing rotation
        let axis_len = 3.0;
        let x_axis = mat4_mul_vec3(rot_mat, vec3(axis_len, 0.0, 0.0));
        let y_axis = mat4_mul_vec3(rot_mat, vec3(0.0, axis_len, 0.0));
        let z_axis = mat4_mul_vec3(rot_mat, vec3(0.0, 0.0, axis_len));
        
        DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, vec3_to_vector3(x_axis), RED);
        DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, vec3_to_vector3(y_axis), GREEN);
        DrawLine3D(Vector3{x:0.0,y:0.0,z:0.0}, vec3_to_vector3(z_axis), BLUE);
        
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
