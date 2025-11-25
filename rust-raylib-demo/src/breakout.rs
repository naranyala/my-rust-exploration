use raylib_ffi::*;
use raylib_ffi::colors::*;

// Keyboard constants
const KEY_LEFT: i32 = 263;
const KEY_RIGHT: i32 = 262;
const KEY_R: i32 = 82;
const KEY_SPACE: i32 = 32;

// Game constants
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const PADDLE_WIDTH: i32 = 100;
const PADDLE_HEIGHT: i32 = 20;
const BALL_RADIUS: i32 = 10;
const BRICK_WIDTH: i32 = 70;
const BRICK_HEIGHT: i32 = 30;
const BRICK_ROWS: i32 = 5;
const BRICK_COLS: i32 = 10;
const BRICK_MARGIN: i32 = 5;

#[derive(Clone, Copy)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Ball {
    position: Vector2,
    velocity: Vector2,
    radius: i32,
    active: bool,
}

impl Ball {
    fn new() -> Self {
        Self {
            position: Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0),
            velocity: Vector2::new(4.0, -4.0),
            radius: BALL_RADIUS,
            active: true,
        }
    }

    fn update(&mut self) {
        if !self.active {
            return;
        }

        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;

        // Wall collisions
        if self.position.x <= self.radius as f32 || self.position.x >= (SCREEN_WIDTH - self.radius) as f32 {
            self.velocity.x *= -1.0;
        }
        if self.position.y <= self.radius as f32 {
            self.velocity.y *= -1.0;
        }

        // Bottom boundary (ball lost)
        if self.position.y >= SCREEN_HEIGHT as f32 {
            self.active = false;
        }
    }

    fn reset(&mut self) {
        self.position = Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0);
        self.velocity = Vector2::new(4.0, -4.0);
        self.active = true;
    }

    fn check_paddle_collision(&mut self, paddle: &Paddle) -> bool {
        if !self.active {
            return false;
        }

        let ball_bottom = self.position.y + self.radius as f32;
        let ball_top = self.position.y - self.radius as f32;
        let ball_left = self.position.x - self.radius as f32;
        let ball_right = self.position.x + self.radius as f32;

        let paddle_top = paddle.position.y;
        let paddle_bottom = paddle.position.y + PADDLE_HEIGHT as f32;
        let paddle_left = paddle.position.x;
        let paddle_right = paddle.position.x + PADDLE_WIDTH as f32;

        // Check if ball is colliding with paddle
        if ball_bottom >= paddle_top && ball_top <= paddle_bottom &&
           ball_right >= paddle_left && ball_left <= paddle_right {
            
            // Calculate hit position on paddle for bounce angle
            let hit_pos = (self.position.x - paddle.position.x) / PADDLE_WIDTH as f32;
            let angle = hit_pos * 2.0 - 1.0; // -1 to 1 range
            
            self.velocity.x = angle * 8.0;
            self.velocity.y = -self.velocity.y.abs();
            
            // Move ball above paddle to prevent multiple collisions
            self.position.y = paddle_top - self.radius as f32;
            
            return true;
        }
        
        false
    }

    fn check_brick_collision(&mut self, brick: &mut Brick) -> bool {
        if !self.active || !brick.active {
            return false;
        }

        let ball_left = self.position.x - self.radius as f32;
        let ball_right = self.position.x + self.radius as f32;
        let ball_top = self.position.y - self.radius as f32;
        let ball_bottom = self.position.y + self.radius as f32;

        let brick_left = brick.position.x;
        let brick_right = brick.position.x + BRICK_WIDTH as f32;
        let brick_top = brick.position.y;
        let brick_bottom = brick.position.y + BRICK_HEIGHT as f32;

        // Check collision
        if ball_right >= brick_left && ball_left <= brick_right &&
           ball_bottom >= brick_top && ball_top <= brick_bottom {
            
            brick.active = false;

            // Determine collision side and bounce accordingly
            let overlap_left = ball_right - brick_left;
            let overlap_right = brick_right - ball_left;
            let overlap_top = ball_bottom - brick_top;
            let overlap_bottom = brick_bottom - ball_top;

            // Find the smallest overlap
            let min_overlap = overlap_left.min(overlap_right).min(overlap_top).min(overlap_bottom);

            if min_overlap == overlap_left || min_overlap == overlap_right {
                self.velocity.x *= -1.0;
            } else {
                self.velocity.y *= -1.0;
            }

            return true;
        }

        false
    }
}

struct Paddle {
    position: Vector2,
    width: i32,
    height: i32,
    speed: f32,
}

impl Paddle {
    fn new() -> Self {
        Self {
            position: Vector2::new(
                (SCREEN_WIDTH / 2 - PADDLE_WIDTH / 2) as f32,
                (SCREEN_HEIGHT - 40) as f32
            ),
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            speed: 8.0,
        }
    }

    fn update(&mut self) {
        unsafe {
            if IsKeyDown(KEY_LEFT) {
                self.position.x -= self.speed;
            }
            if IsKeyDown(KEY_RIGHT) {
                self.position.x += self.speed;
            }

            // Keep paddle within screen bounds
            if self.position.x < 0.0 {
                self.position.x = 0.0;
            }
            if self.position.x > (SCREEN_WIDTH - self.width) as f32 {
                self.position.x = (SCREEN_WIDTH - self.width) as f32;
            }
        }
    }

    fn reset(&mut self) {
        self.position = Vector2::new(
            (SCREEN_WIDTH / 2 - PADDLE_WIDTH / 2) as f32,
            (SCREEN_HEIGHT - 40) as f32
        );
    }
}

struct Brick {
    position: Vector2,
    active: bool,
    color: Color,
}

impl Brick {
    fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            position: Vector2::new(x, y),
            active: true,
            color,
        }
    }
}

struct Game {
    ball: Ball,
    paddle: Paddle,
    bricks: Vec<Brick>,
    score: i32,
    lives: i32,
    game_over: bool,
    level_complete: bool,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            ball: Ball::new(),
            paddle: Paddle::new(),
            bricks: Vec::new(),
            score: 0,
            lives: 3,
            game_over: false,
            level_complete: false,
        };
        game.create_bricks();
        game
    }

    fn create_bricks(&mut self) {
        self.bricks.clear();
        
        let start_x = (SCREEN_WIDTH - (BRICK_COLS * (BRICK_WIDTH + BRICK_MARGIN))) / 2;
        let start_y = 50;
        
        let colors = [RED, ORANGE, YELLOW, GREEN, BLUE];
        
        for row in 0..BRICK_ROWS {
            for col in 0..BRICK_COLS {
                let x = start_x + col * (BRICK_WIDTH + BRICK_MARGIN);
                let y = start_y + row * (BRICK_HEIGHT + BRICK_MARGIN);
                let color = colors[row as usize % colors.len()];
                
                self.bricks.push(Brick::new(x as f32, y as f32, color));
            }
        }
    }

    fn update(&mut self) {
        if self.game_over || self.level_complete {
            return;
        }

        self.paddle.update();
        self.ball.update();

        // Check paddle collision
        self.ball.check_paddle_collision(&self.paddle);

        // Check brick collisions
        let mut bricks_destroyed = 0;
        for brick in &mut self.bricks {
            if self.ball.check_brick_collision(brick) {
                bricks_destroyed += 1;
            }
        }
        self.score += bricks_destroyed * 10;

        // Check if level is complete
        let active_bricks = self.bricks.iter().filter(|b| b.active).count();
        if active_bricks == 0 {
            self.level_complete = true;
        }

        // Check if ball is lost
        if !self.ball.active {
            self.lives -= 1;
            if self.lives <= 0 {
                self.game_over = true;
            } else {
                self.ball.reset();
            }
        }
    }

    fn reset(&mut self) {
        self.ball.reset();
        self.paddle.reset();
        self.create_bricks();
        self.score = 0;
        self.lives = 3;
        self.game_over = false;
        self.level_complete = false;
    }

    fn next_level(&mut self) {
        self.ball.reset();
        self.paddle.reset();
        self.create_bricks();
        self.level_complete = false;
        // Increase ball speed for next level
        self.ball.velocity.x *= 1.2;
        self.ball.velocity.y *= 1.2;
    }

    fn handle_input(&mut self) {
        unsafe {
            if IsKeyPressed(KEY_R) {
                self.reset();
            } else if IsKeyPressed(KEY_SPACE) {
                if self.game_over {
                    self.reset();
                } else if self.level_complete {
                    self.next_level();
                }
            }
        }
    }
}

fn main() {
    unsafe {
        // Initialize window
        InitWindow(SCREEN_WIDTH, SCREEN_HEIGHT, b"Breakout Game\0".as_ptr() as *const i8);
        SetTargetFPS(60);

        let mut game = Game::new();

        // Main game loop
        while !WindowShouldClose() {
            // Handle input
            game.handle_input();
            
            // Update game state
            game.update();

            BeginDrawing();
            
            // Clear background
            ClearBackground(BLACK);
            
            // Draw bricks
            for brick in &game.bricks {
                if brick.active {
                    DrawRectangle(
                        brick.position.x as i32,
                        brick.position.y as i32,
                        BRICK_WIDTH,
                        BRICK_HEIGHT,
                        brick.color
                    );
                    DrawRectangleLines(
                        brick.position.x as i32,
                        brick.position.y as i32,
                        BRICK_WIDTH,
                        BRICK_HEIGHT,
                        WHITE
                    );
                }
            }
            
            // Draw paddle
            DrawRectangle(
                game.paddle.position.x as i32,
                game.paddle.position.y as i32,
                game.paddle.width,
                game.paddle.height,
                WHITE
            );
            
            // Draw ball
            if game.ball.active {
                DrawCircle(
                    game.ball.position.x as i32,
                    game.ball.position.y as i32,
                    game.ball.radius as f32,
                    WHITE
                );
            }
            
            // Draw UI
            let score_text = format!("Score: {}\0", game.score);
            DrawText(score_text.as_ptr() as *const i8, 10, 10, 20, WHITE);
            
            let lives_text = format!("Lives: {}\0", game.lives);
            DrawText(lives_text.as_ptr() as *const i8, SCREEN_WIDTH - 100, 10, 20, WHITE);
            
            // Draw controls
            DrawText(b"Arrow Keys: Move | R: Restart\0".as_ptr() as *const i8, 
                    10, SCREEN_HEIGHT - 25, 16, GRAY);
            
            // Draw game over screen
            if game.game_over {
                DrawRectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color { r: 0, g: 0, b: 0, a: 180 });
                DrawText(b"GAME OVER\0".as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 80, SCREEN_HEIGHT / 2 - 50, 40, RED);
                
                let final_score = format!("Final Score: {}\0", game.score);
                DrawText(final_score.as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 80, SCREEN_HEIGHT / 2, 20, WHITE);
                
                DrawText(b"Press SPACE to restart\0".as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 100, SCREEN_HEIGHT / 2 + 40, 20, YELLOW);
            }
            
            // Draw level complete screen
            if game.level_complete {
                DrawRectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color { r: 0, g: 0, b: 0, a: 180 });
                DrawText(b"LEVEL COMPLETE!\0".as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 120, SCREEN_HEIGHT / 2 - 50, 40, GREEN);
                
                let level_score = format!("Score: {}\0", game.score);
                DrawText(level_score.as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 50, SCREEN_HEIGHT / 2, 20, WHITE);
                
                DrawText(b"Press SPACE for next level\0".as_ptr() as *const i8, 
                        SCREEN_WIDTH / 2 - 120, SCREEN_HEIGHT / 2 + 40, 20, YELLOW);
            }
            
            EndDrawing();
        }

        CloseWindow();
    }
}
