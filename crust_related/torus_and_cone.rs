// rustc --edition 2021 -C panic=abort torus_cone_demo.rs -o torus_cone_demo
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
const DARKBLUE: Color = Color { r: 0, g: 82, b: 172, a: 255 };

const CAMERA_ORBITAL: i32 = 2;
const CAMERA_PERSPECTIVE: i32 = 0;
const PI: f32 = 3.14159265359;

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
fn vec3_to_vector3(v: Vec3) -> Vector3 {
    Vector3 { x: v.x, y: v.y, z: v.z }
}

// ------------------------------------------------------------------
// Mesh Generation
// ------------------------------------------------------------------

// Generate torus vertices
fn generate_torus(major_radius: f32, minor_radius: f32, major_segments: usize, minor_segments: usize) -> ([Vec3; 1024], usize) {
    let mut vertices = [vec3_zero(); 1024];
    let mut count = 0;
    
    for i in 0..major_segments {
        for j in 0..minor_segments {
            let u0 = (i as f32 / major_segments as f32) * 2.0 * PI;
            let u1 = ((i + 1) as f32 / major_segments as f32) * 2.0 * PI;
            let v0 = (j as f32 / minor_segments as f32) * 2.0 * PI;
            let v1 = ((j + 1) as f32 / minor_segments as f32) * 2.0 * PI;
            
            // Calculate 4 vertices of the quad
            let p0 = torus_point(major_radius, minor_radius, u0, v0);
            let p1 = torus_point(major_radius, minor_radius, u1, v0);
            let p2 = torus_point(major_radius, minor_radius, u1, v1);
            let p3 = torus_point(major_radius, minor_radius, u0, v1);
            
            // First triangle
            if count < 1021 {
                vertices[count] = p0; count += 1;
                vertices[count] = p1; count += 1;
                vertices[count] = p2; count += 1;
            }
            
            // Second triangle
            if count < 1021 {
                vertices[count] = p0; count += 1;
                vertices[count] = p2; count += 1;
                vertices[count] = p3; count += 1;
            }
        }
    }
    
    (vertices, count)
}

fn torus_point(major_radius: f32, minor_radius: f32, u: f32, v: f32) -> Vec3 {
    let cos_u = cos_f32(u);
    let sin_u = sin_f32(u);
    let cos_v = cos_f32(v);
    let sin_v = sin_f32(v);
    
    vec3(
        (major_radius + minor_radius * cos_v) * cos_u,
        minor_radius * sin_v,
        (major_radius + minor_radius * cos_v) * sin_u
    )
}

// Generate cone vertices
fn generate_cone(base_radius: f32, height: f32, segments: usize) -> ([Vec3; 512], usize) {
    let mut vertices = [vec3_zero(); 512];
    let mut count = 0;
    
    let apex = vec3(0.0, height, 0.0);
    let base_center = vec3(0.0, 0.0, 0.0);
    
    for i in 0..segments {
        let angle0 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle1 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;
        
        let p0 = vec3(base_radius * cos_f32(angle0), 0.0, base_radius * sin_f32(angle0));
        let p1 = vec3(base_radius * cos_f32(angle1), 0.0, base_radius * sin_f32(angle1));
        
        // Side triangle
        if count < 509 {
            vertices[count] = apex; count += 1;
            vertices[count] = p0; count += 1;
            vertices[count] = p1; count += 1;
        }
        
        // Base triangle
        if count < 509 {
            vertices[count] = base_center; count += 1;
            vertices[count] = p1; count += 1;
            vertices[count] = p0; count += 1;
        }
    }
    
    (vertices, count)
}

// Helper functions from linalg module
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
// Shape Definitions
// ------------------------------------------------------------------
struct Shape {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    color: Color,
    wireframe_color: Color,
    vertices: [Vec3; 1024],
    vertex_count: usize,
}

impl Shape {
    fn new_torus(position: Vec3, rotation: Quat, scale: Vec3, color: Color, wireframe_color: Color) -> Self {
        let (vertices, count) = generate_torus(1.5, 0.5, 20, 16);
        Self {
            position,
            rotation,
            scale,
            color,
            wireframe_color,
            vertices,
            vertex_count: count,
        }
    }
    
    fn new_cone(position: Vec3, rotation: Quat, scale: Vec3, color: Color, wireframe_color: Color) -> Self {
        let (cone_verts, count) = generate_cone(1.0, 2.0, 20);
        let mut vertices = [vec3_zero(); 1024];
        for i in 0..count {
            vertices[i] = cone_verts[i];
        }
        Self {
            position,
            rotation,
            scale,
            color,
            wireframe_color,
            vertices,
            vertex_count: count,
        }
    }

    fn draw(&self) {
        // Create transformation matrix
        let rot_mat = quat_to_mat4(self.rotation);
        let scale_mat = mat4_scale(self.scale);
        let trans_mat = mat4_translate(self.position);
        let transform = mat4_mul(trans_mat, mat4_mul(rot_mat, scale_mat));
        
        // Draw triangles (filled)
        for i in (0..self.vertex_count).step_by(3) {
            if i + 2 < self.vertex_count {
                let v0 = mat4_mul_vec3(transform, self.vertices[i]);
                let v1 = mat4_mul_vec3(transform, self.vertices[i + 1]);
                let v2 = mat4_mul_vec3(transform, self.vertices[i + 2]);
                
                unsafe {
                    DrawLine3D(vec3_to_vector3(v0), vec3_to_vector3(v1), self.color);
                    DrawLine3D(vec3_to_vector3(v1), vec3_to_vector3(v2), self.color);
                    DrawLine3D(vec3_to_vector3(v2), vec3_to_vector3(v0), self.color);
                }
            }
        }
        
        // Draw wireframe
        for i in (0..self.vertex_count).step_by(3) {
            if i + 2 < self.vertex_count {
                let v0 = mat4_mul_vec3(transform, self.vertices[i]);
                let v1 = mat4_mul_vec3(transform, self.vertices[i + 1]);
                let v2 = mat4_mul_vec3(transform, self.vertices[i + 2]);
                
                unsafe {
                    DrawLine3D(vec3_to_vector3(v0), vec3_to_vector3(v1), self.wireframe_color);
                    DrawLine3D(vec3_to_vector3(v1), vec3_to_vector3(v2), self.wireframe_color);
                    DrawLine3D(vec3_to_vector3(v2), vec3_to_vector3(v0), self.wireframe_color);
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
    InitWindow(1200, 900, b"Handmade Torus & Cone Demo\0".as_ptr());
    SetTargetFPS(60);

    let mut camera = Camera3D {
        position: Vector3 { x: 15.0, y: 10.0, z: 15.0 },
        target: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        up: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
        fovy: 45.0,
        projection: CAMERA_PERSPECTIVE,
    };


    // Initialize shapes
    let mut shapes = [
        // Central torus
        Shape::new_torus(vec3(0.0, 2.0, 0.0), quat_identity(), vec3(1.0, 1.0, 1.0), PURPLE, DARKBLUE),
        
        // Surrounding cones
        Shape::new_cone(vec3(6.0, 0.0, 0.0), quat_identity(), vec3(0.8, 1.0, 0.8), RED, DARKBLUE),
        Shape::new_cone(vec3(-6.0, 0.0, 0.0), quat_identity(), vec3(0.8, 1.0, 0.8), BLUE, DARKBLUE),
        Shape::new_cone(vec3(0.0, 0.0, 6.0), quat_identity(), vec3(0.8, 1.0, 0.8), GREEN, DARKBLUE),
        
        // Corner tori
        Shape::new_torus(vec3(5.0, 1.5, 5.0), quat_identity(), vec3(0.6, 0.6, 0.6), ORANGE, DARKBLUE),
        Shape::new_torus(vec3(-5.0, 1.5, -5.0), quat_identity(), vec3(0.6, 0.6, 0.6), YELLOW, DARKBLUE),
    ];

    while !WindowShouldClose() {
        UpdateCamera(&mut camera, CAMERA_ORBITAL);

        let time = GetTime() as f32;

        // Update shape rotations
        for (i, shape) in shapes.iter_mut().enumerate() {
            let speed = 0.4 + (i as f32 * 0.1);
            let axis = match i % 3 {
                0 => vec3(1.0, 1.0, 0.0),
                1 => vec3(0.0, 1.0, 1.0),
                _ => vec3(1.0, 0.0, 1.0),
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
