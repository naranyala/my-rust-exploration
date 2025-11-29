// linalg.rs   (or mod.rs – same file)
// #![no_std]
#![allow(dead_code)]

pub const EPSILON: f32 = 1e-6;

// ←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←
// IMPORTANT: make them public so earth_wireframe.rs can use them!
extern "C" {
    fn sinf(x: f32) -> f32;
    fn cosf(x: f32) -> f32;
    fn sqrtf(x: f32) -> f32;
    fn fabsf(x: f32) -> f32;
    fn floorf(x: f32) -> f32;
    fn tanf(x: f32) -> f32;
}


#[inline] pub fn sin_f32(x: f32) -> f32 { unsafe { sinf(x) } }   // ← added pub
#[inline] pub fn cos_f32(x: f32) -> f32 { unsafe { cosf(x) } }   // ← added pub
#[inline] pub fn sqrt_f32(x: f32) -> f32 { unsafe { sqrtf(x) } }
#[inline] pub fn fabs_f32(x: f32) -> f32 { unsafe { fabsf(x) } }
#[inline] pub fn floor_f32(x: f32) -> f32 { unsafe { floorf(x) } }
#[inline] pub fn tan_f32(x: f32) -> f32 { unsafe { tanf(x) } }
// ←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←

#[inline]
pub fn fmin_f32(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

#[inline]
pub fn fmax_f32(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

// ============================================================================
// Vec3
// ============================================================================

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[inline]
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

#[inline]
pub const fn vec3_zero() -> Vec3 {
    Vec3 { x: 0.0, y: 0.0, z: 0.0 }
}

#[inline]
pub const fn vec3_one() -> Vec3 {
    Vec3 { x: 1.0, y: 1.0, z: 1.0 }
}

#[inline]
pub const fn vec3_add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

#[inline]
pub const fn vec3_sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

#[inline]
pub const fn vec3_mul(a: Vec3, s: f32) -> Vec3 {
    Vec3 { x: a.x * s, y: a.y * s, z: a.z * s }
}

#[inline]
pub fn vec3_div(a: Vec3, s: f32) -> Vec3 {
    let inv = 1.0 / s;
    Vec3 { x: a.x * inv, y: a.y * inv, z: a.z * inv }
}

#[inline]
pub const fn vec3_neg(a: Vec3) -> Vec3 {
    Vec3 { x: -a.x, y: -a.y, z: -a.z }
}

#[inline]
pub fn vec3_abs(a: Vec3) -> Vec3 {
    Vec3 { 
        x: fabs_f32(a.x), 
        y: fabs_f32(a.y), 
        z: fabs_f32(a.z) 
    }
}

#[inline]
pub fn vec3_floor(a: Vec3) -> Vec3 {
    Vec3 {
        x: floor_f32(a.x),
        y: floor_f32(a.y),
        z: floor_f32(a.z),
    }
}

#[inline]
pub fn vec3_frac(a: Vec3) -> Vec3 {
    vec3_sub(a, vec3_floor(a))
}

#[inline]
pub const fn vec3_mul_vec(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x * b.x, y: a.y * b.y, z: a.z * b.z }
}

#[inline]
pub fn vec3_min(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: fmin_f32(a.x, b.x),
        y: fmin_f32(a.y, b.y),
        z: fmin_f32(a.z, b.z),
    }
}

#[inline]
pub fn vec3_max(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: fmax_f32(a.x, b.x),
        y: fmax_f32(a.y, b.y),
        z: fmax_f32(a.z, b.z),
    }
}

#[inline]
pub const fn vec3_dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline]
pub const fn vec3_cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

#[inline]
pub const fn vec3_len_sq(a: Vec3) -> f32 {
    vec3_dot(a, a)
}

#[inline]
pub fn vec3_len(a: Vec3) -> f32 {
    sqrt_f32(vec3_len_sq(a))
}

#[inline]
pub fn vec3_dist_sq(a: Vec3, b: Vec3) -> f32 {
    vec3_len_sq(vec3_sub(a, b))
}

#[inline]
pub fn vec3_dist(a: Vec3, b: Vec3) -> f32 {
    vec3_len(vec3_sub(a, b))
}

#[inline]
pub fn vec3_normalize(a: Vec3) -> Vec3 {
    let len = vec3_len(a);
    if len > EPSILON {
        vec3_div(a, len)
    } else {
        vec3_zero()
    }
}

#[inline]
pub fn vec3_lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    vec3_add(vec3_mul(a, 1.0 - t), vec3_mul(b, t))
}

#[inline]
pub fn vec3_reflect(v: Vec3, n: Vec3) -> Vec3 {
    vec3_sub(v, vec3_mul(n, 2.0 * vec3_dot(v, n)))
}

#[inline]
pub fn vec3_project(a: Vec3, b: Vec3) -> Vec3 {
    let dot = vec3_dot(a, b);
    let len_sq = vec3_len_sq(b);
    if len_sq > EPSILON {
        vec3_mul(b, dot / len_sq)
    } else {
        vec3_zero()
    }
}

#[inline]
pub fn vec3_equal(a: Vec3, b: Vec3, eps: f32) -> bool {
    fabs_f32(a.x - b.x) < eps &&
    fabs_f32(a.y - b.y) < eps &&
    fabs_f32(a.z - b.z) < eps
}

// ============================================================================
// Vec4
// ============================================================================

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 { x, y, z, w }
}

#[inline]
pub const fn vec4_from_vec3(v: Vec3, w: f32) -> Vec4 {
    Vec4 { x: v.x, y: v.y, z: v.z, w }
}

#[inline]
pub const fn vec4_to_vec3(v: Vec4) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

#[inline]
pub const fn vec4_add(a: Vec4, b: Vec4) -> Vec4 {
    Vec4 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
        w: a.w + b.w,
    }
}

#[inline]
pub const fn vec4_mul(a: Vec4, s: f32) -> Vec4 {
    Vec4 {
        x: a.x * s,
        y: a.y * s,
        z: a.z * s,
        w: a.w * s,
    }
}

#[inline]
pub const fn vec4_dot(a: Vec4, b: Vec4) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

// ============================================================================
// Mat4 (Column-major 4x4 matrix)
// ============================================================================

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Mat4 {
    pub m: [f32; 16],  // Column-major: m[col*4 + row]
}

#[inline]
pub const fn mat4_identity() -> Mat4 {
    Mat4 {
        m: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[inline]
pub fn mat4_mul(a: Mat4, b: Mat4) -> Mat4 {
    let mut r = Mat4 { m: [0.0; 16] };
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for i in 0..4 {
                sum += a.m[i * 4 + row] * b.m[col * 4 + i];
            }
            r.m[col * 4 + row] = sum;
        }
    }
    r
}

#[inline]
pub fn mat4_mul_vec4(m: Mat4, v: Vec4) -> Vec4 {
    Vec4 {
        x: m.m[0] * v.x + m.m[4] * v.y + m.m[8]  * v.z + m.m[12] * v.w,
        y: m.m[1] * v.x + m.m[5] * v.y + m.m[9]  * v.z + m.m[13] * v.w,
        z: m.m[2] * v.x + m.m[6] * v.y + m.m[10] * v.z + m.m[14] * v.w,
        w: m.m[3] * v.x + m.m[7] * v.y + m.m[11] * v.z + m.m[15] * v.w,
    }
}

#[inline]
pub fn mat4_mul_vec3(m: Mat4, v: Vec3) -> Vec3 {
    let v4 = vec4_from_vec3(v, 1.0);
    let r = mat4_mul_vec4(m, v4);
    vec4_to_vec3(r)
}

#[inline]
pub const fn mat4_translate(v: Vec3) -> Mat4 {
    let mut m = mat4_identity();
    m.m[12] = v.x;
    m.m[13] = v.y;
    m.m[14] = v.z;
    m
}

#[inline]
pub const fn mat4_scale(v: Vec3) -> Mat4 {
    Mat4 {
        m: [
            v.x, 0.0, 0.0, 0.0,
            0.0, v.y, 0.0, 0.0,
            0.0, 0.0, v.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[inline]
pub fn mat4_rotate_x(angle: f32) -> Mat4 {
    let c = cos_f32(angle);
    let s = sin_f32(angle);
    Mat4 {
        m: [
            1.0, 0.0, 0.0, 0.0,
            0.0, c,   s,   0.0,
            0.0, -s,  c,   0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[inline]
pub fn mat4_rotate_y(angle: f32) -> Mat4 {
    let c = cos_f32(angle);
    let s = sin_f32(angle);
    Mat4 {
        m: [
            c,   0.0, -s,  0.0,
            0.0, 1.0, 0.0, 0.0,
            s,   0.0, c,   0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[inline]
pub fn mat4_rotate_z(angle: f32) -> Mat4 {
    let c = cos_f32(angle);
    let s = sin_f32(angle);
    Mat4 {
        m: [
            c,   s,   0.0, 0.0,
            -s,  c,   0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[inline]
pub fn mat4_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / tan_f32(fov / 2.0);
    let range_inv = 1.0 / (near - far);
    
    Mat4 {
        m: [
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (near + far) * range_inv, -1.0,
            0.0, 0.0, 2.0 * near * far * range_inv, 0.0,
        ]
    }
}

#[inline]
pub fn mat4_look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let f = vec3_normalize(vec3_sub(center, eye));
    let s = vec3_normalize(vec3_cross(f, up));
    let u = vec3_cross(s, f);
    
    let mut m = mat4_identity();
    m.m[0] = s.x;
    m.m[4] = s.y;
    m.m[8] = s.z;
    m.m[1] = u.x;
    m.m[5] = u.y;
    m.m[9] = u.z;
    m.m[2] = -f.x;
    m.m[6] = -f.y;
    m.m[10] = -f.z;
    m.m[12] = -vec3_dot(s, eye);
    m.m[13] = -vec3_dot(u, eye);
    m.m[14] = vec3_dot(f, eye);
    
    m
}

// ============================================================================
// Quaternion
// ============================================================================

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
pub const fn quat_identity() -> Quat {
    Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
}

#[inline]
pub fn quat_angle_axis(angle: f32, axis: Vec3) -> Quat {
    let half_angle = angle * 0.5;
    let s = sin_f32(half_angle);
    let norm_axis = vec3_normalize(axis);
    Quat {
        x: norm_axis.x * s,
        y: norm_axis.y * s,
        z: norm_axis.z * s,
        w: cos_f32(half_angle),
    }
}

#[inline]
pub const fn quat_mul(a: Quat, b: Quat) -> Quat {
    Quat {
        x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    }
}

#[inline]
pub fn quat_to_mat4(q: Quat) -> Mat4 {
    let xx = q.x * q.x;
    let yy = q.y * q.y;
    let zz = q.z * q.z;
    let xy = q.x * q.y;
    let xz = q.x * q.z;
    let yz = q.y * q.z;
    let wx = q.w * q.x;
    let wy = q.w * q.y;
    let wz = q.w * q.z;
    
    Mat4 {
        m: [
            1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz),       2.0 * (xz - wy),       0.0,
            2.0 * (xy - wz),       1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx),       0.0,
            2.0 * (xz + wy),       2.0 * (yz - wx),       1.0 - 2.0 * (xx + yy), 0.0,
            0.0,                   0.0,                   0.0,                   1.0,
        ]
    }
}
