// gp2d_collision/mod.rs - Collision Detection & Physics Helpers
// Depends on: math.rs for vector operations

#![allow(dead_code)]

use crate::math::*;

/* ============================
 * Basic Shapes
 * ============================ */

// Axis-aligned bounding box
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

// Circle
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

// Line segment
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
}

// Ray
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec2,
    pub direction: Vec2, // Should be normalized
}

/* ============================
 * Hit/Collision Results
 * ============================ */

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RaycastResult {
    pub hit: bool,
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

impl Default for RaycastResult {
    fn default() -> Self {
        Self {
            hit: false,
            point: Vec2::zero(),
            normal: Vec2::zero(),
            distance: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollisionResult {
    pub hit: bool,
    pub normal: Vec2,
    pub depth: f32,
}

impl Default for CollisionResult {
    fn default() -> Self {
        Self {
            hit: false,
            normal: Vec2::zero(),
            depth: 0.0,
        }
    }
}

/* ============================
 * AABB Operations
 * ============================ */

impl Aabb {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
    
    pub fn from_center(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
    
    pub fn overlaps(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }
    
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }
    
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
    
    pub fn half_size(&self) -> Vec2 {
        self.size() * 0.5
    }
}

/* ============================
 * Circle Operations
 * ============================ */

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.center.dist_squared(point) <= self.radius * self.radius
    }
    
    pub fn overlaps(&self, other: &Self) -> bool {
        self.center.dist_squared(other.center) <= (self.radius + other.radius).powi(2)
    }
    
    pub fn collision(&self, other: &Self) -> CollisionResult {
        let mut result = CollisionResult::default();
        
        let delta = other.center - self.center;
        let dist_squared = delta.len_squared();
        let radius_sum = self.radius + other.radius;
        
        if dist_squared <= radius_sum * radius_sum {
            let dist = dist_squared.sqrt();
            result.hit = true;
            result.normal = if dist > 0.0 { delta * (1.0 / dist) } else { Vec2::new(1.0, 0.0) };
            result.depth = radius_sum - dist;
        }
        
        result
    }
}

/* ============================
 * Circle vs AABB
 * ============================ */

pub fn circle_aabb_overlap(circle: &Circle, aabb: &Aabb) -> bool {
    // Find closest point on AABB to circle center
    let closest_x = circle.center.x.max(aabb.min.x).min(aabb.max.x);
    let closest_y = circle.center.y.max(aabb.min.y).min(aabb.max.y);
    let closest = Vec2::new(closest_x, closest_y);
    
    circle.center.dist_squared(closest) <= circle.radius * circle.radius
}

/* ============================
 * Ray Operations
 * ============================ */

impl Ray {
    pub fn new(origin: Vec2, direction: Vec2) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
}

// Ray vs Circle
pub fn raycast_circle(ray: &Ray, circle: &Circle) -> RaycastResult {
    let mut result = RaycastResult::default();
    
    let oc = ray.origin - circle.center;
    let b = oc.dot(ray.direction);
    let c = oc.dot(oc) - circle.radius * circle.radius;
    let discriminant = b * b - c;
    
    if discriminant < 0.0 {
        return result;
    }
    
    let mut t = -b - discriminant.sqrt();
    if t < 0.0 {
        t = -b + discriminant.sqrt();
    }
    if t < 0.0 {
        return result;
    }
    
    result.hit = true;
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    result.normal = (result.point - circle.center).normalize();
    
    result
}

// Ray vs AABB
pub fn raycast_aabb(ray: &Ray, aabb: &Aabb) -> RaycastResult {
    let mut result = RaycastResult::default();
    
    let tx1 = (aabb.min.x - ray.origin.x) / ray.direction.x;
    let tx2 = (aabb.max.x - ray.origin.x) / ray.direction.x;
    let mut tmin = tx1.min(tx2);
    let mut tmax = tx1.max(tx2);
    
    let ty1 = (aabb.min.y - ray.origin.y) / ray.direction.y;
    let ty2 = (aabb.max.y - ray.origin.y) / ray.direction.y;
    tmin = tmin.max(ty1.min(ty2));
    tmax = tmax.min(ty1.max(ty2));
    
    if tmax < 0.0 || tmin > tmax {
        return result;
    }
    
    let t = if tmin >= 0.0 { tmin } else { tmax };
    result.hit = true;
    result.distance = t;
    result.point = ray.origin + ray.direction * t;
    
    // Calculate normal
    let center = aabb.center();
    let to_point = result.point - center;
    let half_size = aabb.half_size();
    
    let dx = (to_point.x / half_size.x).abs();
    let dy = (to_point.y / half_size.y).abs();
    
    // result.normal = if dx > dy {
    //     Vec2::new(if to_point.x > 0.0 { 1.0 } else { -1.0 }, 0.0)
    // } else {
    //     Vec2::new(0.0, if to_point.y > 0.0 { 1.0 } else { -1.0 })
    // };

    result.normal = if dx > dy + 0.0001 {
        Vec2::new(if to_point.x > 0.0 { 1.0 } else { -1.0 }, 0.0)
    } else {
        Vec2::new(0.0, if to_point.y > 0.0 { 1.0 } else { -1.0 })
    };
    
    result
}

/* ============================
 * Polygon Collision (SAT)
 * ============================ */

// Project polygon onto axis
pub fn project_polygon(vertices: &[Vec2], axis: Vec2) -> (f32, f32) {
    let mut min = vertices[0].dot(axis);
    let mut max = min;
    
    for vertex in &vertices[1..] {
        let projection = vertex.dot(axis);
        if projection < min {
            min = projection;
        }
        if projection > max {
            max = projection;
        }
    }
    
    (min, max)
}

// SAT overlap test for convex polygons
pub fn polygon_overlap(poly_a: &[Vec2], poly_b: &[Vec2]) -> bool {
    // Test edges of polygon A
    for i in 0..poly_a.len() {
        let edge = poly_a[(i + 1) % poly_a.len()] - poly_a[i];
        let axis = edge.perp().normalize();
        
        let (min_a, max_a) = project_polygon(poly_a, axis);
        let (min_b, max_b) = project_polygon(poly_b, axis);
        
        if max_a < min_b || max_b < min_a {
            return false;
        }
    }
    
    // Test edges of polygon B
    for i in 0..poly_b.len() {
        let edge = poly_b[(i + 1) % poly_b.len()] - poly_b[i];
        let axis = edge.perp().normalize();
        
        let (min_a, max_a) = project_polygon(poly_a, axis);
        let (min_b, max_b) = project_polygon(poly_b, axis);
        
        if max_a < min_b || max_b < min_a {
            return false;
        }
    }
    
    true // All axes passed, shapes overlap
}

/* ============================
 * Physics Helpers
 * ============================ */

// Bounce velocity off surface
pub fn reflect_velocity(velocity: Vec2, normal: Vec2, bounce_factor: f32) -> Vec2 {
    velocity.reflect(normal) * bounce_factor
}

// Slide velocity along surface (remove normal component)
pub fn slide_velocity(velocity: Vec2, normal: Vec2) -> Vec2 {
    let normal_component = velocity.dot(normal);
    velocity - normal * normal_component
}

// Separate two overlapping circles
pub fn separate_circles(circle_a: &mut Circle, circle_b: &mut Circle) {
    let collision = circle_a.collision(circle_b);
    if collision.hit {
        let half_depth = collision.depth * 0.5;
        circle_a.center = circle_a.center - collision.normal * half_depth;
        circle_b.center = circle_b.center + collision.normal * half_depth;
    }
}

// Additional utility functions
impl Segment {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }
    
    pub fn length(&self) -> f32 {
        self.a.dist(self.b)
    }
    
    pub fn direction(&self) -> Vec2 {
        (self.b - self.a).normalize()
    }
}

// Ray vs Segment
pub fn raycast_segment(ray: &Ray, segment: &Segment) -> RaycastResult {
    let mut result = RaycastResult::default();
    
    let segment_vec = segment.b - segment.a;
    let segment_length = segment_vec.len();
    let segment_dir = segment_vec * (1.0 / segment_length);
    
    // Check if ray and segment are parallel
    let cross = ray.direction.perp().dot(segment_dir);
    if cross.abs() < 1e-6 {
        return result;
    }
    
    let t = (segment.a - ray.origin).perp().dot(segment_dir) / cross;
    let u = (ray.origin - segment.a).perp().dot(ray.direction) / cross;
    
    if t >= 0.0 && u >= 0.0 && u <= segment_length {
        result.hit = true;
        result.distance = t;
        result.point = ray.origin + ray.direction * t;
        result.normal = segment_dir.perp().normalize();
    }
    
    result
}
