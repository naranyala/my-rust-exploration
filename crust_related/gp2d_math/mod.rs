// gp2d_math/mod.rs - Core 2D Math Library (NO collision detection)
// Pure math operations: vectors, matrices, curves, animation, easing

#![allow(dead_code)]
use std::f32::consts::PI;

/* ============================
 * 2D Vector - Core Operations
 * ============================ */
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}



impl Vec2 {
    // Construction & arithmetic
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
    
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
    
    pub fn scale(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
    
    pub fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
    
    pub fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }

    // Dot product & length
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn len_squared(self) -> f32 {
        self.dot(self)
    }
    
    pub fn len(self) -> f32 {
        self.len_squared().sqrt()
    }
    
    pub fn dist_squared(self, other: Self) -> f32 {
        self.sub(other).len_squared()
    }
    
    pub fn dist(self, other: Self) -> f32 {
        self.dist_squared(other).sqrt()
    }

    // Normalization
    pub fn normalize(self) -> Self {
        let len = self.len();
        if len > 0.0 {
            self.scale(1.0 / len)
        } else {
            self
        }
    }

    // Perpendicular (90° rotation)
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    // Interpolation
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    // Rotation
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
    
    pub fn from_angle(rad: f32) -> Self {
        Self::new(rad.cos(), rad.sin())
    }
    
    pub fn rotate(self, rad: f32) -> Self {
        let (c, s) = (rad.cos(), rad.sin());
        Self::new(
            self.x * c - self.y * s,
            self.x * s + self.y * c,
        )
    }

    // Reflection
    pub fn reflect(self, normal: Self) -> Self {
        self.sub(normal.scale(2.0 * self.dot(normal)))
    }
}

// Operator overloads for convenience
impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        self.scale(scalar)
    }
}

impl std::ops::Mul<Vec2> for Vec2 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        self.neg()
    }
}

/* ============================
 * Bezier Curves
 * ============================ */

pub fn bezier_quad(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let a = p0.lerp(p1, t);
    let b = p1.lerp(p2, t);
    a.lerp(b, t)
}

pub fn bezier_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;
    
    let mut p = p0 * uuu;
    p = p + (p1 * (3.0 * uu * t));
    p = p + (p2 * (3.0 * u * tt));
    p = p + (p3 * ttt);
    p
}

/* ============================
 * 3x3 Matrix (2D Transforms)
 * ============================ */
#[derive(Copy, Clone, Debug)]
pub struct Mat3 {
    pub m: [f32; 9],
}

impl Mat3 {
    pub fn identity() -> Self {
        let mut m = [0.0; 9];
        m[0] = 1.0;
        m[4] = 1.0;
        m[8] = 1.0;
        Self { m }
    }
    
    pub fn multiply(self, other: Self) -> Self {
        let mut result = Self { m: [0.0; 9] };
        
        for col in 0..3 {
            for row in 0..3 {
                for k in 0..3 {
                    result.m[col * 3 + row] += self.m[k * 3 + row] * other.m[col * 3 + k];
                }
            }
        }
        result
    }
    
    pub fn translate(tx: f32, ty: f32) -> Self {
        let mut m = Self::identity();
        m.m[6] = tx;
        m.m[7] = ty;
        m
    }
    
    pub fn scale(sx: f32, sy: f32) -> Self {
        let mut m = Self::identity();
        m.m[0] = sx;
        m.m[4] = sy;
        m
    }
    
    pub fn rotate(angle: f32) -> Self {
        let (c, s) = (angle.cos(), angle.sin());
        let mut m = Self::identity();
        m.m[0] = c;
        m.m[3] = -s;
        m.m[1] = s;
        m.m[4] = c;
        m
    }

    // Transform application
    pub fn transform_point(self, point: Vec2) -> Vec2 {
        Vec2::new(
            self.m[0] * point.x + self.m[3] * point.y + self.m[6],
            self.m[1] * point.x + self.m[4] * point.y + self.m[7],
        )
    }
    
    pub fn transform_vector(self, vector: Vec2) -> Vec2 {
        Vec2::new(
            self.m[0] * vector.x + self.m[3] * vector.y,
            self.m[1] * vector.x + self.m[4] * vector.y,
        )
    }
}

/* ============================
 * Easing Functions
 * ============================ */

pub fn ease_linear(t: f32) -> f32 { t }

// Quadratic
pub fn ease_in_quad(t: f32) -> f32 { t * t }
pub fn ease_out_quad(t: f32) -> f32 { t * (2.0 - t) }
pub fn ease_in_out_quad(t: f32) -> f32 { 
    if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
}

// Cubic
pub fn ease_in_cubic(t: f32) -> f32 { t * t * t }
pub fn ease_out_cubic(t: f32) -> f32 { 
    let s = t - 1.0;
    s * s * s + 1.0
}
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let s = t - 1.0;
        s * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
    }
}

// Exponential
pub fn ease_in_expo(t: f32) -> f32 { 
    if t == 0.0 { 0.0 } else { 2.0f32.powf(10.0 * (t - 1.0)) }
}
pub fn ease_out_expo(t: f32) -> f32 { 
    if t == 1.0 { 1.0 } else { 1.0 - 2.0f32.powf(-10.0 * t) }
}

// Elastic
pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 || t == 1.0 { return t; }
    2.0f32.powf(-10.0 * t) * ((t - 0.075) * (2.0 * PI) / 0.3).sin() + 1.0
}

/* ============================
 * Timing & Animation
 * ============================ */

// Time normalization
pub fn time_normalize(t: f32, start: f32, end: f32) -> f32 {
    (t - start) / (end - start)
}

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

pub fn clamp01(v: f32) -> f32 {
    clamp(v, 0.0, 1.0)
}

// Simple timer
#[derive(Copy, Clone, Debug)]
pub struct Timer {
    pub start: f32,
    pub duration: f32,
    pub loop_: bool,
}

impl Timer {
    pub fn new(duration: f32, loop_: bool) -> Self {
        Self {
            start: 0.0,
            duration,
            loop_,
        }
    }
    
    pub fn progress(&self, global_time: f32) -> f32 {
        let mut local_time = global_time - self.start;
        if self.duration > 0.0 && self.loop_ {
            local_time = local_time % self.duration;
        }
        if self.duration > 0.0 {
            local_time / self.duration
        } else {
            0.0
        }
    }
    
    pub fn done(&self, global_time: f32) -> bool {
        !self.loop_ && (global_time - self.start >= self.duration)
    }
    
    pub fn restart(&mut self, now: f32) {
        self.start = now;
    }
}

/* ============================
 * Keyframe Animation
 * ============================ */

#[derive(Copy, Clone, Debug)]
pub struct ColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct KeyFloat {
    pub time: f32,
    pub value: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct KeyVec2 {
    pub time: f32,
    pub value: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct KeyColor {
    pub time: f32,
    pub value: ColorF,
}

pub fn sample_float(keys: &[KeyFloat], t: f32, loop_: bool) -> f32 {
    if keys.is_empty() {
        return 0.0;
    }
    
    let mut time = t;
    if loop_ && keys.len() > 1 {
        let total_duration = keys.last().unwrap().time - keys[0].time;
        if total_duration > 0.0 {
            time = ((t - keys[0].time) % total_duration) + keys[0].time;
        }
    }
    
    if keys.len() == 1 || time <= keys[0].time {
        return keys[0].value;
    }
    if time >= keys.last().unwrap().time {
        return keys.last().unwrap().value;
    }

    for i in 1..keys.len() {
        if time < keys[i].time {
            let a = keys[i - 1].time;
            let b = keys[i].time;
            let p = ease_in_out_cubic((time - a) / (b - a));
            return keys[i - 1].value + (keys[i].value - keys[i - 1].value) * p;
        }
    }
    
    keys.last().unwrap().value
}

pub fn sample_vec2(keys: &[KeyVec2], t: f32, loop_: bool) -> Vec2 {
    if keys.is_empty() {
        return Vec2::zero();
    }
    
    let mut time = t;
    if loop_ && keys.len() > 1 {
        let total_duration = keys.last().unwrap().time - keys[0].time;
        if total_duration > 0.0 {
            time = ((t - keys[0].time) % total_duration) + keys[0].time;
        }
    }
    
    if keys.len() == 1 || time <= keys[0].time {
        return keys[0].value;
    }
    if time >= keys.last().unwrap().time {
        return keys.last().unwrap().value;
    }

    for i in 1..keys.len() {
        if time < keys[i].time {
            let a = keys[i - 1].time;
            let b = keys[i].time;
            let p = ease_in_out_cubic((time - a) / (b - a));
            return keys[i - 1].value.lerp(keys[i].value, p);
        }
    }
    
    keys.last().unwrap().value
}

/* ============================
 * Path Following (Bezier Chains)
 * ============================ */

#[derive(Copy, Clone, Debug)]
pub struct BezierSegment {
    pub points: [Vec2; 4],
}

pub fn path_sample(segments: &[BezierSegment], t: f32) -> Vec2 {
    if segments.is_empty() {
        return Vec2::zero();
    }
    
    let segment_t = t * segments.len() as f32;
    let idx = (segment_t as usize).min(segments.len() - 1);
    let local_t = segment_t - idx as f32;

    let segment = &segments[idx];
    bezier_cubic(
        segment.points[0],
        segment.points[1],
        segment.points[2],
        segment.points[3],
        local_t,
    )
}


