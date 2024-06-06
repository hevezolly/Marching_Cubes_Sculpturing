use core::buffers::buffer::BufferDataInterface;
use core::buffers::buffer::Usage;
use core::buffers::buffer::UsageFrequency;
use core::buffers::buffer::UsagePattern;
use core::buffers::buffer::VertexBuffer;
use core::context::synchronization_context::SynchronizationContext;
use core::shaders::shader::Shader;
use core::textures::image_provider::Image;
use core::textures::image_provider::ImageFormat;
use core::textures::texture::FilterMode;
use core::textures::texture::MipMapFilterMode;
use core::textures::texture::Texture;
use core::textures::texture::TextureAccess;
use core::textures::texture::WrapMode;
use core::textures::TextureUnit;
use core::GL;
use core::buffers::buffer::Buffer;
use core::shaders::shader_programm::ShaderProgramm;
use std::cmp::max;
use std::cmp::min;
// use core::shaders::Shader;
use std::ffi::c_void;
use std::ops::Index;
use std::path::Path;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use ::egui::text;
use egui_glfw_gl::egui::paint::stats;
use egui_glfw_gl::egui::Color32;
use egui_glfw_gl::egui::CtxRef;
use egui_glfw_gl::egui::InputState;
use egui_glfw_gl::egui::Pos2;
use egui_glfw_gl::egui::Stroke;
use egui_glfw_gl::gl;
use egui_glfw_gl::egui;
use glam::ivec3;
use glam::vec4;
use glam::IVec2;
use glam::IVec3;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::vec2;
use glam::vec3;
use glam::Vec4;
use glam::Vec4Swizzles;

use crate::algorithms::camera::Camera;
use crate::algorithms::camera::perspective::PerspectiveCamera;
use crate::application::support::brush;
use crate::application::support::brush::circle_bruhs::CircleBrush;

use super::chunk::Chunk;
use super::support::shaders::shaders_loader::ShaderStorage;

pub struct ExecutrionLogick {
    // command_buffer: Buffer,
    camera: PerspectiveCamera,
    sync_context: SynchronizationContext,
    programm_storage: ShaderStorage,
    chunk: Chunk,
    slice: f32,
    debug: bool,
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
pub const CHUNK_SIZE: i32 = 32;
pub const NUM_OF_CUBES: IVec3 = IVec3 { x: CHUNK_SIZE, y: CHUNK_SIZE, z: CHUNK_SIZE };

const FPS: usize = 60;

const FRAME_TIME: Duration = Duration::from_millis(1000 / FPS as u64);

impl ExecutrionLogick {
    pub fn init() -> ExecutrionLogick {
        
        // let vertex_data = model_vertex_buffer.get_all_data();
        // dbg!(vertex_data);

        let mut camera = PerspectiveCamera::new(90., 0.01, 100.);
        camera.transform.set_position(vec3(0.5, 0.5, -1.));
        let sync_context = SynchronizationContext::new();
        let programm_storage = ShaderStorage::new();

        let mut chunk = Chunk::empty(sync_context.clone(), programm_storage.clone());

        chunk.fill_sphere();
        chunk.march(true);

        ExecutrionLogick { 
            camera, 
            slice: 0.,
            chunk,
            debug: false,
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
        }
    }


    pub fn update(&mut self, input: &InputState) {
        const speed: f32 = 0.1;

        if input.key_pressed(egui::Key::S) {
            let pos = self.camera.transform.position() + vec3(0., 0., -speed);
            self.camera.transform.set_position(pos);
        }
        if input.key_pressed(egui::Key::W) {
            let pos = self.camera.transform.position() + vec3(0., 0., speed);
            self.camera.transform.set_position(pos);
        }
        if input.key_pressed(egui::Key::A) {
            let pos = self.camera.transform.position() + vec3(-speed, 0., 0.);
            self.camera.transform.set_position(pos);
        }
        if input.key_pressed(egui::Key::D) {
            let pos = self.camera.transform.position() + vec3(speed, 0., 0.);
            self.camera.transform.set_position(pos);
        }
        if input.key_pressed(egui::Key::Space) {
            let pos = self.camera.transform.position() + vec3(0., speed, 0.);
            self.camera.transform.set_position(pos);
        }
        if input.key_pressed(egui::Key::Z) {
            let pos = self.camera.transform.position() + vec3(0., -speed, 0.);
            self.camera.transform.set_position(pos);
        }

        if input.pointer.button_down(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = input.pointer.hover_pos() {
                let size = vec2(input.screen_rect.width(), input.screen_rect.height());
                let viewport =  vec2(mouse_pos.x, mouse_pos.y) / size;
                let ray = self.camera.viewport_point_to_ray(vec3(viewport.x, 1. - viewport.y, 0.));

                let scale = Vec3::ONE;// / NUM_OF_CUBES.as_vec3();
                if let Some(position) = self.chunk.raycast(ray) {
                    let brush = CircleBrush::new(
                        self.programm_storage.clone(),
                        position, 0.1, 0.01, 3.);
                    self.chunk.apply_brush(&brush);
                    self.chunk.march(true);


                    // let start = Instant::now();
                    // GL!(gl::Finish());
                    // dbg!(start.elapsed());
                    // self.draw_point(egui_ctx, position, size);
                }
            }
        }
    }

    pub fn draw(&mut self, params: Parameters) {

        GL!(gl::Enable(gl::DEPTH_TEST));
        GL!(gl::ClearColor(0.455, 0.302, 0.663, 1.0));
        GL!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        
        
        self.camera.set_aspect_ratio(params.height as f32 / params.width as f32);
        
        if !self.debug {
            self.chunk.draw(&self.camera);
        }
        else {
            self.chunk.draw_3d_texture(&self.camera, self.slice);
            self.chunk.draw_debug_vew(&self.camera);
        }
        // self.chunk.draw_3d_texture(&self.camera, self.slice);


        GL!(gl::Disable(gl::DEPTH_TEST));
    }

    pub fn draw_ui(&mut self, egui_ctx: &CtxRef, params: Parameters) {
        egui::Window::new("Settings").show(egui_ctx, |ui| {

            let mut fov = self.camera.fov();
            ui.add(egui::Slider::new(&mut fov, 1.0..=179.).text("fov"));
            self.camera.set_fov(fov);

            ui.add(egui::Slider::new(&mut self.slice, 0.0..=1.0).text("slice"));

            ui.checkbox(&mut self.debug, "debug");
        });

        // if egui_ctx.input().pointer.button_down(egui::PointerButton::Primary) {
        //     if let Some(mouse_pos) = egui_ctx.input().pointer.hover_pos() {
        //         let screen_size = vec2(egui_ctx.input().screen_rect.width(), egui_ctx.input().screen_rect.height());
        //         let viewport =  vec2(mouse_pos.x, mouse_pos.y) / screen_size;
        //         let ray = self.camera.viewport_point_to_ray(vec3(viewport.x, 1. - viewport.y, 0.));

        //         let scale = Vec3::ONE;// / NUM_OF_CUBES.as_vec3();

        //         if let Some(position) = self.chunk.raycast(ray, |center, size| 
        //                 draw_box(&self.camera, &egui_ctx, center, size, screen_size, Color32::RED)) {
        //             draw_point(&self.camera, &egui_ctx, position, screen_size);
        //             let brush = CircleBrush::new(
        //                 self.programm_storage.clone(),
        //                 position, 0.1, -0.01, 1.);
        //             self.chunk.apply_brush(&brush);
        //             self.chunk.march(true);


        //             // let start = Instant::now();
        //             // GL!(gl::Finish());
        //             // dbg!(start.elapsed());
        //             // self.draw_point(egui_ctx, position, size);
        //         }
        //     }
        // }
    }
}

pub struct Parameters {
    pub width: i32,
    pub height: i32
}

    
fn draw_box(camera: &impl Camera,
    egui_ctx: &CtxRef, 
    corner: Vec3, 
    size: Vec3, 
    screen_size: Vec2,
    color: Color32) {
    let painter = egui_ctx.debug_painter();
    let corner = vec4(corner.x, corner.y, corner.z, 1.);

    // let size = vec2(screen_size.x, screen_size.y) / egui_ctx.pixels_per_point();
    //  egui_ctx.pixels_per_point();

    let lbb = corner;
    let lbf = corner + Vec4::Z * size.z;
    let ltb = corner + Vec4::Y * size.y;
    let ltf = corner + Vec4::Y * size.y + Vec4::Z * size.z;
    let rbb = corner + Vec4::X * size.x;
    let rbf = corner + Vec4::X * size.x + Vec4::Z * size.z;
    let rtb = corner + Vec4::X * size.x + Vec4::Y * size.y;
    let rtf = corner + Vec4::X * size.x + Vec4::Y * size.y + Vec4::Z * size.z;

    let lbb = camera.full_matrix() * lbb;
    let lbf = camera.full_matrix() * lbf;
    let ltb = camera.full_matrix() * ltb;
    let ltf = camera.full_matrix() * ltf;
    let rbb = camera.full_matrix() * rbb;
    let rbf = camera.full_matrix() * rbf;
    let rtb = camera.full_matrix() * rtb;
    let rtf = camera.full_matrix() * rtf;

    let lbb = ((lbb.xy() / lbb.w) + Vec2::ONE) * 0.5;
    let lbf = ((lbf.xy() / lbf.w) + Vec2::ONE) * 0.5;
    let ltb = ((ltb.xy() / ltb.w) + Vec2::ONE) * 0.5;
    let ltf = ((ltf.xy() / ltf.w) + Vec2::ONE) * 0.5;
    let rbb = ((rbb.xy() / rbb.w) + Vec2::ONE) * 0.5;
    let rbf = ((rbf.xy() / rbf.w) + Vec2::ONE) * 0.5;
    let rtb = ((rtb.xy() / rtb.w) + Vec2::ONE) * 0.5;
    let rtf = ((rtf.xy() / rtf.w) + Vec2::ONE) * 0.5;

    let lbb = Pos2::new(lbb.x * screen_size.x, (1. - lbb.y) * screen_size.y);
    let lbf = Pos2::new(lbf.x * screen_size.x, (1. - lbf.y) * screen_size.y);
    let ltb = Pos2::new(ltb.x * screen_size.x, (1. - ltb.y) * screen_size.y);
    let ltf = Pos2::new(ltf.x * screen_size.x, (1. - ltf.y) * screen_size.y);
    let rbb = Pos2::new(rbb.x * screen_size.x, (1. - rbb.y) * screen_size.y);
    let rbf = Pos2::new(rbf.x * screen_size.x, (1. - rbf.y) * screen_size.y);
    let rtb = Pos2::new(rtb.x * screen_size.x, (1. - rtb.y) * screen_size.y);
    let rtf = Pos2::new(rtf.x * screen_size.x, (1. - rtf.y) * screen_size.y);

    let stroke = Stroke::new(1., color);

    painter.line_segment([lbb, rbb], stroke);
    painter.line_segment([ltb, lbb], stroke);
    painter.line_segment([rtb, rbb], stroke);
    painter.line_segment([ltb, rtb], stroke);

    painter.line_segment([lbf, rbf], stroke);
    painter.line_segment([ltf, lbf], stroke);
    painter.line_segment([rtf, rbf], stroke);
    painter.line_segment([ltf, rtf], stroke);

    painter.line_segment([lbb, lbf], stroke);
    painter.line_segment([rbb, rbf], stroke);
    painter.line_segment([rtb, rtf], stroke);
    painter.line_segment([ltb, ltf], stroke);

}

fn draw_point(camera: &impl Camera, egui_ctx: &CtxRef, center: Vec3, screen_size: Vec2) {
    let painter = egui_ctx.debug_painter();
    let center = vec4(center.x, center.y, center.z, 1.);
    let center = camera.full_matrix() * center;
    let scale = 10. / center.w;
    let center = ((center.xy() / center.w) + Vec2::ONE) * 0.5;
    let center = Pos2::new(center.x * screen_size.x, (1. - center.y) * screen_size.y);

    painter.circle_filled(center, scale, Color32::YELLOW);
}