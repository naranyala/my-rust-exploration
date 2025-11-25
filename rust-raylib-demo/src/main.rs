use raylib_ffi::*;
use raylib_ffi::colors::*;
use std::ffi::CString;

const MOUSE_BUTTON_LEFT: i32 = 0;
const KEY_ESCAPE: i32 = 256;

struct AccordionSection {
    title: String,
    content: String,
    is_open: bool,
}

impl AccordionSection {
    fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            is_open: false,
        }
    }
}

fn main() {
    unsafe {
        let title = CString::new("Accordion Demo").unwrap();
        InitWindow(500, 600, title.as_ptr());
        SetTargetFPS(60);

        let mut sections = vec![
            AccordionSection::new("Section 1", "This is the content of section 1."),
            AccordionSection::new("Section 2", "Here lies section 2’s content."),
            AccordionSection::new("Section 3", "Section 3 has some text too."),
        ];

        let section_x = 50;
        let section_y = 80;
        let section_width = 400;
        let header_height = 40;
        let content_height = 80;

        while !WindowShouldClose() {
            // Handle mouse clicks
            if IsMouseButtonPressed(MOUSE_BUTTON_LEFT) {
                let mouse_pos = GetMousePosition();
                let mx = mouse_pos.x as i32;
                let my = mouse_pos.y as i32;

                let mut current_y = section_y;
                for section in sections.iter_mut() {
                    // Header area
                    if mx >= section_x && mx <= section_x + section_width &&
                       my >= current_y && my <= current_y + header_height {
                        // Toggle open/close
                        section.is_open = !section.is_open;
                    }
                    current_y += header_height;
                    if section.is_open {
                        current_y += content_height;
                    }
                }
            }

            // Escape closes all
            if IsKeyPressed(KEY_ESCAPE) {
                for section in sections.iter_mut() {
                    section.is_open = false;
                }
            }

            BeginDrawing();
            ClearBackground(RAYWHITE);

            let demo_title = CString::new("Accordion Demo (click headers)").unwrap();
            DrawText(demo_title.as_ptr(), 50, 30, 24, DARKGRAY);

            // Draw sections
            let mut current_y = section_y;
            for section in &sections {
                // Header
                let header_color = if section.is_open { SKYBLUE } else { LIGHTGRAY };
                DrawRectangle(section_x, current_y, section_width, header_height, header_color);
                DrawRectangleLines(section_x, current_y, section_width, header_height, DARKGRAY);

                let title_text = CString::new(section.title.clone()).unwrap();
                DrawText(title_text.as_ptr(), section_x + 10, current_y + 10, 20, BLACK);

                current_y += header_height;

                // Content if open
                if section.is_open {
                    DrawRectangle(section_x, current_y, section_width, content_height, WHITE);
                    DrawRectangleLines(section_x, current_y, section_width, content_height, GRAY);

                    let content_text = CString::new(section.content.clone()).unwrap();
                    DrawText(content_text.as_ptr(), section_x + 10, current_y + 10, 18, DARKGREEN);

                    current_y += content_height;
                }
            }

            EndDrawing();
        }

        CloseWindow();
    }
}

