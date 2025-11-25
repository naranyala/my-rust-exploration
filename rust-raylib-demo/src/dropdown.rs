use raylib_ffi::*;
use raylib_ffi::colors::*;
use std::ffi::CString;

const MOUSE_BUTTON_LEFT: i32 = 0;
const KEY_ESCAPE: i32 = 256;
const KEY_BACKSPACE: i32 = 259;


struct FuzzyDropdown {
    items: Vec<String>,
    filtered_items: Vec<(String, usize)>, // (item, original_index)
    search_text: String,
    is_open: bool,
    selected_index: Option<usize>,
    hovered_index: Option<usize>,
    item_offset: i32, // scroll offset in item units (number of items), not pixels
}

impl FuzzyDropdown {
    fn new(items: Vec<String>) -> Self {
        let filtered = items
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();

        Self {
            items,
            filtered_items: filtered,
            search_text: String::new(),
            is_open: false,
            selected_index: None,
            hovered_index: None,
            item_offset: 0,
        }
    }

    fn fuzzy_match(query: &str, target: &str) -> Option<i32> {
        if query.is_empty() {
            return Some(0);
        }

        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        let mut query_chars = query_lower.chars().peekable();
        let mut score = 0;
        let mut last_match_index = 0;
        let mut consecutive_matches = 0;

        for (i, target_char) in target_lower.chars().enumerate() {
            if let Some(&query_char) = query_chars.peek() {
                if query_char == target_char {
                    query_chars.next();

                    // Bonus for consecutive matches
                    if i == last_match_index + 1 {
                        consecutive_matches += 1;
                        score += 5 + consecutive_matches;
                    } else {
                        consecutive_matches = 0;
                        score += 1;
                    }

                    // Bonus for matching at word start
                    if i == 0 || target_lower.chars().nth(i - 1).unwrap().is_whitespace() {
                        score += 10;
                    }

                    last_match_index = i;
                }
            } else {
                break;
            }
        }

        if query_chars.peek().is_none() {
            Some(score)
        } else {
            None
        }
    }

    fn update_filter(&mut self) {
        // Build a scored list, sort, then strip scores
        let mut scored: Vec<(String, usize, i32)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| Self::fuzzy_match(&self.search_text, item).map(|score| (item.clone(), i, score)))
            .collect();

        scored.sort_by(|a, b| b.2.cmp(&a.2)); // descending by score

        self.filtered_items = scored.into_iter().map(|(item, idx, _)| (item, idx)).collect();

        self.item_offset = 0;
        self.hovered_index = None;
    }

    fn add_char(&mut self, c: char) {
        self.search_text.push(c);
        self.update_filter();
    }

    fn backspace(&mut self) {
        self.search_text.pop();
        self.update_filter();
    }

    fn clear_search(&mut self) {
        self.search_text.clear();
        self.update_filter();
    }

    fn select_item(&mut self, filtered_index: usize) {
        if filtered_index < self.filtered_items.len() {
            self.selected_index = Some(self.filtered_items[filtered_index].1);
            self.is_open = false;
            self.clear_search();
        }
    }

    fn get_selected_text(&self) -> String {
        self.selected_index
            .and_then(|i| self.items.get(i))
            .cloned()
            .unwrap_or_else(|| "Select an item...".to_string())
    }
}

fn main() {
    unsafe {
        // Window title: use a static C string for safety
        let title = CString::new("Fuzzy Search Dropdown").unwrap();
        InitWindow(500, 600, title.as_ptr());
        SetTargetFPS(60);

        // Sample data
        let items = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
            "Date".to_string(),
            "Elderberry".to_string(),
            "Fig".to_string(),
            "Grape".to_string(),
            "Honeydew".to_string(),
            "Kiwi".to_string(),
            "Lemon".to_string(),
            "Mango".to_string(),
            "Nectarine".to_string(),
            "Orange".to_string(),
            "Papaya".to_string(),
            "Raspberry".to_string(),
            "Strawberry".to_string(),
            "Tangerine".to_string(),
            "Watermelon".to_string(),
            "Blueberry".to_string(),
            "Blackberry".to_string(),
        ];

        let mut dropdown = FuzzyDropdown::new(items);

        let dropdown_x = 50;
        let dropdown_y = 100;
        let dropdown_width = 400;
        let dropdown_height = 40;
        let max_visible_items = 8;
        let item_height = 35;

        // Pre-create static texts
        let title_text = CString::new("Fuzzy Search Dropdown Demo").unwrap();
        let subtitle_text = CString::new("Click to open, type to filter").unwrap();

        while !WindowShouldClose() {
            // Handle mouse input
            if IsMouseButtonPressed(MOUSE_BUTTON_LEFT) {
                let mouse_pos = GetMousePosition();
                let mx = mouse_pos.x as i32;
                let my = mouse_pos.y as i32;

                // Header toggle
                if mx >= dropdown_x
                    && mx <= dropdown_x + dropdown_width
                    && my >= dropdown_y
                    && my <= dropdown_y + dropdown_height
                {
                    dropdown.is_open = !dropdown.is_open;
                    if !dropdown.is_open {
                        dropdown.clear_search();
                    }
                } else if dropdown.is_open {
                    // Click in list
                    let list_y = dropdown_y + dropdown_height + 5;
                    let total_items = dropdown.filtered_items.len() as i32;
                    let visible_items = total_items.min(max_visible_items as i32);
                    let list_height = visible_items * item_height;

                    // Determine clicked visible index
                    if mx >= dropdown_x && mx <= dropdown_x + dropdown_width && my >= list_y && my <= list_y + list_height {
                        let relative_y = my - list_y;
                        let vis_index = relative_y / item_height;
                        let actual_index = vis_index + dropdown.item_offset;
                        if actual_index >= 0 && (actual_index as usize) < dropdown.filtered_items.len() {
                            dropdown.select_item(actual_index as usize);
                        }
                    } else {
                        // Click outside: close
                        dropdown.is_open = false;
                        dropdown.clear_search();
                    }
                }
            }

            // Handle mouse hover
            if dropdown.is_open {
                let mouse_pos = GetMousePosition();
                let mx = mouse_pos.x as i32;
                let my = mouse_pos.y as i32;
                let list_y = dropdown_y + dropdown_height + 5;

                dropdown.hovered_index = None;

                let total_items = dropdown.filtered_items.len() as i32;
                let visible_items = total_items.min(max_visible_items as i32);
                let list_height = visible_items * item_height;

                if mx >= dropdown_x && mx <= dropdown_x + dropdown_width && my >= list_y && my <= list_y + list_height {
                    let relative_y = my - list_y;
                    let vis_index = relative_y / item_height;
                    let actual_index = vis_index + dropdown.item_offset;
                    if actual_index >= 0 && (actual_index as usize) < dropdown.filtered_items.len() {
                        dropdown.hovered_index = Some(actual_index as usize);
                    }
                }
            }

            // Handle keyboard input
            if dropdown.is_open {
                let key = GetCharPressed();
                if key > 0 {
                    if let Some(c) = char::from_u32(key as u32) {
                        if c.is_alphanumeric() || c.is_whitespace() {
                            dropdown.add_char(c);
                        }
                    }
                }

                if IsKeyPressed(KEY_BACKSPACE as i32) {
                    dropdown.backspace();
                }

                if IsKeyPressed(KEY_ESCAPE as i32) {
                    dropdown.is_open = false;
                    dropdown.clear_search();
                }

                // Handle scroll (item-based)
                let wheel = GetMouseWheelMove();
                if wheel != 0.0 {
                    let total_items = dropdown.filtered_items.len() as i32;
                    let max_offset = (total_items - max_visible_items as i32).max(0);
                    // Scroll by whole items per wheel notch
                    dropdown.item_offset -= wheel.round() as i32;
                    if dropdown.item_offset < 0 {
                        dropdown.item_offset = 0;
                    } else if dropdown.item_offset > max_offset {
                        dropdown.item_offset = max_offset;
                    }
                }
            }

            BeginDrawing();
            ClearBackground(RAYWHITE);

            // Draw title
            DrawText(title_text.as_ptr(), 50, 30, 28, DARKGRAY);
            DrawText(subtitle_text.as_ptr(), 50, 65, 16, GRAY);

            // Draw dropdown header
            let header_color = if dropdown.is_open { SKYBLUE } else { LIGHTGRAY };
            DrawRectangle(dropdown_x, dropdown_y, dropdown_width, dropdown_height, header_color);
            DrawRectangleLines(dropdown_x, dropdown_y, dropdown_width, dropdown_height, DARKGRAY);

            // Draw selected text or search text (CString each frame; lifetime covers the call)
            let display_text_str = if dropdown.is_open && !dropdown.search_text.is_empty() {
                dropdown.search_text.clone()
            } else {
                dropdown.get_selected_text()
            };
            let display_text = CString::new(display_text_str).unwrap();
            DrawText(display_text.as_ptr(), dropdown_x + 10, dropdown_y + 10, 20, BLACK);

            // Draw dropdown arrow (ASCII-safe)
            let arrow_char = if dropdown.is_open { "^" } else { "v" };
            let arrow = CString::new(arrow_char).unwrap();
            DrawText(
                arrow.as_ptr(),
                dropdown_x + dropdown_width - 30,
                dropdown_y + 10,
                20,
                DARKGRAY,
            );

            // Draw dropdown list
            if dropdown.is_open {
                let list_y = dropdown_y + dropdown_height + 5;
                let total_items = dropdown.filtered_items.len() as i32;
                let visible_items = total_items.min(max_visible_items as i32);
                let list_height = visible_items * item_height;

                DrawRectangle(dropdown_x, list_y, dropdown_width, list_height, WHITE);
                DrawRectangleLines(dropdown_x, list_y, dropdown_width, list_height, DARKGRAY);

                // Draw visible items with correct offset
                for vis_i in 0..visible_items {
                    let actual_index = (vis_i + dropdown.item_offset) as usize;
                    if actual_index >= dropdown.filtered_items.len() {
                        break;
                    }

                    let item_y = list_y + (vis_i * item_height);

                    let is_hovered = dropdown.hovered_index == Some(actual_index);
                    if is_hovered {
                        DrawRectangle(dropdown_x, item_y, dropdown_width, item_height, SKYBLUE);
                    }

                    let item_text = CString::new(dropdown.filtered_items[actual_index].0.clone()).unwrap();
                    DrawText(item_text.as_ptr(), dropdown_x + 10, item_y + 8, 18, BLACK);
                }

                // Draw search hint (ensure ASCII)
                if dropdown.search_text.is_empty() {
                    let hint = CString::new(format!(
                        "Type to search... ({} items)",
                        dropdown.filtered_items.len()
                    ))
                    .unwrap();
                    DrawText(hint.as_ptr(), dropdown_x + 10, list_y + list_height + 10, 14, GRAY);
                } else {
                    let hint = CString::new(format!("Found {} items", dropdown.filtered_items.len())).unwrap();
                    DrawText(hint.as_ptr(), dropdown_x + 10, list_y + list_height + 10, 14, DARKGREEN);
                }
            }

            // Draw selected item info
            if let Some(idx) = dropdown.selected_index {
                let info = CString::new(format!("Selected: {} (index: {})", dropdown.items[idx], idx)).unwrap();
                DrawText(info.as_ptr(), 50, 500, 18, DARKPURPLE);
            }

            EndDrawing();
        }

        CloseWindow();
    }
}

