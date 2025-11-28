#![no_std]
#![no_main]

// ---------- C bindings ----------
#[link(name = "raylib")]
#[link(name = "m")]
#[link(name = "pthread")]
#[link(name = "dl")]
#[link(name = "rt")]
#[link(name = "GL")]
#[link(name = "c")]
extern "C" {
    fn InitWindow(width: i32, height: i32, title: *const u8);
    fn CloseWindow();
    fn WindowShouldClose() -> bool;
    fn SetTargetFPS(fps: i32);

    fn BeginDrawing();
    fn EndDrawing();
    fn ClearBackground(color: Color);
    fn DrawText(text: *const u8, posX: i32, posY: i32, fontSize: i32, color: Color);
    fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color);

    // mouse
    fn IsMouseButtonPressed(button: i32) -> bool;
    fn GetMouseX() -> i32;
    fn GetMouseY() -> i32;
}

const MOUSE_LEFT: i32 = 0;

// ---------- colors ----------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color {
    r: u8, g: u8, b: u8, a: u8,
}
const RAYWHITE: Color = Color{r:245,g:245,b:245,a:255};
const BLACK:    Color = Color{r:0,g:0,b:0,a:255};
const RED:      Color = Color{r:230,g:41,b:55,a:255};
const GREEN:    Color = Color{r:0,g:228,b:48,a:255};
const BLUE:     Color = Color{r:0,g:121,b:241,a:255};

// ---------- helper: u32 -> &str ----------
fn u32_to_str(mut n: u32) -> &'static str {
    static mut BUF: [u8; 11] = [0; 11]; // 10 digits + NUL
    unsafe {
        let mut i = 9;
        if n == 0 { BUF[i] = b'0'; } else {
            while n > 0 { BUF[i] = (n % 10) as u8 + b'0'; n /= 10; i -= 1; }
            i += 1;
        }
        BUF[10] = 0;
        core::str::from_utf8_unchecked(&BUF[i..11])
    }
}

// ---------- entry ----------
#[no_mangle]
pub unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    InitWindow(800, 450, b"GUI Counter\0".as_ptr());
    SetTargetFPS(60);

    let mut counter: u32 = 1;

    // button layouts
    let btn_plus = ( 80, 200, 140, 60 ); // x,y,w,h
    let btn_mul   = (250, 200, 140, 60 );

    while !WindowShouldClose() {
        // ---- click detection ----
        if IsMouseButtonPressed(MOUSE_LEFT) {
            let mx = GetMouseX();
            let my = GetMouseY();

            // +1 button
            if mx >= btn_plus.0 && mx <= btn_plus.0 + btn_plus.2 &&
               my >= btn_plus.1 && my <= btn_plus.1 + btn_plus.3 {
                counter = counter.wrapping_add(1);
            }
            // ×2 button
            if mx >= btn_mul.0 && mx <= btn_mul.0 + btn_mul.2 &&
               my >= btn_mul.1 && my <= btn_mul.1 + btn_mul.3 {
                counter = counter.wrapping_mul(2);
            }
        }

        // ---- draw ----
        BeginDrawing();
        ClearBackground(RAYWHITE);

        // title & counter
        DrawText(b"Counter:\0".as_ptr(), 80, 80, 40, BLACK);
        DrawText(u32_to_str(counter).as_ptr(), 280, 82, 40, RED);

        // +1 button
        DrawRectangle(btn_plus.0, btn_plus.1, btn_plus.2, btn_plus.3, BLUE);
        DrawText(b"+1\0".as_ptr(), btn_plus.0 + 50, btn_plus.1 + 20, 30, RAYWHITE);

        // ×2 button
        DrawRectangle(btn_mul.0, btn_mul.1, btn_mul.2, btn_mul.3, GREEN);
        DrawText(b"x2\0".as_ptr(), btn_mul.0 + 50, btn_mul.1 + 20, 30, RAYWHITE);

        EndDrawing();
    }

    CloseWindow();
    0
}

// ---------- no-std stubs ----------
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
