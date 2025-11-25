#![windows_subsystem = "windows"]

use winsafe::{self as w, gui, prelude::*};

fn main() {
    if let Err(e) = MyWindow::create_and_run() {
        eprintln!("{}", e);
    }
}

#[derive(Clone)]
pub struct MyWindow {
    wnd: gui::WindowMain,   // responsible for managing the window
    btn_hello: gui::Button, // a button
}

impl MyWindow {
    pub fn create_and_run() -> w::AnyResult<i32> {
        let wnd = gui::WindowMain::new(
            // instantiate the window manager
            gui::WindowMainOpts {
                title: "My window title",
                size: gui::dpi(300, 150),
                ..Default::default() // leave all other options as default
            },
        );

        let btn_hello = gui::Button::new(
            &wnd, // the window manager is the parent of our button
            gui::ButtonOpts {
                text: "&Click me",
                position: gui::dpi(20, 20),
                ..Default::default()
            },
        );

        let new_self = Self { wnd, btn_hello };
        new_self.events(); // attach our events

        new_self.wnd.run_main(None) // show the main window; will block until closed
    }

    fn events(&self) {
        let wnd = self.wnd.clone(); // clone so it can be passed into the closure
        self.btn_hello.on().bn_clicked(move || {
            wnd.hwnd().SetWindowText("Hello, world!")?; // call native Windows API
            Ok(())
        });
    }
}
