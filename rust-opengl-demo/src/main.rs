extern crate glfw;
extern crate gl;

use glfw::{Action, Context, Key};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Create window with OpenGL 3.3 core profile
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(800, 600, "GLFW + Rust", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // Main loop
    while !window.should_close() {
        // Input
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.swap_buffers();
    }
}

