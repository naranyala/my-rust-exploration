use raylib_ffi::*;
use raylib_ffi::colors::*;

// Mouse button constants (from raylib.h)
const MOUSE_BUTTON_LEFT: i32 = 0;
const MOUSE_BUTTON_RIGHT: i32 = 1;

struct Calculator {
    display: String,
    current_value: f64,
    previous_value: f64,
    operator: char,
    should_reset_display: bool,
}

impl Calculator {
    fn new() -> Self {
        Self {
            display: String::from("0"),
            current_value: 0.0,
            previous_value: 0.0,
            operator: ' ',
            should_reset_display: false,
        }
    }

    fn append_number(&mut self, num: char) {
        if self.should_reset_display {
            self.display.clear();
            self.should_reset_display = false;
        }
        
        if self.display == "0" {
            self.display = num.to_string();
        } else {
            self.display.push(num);
        }
    }

    fn append_decimal(&mut self) {
        if self.should_reset_display {
            self.display = String::from("0");
            self.should_reset_display = false;
        }
        
        if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    fn set_operator(&mut self, op: char) {
        if self.operator != ' ' {
            self.calculate();
        }
        
        self.current_value = self.display.parse().unwrap_or(0.0);
        self.operator = op;
        self.should_reset_display = true;
    }

    fn calculate(&mut self) {
        let current = self.display.parse().unwrap_or(0.0);
        
        match self.operator {
            '+' => self.current_value += current,
            '-' => self.current_value -= current,
            '*' => self.current_value *= current,
            '/' => {
                if current != 0.0 {
                    self.current_value /= current;
                } else {
                    self.display = String::from("Error");
                    self.operator = ' ';
                    return;
                }
            }
            _ => self.current_value = current,
        }

        // Format the result
        if self.current_value.fract() == 0.0 {
            self.display = format!("{}", self.current_value as i64);
        } else {
            self.display = format!("{:.6}", self.current_value);
            // Trim trailing zeros
            if self.display.contains('.') {
                while self.display.ends_with('0') {
                    self.display.pop();
                }
                if self.display.ends_with('.') {
                    self.display.pop();
                }
            }
        }

        self.operator = ' ';
        self.should_reset_display = true;
    }

    fn clear(&mut self) {
        self.display = String::from("0");
        self.current_value = 0.0;
        self.previous_value = 0.0;
        self.operator = ' ';
        self.should_reset_display = false;
    }

    fn delete(&mut self) {
        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = String::from("0");
        }
    }
}

fn main() {
    unsafe {
        // Initialize window
        InitWindow(400, 550, b"Rust Calculator\0".as_ptr() as *const i8);
        SetTargetFPS(60);

        let mut calculator = Calculator::new();
        let button_width = 80;
        let button_height = 60;
        let margin = 10;
        let start_x = 20;
        let start_y = 120;

        // Define button layout
        let buttons = [
            // Row 1
            ('7', start_x, start_y),
            ('8', start_x + button_width + margin, start_y),
            ('9', start_x + 2 * (button_width + margin), start_y),
            ('/', start_x + 3 * (button_width + margin), start_y),
            // Row 2
            ('4', start_x, start_y + button_height + margin),
            ('5', start_x + button_width + margin, start_y + button_height + margin),
            ('6', start_x + 2 * (button_width + margin), start_y + button_height + margin),
            ('*', start_x + 3 * (button_width + margin), start_y + button_height + margin),
            // Row 3
            ('1', start_x, start_y + 2 * (button_height + margin)),
            ('2', start_x + button_width + margin, start_y + 2 * (button_height + margin)),
            ('3', start_x + 2 * (button_width + margin), start_y + 2 * (button_height + margin)),
            ('-', start_x + 3 * (button_width + margin), start_y + 2 * (button_height + margin)),
            // Row 4
            ('0', start_x, start_y + 3 * (button_height + margin)),
            ('.', start_x + button_width + margin, start_y + 3 * (button_height + margin)),
            ('=', start_x + 2 * (button_width + margin), start_y + 3 * (button_height + margin)),
            ('+', start_x + 3 * (button_width + margin), start_y + 3 * (button_height + margin)),
            // Row 5
            ('C', start_x, start_y + 4 * (button_height + margin)),
            ('D', start_x + button_width + margin, start_y + 4 * (button_height + margin)),
        ];

        // Main loop
        while !WindowShouldClose() {
            // Handle input
            if IsMouseButtonPressed(MOUSE_BUTTON_LEFT) {
                let mouse_pos = GetMousePosition();
                
                for &(symbol, x, y) in &buttons {
                    if mouse_pos.x >= x as f32 && 
                       mouse_pos.x <= (x + button_width) as f32 &&
                       mouse_pos.y >= y as f32 && 
                       mouse_pos.y <= (y + button_height) as f32 {
                        
                        match symbol {
                            '0'..='9' => calculator.append_number(symbol),
                            '.' => calculator.append_decimal(),
                            '+' | '-' | '*' | '/' => calculator.set_operator(symbol),
                            '=' => calculator.calculate(),
                            'C' => calculator.clear(),
                            'D' => calculator.delete(),
                            _ => {}
                        }
                    }
                }
            }

            BeginDrawing();
            
            // Clear background
            ClearBackground(DARKGRAY);
            
            // Draw title
            DrawText(b"Rust Calculator\0".as_ptr() as *const i8, 120, 20, 24, RAYWHITE);
            
            // Draw display background
            DrawRectangle(20, 60, 340, 50, RAYWHITE);
            DrawRectangleLines(20, 60, 340, 50, BLACK);
            
            // Draw display text (right-aligned)
            let display_text = if calculator.display.len() > 15 {
                // Truncate very long numbers
                format!("{}\0", &calculator.display[calculator.display.len()-15..])
            } else {
                format!("{}\0", calculator.display)
            };
            
            let text_width = MeasureText(display_text.as_ptr() as *const i8, 20);
            let text_x = 20 + 340 - text_width - 10; // Right align with padding
            DrawText(display_text.as_ptr() as *const i8, text_x, 70, 20, BLACK);
            
            // Draw buttons
            for &(symbol, x, y) in &buttons {
                // Different colors for different button types
                let color = match symbol {
                    '0'..='9' | '.' => LIGHTGRAY,
                    '+' | '-' | '*' | '/' | '=' => ORANGE,
                    'C' | 'D' => MAROON,
                    _ => LIGHTGRAY,
                };
                
                DrawRectangle(x, y, button_width, button_height, color);
                DrawRectangleLines(x, y, button_width, button_height, BLACK);
                
                let button_text = format!("{}\0", symbol);
                let text_width = MeasureText(button_text.as_ptr() as *const i8, 20);
                let text_x = x + (button_width - text_width) / 2;
                let text_y = y + (button_height - 20) / 2;
                
                DrawText(button_text.as_ptr() as *const i8, text_x, text_y, 20, BLACK);
            }
            
            // Draw instructions
            DrawText(b"Click buttons to calculate\0".as_ptr() as *const i8, 20, 500, 16, RAYWHITE);
            
            EndDrawing();
        }

        CloseWindow();
    }
}
