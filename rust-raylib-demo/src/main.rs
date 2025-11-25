// Raylib + Fine-grained Reactivity — Complete Rewrite
use raylib_ffi::*;
use raylib_ffi::colors::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::ffi::CString;

// ============== REACTIVITY PRIMITIVES ==============

/// A reactive signal that notifies subscribers when its value changes
#[derive(Clone)]
pub struct Signal<T> {
    value: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<Vec<Rc<dyn Fn()>>>>,
}

impl<T: Clone + std::fmt::Debug> Signal<T> {
    pub fn new(value: T) -> Self {
        println!("[Signal] Creating new signal with value: {:?}", value);
        Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    pub fn set(&self, new_value: T) {
        println!("[Signal] Setting value to: {:?}", new_value);
        *self.value.borrow_mut() = new_value;
        self.notify();
    }

    fn notify(&self) {
        let subs = self.subscribers.borrow();
        println!("[Signal] Notifying {} subscribers", subs.len());
        for (i, callback) in subs.iter().enumerate() {
            println!("[Signal] Running subscriber #{}", i + 1);
            callback();
        }
    }

    pub fn subscribe(&self, callback: impl Fn() + 'static) {
        println!("[Signal] Adding subscriber (total will be: {})", self.subscribers.borrow().len() + 1);
        self.subscribers.borrow_mut().push(Rc::new(callback));
    }
}

/// A computed value that automatically updates when its dependencies change
pub struct Computed<T> {
    value: Rc<RefCell<T>>,
}

impl<T: Clone + std::fmt::Debug + 'static> Computed<T> {
    pub fn new<U, F>(source: &Signal<U>, compute: F) -> Self
    where
        U: Clone + std::fmt::Debug + 'static,
        F: Fn(U) -> T + 'static,
    {
        println!("[Computed] Creating computed value");
        
        // Compute initial value
        let initial = compute(source.get());
        println!("[Computed] Initial value: {:?}", initial);
        let value_rc = Rc::new(RefCell::new(initial));

        // Set up reactive dependency
        let value_clone = value_rc.clone();
        let source_clone = source.clone();
        let compute_rc = Rc::new(compute);
        
        source.subscribe(move || {
            println!("[Computed Effect] Triggered!");
            let source_val = source_clone.get();
            println!("[Computed Effect] Source value: {:?}", source_val);
            let new_val = compute_rc(source_val);
            println!("[Computed Effect] Computed value: {:?}", new_val);
            *value_clone.borrow_mut() = new_val;
        });

        println!("[Computed] Subscription complete");

        Self { value: value_rc }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }
}

// ============== RAYLIB HELPERS ==============

fn cstr(s: &str) -> *const std::os::raw::c_char {
    CString::new(s).unwrap().into_raw()
}

fn draw_text_centered(text: &str, cx: i32, y: i32, size: i32, color: Color) {
    let c_text = cstr(text);
    let w = unsafe { MeasureText(c_text, size) };
    unsafe { DrawText(c_text, cx - w / 2, y, size, color) };
}

// ============== MAIN ==============

fn main() {
    unsafe {
        InitWindow(400, 300, cstr("Reactive Counter"));
        SetTargetFPS(60);

        println!("\n=== INITIALIZING REACTIVE SYSTEM ===");
        let counter = Signal::new(0);
        let doubled = Computed::new(&counter, |v| {
            println!("[Compute Fn] {} * 2 = {}", v, v * 2);
            v * 2
        });
        println!("=== INITIALIZATION COMPLETE ===\n");

        let inc_rect = (50, 180, 100, 50);
        let reset_rect = (250, 180, 100, 50);

        while !WindowShouldClose() {
            // Input handling
            if IsMouseButtonPressed(0) {
                let mouse = GetMousePosition();
                let (mx, my) = (mouse.x as i32, mouse.y as i32);

                // Increment button
                let (x, y, w, h) = inc_rect;
                if mx >= x && mx < x + w && my >= y && my < y + h {
                    println!("\n=== INCREMENT CLICKED ===");
                    let new_val = counter.get() + 1;
                    counter.set(new_val);
                    println!("Counter is now: {}", counter.get());
                    println!("Doubled is now: {}", doubled.get());
                    println!("=== INCREMENT COMPLETE ===\n");
                }

                // Reset button
                let (x, y, w, h) = reset_rect;
                if mx >= x && mx < x + w && my >= y && my < y + h {
                    println!("\n=== RESET CLICKED ===");
                    counter.set(0);
                    println!("Counter is now: {}", counter.get());
                    println!("Doubled is now: {}", doubled.get());
                    println!("=== RESET COMPLETE ===\n");
                }
            }

            // Rendering
            BeginDrawing();
            ClearBackground(DARKGRAY);

            // Title
            draw_text_centered("Reactive Counter", 200, 30, 24, RAYWHITE);
            
            // Display values
            let counter_val = counter.get();
            let doubled_val = doubled.get();
            draw_text_centered(&format!("Counter: {}", counter_val), 200, 90, 32, YELLOW);
            draw_text_centered(&format!("Doubled: {}", doubled_val), 200, 140, 32, GREEN);

            // Increment button
            let (x, y, w, h) = inc_rect;
            DrawRectangle(x, y, w, h, BLUE);
            DrawRectangleLines(x, y, w, h, BLACK);
            draw_text_centered("Increment", x + w / 2, y + h / 2 - 10, 18, RAYWHITE);

            // Reset button
            let (x, y, w, h) = reset_rect;
            DrawRectangle(x, y, w, h, MAROON);
            DrawRectangleLines(x, y, w, h, BLACK);
            draw_text_centered("Reset", x + w / 2, y + h / 2 - 10, 18, RAYWHITE);

            EndDrawing();
        }

        CloseWindow();
    }
}
