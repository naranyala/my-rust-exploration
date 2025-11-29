#[path = "gp2d_math/mod.rs"]
mod math;
mod raylib;

use math::*;
use raylib::*;

// ============================
// Game Constants
// ============================
const GRID_SIZE: i32 = 20;
const CELL_SIZE: i32 = 30;
const WINDOW_WIDTH: i32 = GRID_SIZE * CELL_SIZE;
const WINDOW_HEIGHT: i32 = GRID_SIZE * CELL_SIZE + 60; // Extra space for UI

const MOVE_DELAY: f32 = 0.15; // Seconds between moves

// ============================
// Direction
// ============================
#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, -1.0),
            Direction::Down => Vec2::new(0.0, 1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        }
    }

    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

// ============================
// Snake Game
// ============================
struct SnakeGame {
    snake: Vec<Vec2>,
    direction: Direction,
    next_direction: Direction,
    food: Vec2,
    score: i32,
    game_over: bool,
    move_timer: f32,
    rng_seed: u64,
}

impl SnakeGame {
    fn new() -> Self {
        let mut game = Self {
            snake: vec![Vec2::new(10.0, 10.0)],
            direction: Direction::Right,
            next_direction: Direction::Right,
            food: Vec2::new(15.0, 10.0),
            score: 0,
            game_over: false,
            move_timer: 0.0,
            rng_seed: 12345,
        };
        game.spawn_food();
        game
    }

    fn spawn_food(&mut self) {
        loop {
            let x = self.rand_int(0, GRID_SIZE - 1) as f32;
            let y = self.rand_int(0, GRID_SIZE - 1) as f32;
            let pos = Vec2::new(x, y);

            // Check if food spawns on snake
            let mut valid = true;
            for segment in &self.snake {
                if segment.x == pos.x && segment.y == pos.y {
                    valid = false;
                    break;
                }
            }

            if valid {
                self.food = pos;
                break;
            }
        }
    }

    fn rand_int(&mut self, min: i32, max: i32) -> i32 {
        self.rng_seed = self.rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let f = (self.rng_seed >> 32) as f32 / 4294967296.0;
        min + (f * (max - min + 1) as f32) as i32
    }

    fn update(&mut self, dt: f32) {
        if self.game_over {
            return;
        }

        self.move_timer += dt;

        if self.move_timer >= MOVE_DELAY {
            self.move_timer = 0.0;
            self.direction = self.next_direction;

            // Calculate new head position
            let head = self.snake[0];
            let dir = self.direction.to_vec2();
            let new_head = Vec2::new(head.x + dir.x, head.y + dir.y);

            // Check wall collision
            if new_head.x < 0.0 || new_head.x >= GRID_SIZE as f32 ||
               new_head.y < 0.0 || new_head.y >= GRID_SIZE as f32 {
                self.game_over = true;
                return;
            }

            // Check self collision
            for segment in &self.snake {
                if segment.x == new_head.x && segment.y == new_head.y {
                    self.game_over = true;
                    return;
                }
            }

            // Move snake
            self.snake.insert(0, new_head);

            // Check food collision
            if new_head.x == self.food.x && new_head.y == self.food.y {
                self.score += 10;
                self.spawn_food();
            } else {
                self.snake.pop();
            }
        }
    }

    fn handle_input(&mut self) {
        unsafe {
            if IsKeyPressed(KEY_UP) && self.direction != Direction::Down {
                self.next_direction = Direction::Up;
            }
            if IsKeyPressed(KEY_DOWN) && self.direction != Direction::Up {
                self.next_direction = Direction::Down;
            }
            if IsKeyPressed(KEY_LEFT) && self.direction != Direction::Right {
                self.next_direction = Direction::Left;
            }
            if IsKeyPressed(KEY_RIGHT) && self.direction != Direction::Left {
                self.next_direction = Direction::Right;
            }
            if IsKeyPressed(KEY_R) {
                *self = Self::new();
            }
        }
    }

    fn draw(&self) {
        unsafe {
            ClearBackground(Color { r: 20, g: 20, b: 30, a: 255 });

            // Draw grid
            for x in 0..GRID_SIZE {
                for y in 0..GRID_SIZE {
                    let color = if (x + y) % 2 == 0 {
                        Color { r: 30, g: 30, b: 40, a: 255 }
                    } else {
                        Color { r: 25, g: 25, b: 35, a: 255 }
                    };
                    DrawRectangle(
                        x * CELL_SIZE,
                        y * CELL_SIZE,
                        CELL_SIZE,
                        CELL_SIZE,
                        color
                    );
                }
            }

            // Draw food
            DrawRectangle(
                self.food.x as i32 * CELL_SIZE + 2,
                self.food.y as i32 * CELL_SIZE + 2,
                CELL_SIZE - 4,
                CELL_SIZE - 4,
                RED
            );

            // Draw snake
            for (i, segment) in self.snake.iter().enumerate() {
                let color = if i == 0 {
                    GREEN // Head
                } else {
                    Color { r: 0, g: 180, b: 40, a: 255 } // Body
                };
                
                DrawRectangle(
                    segment.x as i32 * CELL_SIZE + 1,
                    segment.y as i32 * CELL_SIZE + 1,
                    CELL_SIZE - 2,
                    CELL_SIZE - 2,
                    color
                );

                // Draw eyes on head
                if i == 0 {
                    let eye_size = 4.0;
                    let eye_offset = CELL_SIZE as f32 * 0.3;
                    let cx = segment.x as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 * 0.5;
                    let cy = segment.y as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 * 0.5;

                    let (dx, dy) = match self.direction {
                        Direction::Right => (eye_offset, -eye_offset / 2.0),
                        Direction::Left => (-eye_offset, -eye_offset / 2.0),
                        Direction::Up => (0.0, -eye_offset),
                        Direction::Down => (0.0, eye_offset),
                    };

                    DrawCircle((cx + dx) as i32, (cy + dy - 3.0) as i32, eye_size, BLACK);
                    DrawCircle((cx + dx) as i32, (cy + dy + 3.0) as i32, eye_size, BLACK);
                }
            }

            // Draw UI
            let ui_y = GRID_SIZE * CELL_SIZE + 10;
            DrawText(b"SNAKE GAME\0".as_ptr() as *const i8, 10, ui_y, 20, RAYWHITE);
            
            let score_text = format!("Score: {}\0", self.score);
            DrawText(score_text.as_ptr() as *const i8, 200, ui_y, 20, YELLOW);

            DrawText(b"Arrow Keys: Move | R: Restart\0".as_ptr() as *const i8, 10, ui_y + 25, 16, DARKGRAY);

            if self.game_over {
                // Draw game over overlay
                DrawRectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color { r: 0, g: 0, b: 0, a: 180 });
                
                let msg = b"GAME OVER!\0";
                DrawText(msg.as_ptr() as *const i8, WINDOW_WIDTH / 2 - 100, WINDOW_HEIGHT / 2 - 40, 40, RED);
                
                let restart = b"Press R to Restart\0";
                DrawText(restart.as_ptr() as *const i8, WINDOW_WIDTH / 2 - 100, WINDOW_HEIGHT / 2 + 20, 20, RAYWHITE);
            }
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

// ============================
// Main
// ============================
fn main() {
    unsafe {
        InitWindow(WINDOW_WIDTH, WINDOW_HEIGHT, b"Snake Game\0".as_ptr() as *const i8);
        SetTargetFPS(60);

        let mut game = SnakeGame::new();

        while !WindowShouldClose() {
            let dt = GetFrameTime();

            game.handle_input();
            game.update(dt);

            BeginDrawing();
            game.draw();
            EndDrawing();
        }

        CloseWindow();
    }
}
