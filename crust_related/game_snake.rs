// rustc --edition 2021 -C panic=abort snake_game.rs -o snake_game 
// (Requires linking against raylib and its dependencies)

#![no_std]
#![no_main]

// --- raylib C Function Bindings and Color Definitions ---

// Link against raylib and required system libraries
#[link(name = "raylib")]
#[link(name = "m")]
#[link(name = "pthread")]
#[link(name = "dl")]
#[link(name = "rt")]
#[link(name = "GL")]
#[link(name = "c")]
extern "C" {
    // Core functions
    pub fn InitWindow(width: i32, height: i32, title: *const u8);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn SetTargetFPS(fps: i32);
    pub fn GetFrameTime() -> f32;
    
    // Drawing functions
    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);
    pub fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    
    // Input/Random
    pub fn IsKeyPressed(key: i32) -> bool;
    pub fn GetRandomValue(min: i32, max: i32) -> i32;
}

// Key definitions for raylib
const KEY_RIGHT: i32 = 262;
const KEY_LEFT: i32 = 263;
const KEY_DOWN: i32 = 264;
const KEY_UP: i32 = 265;
const KEY_SPACE: i32 = 32;

// Color struct - must match raylib's Color layout
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// Common colors
pub const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };
pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
pub const DARKGREEN: Color = Color { r: 0, g: 100, b: 0, a: 255 };
pub const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
pub const GOLD: Color = Color { r: 255, g: 203, b: 0, a: 255 };
pub const GRAY: Color = Color { r: 130, g: 130, b: 130, a: 255 };

// --- Snake Game Logic Structs and Constants ---

const MAX_SNAKE_LENGTH: usize = 400;
const CELL_SIZE: i32 = 20;
const GRID_WIDTH: i32 = 40;
const GRID_HEIGHT: i32 = 30;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up, Down, Left, Right,
}

#[derive(Copy, Clone, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

struct Game {
    snake: [Position; MAX_SNAKE_LENGTH],
    length: usize,
    direction: Direction,
    next_direction: Direction,
    food: Position,
    game_over: bool,
    frames_since_move: f32,
    move_interval: f32,
}

// Helper to check if a position is occupied by the snake
fn is_occupied(pos: Position, snake: &[Position], length: usize) -> bool {
    for i in 0..length {
        if snake[i] == pos {
            return true;
        }
    }
    false
}

// A helper for generating the food position
unsafe fn generate_food_position(snake: &[Position], length: usize) -> Position {
    let mut food_pos: Position;
    loop {
        let x = GetRandomValue(0, GRID_WIDTH - 1);
        let y = GetRandomValue(0, GRID_HEIGHT - 1);
        food_pos = Position { x, y };

        if !is_occupied(food_pos, snake, length) {
            break;
        }
    }
    food_pos
}

// --- Manual Integer-to-ASCII (itoa) Conversion ---
// Converts a number into a null-terminated byte array for DrawText.
// Buffer size 12 is enough for i32 (up to 10 digits + null terminator + 1 for safety)
fn itoa_to_bytes(mut n: i32, buffer: &mut [u8; 12]) -> *const u8 {
    // Handle zero case
    if n == 0 {
        buffer[0] = b'0';
        buffer[1] = 0; // Null terminator
        return buffer.as_ptr();
    }

    let mut i = buffer.len() - 1;
    buffer[i] = 0; // Start with the null terminator

    // Write digits from right to left (end of buffer)
    while n > 0 {
        i -= 1;
        let digit = (n % 10) as u8;
        buffer[i] = b'0' + digit;
        n /= 10;
    }
    
    // Return a pointer to the start of the valid digits (index i)
    // Safety: we rely on the fact that i will be < buffer.len()
    unsafe { buffer.as_ptr().add(i) }
}


// --- Main Entry Point ---

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let width = GRID_WIDTH * CELL_SIZE;
    let height = GRID_HEIGHT * CELL_SIZE;
    let title = b"Crust Raylib Snake!\0";
    
    InitWindow(width, height, title.as_ptr());
    SetTargetFPS(60); 

    let mut initial_snake_array = [Position { x: 0, y: 0 }; MAX_SNAKE_LENGTH];
    initial_snake_array[0] = Position { x: 5, y: 15 };
    initial_snake_array[1] = Position { x: 4, y: 15 };
    initial_snake_array[2] = Position { x: 3, y: 15 };
    let initial_length = 3;

    let mut game = Game {
        direction: Direction::Right,
        next_direction: Direction::Right,
        food: generate_food_position(&initial_snake_array, initial_length),
        snake: initial_snake_array,
        length: initial_length,
        game_over: false,
        frames_since_move: 0.0,
        move_interval: 0.12, 
    };

    // Buffer for score conversion
    let mut score_buffer: [u8; 12] = [0; 12];
    
    // Game loop
    while !WindowShouldClose() {
        let delta = GetFrameTime();
        
        // --- Input Handling ---
        // ... (Input handling remains the same)
        if IsKeyPressed(KEY_RIGHT) && game.direction != Direction::Left {
            game.next_direction = Direction::Right;
        } else if IsKeyPressed(KEY_LEFT) && game.direction != Direction::Right {
            game.next_direction = Direction::Left;
        } else if IsKeyPressed(KEY_UP) && game.direction != Direction::Down {
            game.next_direction = Direction::Up;
        } else if IsKeyPressed(KEY_DOWN) && game.direction != Direction::Up {
            game.next_direction = Direction::Down;
        } else if IsKeyPressed(KEY_SPACE) && game.game_over {
            // Reset game state
            let mut reset_snake = [Position { x: 0, y: 0 }; MAX_SNAKE_LENGTH];
            reset_snake[0] = Position { x: 5, y: 15 };
            reset_snake[1] = Position { x: 4, y: 15 };
            reset_snake[2] = Position { x: 3, y: 15 };

            game = Game {
                direction: Direction::Right,
                next_direction: Direction::Right,
                food: generate_food_position(&reset_snake, 3),
                snake: reset_snake,
                length: 3,
                game_over: false,
                frames_since_move: 0.0,
                move_interval: 0.12,
            };
        }


        // --- Game Update Logic ---
        if !game.game_over {
            game.frames_since_move += delta;

            if game.frames_since_move >= game.move_interval {
                game.frames_since_move = 0.0;
                game.direction = game.next_direction; 

                // 1. Calculate new head position
                let mut head = game.snake[0];
                match game.direction {
                    Direction::Up => head.y -= 1,
                    Direction::Down => head.y += 1,
                    Direction::Left => head.x -= 1,
                    Direction::Right => head.x += 1,
                }
                
                // 2. Collision Checks
                if head.x < 0 || head.x >= GRID_WIDTH || head.y < 0 || head.y >= GRID_HEIGHT {
                    game.game_over = true;
                }
                for i in 1..game.length {
                    if head == game.snake[i] {
                        game.game_over = true;
                        break;
                    }
                }

                if !game.game_over {
                    // 3. Move snake body (Shift all segments back one position)
                    for i in (1..game.length).rev() {
                        game.snake[i] = game.snake[i-1];
                    }
                    game.snake[0] = head; // Set new head

                    // 4. Food consumption
                    if head == game.food {
                        if game.length < MAX_SNAKE_LENGTH {
                            let old_tail_pos = game.snake[game.length - 1]; 
                            game.length += 1;
                            game.snake[game.length - 1] = old_tail_pos;
                            game.food = generate_food_position(&game.snake, game.length);
                        } else {
                            game.food = generate_food_position(&game.snake, game.length);
                        }
                    } 
                }
            }
        }
        
        // --- Drawing ---
        BeginDrawing();
        ClearBackground(GRAY);
        
        if game.game_over {
            let msg = b"GAME OVER! Press SPACE to restart.\0";
            DrawText(msg.as_ptr(), 
                     width / 2 - 300, height / 2 - 50, 40, RED);
        } else {
            // Draw Food (GOLD)
            DrawRectangle(game.food.x * CELL_SIZE, game.food.y * CELL_SIZE, CELL_SIZE, CELL_SIZE, GOLD);

            // Draw Snake (DARKGREEN)
            for i in 0..game.length {
                let segment = game.snake[i];
                let color = if i == 0 { BLACK } else { DARKGREEN }; 
                DrawRectangle(segment.x * CELL_SIZE, segment.y * CELL_SIZE, CELL_SIZE, CELL_SIZE, color);
            }
        }

        // Draw Score Label
        DrawText(b"SCORE: \0".as_ptr(), 20, 20, 20, BLACK);
        
        // Draw Dynamic Score Value (Snake length minus initial 3 segments)
        let score = (game.length as i32) - 3;
        let score_ptr = itoa_to_bytes(score, &mut score_buffer);
        DrawText(score_ptr, 100, 20, 20, BLACK); // Positioned next to the label

        
        EndDrawing();
    }
    
    CloseWindow();
    0
}

// --- Required Panic/Personality Handlers ---

#[panic_handler]
pub unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
