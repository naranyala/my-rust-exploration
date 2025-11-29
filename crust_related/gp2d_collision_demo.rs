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
// Physics Ball
// ============================
#[derive(Clone)]
struct PhysicsBall {
    circle: Circle,
    velocity: Vec2,
    color: Color,
    trail: Vec<Vec2>,
    spawn_time: f32,
}

impl PhysicsBall {
    fn new(center: Vec2, radius: f32, color: Color, spawn_time: f32) -> Self {
        Self {
            circle: Circle::new(center, radius),
            velocity: Vec2::zero(),
            color,
            trail: Vec::new(),
            spawn_time,
        }
    }

    fn update(&mut self, dt: f32, bounds: &Aabb) {
        self.trail.push(self.circle.center);
        if self.trail.len() > 25 {
            self.trail.remove(0);
        }

        // Gravity
        self.velocity.y += 620.0 * dt;

        // Integrate
        self.circle.center = self.circle.center + self.velocity * dt;

        // Bounds collision
        if self.circle.center.x - self.circle.radius < bounds.min.x {
            self.circle.center.x = bounds.min.x + self.circle.radius;
            self.velocity.x = -self.velocity.x * 0.8;
        }
        if self.circle.center.x + self.circle.radius > bounds.max.x {
            self.circle.center.x = bounds.max.x - self.circle.radius;
            self.velocity.x = -self.velocity.x * 0.8;
        }
        if self.circle.center.y - self.circle.radius < bounds.min.y {
            self.circle.center.y = bounds.min.y + self.circle.radius;
            self.velocity.y = -self.velocity.y * 0.8;
        }
        if self.circle.center.y + self.circle.radius > bounds.max.y {
            self.circle.center.y = bounds.max.y - self.circle.radius;
            self.velocity.y = -self.velocity.y * 0.8;
        }

        // Drag
        self.velocity = self.velocity * 0.995f32.powf(dt * 60.0);
    }

    #[allow(dead_code)]
    fn apply_impulse(&mut self, impulse: Vec2) {
        self.velocity = self.velocity + impulse;
    }
}

// ============================
// Demo State
// ============================
struct DemoState {
    balls: Vec<PhysicsBall>,
    obstacles: Vec<Circle>,
    walls: Vec<Aabb>,
    bounds: Aabb,
    spawn_timer: Timer,
    global_time: f32,
    show_rays: bool,
    bezier_points: [Vec2; 4],
    bezier_t: f32,
    bezier_dir: f32,
}

impl DemoState {
    fn new(w: f32, h: f32) -> Self {
        Self {
            balls: Vec::new(),
            obstacles: vec![
                Circle::new(Vec2::new(200.0, 200.0), 40.0),
                Circle::new(Vec2::new(600.0, 300.0), 60.0),
                Circle::new(Vec2::new(400.0, 500.0), 50.0),
            ],
            walls: vec![
                Aabb::from_center(Vec2::new(400.0, 50.0), Vec2::new(350.0, 20.0)),
                Aabb::from_center(Vec2::new(150.0, 400.0), Vec2::new(20.0, 200.0)),
                Aabb::from_center(Vec2::new(650.0, 400.0), Vec2::new(20.0, 200.0)),
            ],
            bounds: Aabb::new(Vec2::zero(), Vec2::new(w, h)),
            spawn_timer: Timer::new(0.8, false),
            global_time: 0.0,
            show_rays: false,
            bezier_points: [
                Vec2::new(100.0, 100.0),
                Vec2::new(300.0, 400.0),
                Vec2::new(500.0, 100.0),
                Vec2::new(700.0, 400.0),
            ],
            bezier_t: 0.0,
            bezier_dir: 1.0,
        }
    }

    fn spawn_ball(&mut self, pos: Vec2) {
        let colors = [RED, BLUE, GREEN, YELLOW, PURPLE, ORANGE];
        let color = colors[self.balls.len() % colors.len()];
        let mut ball = PhysicsBall::new(pos, 15.0, color, self.global_time);

        ball.velocity = Vec2::new(
            (self.global_time * 1.7).sin() * 280.0,
            -180.0 - (self.global_time * 2.3).cos().abs() * 220.0,
        );

        self.balls.push(ball);
    }

    fn update(&mut self, dt: f32, _mouse_pos: Vec2) {
        self.global_time += dt;

        // Bezier animation
        self.bezier_t += dt * 0.35 * self.bezier_dir;
        if self.bezier_t >= 1.0 { self.bezier_t = 1.0; self.bezier_dir = -1.0; }
        if self.bezier_t <= 0.0 { self.bezier_t = 0.0; self.bezier_dir = 1.0; }

        // Auto spawn
        if self.spawn_timer.progress(self.global_time) >= 1.0 {
            self.spawn_ball(Vec2::new(60.0, 60.0));
            self.spawn_timer.restart(self.global_time);
        }

        // Update balls
        for ball in &mut self.balls {
            ball.update(dt, &self.bounds);

            // Obstacles
            for obs in &self.obstacles {
                let col = ball.circle.collision(obs);
                if col.hit {
                    // Static resolution: move ball out by full depth
                    ball.circle.center = ball.circle.center - col.normal * col.depth;
                    ball.velocity = reflect_velocity(ball.velocity, col.normal, 0.9);
                }
            }

            // Walls (simple push-out)
            for wall in &self.walls {
                if circle_aabb_overlap(&ball.circle, wall) {
                    let closest = Vec2::new(
                        ball.circle.center.x.max(wall.min.x).min(wall.max.x),
                        ball.circle.center.y.max(wall.min.y).min(wall.max.y),
                    );
                    let pen = ball.circle.center - closest;
                    let dist = pen.len();
                    if dist > 0.0001 {
                        let normal = pen * (1.0 / dist);
                        ball.circle.center = closest + normal * (ball.circle.radius + 0.5);
                        ball.velocity = reflect_velocity(ball.velocity, normal, 0.8);
                    }
                }
            }
        }

        // Ball-vs-ball (O(n²) but fine for demo)
        let len = self.balls.len();
        for i in 0..len {
            for j in (i+1)..len {
                let (a, b) = if i < j {
                    let (left, right) = self.balls.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                } else {
                    continue;
                };

                let col = a.circle.collision(&b.circle);
                if col.hit {
                    separate_circles(&mut a.circle, &mut b.circle);

                    let v1 = a.velocity;
                    let v2 = b.velocity;
                    let n = col.normal;

                    let v1n = v1.dot(n);
                    let v2n = v2.dot(n);
                    let rel = v1n - v2n;

                    if rel > 0.0 { continue; } // moving apart

                    let impulse = (1.0 + 0.9) * rel * 0.5;
                    a.velocity = v1 - n * impulse;
                    b.velocity = v2 + n * impulse;
                }
            }
        }

        // Remove old/slow balls
        self.balls.retain(|b| {
            b.velocity.len() > 8.0 || (self.global_time - b.spawn_time) < 10.0
        });
    }

    fn draw(&self) {
        unsafe {
            ClearBackground(RAYWHITE);

            // Bounds
            DrawRectangleLines(0, 0, 800, 600, DARKGRAY);

            // Obstacles
            for o in &self.obstacles {
                DrawCircle(o.center.x as i32, o.center.y as i32, o.radius, DARKGRAY);
            }

            // Walls
            for w in &self.walls {
                DrawRectangle(
                    w.min.x as i32, w.min.y as i32,
                    (w.max.x - w.min.x) as i32,
                    (w.max.y - w.min.y) as i32,
                    DARKGRAY,
                );
            }

            // Bezier curve
            let mut prev = self.bezier_points[0];
            for i in 1..=30 {
                let t = i as f32 / 30.0;
                let p = bezier_cubic(
                    self.bezier_points[0],
                    self.bezier_points[1],
                    self.bezier_points[2],
                    self.bezier_points[3],
                    t,
                );
                DrawLine(prev.x as i32, prev.y as i32, p.x as i32, p.y as i32, PURPLE);
                prev = p;
            }
            let bp = bezier_cubic(
                self.bezier_points[0],
                self.bezier_points[1],
                self.bezier_points[2],
                self.bezier_points[3],
                self.bezier_t,
            );
            DrawCircle(bp.x as i32, bp.y as i32, 10.0, YELLOW);

            // Balls + trails
            for ball in &self.balls {
                for (i, &pos) in ball.trail.iter().enumerate() {
                    let alpha = (i as f32 / ball.trail.len() as f32) * 0.6;
                    let c = Color {
                        r: ball.color.r,
                        g: ball.color.g,
                        b: ball.color.b,
                        a: (alpha * 255.0) as u8,
                    };
                    DrawCircle(pos.x as i32, pos.y as i32, ball.circle.radius * 0.7, c);
                }
                DrawCircle(ball.circle.center.x as i32, ball.circle.center.y as i32, ball.circle.radius, ball.color);
            }

            // Ray debug
            if self.show_rays {
                for ball in &self.balls {
                    if ball.velocity.len() > 1.0 {
                        let ray = Ray::new(ball.circle.center, ball.velocity.normalize());
                        for obs in &self.obstacles {
                            let hit = raycast_circle(&ray, obs);
                            if hit.hit {
                                DrawLine(ray.origin.x as i32, ray.origin.y as i32,
                                         hit.point.x as i32, hit.point.y as i32, RED);
                                DrawCircle(hit.point.x as i32, hit.point.y as i32, 5.0, RED);
                            }
                        }
                    }
                }
            }

            // UI
            DrawText(b"Rust 2D Math + Collision Demo\0".as_ptr() as *const i8, 20, 20, 20, BLACK);
            DrawText(b"LClick: spawn | R: reset | SPACE: rays\0".as_ptr() as *const i8, 20, 45, 16, BLACK);
            DrawText(cstr(format_args!("Balls: {} | FPS: {:.0}", self.balls.len(), 1.0/GetFrameTime())), 20, 70, 16, BLACK);
            DrawText(cstr(format_args!("Time: {:.1}s", self.global_time)), 20, 92, 16, BLACK);
        }
    }

    fn reset(&mut self) {
        self.balls.clear();
        self.global_time = 0.0;
        self.bezier_t = 0.0;
        self.bezier_dir = 1.0;
    }
}

// ============================
// Entry Point
// ============================
fn main() {
    unsafe {
        InitWindow(800, 600, b"Rust GP2D Physics Demo\0".as_ptr() as *const i8);
        SetTargetFPS(60);

        let mut demo = DemoState::new(800.0, 600.0);

        while !WindowShouldClose() {
            let dt = GetFrameTime();

            let mouse_raw = GetMousePosition();
            let mouse_pos = Vec2::new(mouse_raw.x, mouse_raw.y);

            if IsMouseButtonPressed(MOUSE_LEFT) {
                demo.spawn_ball(mouse_pos);
            }
            if IsKeyPressed(KEY_R) {
                demo.reset();
            }
            if IsKeyPressed(KEY_SPACE) {
                demo.show_rays = !demo.show_rays;
            }

            demo.update(dt, mouse_pos);

            BeginDrawing();
            demo.draw();
            EndDrawing();
        }

        CloseWindow();
    }
}


