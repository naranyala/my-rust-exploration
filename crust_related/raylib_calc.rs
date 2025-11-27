// rustc --edition 2021 -C panic=abort calculator.rs -o calculator

#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn rust_eh_personality() { loop {} }

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
    pub fn DrawRectangleLines(posX: i32, posY: i32, width: i32, height: i32, color: Color);
    
    // Input functions
    pub fn IsMouseButtonPressed(button: i32) -> bool;
    pub fn GetMouseX() -> i32;
    pub fn GetMouseY() -> i32;
}

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
pub const DARKGRAY: Color = Color { r: 80, g: 80, b: 80, a: 255 };
pub const LIGHTGRAY: Color = Color { r: 200, g: 200, b: 200, a: 255 };
pub const ORANGE: Color = Color { r: 255, g: 161, b: 0, a: 255 };
pub const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Button {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub text: *const u8,
    pub color: Color,
}

// Check if button is clicked
pub unsafe fn is_button_clicked(btn: *const Button, mouse_x: i32, mouse_y: i32) -> bool {
    let b = *btn;
    mouse_x >= b.x && mouse_x <= b.x + b.width &&
    mouse_y >= b.y && mouse_y <= b.y + b.height
}

// Draw a button
pub unsafe fn draw_button(btn: *const Button) {
    let b = *btn;
    DrawRectangle(b.x, b.y, b.width, b.height, b.color);
    DrawRectangleLines(b.x, b.y, b.width, b.height, BLACK);
    
    let text_x = b.x + b.width / 2 - 10;
    let text_y = b.y + b.height / 2 - 15;
    DrawText(b.text, text_x, text_y, 30, BLACK);
}

// Simple itoa implementation
pub unsafe fn int_to_string(mut num: i32, buffer: *mut u8, _size: usize) -> usize {
    if num == 0 {
        *buffer.offset(0) = b'0';
        *buffer.offset(1) = 0;
        return 1;
    }
    
    let mut is_negative = false;
    if num < 0 {
        is_negative = true;
        num = -num;
    }
    
    let mut len = 0;
    let mut temp = num;
    while temp > 0 {
        temp /= 10;
        len += 1;
    }
    
    if is_negative {
        len += 1;
    }
    
    let mut idx = len;
    *buffer.offset(idx as isize) = 0;
    
    temp = num;
    while temp > 0 {
        idx -= 1;
        *buffer.offset(idx as isize) = b'0' + (temp % 10) as u8;
        temp /= 10;
    }
    
    if is_negative {
        *buffer.offset(0) = b'-';
    }
    
    len
}

#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Window setup
    let width = 400;
    let height = 600;
    let title = b"Crust Calculator\0";
    
    InitWindow(width, height, title.as_ptr());
    SetTargetFPS(60);
    
    // Calculator state
    let mut current_value: i32 = 0;
    let mut stored_value: i32 = 0;
    let mut operation: u8 = 0; // 0=none, 1=add, 2=sub, 3=mul, 4=div
    let mut new_number = true;
    
    // Display buffer
    let mut display_buffer: [u8; 32] = [0; 32];
    
    // Button dimensions
    let btn_width = 80;
    let btn_height = 70;
    let spacing = 10;
    let start_x = 20;
    let start_y = 150;
    
    // Number buttons 0-9
    let mut num_buttons: [Button; 10] = [Button {
        x: 0, y: 0, width: btn_width, height: btn_height,
        text: b"0\0".as_ptr(), color: LIGHTGRAY
    }; 10];
    
    let num_texts: [*const u8; 10] = [
        b"0\0".as_ptr(), b"1\0".as_ptr(), b"2\0".as_ptr(), b"3\0".as_ptr(),
        b"4\0".as_ptr(), b"5\0".as_ptr(), b"6\0".as_ptr(), b"7\0".as_ptr(),
        b"8\0".as_ptr(), b"9\0".as_ptr(),
    ];
    
    // Layout: 7 8 9 / 4 5 6 * 1 2 3 - 0 C = +
    for i in 0..10 {
        let row: i32;
        let col: i32;
        
        if i == 0 {
            row = 3;
            col = 0;
        } else {
            row = 2 - ((i - 1) / 3) as i32;
            col = ((i - 1) % 3) as i32;
        }
        
        num_buttons[i].x = start_x + col * (btn_width + spacing);
        num_buttons[i].y = start_y + row * (btn_height + spacing);
        num_buttons[i].text = num_texts[i];
    }
    
    // Operation buttons
    let op_buttons: [Button; 6] = [
        Button { // +
            x: start_x + 3 * (btn_width + spacing),
            y: start_y + 3 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"+\0".as_ptr(), color: ORANGE
        },
        Button { // -
            x: start_x + 3 * (btn_width + spacing),
            y: start_y + 2 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"-\0".as_ptr(), color: ORANGE
        },
        Button { // *
            x: start_x + 3 * (btn_width + spacing),
            y: start_y + 1 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"*\0".as_ptr(), color: ORANGE
        },
        Button { // /
            x: start_x + 3 * (btn_width + spacing),
            y: start_y + 0 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"/\0".as_ptr(), color: ORANGE
        },
        Button { // =
            x: start_x + 2 * (btn_width + spacing),
            y: start_y + 3 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"=\0".as_ptr(), color: BLUE
        },
        Button { // C (Clear)
            x: start_x + 1 * (btn_width + spacing),
            y: start_y + 3 * (btn_height + spacing),
            width: btn_width, height: btn_height,
            text: b"C\0".as_ptr(), color: BLUE
        },
    ];
    
    // Game loop
    while !WindowShouldClose() {
        // Input handling
        if IsMouseButtonPressed(0) { // Left mouse button
            let mouse_x = GetMouseX();
            let mouse_y = GetMouseY();
            
            // Check number buttons
            for i in 0..10 {
                if is_button_clicked(&num_buttons[i] as *const Button, mouse_x, mouse_y) {
                    if new_number {
                        current_value = i as i32;
                        new_number = false;
                    } else {
                        current_value = current_value * 10 + i as i32;
                    }
                }
            }
            
            // Check operation buttons
            // + button
            if is_button_clicked(&op_buttons[0] as *const Button, mouse_x, mouse_y) {
                stored_value = current_value;
                operation = 1;
                new_number = true;
            }
            // - button
            if is_button_clicked(&op_buttons[1] as *const Button, mouse_x, mouse_y) {
                stored_value = current_value;
                operation = 2;
                new_number = true;
            }
            // * button
            if is_button_clicked(&op_buttons[2] as *const Button, mouse_x, mouse_y) {
                stored_value = current_value;
                operation = 3;
                new_number = true;
            }
            // / button
            if is_button_clicked(&op_buttons[3] as *const Button, mouse_x, mouse_y) {
                stored_value = current_value;
                operation = 4;
                new_number = true;
            }
            // = button
            if is_button_clicked(&op_buttons[4] as *const Button, mouse_x, mouse_y) {
                match operation {
                    1 => current_value = stored_value + current_value,
                    2 => current_value = stored_value - current_value,
                    3 => current_value = stored_value * current_value,
                    4 => {
                        if current_value != 0 {
                            current_value = stored_value / current_value;
                        }
                    },
                    _ => {},
                }
                operation = 0;
                new_number = true;
            }
            // C button (Clear)
            if is_button_clicked(&op_buttons[5] as *const Button, mouse_x, mouse_y) {
                current_value = 0;
                stored_value = 0;
                operation = 0;
                new_number = true;
            }
        }
        
        // Drawing
        BeginDrawing();
        ClearBackground(RAYWHITE);
        
        // Draw title
        DrawText(b"Crust Calculator\0".as_ptr(), 80, 20, 30, BLACK);
        
        // Draw display
        DrawRectangle(20, 70, 360, 60, DARKGRAY);
        int_to_string(current_value, display_buffer.as_mut_ptr(), 32);
        DrawText(display_buffer.as_ptr(), 300, 85, 40, RAYWHITE);
        
        // Draw all buttons
        for i in 0..10 {
            draw_button(&num_buttons[i] as *const Button);
        }
        for i in 0..6 {
            draw_button(&op_buttons[i] as *const Button);
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
