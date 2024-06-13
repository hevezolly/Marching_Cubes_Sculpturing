use core::{OpenglAlias, GL};
use std::panic::UnwindSafe;
use std::ptr::null;
use std::{sync::mpsc::Receiver, time::Instant};

use egui_glfw_gl::{EguiInputState, gl};
use egui_glfw_gl::glfw::{Glfw, Window, WindowEvent, self, Context, Key, Action};
use egui_glfw_gl::egui::{self, Rect, Pos2, vec2};

use self::app_logick::{ExecutrionLogick, Parameters};

mod app_logick;
mod cunks;
mod support;

pub struct EguiContext {
    pub painter: egui_glfw_gl::Painter,
    pub egui_ctx: egui::CtxRef,
    pub egui_input_state: EguiInputState,
    pub native_pixels_per_point: f32,
}

impl EguiContext {
    fn new(window: &mut Window, width: u32, height: u32) -> EguiContext {
        let painter = egui_glfw_gl::Painter::new(window, width, height);
        let egui_ctx = egui::CtxRef::default();

        

        let (width, height) = window.get_framebuffer_size();
        // let native_pixels_per_point = window.get_content_scale().0;
        // dbg!(window.get_content_scale());

        let egui_input_state = egui_glfw_gl::EguiInputState::new(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::new(0f32, 0f32),
                vec2(width as f32, height as f32),
            )),
            pixels_per_point: Some(1.),
            ..Default::default()
        });

        EguiContext { painter, egui_ctx, egui_input_state, native_pixels_per_point: 1. }
    }
}


pub struct Application {
    pub glfw: Glfw,
    pub window: Window,
    pub events: Receiver<(f64, WindowEvent)>,
    pub egui: EguiContext,
    pub start_time: Instant,
    execution_logick: Option<ExecutrionLogick>,
}


fn configure_glfw(glfw: &mut Glfw) {
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::Resizable(false));
}

fn configure_window(window: &mut Window) {
    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.make_current();
}

extern "system"
fn message_callback(_source: gl::types::GLenum,
    err_type: gl::types::GLenum,
    _id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void)
{
    let str = unsafe{std::ffi::CStr::from_ptr(message)}.to_str().unwrap_or("unreadable error");
    println!("GL Callback. type: {}, severity: {}, message: {}", err_type, severity, str);
}

impl Application {
    pub fn new(width: u32, height: u32) -> Application {
        
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        configure_glfw(&mut glfw);

        let (mut window, events) = glfw.create_window(
            width, 
            height, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        configure_window(&mut window);
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));


        let start_time = Instant::now();

        let egui = EguiContext::new(&mut window, width, height);

        Application { glfw, window, events, egui, start_time, execution_logick: None }
    }

    pub fn init_debugging(&self) {
        GL!(gl::Enable(gl::DEBUG_OUTPUT));
        GL!(gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
        GL!(gl::DebugMessageCallback(Some(message_callback), null()));
        GL!(gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, null(), gl::FALSE));
        // GL!(gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, null(), gl::TRUE));
        
        GL!(gl::DebugMessageControl(gl::DONT_CARE, gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR, gl::DONT_CARE, 0, null(), gl::TRUE));
        GL!(gl::DebugMessageControl(gl::DONT_CARE, gl::DEBUG_TYPE_ERROR, gl::DONT_CARE, 0, null(), gl::TRUE));

    }

    pub fn init_logick(&mut self) {
        let logic = ExecutrionLogick::init();
        self.execution_logick = Some(logic)
    }

    pub fn begin_frame(&mut self) {
        self.egui.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui.egui_ctx.begin_frame(self.egui.egui_input_state.input.take());
        self.egui.egui_input_state.input.pixels_per_point = Some(self.egui.native_pixels_per_point);
        if let Some(logic) = &mut self.execution_logick {
            logic.on_frame_begin();
        }
    }

    pub fn update(&mut self) {
        if let Some(logic) = &mut self.execution_logick {
            logic.update(&self.egui.egui_ctx)
        }
    } 

    pub fn draw_frame(&mut self) {
        if let Some(logic) = &mut self.execution_logick {
            let (width, height) = self.window.get_framebuffer_size();
            logic.draw(Parameters { width, height });
        }
    }

    pub fn end_frame(&mut self) {
        let (
            platform_output,
            shapes,
        ) = self.egui.egui_ctx.end_frame();

        if !platform_output.copied_text.is_empty() {
            egui_glfw_gl::copy_to_clipboard(&mut self.egui.egui_input_state, platform_output.copied_text);
        }

        let clipped_shapes = self.egui.egui_ctx.tessellate(shapes);
        self.egui.painter.paint_jobs(None, 
            clipped_shapes, 
            self.egui.egui_ctx.texture().as_ref(),
             1.);


        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => {egui_glfw_gl::handle_event(event, &mut self.egui.egui_input_state);}
            }
        }

        if let Some(logic) = &mut self.execution_logick {
            logic.on_frame_end();
        }


        self.window.swap_buffers();
        self.glfw.poll_events();
    }

    pub fn draw_ui(&mut self) {
        if let Some(logic) = &mut self.execution_logick {
            let (width, height) = self.window.get_framebuffer_size();
            logic.draw_ui(&self.egui.egui_ctx, Parameters { width, height });
        }
    }
}