use core::context::synchronization_context::SynchronizationContext;
use core::GL;
use std::default;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use egui_glfw_gl::egui::Color32;
use egui_glfw_gl::egui::CtxRef;
use egui_glfw_gl::egui::InputState;
use egui_glfw_gl::egui::Pos2;
use egui_glfw_gl::egui::Stroke;
use egui_glfw_gl::gl;
use egui_glfw_gl::egui;
use glam::vec4;
use glam::BVec3;
use glam::IVec3;
use glam::Quat;
use glam::Vec2;
use glam::Vec3;
use glam::vec2;
use glam::vec3;
use glam::Vec4;
use glam::Vec4Swizzles;

use crate::algorithms::camera::Camera;
use crate::algorithms::camera::perspective::PerspectiveCamera;
use crate::application::support::brush::circle_bruhs::CircleBrush;
use crate::application::support::brush::Brush;
use crate::application::support::brush::BrushApplicationParameters;

use super::cunks::field::Field;
use super::support::bounds::Bounds;
use super::support::brush::circle_bruhs::InverseCircleBrush;
use super::support::brush::flatten_brush::FlattenBrush;
use super::support::brush::BrushSettings;
use super::support::camera_ref::CameraRef;
use super::support::debugger::Debugger;
use super::support::shaders::shaders_loader::ShaderStorage;


struct DebugSettings {
    debug: bool,
    draw_model: bool,
    draw_sdf: bool,
    parity: BVec3,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self { debug: false, draw_model: true, draw_sdf: true, 
            parity: BVec3 { x: true, y: true, z: true } }
    }
}
pub struct ExecutrionLogick {
    // command_buffer: Buffer,
    camera: PerspectiveCamera,
    sync_context: SynchronizationContext,
    delta_time_ratio: f32,
    programm_storage: ShaderStorage,
    debugger: Debugger,
    field: Field,
    slice: f32,
    brushes: Vec<Arc<Mutex<dyn BrushSettings>>>,
    selected_brush: usize,
    debug: DebugSettings,
    instant: Instant,
    // image: Image
    // programm: ShaderProgramm,
}

pub const fn ceil_div(val: usize, divider: usize) -> usize {
    let div = val / divider;
    if val % divider > 0 {
        div + 1
    }
    else {
        div
    }
}

pub const BLOCKY: bool = false;
pub const FLAT_SHADING: bool = false;
pub const CHUNK_SIZE: i32 = 48;
pub const NUM_OF_CUBES: IVec3 = IVec3 { x: CHUNK_SIZE, y: CHUNK_SIZE, z: CHUNK_SIZE };

const FPS: usize = 60;

const FRAME_TIME: Duration = Duration::from_millis(1000 / FPS as u64);

macro_rules! brush_vec {
    [$($y:expr),+$(,)?] => (
        vec![$(Arc::new(Mutex::new($y)) as Arc<Mutex<dyn BrushSettings>>),+]
    )
}

impl ExecutrionLogick {
    pub fn init() -> ExecutrionLogick {
        
        // let vertex_data = model_vertex_buffer.get_all_data();
        // dbg!(vertex_data);
        
        // let bounds = Bounds::min_max(IVec3::ZERO, IVec3::ONE);

        // for c in bounds.iterate_cords() {
        //     dbg!(c);
        // }

        let mut camera = PerspectiveCamera::new(60., 0.01, 100.);
        camera.transform.set_position(vec3(0.5, 0.5, -1.));
        let sync_context = SynchronizationContext::new();
        let programm_storage = ShaderStorage::new();


        let debugger = Debugger::new();

        let field = Field::new(sync_context.clone(), programm_storage.clone(), debugger.clone());

        let brushes = brush_vec![
            CircleBrush::new(programm_storage.clone()),
            InverseCircleBrush::new(programm_storage.clone()),
            
            // not working properly :(
            FlattenBrush::new(sync_context.clone(), programm_storage.clone()),
        ];

        // let brushes = brush_vec!(CircleBrush::new(programm_storage.clone()),
        //         InverseCircleBrush::new(programm_storage.clone()),
        // );
        ExecutrionLogick { 

            debugger,
            camera, 
            delta_time_ratio: 1.,
            slice: 0.,
            field,
            brushes,
            selected_brush: 0,
            // ao_upper_edge: 0.1,
            debug: Default::default(),
            sync_context,
            instant: Instant::now(),
            programm_storage,
        }

    }

    pub fn on_frame_begin(&mut self) {
        self.instant = Instant::now();
    }

    pub fn on_frame_end(&mut self) {
        GL!(gl::Finish());
        let elapsed = self.instant.elapsed();
        println!("frame_time: {}ms", elapsed.as_millis());
        if elapsed < FRAME_TIME {
            sleep(FRAME_TIME - elapsed);
            self.delta_time_ratio = 1.
        }
        else {
            self.delta_time_ratio = (elapsed.as_nanos() / FRAME_TIME.as_nanos()) as f32;
        }
    }


    pub fn update(&mut self, egui_ctx: &CtxRef) {
        const speed: f32 = 0.025;
        const ang_speed: f32 = 0.025;

        let input = egui_ctx.input();

        let f = self.camera.transform.forward();
        let proj_forward = (f - f.dot(Vec3::Y) * Vec3::Y).normalize();

        if input.key_down(egui::Key::S) {
            let pos = self.camera.transform.position() + proj_forward * -speed;
            self.camera.transform.set_position(pos);
        }
        if input.key_down(egui::Key::W) {
            let pos = self.camera.transform.position() + proj_forward * speed;
            self.camera.transform.set_position(pos);
        }
        if input.key_down(egui::Key::A) {
            let pos = self.camera.transform.position() + self.camera.transform.right() * -speed;
            self.camera.transform.set_position(pos);
        }
        if input.key_down(egui::Key::D) {
            let pos = self.camera.transform.position() + self.camera.transform.right() * speed;
            self.camera.transform.set_position(pos);
        }
        if input.key_down(egui::Key::Space) {
            let pos = self.camera.transform.position() + Vec3::Y * speed;
            self.camera.transform.set_position(pos);
        }
        if input.key_down(egui::Key::Z) {
            let pos = self.camera.transform.position() + Vec3::Y * -speed;
            self.camera.transform.set_position(pos);
        }

        if input.key_down(egui::Key::ArrowUp) {
            let rot = Quat::from_axis_angle(self.camera.transform.right(), ang_speed) * self.camera.transform.rotation();
            self.camera.transform.set_rotation(rot);
        }
        if input.key_down(egui::Key::ArrowDown) {
            let rot = Quat::from_axis_angle(self.camera.transform.right(), -ang_speed) * self.camera.transform.rotation();
            self.camera.transform.set_rotation(rot);
        }
        if input.key_down(egui::Key::ArrowLeft) {
            let rot = Quat::from_axis_angle(Vec3::Y, -ang_speed) * self.camera.transform.rotation();
            self.camera.transform.set_rotation(rot);
        }
        if input.key_down(egui::Key::ArrowRight) {
            let rot = Quat::from_axis_angle(Vec3::Y, ang_speed) * self.camera.transform.rotation();
            self.camera.transform.set_rotation(rot);
        }
        

        if !egui_ctx.is_pointer_over_area() && 
            input.pointer.button_down(egui::PointerButton::Primary) {

            let hit = if let Some(mouse_pos) = input.pointer.hover_pos() {
                let size = vec2(input.screen_rect.width(), input.screen_rect.height());
                let viewport =  vec2(mouse_pos.x, mouse_pos.y) / size;
                let ray = self.camera.viewport_point_to_ray(vec3(viewport.x, 1. - viewport.y, 0.));
        
                self.field.raycast(ray)
            }
            else {
                None
            };

            if let Some(position) = hit {

                let settings = BrushApplicationParameters::new(
                    position, self.delta_time_ratio
                );

                let brush = Brush::new(
                    self.brushes[self.selected_brush].clone(),
                    settings
                );

                self.field.apply_brush(&brush);
            }
        }
    }

    pub fn draw(&mut self, params: Parameters) {

        GL!(gl::Enable(gl::DEPTH_TEST));
        GL!(gl::ClearColor(0.455, 0.302, 0.663, 1.0));
        GL!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        
        
        self.camera.set_aspect_ratio(params.height as f32 / params.width as f32);
        
        if !self.debug.debug {
            self.field.draw(&self.camera);
        }
        else {
            if self.debug.draw_model {
                self.field.draw(&self.camera);
            }
            if self.debug.draw_sdf {
                self.field.draw_distance_field(
                    &self.camera, 
                    self.slice, 
                    self.debug.parity);
            }
        }


        GL!(gl::Disable(gl::DEPTH_TEST));
    }

    pub fn draw_ui(&mut self, egui_ctx: &CtxRef, params: Parameters) {
        egui::Window::new("Settings").show(egui_ctx, |ui| {

            // let mut fov = self.camera.fov();
            // ui.add(egui::Slider::new(&mut fov, 1.0..=179.).text("fov"));
            // self.camera.set_fov(fov);

            egui::ComboBox::from_label("Select brush")
                .selected_text(format!("{:?}", self.brushes[self.selected_brush].lock().unwrap().brush_name()))
                .show_ui(ui, |ui| {
                    for i in 0..self.brushes.len() {
                        ui.selectable_value(&mut self.selected_brush, 
                            i, self.brushes[i].lock().unwrap().brush_name());
                    }
                }
            );

            ui.add_space(10.);


            ui.label(self.brushes[self.selected_brush].lock().unwrap().brush_name());
            self.brushes[self.selected_brush].lock().unwrap().display_ui(ui);
            ui.add_space(10.);    
                    
            
            let mut new_debug = self.debug.debug;
            ui.checkbox(&mut new_debug, "debug");
            if new_debug != self.debug.debug {
                self.debugger.set_debug_enabled(new_debug);
                self.debug.debug = new_debug;
            }
            if self.debug.debug {

                ui.checkbox(&mut self.debug.draw_model, "draw model");
                ui.checkbox(&mut self.debug.draw_sdf, "draw sdf");

                if self.debug.draw_sdf {
                    ui.add(egui::Slider::new(&mut self.slice, 0.0..=1.0).text("slice"));
                }

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.debug.parity.x, "x");
                    ui.checkbox(&mut self.debug.parity.y, "y");
                    ui.checkbox(&mut self.debug.parity.z, "z");
                });


            }
        });

        self.debugger.perform_draw(&egui_ctx, &self.camera);
    }
}

pub struct Parameters {
    pub width: i32,
    pub height: i32
}