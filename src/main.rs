// #![windows_subsystem = "windows"]
pub mod application;
pub mod algorithms;


use std::{fs::File, io::Write, panic};

use application::Application;


const WIN_WIDTH: u32 = 1024;
const WIN_HEIGHT: u32 = 980;

#[macro_use]
extern crate macros;

fn main() {

    let mut app = Application::new(WIN_WIDTH, WIN_HEIGHT);    

    app.init_debugging();
    app.init_logick();

    while !app.window.should_close() {
        app.begin_frame();

        app.draw_frame();
        
        app.draw_ui();
        
        
        app.end_frame();
    }
}
