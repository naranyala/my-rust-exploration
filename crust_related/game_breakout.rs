// rustc --edition 2021 -C panic=abort breakout.rs -o breakout

#![no_std]
#![no_main]

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
    
    // Drawing functions
    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn ClearBackground(color: Color);
    pub fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    pub fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    pub fn DrawCircle(centerX: i32, centerY: i32, radius: f32, color: Color);
    
    // Input functions
    pub fn IsKeyDown(key: i32) -> bool;
    pub fn GetMouseX() -> i32;
    
    // Timing
    pub fn GetFrameTime() -> f32;
}

// Key constants
pub const KEY_LEFT: i32 = 263;
pub const KEY_RIGHT: i32 = 262;
pub const KEY_SPACE: i32 = 32;
pub const KEY_R: i32 = 82;

// Color struct
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
pub const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
pub const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };
pub const GREEN: Color = Color { r: 0, g: 228, b: 48, a: 255 };
pub const YELLOW: Color = Color { r: 253, g: 249, b: 0, a: 255 };
pub const ORANGE: Color = Color { r: 255, g: 161, b: 0, a: 255 };
pub const PURPLE: Color = Color { r: 200, g: 122, b: 255, a: 255 };
pub const DARKGRAY: Color = Color { r: 80, g: 80, b: 80, a: 255 };

// Game structs
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Paddle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub active: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Brick {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub color: Color,
}

// Collision detection
pub unsafe fn check_ball_paddle_collision(ball: *mut Ball, paddle: *const Paddle) -> bool {
    let b = *ball;
    let p = *paddle;
    
    if b.y + b.radius >= p.y &&
       b.y - b.radius <= p.y + p.height &&
       b.x >= p.x &&
       b.x <= p.x + p.width {
        (*ball).velocity_y = -(*ball).velocity_y;
        (*ball).y = p.y - b.radius;
        
        // Add angle based on hit position
        let hit_pos = (b.x - p.x) / p.width;
        (*ball).velocity_x = (hit_pos - 0.5) * 400.0;
        true
    } else {
        false
    }
}

pub unsafe fn check_ball_brick_collision(ball: *mut Ball, brick: *mut Brick) -> bool {
    let b = *ball;
    let br = *brick;
    
    if !br.active {
        return false;
    }
    
    if b.x + b.radius >= br.x &&
       b.x - b.radius <= br.x + br.width &&
       b.y + b.radius >= br.y &&
       b.y - b.radius <= br.y + br.height {
        
        (*brick).active = false;
        
        // Determine collision side
        let overlap_left = (b.x + b.radius) - br.x;
        let overlap_right = (br.x + br.width) - (b.x - b.radius);
        let overlap_top = (b.y + b.radius) - br.y;
        let overlap_bottom = (br.y + br.height) - (b.y - b.radius);
        
        // Find minimum overlap to determine collision side
        let mut min_overlap = overlap_left;
        let mut side = 0; // 0=left, 1=right, 2=top, 3=bottom
        
        if overlap_right < min_overlap {
            min_overlap = overlap_right;
            side = 1;
        }
        if overlap_top < min_overlap {
            min_overlap = overlap_top;
            side = 2;
        }
        if overlap_bottom < min_overlap {
            side = 3;
        }
        
        // Bounce based on collision side
        if side == 0 || side == 1 {
            (*ball).velocity_x = -(*ball).velocity_x;
        } else {
            (*ball).velocity_y = -(*ball).velocity_y;
        }
        
        true
    } else {
        false
    }
}

// Simple itoa
pub unsafe fn int_to_string(mut num: i32, buffer: *mut u8) {
    if num == 0 {
        *buffer.offset(0) = b'0';
        *buffer.offset(1) = 0;
        return;
    }
    
    let mut len = 0;
    let mut temp = num;
    while temp > 0 {
        temp /= 10;
        len += 1;
    }
    
    let mut idx = len;
    *buffer.offset(idx as isize) = 0;
    
    while num > 0 {
        idx -= 1;
        *buffer.offset(idx as isize) = b'0' + (num % 10) as u8;
        num /= 10;
    }
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Window setup
    let width = 800;
    let height = 600;
    let title = b"Crust Breakout\0";
    
    InitWindow(width, height, title.as_ptr());
    SetTargetFPS(60);
    
    // Game state
    let mut game_over = false;
    let mut game_won = false;
    let mut score = 0;
    let mut lives = 3;
    
    // Paddle
    let mut paddle = Paddle {
        x: (width / 2 - 60) as f32,
        y: (height - 50) as f32,
        width: 120.0,
        height: 20.0,
        speed: 500.0,
    };
    
    // Ball
    let mut ball = Ball {
        x: (width / 2) as f32,
        y: (height - 80) as f32,
        radius: 8.0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        active: false,
    };
    
    // Bricks
    let brick_rows = 5;
    let brick_cols = 10;
    let brick_width = 70.0;
    let brick_height = 25.0;
    let brick_padding = 5.0;
    let brick_offset_x = 35.0;
    let brick_offset_y = 50.0;
    
    let mut bricks: [Brick; 50] = [Brick {
        x: 0.0, y: 0.0,
        width: brick_width,
        height: brick_height,
        active: true,
        color: RED,
    }; 50];
    
    let colors = [RED, ORANGE, YELLOW, GREEN, BLUE];
    
    // Initialize bricks
    for row in 0..brick_rows {
        for col in 0..brick_cols {
            let idx = row * brick_cols + col;
            bricks[idx].x = brick_offset_x + col as f32 * (brick_width + brick_padding);
            bricks[idx].y = brick_offset_y + row as f32 * (brick_height + brick_padding);
            bricks[idx].color = colors[row];
        }
    }
    
    let mut score_buffer: [u8; 16] = [0; 16];
    let mut lives_buffer: [u8; 16] = [0; 16];
    
    // Game loop
    while !WindowShouldClose() {
        let delta = GetFrameTime();
        
        if !game_over && !game_won {
            // Paddle movement
            if IsKeyDown(KEY_LEFT) {
                paddle.x -= paddle.speed * delta;
                if paddle.x < 0.0 {
                    paddle.x = 0.0;
                }
            }
            if IsKeyDown(KEY_RIGHT) {
                paddle.x += paddle.speed * delta;
                if paddle.x + paddle.width > width as f32 {
                    paddle.x = width as f32 - paddle.width;
                }
            }
            
            // Mouse control (alternative)
            let mouse_x = GetMouseX() as f32;
            paddle.x = mouse_x - paddle.width / 2.0;
            if paddle.x < 0.0 {
                paddle.x = 0.0;
            }
            if paddle.x + paddle.width > width as f32 {
                paddle.x = width as f32 - paddle.width;
            }
            
            // Launch ball
            if !ball.active && IsKeyDown(KEY_SPACE) {
                ball.active = true;
                ball.velocity_x = 200.0;
                ball.velocity_y = -300.0;
            }
            
            // Update ball if active
            if ball.active {
                ball.x += ball.velocity_x * delta;
                ball.y += ball.velocity_y * delta;
                
                // Wall collisions
                if ball.x - ball.radius <= 0.0 || ball.x + ball.radius >= width as f32 {
                    ball.velocity_x = -ball.velocity_x;
                }
                if ball.y - ball.radius <= 0.0 {
                    ball.velocity_y = -ball.velocity_y;
                }
                
                // Bottom boundary (lose life)
                if ball.y - ball.radius >= height as f32 {
                    lives -= 1;
                    if lives <= 0 {
                        game_over = true;
                    } else {
                        ball.x = paddle.x + paddle.width / 2.0;
                        ball.y = paddle.y - 30.0;
                        ball.velocity_x = 0.0;
                        ball.velocity_y = 0.0;
                        ball.active = false;
                    }
                }
                
                // Paddle collision
                check_ball_paddle_collision(&mut ball as *mut Ball, &paddle as *const Paddle);
                
                // Brick collisions
                for i in 0..50 {
                    if check_ball_brick_collision(&mut ball as *mut Ball, &mut bricks[i] as *mut Brick) {
                        score += 10;
                    }
                }
                
                // Check win condition
                let mut all_destroyed = true;
                for i in 0..50 {
                    if bricks[i].active {
                        all_destroyed = false;
                        break;
                    }
                }
                if all_destroyed {
                    game_won = true;
                }
            } else {
                // Ball follows paddle when not active
                ball.x = paddle.x + paddle.width / 2.0;
                ball.y = paddle.y - 30.0;
            }
        }
        
        // Restart game
        if (game_over || game_won) && IsKeyDown(KEY_R) {
            game_over = false;
            game_won = false;
            score = 0;
            lives = 3;
            
            paddle.x = (width / 2 - 60) as f32;
            ball.x = (width / 2) as f32;
            ball.y = (height - 80) as f32;
            ball.velocity_x = 0.0;
            ball.velocity_y = 0.0;
            ball.active = false;
            
            for i in 0..50 {
                bricks[i].active = true;
            }
        }
        
        // Drawing
        BeginDrawing();
        ClearBackground(RAYWHITE);
        
        // Draw title
        DrawText(b"CRUST BREAKOUT\0".as_ptr(), 20, 10, 20, DARKGRAY);
        
        // Draw score
        int_to_string(score, score_buffer.as_mut_ptr());
        DrawText(b"Score: \0".as_ptr(), width - 200, 10, 20, DARKGRAY);
        DrawText(score_buffer.as_ptr(), width - 100, 10, 20, DARKGRAY);
        
        // Draw lives
        int_to_string(lives, lives_buffer.as_mut_ptr());
        DrawText(b"Lives: \0".as_ptr(), width - 200, 35, 20, DARKGRAY);
        DrawText(lives_buffer.as_ptr(), width - 100, 35, 20, DARKGRAY);
        
        if !game_over && !game_won {
            // Draw paddle
            DrawRectangle(paddle.x as i32, paddle.y as i32, paddle.width as i32, paddle.height as i32, DARKGRAY);
            
            // Draw ball
            DrawCircle(ball.x as i32, ball.y as i32, ball.radius, BLACK);
            
            // Draw bricks
            for i in 0..50 {
                if bricks[i].active {
                    DrawRectangle(
                        bricks[i].x as i32,
                        bricks[i].y as i32,
                        bricks[i].width as i32,
                        bricks[i].height as i32,
                        bricks[i].color
                    );
                }
            }
            
            // Draw instructions if ball not active
            if !ball.active {
                DrawText(b"Press SPACE to launch!\0".as_ptr(), width / 2 - 150, height / 2, 20, DARKGRAY);
            }
        } else if game_over {
            DrawText(b"GAME OVER!\0".as_ptr(), width / 2 - 120, height / 2 - 40, 40, RED);
            DrawText(b"Press R to restart\0".as_ptr(), width / 2 - 120, height / 2 + 20, 20, DARKGRAY);
        } else if game_won {
            DrawText(b"YOU WIN!\0".as_ptr(), width / 2 - 100, height / 2 - 40, 40, GREEN);
            DrawText(b"Press R to restart\0".as_ptr(), width / 2 - 120, height / 2 + 20, 20, DARKGRAY);
        }
        
        EndDrawing();
    }
    
    CloseWindow();
    0
}

#[panic_handler]
pub unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }
