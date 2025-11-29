// earth_wireframe.rs
// Compile:
// rustc earth_wireframe.rs -C link-arg=-lraylib -C link-arg=-lGL -C link-arg=-lm \
//   -C link-arg=-lpthread -C link-arg=-ldl -C link-arg=-lrt -o earth_rust

mod linalg;
use linalg::*;

// ===================================================================
// Raylib FFI
// ===================================================================
#[link(name = "raylib")]
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
    fn GetMousePosition() -> Vector2;
    fn IsMouseButtonPressed(button: i32) -> bool;
    fn IsMouseButtonReleased(button: i32) -> bool;
    fn GetMouseWheelMove() -> f32;
    fn DrawText(text: *const u8, x: i32, y: i32, font_size: i32, color: Color);
    fn DrawFPS(x: i32, y: i32);
}

// ===================================================================
// Raylib Types
// ===================================================================
#[repr(C)] #[derive(Copy, Clone)] pub struct Vector2 { pub x: f32, pub y: f32 }
#[repr(C)] #[derive(Copy, Clone)] pub struct Vector3 { pub x: f32, pub y: f32, pub z: f32 }
#[repr(C)] #[derive(Copy, Clone)] pub struct Color   { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }
#[repr(C)] #[derive(Copy, Clone)] pub struct Camera3D {
    pub position: Vector3,
    pub target:   Vector3,
    pub up:       Vector3,
    pub fovy:     f32,
    pub projection: i32,
}

// ===================================================================
// Colors
// ===================================================================
const BLACK:    Color = Color { r:   0, g:   0, b:   0, a: 255 };
const WHITE:    Color = Color { r: 255, g: 255, b: 255, a: 255 };
const RED:      Color = Color { r: 230, g:  41, b:  55, a: 255 };
const GREEN:    Color = Color { r:   0, g: 228, b:  48, a: 255 };
const YELLOW:   Color = Color { r: 255, g: 249, b:   0, a: 255 };
const ORANGE:   Color = Color { r: 255, g: 161, b:   0, a: 255 };
const SKYBLUE:  Color = Color { r: 100, g: 180, b: 255, a: 255 };
const DARKGRAY: Color = Color { r:  80, g:  80, b:  80, a: 255 };

// ===================================================================
// Constants
// ===================================================================
const RADIUS: f32 = 2.0;
const SEG_LAT: i32 = 36;
const SEG_LON: i32 = 72;

// ===================================================================
// Helpers
// ===================================================================
#[inline]
fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

#[inline]
fn to_vec3(v: Vector3) -> Vec3 {
    vec3(v.x, v.y, v.z)
}

#[inline]
fn v3(v: Vec3) -> Vector3 {
    Vector3 { x: v.x, y: v.y, z: v.z }
}

// ===================================================================
// Main
// ===================================================================
fn main() {
    unsafe {
        InitWindow(1200, 800, b"Rust + linalg.rs - Wireframe Earth\0".as_ptr());
        SetTargetFPS(60);

        let mut camera = Camera3D {
            position: Vector3 { x: 0.0, y: 0.0, z: 10.0 },
            target:   Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            up:       Vector3 { x: 0.0, y: 1.0, z: 0.0 },
            fovy: 45.0,
            projection: 0,
        };

        let mut earth_quat = quat_identity();
        let mut cam_dist = 10.0f32;
        let mut dragging = false;
        let mut last_mouse = Vector2 { x: 0.0, y: 0.0 };
        
        // Control sensitivity settings
        let _rotation_speed = 0.005;  // Lower = slower rotation
        let zoom_speed = 1.5;         // Higher = faster zoom

        while !WindowShouldClose() {
            // Mouse orbit
            if IsMouseButtonPressed(0) {
                last_mouse = GetMousePosition();
                dragging = true;
            }
            if IsMouseButtonReleased(0) {
                dragging = false;
            }

            if dragging {
                let mouse = GetMousePosition();
                let dx = mouse.x - last_mouse.x;
                let dy = mouse.y - last_mouse.y;
                last_mouse = mouse;

                if dx != 0.0 || dy != 0.0 {
                    let sensitivity = 0.005;

                    // Horizontal orbit (around Y axis)
                    let yaw_quat = quat_angle_axis(-dx * sensitivity, vec3(0.0, 1.0, 0.0));

                    // Vertical orbit (around camera's right axis)
                    let camera_right = vec3_normalize(vec3_cross(
                        vec3_sub(to_vec3(camera.position), to_vec3(camera.target)),
                        vec3(0.0, 1.0, 0.0),
                    ));
                    let pitch_quat = quat_angle_axis(-dy * sensitivity, camera_right);

                    // Combine rotations
                    earth_quat = quat_mul(pitch_quat, quat_mul(yaw_quat, earth_quat));
                }
            }


            // Zoom - FIXED: only handle once
            let wheel = GetMouseWheelMove();
            if wheel != 0.0 {
                cam_dist -= wheel * zoom_speed;
                cam_dist = clamp(cam_dist, 3.0, 30.0);
            }

            // Update camera position based on Earth rotation - FIXED: only set once
            let model = quat_to_mat4(earth_quat);
            let cam_pos_local = vec3(0.0, 0.0, cam_dist);
            let cam_pos_world = mat4_mul_vec3(model, cam_pos_local);
            camera.position = v3(cam_pos_world);
            camera.target = v3(mat4_mul_vec3(model, vec3_zero()));

            BeginDrawing();
            ClearBackground(BLACK);
            BeginMode3D(camera);

            // Longitude lines
            for i in 0..SEG_LON {
                let lon = 2.0 * std::f32::consts::PI * i as f32 / SEG_LON as f32;
                let col = if i % (SEG_LON / 12) == 0 { YELLOW } else { DARKGRAY };

                let mut prev = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut first = true;

                for j in 0..=SEG_LAT {
                    let phi = std::f32::consts::PI * j as f32 / SEG_LAT as f32;
                    let local = vec3(
                        RADIUS * sin_f32(phi) * cos_f32(lon),
                        RADIUS * cos_f32(phi),
                        RADIUS * sin_f32(phi) * sin_f32(lon),
                    );
                    let world = mat4_mul_vec3(model, local);
                    let pos = v3(world);

                    if !first { DrawLine3D(prev, pos, col); }
                    prev = pos;
                    first = false;
                }
            }

            // Latitude lines
            for j in 1..SEG_LAT {
                let phi = std::f32::consts::PI * j as f32 / SEG_LAT as f32;
                let y = RADIUS * cos_f32(phi);
                let r = RADIUS * sin_f32(phi);
                let col = if j == SEG_LAT / 2 { GREEN }
                          else if j == SEG_LAT / 4 || j == 3 * SEG_LAT / 4 { ORANGE }
                          else { SKYBLUE };

                let mut prev = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut first = true;

                for i in 0..=SEG_LON {
                    let theta = 2.0 * std::f32::consts::PI * i as f32 / SEG_LON as f32;
                    let local = vec3(r * cos_f32(theta), y, r * sin_f32(theta));
                    let world = mat4_mul_vec3(model, local);
                    let pos = v3(world);

                    if !first { DrawLine3D(prev, pos, col); }
                    prev = pos;
                    first = false;
                }
            }

            // Equator (red, extra smooth)
            {
                let mut prev = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut first = true;
                for i in 0..=128 {
                    let a = 2.0 * std::f32::consts::PI * i as f32 / 128.0;
                    let local = vec3(RADIUS * cos_f32(a), 0.0, RADIUS * sin_f32(a));
                    let world = mat4_mul_vec3(model, local);
                    let pos = v3(world);
                    if !first { DrawLine3D(prev, pos, RED); }
                    prev = pos;
                    first = false;
                }
            }

            EndMode3D();

            DrawText(b"Left Drag = Rotate | Wheel = Zoom\0".as_ptr(), 10, 10, 20, WHITE);
            DrawText(b"100% linalg.rs + Rust\0".as_ptr(), 10, 40, 20, YELLOW);
            DrawFPS(10, 70);

            EndDrawing();
        }

        CloseWindow();
    }
}
