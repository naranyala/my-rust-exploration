#[path = "gp2d_math/mod.rs"]
mod math;
#[path = "gp2d_collision/mod.rs"]
mod collision;
mod raylib;
mod helpers;

use math::*;
use collision::*;
use raylib::*;
use helpers::*;



// ============================
// Physics Engine
// ============================

#[derive(Clone, Debug)]
enum Shape {
    Circle(Circle),
    Polygon(Vec<Vec2>),
}

impl Shape {
    fn move_by(&mut self, delta: Vec2) {
        match self {
            Shape::Circle(c) => c.center = c.center + delta,
            Shape::Polygon(verts) => {
                for v in verts { *v = *v + delta; }
            }
        }
    }

    fn center(&self) -> Vec2 {
        match self {
            Shape::Circle(c) => c.center,
            Shape::Polygon(verts) => {
                let mut sum = Vec2::zero();
                for v in verts { sum = sum + *v; }
                sum * (1.0 / verts.len() as f32)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Body {
    shape: Shape,
    velocity: Vec2,
    inv_mass: f32,
    restitution: f32,
    color: Color,
}

impl Body {
    fn new_circle(pos: Vec2, radius: f32, mass: f32, color: Color) -> Self {
        Self {
            shape: Shape::Circle(Circle::new(pos, radius)),
            velocity: Vec2::zero(),
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            restitution: 0.7,
            color,
        }
    }

    fn new_box(pos: Vec2, w: f32, h: f32, mass: f32, color: Color) -> Self {
        let hw = w * 0.5;
        let hh = h * 0.5;
        let verts = vec![
            pos + Vec2::new(-hw, -hh),
            pos + Vec2::new(hw, -hh),
            pos + Vec2::new(hw, hh),
            pos + Vec2::new(-hw, hh),
        ];
        Self {
            shape: Shape::Polygon(verts),
            velocity: Vec2::zero(),
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            restitution: 0.5,
            color,
        }
    }

    fn new_poly(pos: Vec2, radius: f32, sides: usize, mass: f32, color: Color) -> Self {
        let mut verts = Vec::new();
        for i in 0..sides {
            let angle = (i as f32 / sides as f32) * std::f32::consts::PI * 2.0;
            verts.push(pos + Vec2::from_angle(angle) * radius);
        }
        Self {
            shape: Shape::Polygon(verts),
            velocity: Vec2::zero(),
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            restitution: 0.6,
            color,
        }
    }

    fn integrate(&mut self, dt: f32, gravity: Vec2) {
        if self.inv_mass == 0.0 { return; }
        
        self.velocity = self.velocity + gravity * dt;
        let delta = self.velocity * dt;
        self.shape.move_by(delta);
    }
}

// SAT Collision Detection
fn get_polygon_collision(poly_a: &[Vec2], poly_b: &[Vec2]) -> CollisionResult {
    let mut result = CollisionResult::default();
    result.depth = std::f32::MAX;

    let mut edges = Vec::new();
    for i in 0..poly_a.len() { edges.push(poly_a[(i + 1) % poly_a.len()] - poly_a[i]); }
    for i in 0..poly_b.len() { edges.push(poly_b[(i + 1) % poly_b.len()] - poly_b[i]); }

    for edge in edges {
        let axis = edge.perp().normalize();
        let (min_a, max_a) = project_polygon(poly_a, axis);
        let (min_b, max_b) = project_polygon(poly_b, axis);

        if max_a < min_b || max_b < min_a {
            return CollisionResult::default(); // Separating axis found
        }

        let overlap = (max_a.min(max_b) - min_a.max(min_b)).abs();
        if overlap < result.depth {
            result.depth = overlap;
            result.normal = axis;
        }
    }

    // Ensure normal points from A to B
    let center_a = {
        let mut sum = Vec2::zero();
        for v in poly_a { sum = sum + *v; }
        sum * (1.0 / poly_a.len() as f32)
    };
    let center_b = {
        let mut sum = Vec2::zero();
        for v in poly_b { sum = sum + *v; }
        sum * (1.0 / poly_b.len() as f32)
    };
    
    if result.normal.dot(center_b - center_a) < 0.0 {
        result.normal = -result.normal;
    }

    result.hit = true;
    result
}

fn check_collision(a: &Body, b: &Body) -> CollisionResult {
    match (&a.shape, &b.shape) {
        (Shape::Circle(c1), Shape::Circle(c2)) => c1.collision(c2),
        (Shape::Polygon(p1), Shape::Polygon(p2)) => get_polygon_collision(p1, p2),
        (Shape::Polygon(p), Shape::Circle(c)) => {
            // Circle vs Poly (approximate circle as poly for simplicity in this demo)
            let mut cp = Vec::new();
            for i in 0..16 {
                let angle = i as f32 * std::f32::consts::PI * 2.0 / 16.0;
                cp.push(c.center + Vec2::from_angle(angle) * c.radius);
            }
            get_polygon_collision(p, &cp)
        },
        (Shape::Circle(c), Shape::Polygon(p)) => {
            let mut cp = Vec::new();
            for i in 0..16 {
                let angle = i as f32 * std::f32::consts::PI * 2.0 / 16.0;
                cp.push(c.center + Vec2::from_angle(angle) * c.radius);
            }
            let mut res = get_polygon_collision(&cp, p);
            if res.hit { res.normal = -res.normal; }
            res
        }
    }
}

fn resolve_collision(a: &mut Body, b: &mut Body, res: CollisionResult) {
    // Positional Correction
    let percent = 0.8; // Penetration percentage to correct
    let slop = 0.01;
    let correction = res.normal * ((res.depth.max(slop) - slop).max(0.0) / (a.inv_mass + b.inv_mass) * percent);
    
    if a.inv_mass > 0.0 {
        a.shape.move_by(-correction * a.inv_mass);
    }
    if b.inv_mass > 0.0 {
        b.shape.move_by(correction * b.inv_mass);
    }

    // Velocity Resolution
    let rel_vel = b.velocity - a.velocity;
    let vel_along_normal = rel_vel.dot(res.normal);

    if vel_along_normal > 0.0 { return; } // Moving apart

    let e = a.restitution.min(b.restitution);
    let j = -(1.0 + e) * vel_along_normal;
    let j = j / (a.inv_mass + b.inv_mass);

    let impulse = res.normal * j;
    
    if a.inv_mass > 0.0 {
        a.velocity = a.velocity - impulse * a.inv_mass;
    }
    if b.inv_mass > 0.0 {
        b.velocity = b.velocity + impulse * b.inv_mass;
    }
}

// ============================
// Demo State
// ============================

struct DemoState {
    bodies: Vec<Body>,
    rng_seed: u64,
    gravity: Vec2,
    drag_idx: Option<usize>,
}

impl DemoState {
    fn new() -> Self {
        let mut s = Self {
            bodies: Vec::new(),
            rng_seed: 98765,
            gravity: Vec2::new(0.0, 500.0),
            drag_idx: None,
        };

        // Create container walls (static bodies)
        let wall_color = DARKGRAY;
        let thickness = 50.0;
        let w = 800.0;
        let h = 600.0;
        
        // Floor
        s.bodies.push(Body::new_box(Vec2::new(w/2.0, h + thickness/2.0 - 10.0), w, thickness, 0.0, wall_color));
        // Left Wall
        s.bodies.push(Body::new_box(Vec2::new(-thickness/2.0 + 10.0, h/2.0), thickness, h, 0.0, wall_color));
        // Right Wall
        s.bodies.push(Body::new_box(Vec2::new(w + thickness/2.0 - 10.0, h/2.0), thickness, h, 0.0, wall_color));
        // Funnel
        s.bodies.push(Body::new_box(Vec2::new(100.0, 200.0), 300.0, 20.0, 0.0, wall_color));
        s.bodies.push(Body::new_box(Vec2::new(700.0, 200.0), 300.0, 20.0, 0.0, wall_color));

        // Rotate funnel walls
        {
            let c = s.bodies[3].shape.center();
            if let Shape::Polygon(verts) = &mut s.bodies[3].shape {
                for v in verts { *v = (*v - c).rotate(0.4) + c; }
            }
        }
        {
            let c = s.bodies[4].shape.center();
            if let Shape::Polygon(verts) = &mut s.bodies[4].shape {
                for v in verts { *v = (*v - c).rotate(-0.4) + c; }
            }
        }

        s
    }

    fn spawn_random(&mut self, pos: Vec2) {
        let r = rand_range(&mut self.rng_seed, 0.0, 3.0);
        let color = match rand_range(&mut self.rng_seed, 0.0, 3.0) as i32 {
            0 => RED, 1 => GREEN, 2 => BLUE, _ => ORANGE
        };
        
        let body = if r < 1.0 {
            Body::new_circle(pos, rand_range(&mut self.rng_seed, 15.0, 25.0), 10.0, color)
        } else if r < 2.0 {
            Body::new_box(pos, rand_range(&mut self.rng_seed, 30.0, 50.0), rand_range(&mut self.rng_seed, 30.0, 50.0), 10.0, color)
        } else {
            Body::new_poly(pos, rand_range(&mut self.rng_seed, 20.0, 35.0), rand_range(&mut self.rng_seed, 3.0, 6.0) as usize, 10.0, color)
        };
        self.bodies.push(body);
    }

    fn update(&mut self, dt: f32, mouse_pos: Vec2) {
        unsafe {
            // Input
            if IsMouseButtonPressed(MOUSE_RIGHT) {
                // Explosion
                for b in &mut self.bodies {
                    if b.inv_mass == 0.0 { continue; }
                    let dir = b.shape.center() - mouse_pos;
                    let dist = dir.len();
                    if dist < 200.0 && dist > 0.001 {
                        let force = dir.normalize() * (100000.0 / dist);
                        b.velocity = b.velocity + force * b.inv_mass;
                    }
                }
            }

            if IsMouseButtonPressed(MOUSE_LEFT) {
                for (i, b) in self.bodies.iter().enumerate().rev() {
                    if b.inv_mass > 0.0 { // Only dynamic
                        let center = b.shape.center();
                        if center.dist(mouse_pos) < 40.0 { // Simple pick
                            self.drag_idx = Some(i);
                            break;
                        }
                    }
                }
            }

            if IsMouseButtonReleased(MOUSE_LEFT) {
                self.drag_idx = None;
            }

            // Dragging
            if let Some(idx) = self.drag_idx {
                if idx < self.bodies.len() {
                    let b = &mut self.bodies[idx];
                    let target_vel = (mouse_pos - b.shape.center()) * 10.0;
                    b.velocity = b.velocity.lerp(target_vel, 0.2);
                }
            }

            // Auto spawn
            let r = rand_range(&mut self.rng_seed, 0.0, 100.0);
            if r < 2.0 {
                let x = rand_range(&mut self.rng_seed, 300.0, 500.0);
                self.spawn_random(Vec2::new(x, -50.0));
            }

            // Physics Step
            let steps = 4;
            let sub_dt = dt / steps as f32;

            for _ in 0..steps {
                // Integrate
                for b in &mut self.bodies {
                    b.integrate(sub_dt, self.gravity);
                }

                // Collisions (Naive O(N^2))
                // We need to use indices to avoid borrow checker hell with nested loops
                let len = self.bodies.len();
                for i in 0..len {
                    for j in (i+1)..len {
                        let (b1, b2) = {
                            let (left, right) = self.bodies.split_at_mut(j);
                            (&mut left[i], &mut right[0])
                        };

                        if b1.inv_mass == 0.0 && b2.inv_mass == 0.0 { continue; }

                        // Broadphase check (AABB approx)
                        let c1 = b1.shape.center();
                        let c2 = b2.shape.center();
                        if c1.dist_squared(c2) > 100.0 * 100.0 { continue; } 

                        let res = check_collision(b1, b2);
                        if res.hit {
                            resolve_collision(b1, b2, res);
                        }
                    }
                }
            }

            // Cleanup fallen
            self.bodies.retain(|b| b.shape.center().y < 1000.0);
        }
    }

    fn draw(&self) {
        unsafe {
            ClearBackground(RAYWHITE);
            
            for b in &self.bodies {
                match &b.shape {
                    Shape::Circle(c) => {
                        DrawCircle(c.center.x as i32, c.center.y as i32, c.radius, b.color);
                        DrawCircle(c.center.x as i32, c.center.y as i32, c.radius * 0.8, Color{r:0,g:0,b:0,a:20});
                    },
                    Shape::Polygon(verts) => {
                        // Triangulate fan
                        let c = b.shape.center();
                        let center_v = Vector2 { x: c.x, y: c.y };
                        for i in 0..verts.len() {
                            let p1 = verts[i];
                            let p2 = verts[(i + 1) % verts.len()];
                            DrawTriangle(
                                center_v,
                                Vector2 { x: p1.x, y: p1.y },
                                Vector2 { x: p2.x, y: p2.y },
                                b.color
                            );
                            DrawLine(p1.x as i32, p1.y as i32, p2.x as i32, p2.y as i32, BLACK);
                        }
                    }
                }
            }

            DrawText(b"Advanced Physics Demo\0".as_ptr() as *const i8, 10, 10, 20, BLACK);
            DrawText(b"Left Drag: Grab | Right Click: Explode\0".as_ptr() as *const i8, 10, 35, 16, DARKGRAY);
            DrawText(cstr(format_args!("Bodies: {}", self.bodies.len())), 10, 60, 16, BLACK);
        }
    }
}

fn main() {
    unsafe {
        InitWindow(800, 600, b"GP2D Advanced Physics\0".as_ptr() as *const i8);
        SetTargetFPS(60);

        let mut state = DemoState::new();

        while !WindowShouldClose() {
            let dt = GetFrameTime();
            let m = GetMousePosition();
            let mouse_pos = Vec2::new(m.x, m.y);

            state.update(dt, mouse_pos);

            BeginDrawing();
            state.draw();
            EndDrawing();
        }

        CloseWindow();
    }
}
