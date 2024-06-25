use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage, VertexBuffer}, context::synchronization_context::{BufferUpdateBarrier, CommandBarrier, ShaderStorageBarrier, SynchronizationContext}, textures::{texture::{Texture, TextureAccess}, TextureUnit}, GL};
use std::ffi::c_void;

use egui_glfw_gl::gl;
use glam::{vec3, IVec3, Mat4, Vec3};

use crate::{algorithms::camera::Camera, application::{app_logick::{BLOCKY, FLAT_SHADING}, cunks::{chunk::TEXTURE_OFFSET, collision_shape::COMPRESS_COLLISION, marching_cubes::WORK_GROUP, DrawParameters}, support::{shaders::{dispatch_compute_for, shaders_loader::ShaderType, ModelProgramm}, triangulation_table::static_triangle_buffer}}, dispatch_size, shader_ref};

use super::{CubeMarcher, MarchParameters, ModelVertex};


#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes/marching_cubes.compute")]
pub struct MarchingCubesUniforms {
    #[name("scalarField")]
    pub scalar_field: TextureUnit,
    pub origin_offset: Vec3,
    pub field_scale: Vec3,
    pub num_boxes: IVec3,
    pub surface_level: f32,
    pub texture_sample_offset: IVec3,
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/drawing/display_model.vert", 
              "resources/shader_sources/drawing/display_model.frag")]
struct ModelDisplayUniform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
    light_direction: Vec3
}

#[repr(C)]
#[derive(Debug, VertexDef, Clone)]
struct IndirectArrayCommand {
    count: u32,
    primitive_count: u32,
    first: u32,
    base_instance: u32,
}

impl Default for IndirectArrayCommand {
    fn default() -> Self {
        Self { count: 0, primitive_count: 1, first: 0, base_instance: 0 }
    }
}

shader_ref!(MarchingCubeProgramm, ShaderType::Compute("resources/shader_sources/marching_cubes/marching_cubes.compute"), 
    if COMPRESS_COLLISION {"COMPRESS_COLLISION"} else {""},
    if BLOCKY {"BLOCKY"} else {""},
    if FLAT_SHADING {"FLAT_SHADING"} else {""},
    dispatch_size!(WORK_GROUP));

pub struct FullCubeMarcher {
    command_buffer: Buffer,
}

impl CubeMarcher for FullCubeMarcher {
    
    fn march<'a>(&mut self, step: usize, parameters: &mut MarchParameters) {
        assert!(step == 0);
        self.command_buffer.update_data(0,&[IndirectArrayCommand::default()]);
        parameters.sync_context.force_sync(BufferUpdateBarrier);

        parameters.distance_field.bind_image(1, TextureAccess::Read);

        let triangle_buffer = static_triangle_buffer();
        parameters.programm_storage.access().get::<MarchingCubeProgramm>().unwrap()
        .bind().set_uniforms(MarchingCubesUniforms { 
            texture_sample_offset: TEXTURE_OFFSET,
            scalar_field: 1.into(), 
            origin_offset: Vec3::ZERO, 
            field_scale: parameters.chunk_scale_factor, 
            num_boxes: parameters.num_of_cubes, 
            surface_level: parameters.surface_level }
        ).unwrap()
        .set_buffer(&self.command_buffer, 1)
        .set_buffer(&parameters.model_vertex_buffer, 2)
        .set_buffer(triangle_buffer, 3)
        // .set_buffer(parameters.bit_field.buffer(), 4)
        .set_buffer(parameters.collision_field.buffer(), 5);
        
        parameters.sync_context.dirty(ShaderStorageBarrier | CommandBarrier);
        dispatch_compute_for(parameters.num_of_cubes, WORK_GROUP);
    }
    
    fn draw(&mut self, params: &mut MarchParameters) {
        params.sync_context.sync(ShaderStorageBarrier | CommandBarrier);

        params.model_vertex_buffer.bind();

        // GL!(gl::DrawArrays(gl::TRIANGLES, params.offset * 3, params.draw_count * 3));
        self.command_buffer.bind(gl::DRAW_INDIRECT_BUFFER);
        GL!(gl::DrawArraysIndirect(gl::TRIANGLES, (0) as *const c_void));

        params.model_vertex_buffer.unbind();
        self.command_buffer.unbind();
    }
    
    fn march_steps_count(&self) -> usize {
        1
    }
}

impl FullCubeMarcher {
    pub fn new() -> Self {
        let command_buffer: Buffer = Buffer::from_data(&[IndirectArrayCommand::default()], 
            Usage::dynamic_copy());
        
        FullCubeMarcher { command_buffer }
    }
}