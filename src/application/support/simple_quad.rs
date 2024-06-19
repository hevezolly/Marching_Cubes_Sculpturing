use core::{buffers::buffer::{Buffer, Usage, VertexBuffer}, shaders::{shader::Shader, shader_programm::ShaderProgramm}, textures::TextureUnit, GL};
use std::{ffi::c_void, sync::Mutex};

use egui_glfw_gl::gl;
use glam::{vec2, vec3, Mat4, Vec2, Vec3};

use crate::{algorithms::camera::Camera, application::cunks::DrawParameters};

use super::shaders::{shaders_loader::ShaderStorage, QuadProgramm};

#[derive(Uniforms)]
// #[for_shaders("resources/shader_sources/display_quad.vert", 
//               "resources/shader_sources/display_quad.frag")]
struct QuadDisplayUniform {
    view: Mat4,
    projection: Mat4,
    volume: TextureUnit,
    slice: f32
}

#[repr(C)]
#[derive(VertexDef)]
struct DefaultVertex {
    psition: Vec3,
    uv: Vec2
}

fn quad() -> (VertexBuffer<DefaultVertex>, Buffer) 
{
    let positions = [
            DefaultVertex { psition: vec3(0., 0., 0.), uv: vec2(0., 0.) },
            DefaultVertex { psition: vec3(1., 0., 0.), uv: vec2(1., 0.) },
            DefaultVertex { psition: vec3(1., 1., 0.), uv: vec2(1., 1.) },
            DefaultVertex { psition: vec3(0., 1., 0.), uv: vec2(0., 1.) },
        ];

        let indecies = [
            0, 1, 2, 0, 2, 3u32
        ];

        let vertex_buffer = VertexBuffer::from_data(&positions, Usage::static_draw());

        let index_buffer = Buffer::from_data(&indecies, Usage::static_draw());
    (vertex_buffer, index_buffer)
}

fn static_quad_buffers<'a>() -> &'a mut (VertexBuffer<DefaultVertex>, Buffer) {
    static mut MAP: Mutex<Option<(VertexBuffer<DefaultVertex>, Buffer)>> = Mutex::new(None);
        
    unsafe { MAP.get_mut().unwrap() }
            .get_or_insert_with(|| quad())
}


pub struct SimpleQuad {
    programm_storage: ShaderStorage
}

impl SimpleQuad {
    pub fn new(mut programm_storage: ShaderStorage) -> Result<SimpleQuad, String> {

        programm_storage.access().preload::<QuadProgramm>().unwrap();
        
        Ok(SimpleQuad {programm_storage })
    }

    pub fn draw(&mut self, parameters: DrawParameters, texture: TextureUnit, slice: f32) {
        let (vertex, index) = static_quad_buffers();
        vertex.bind();
        index.bind_as_index();

        self.programm_storage.access().get::<QuadProgramm>().unwrap().bind().set_uniforms(QuadDisplayUniform {
            view: parameters.camera.view_matrix() * *parameters.model,
            projection: parameters.camera.projection_matrix(),
            volume: texture,
            slice
        }).unwrap();

        // GL!(gl::DrawArrays(gl::TRIANGLES, self.offset * 3, self.draw_count * 3));
        GL!(gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, (0) as *const c_void));

        vertex.unbind();
        index.unbind();
    }
}
