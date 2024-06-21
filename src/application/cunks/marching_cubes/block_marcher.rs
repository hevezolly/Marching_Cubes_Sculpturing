use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage}, context::synchronization_context::{BufferUpdateBarrier, CommandBarrier, ShaderStorageBarrier}, shaders::shader::Shader, textures::{texture::TextureAccess, TextureUnit}, GL};
use std::ffi::c_void;

use egui_glfw_gl::gl;
use glam::{vec3, IVec3, Mat4, Vec3};

use crate::{algorithms::camera::Camera, application::{app_logick::{BLOCKY, FLAT_SHADING}, cunks::{chunk::TEXTURE_OFFSET, collision_shape::COMPRESS_COLLISION, marching_cubes::WORK_GROUP, DrawParameters}, support::{shaders::{dispatch_compute_for, shaders_loader::ShaderType, ModelProgramm}, triangulation_table::static_triangle_buffer}}, shader_ref};

use super::{CubeMarcher, MarchParameters};


pub struct BlockCubeMarcher {
    command_buffer: Buffer,
    counter_buffer: Buffer,
    model_index_buffer: Buffer,
}

#[repr(C)]
#[derive(Debug, VertexDef, Clone)]
struct IndirectElementsCommand {
    count: u32,
    prim_count: u32,
    first_index: u32,
    base_vertex: u32,
    base_instance: u32,
}

impl Default for IndirectElementsCommand {
    fn default() -> Self {
        Self { count: 0, prim_count: 1, first_index: 0, base_vertex: 0, base_instance: 0 }
    }
}

shader_ref!(MarchingCubeProgramm, ShaderType::Compute("resources/shader_sources/marching_cubes/marching_cubes.compute"), 
    if COMPRESS_COLLISION {"COMPRESS_COLLISION"} else {""},
    if BLOCKY {"BLOCKY"} else {""},
    "BLOCK_UPDATE",
    if FLAT_SHADING {"FLAT_SHADING"} else {""},
    format!("DISPATCH_SIZE local_size_x = {}, local_size_y = {}, local_size_z = {}",
    WORK_GROUP.x,
    WORK_GROUP.y,
    WORK_GROUP.z));

shader_ref!(MarchingCubeIndexerProgramm, ShaderType::Compute("resources/shader_sources/marching_cubes/marching_cubes_indexer.compute"),
    format!("DISPATCH_SIZE local_size_x = {}, local_size_y = {}, local_size_z = {}",
    WORK_GROUP.x,
    WORK_GROUP.y,
    WORK_GROUP.z));

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes/marching_cubes.compute")]
struct MarchingCubesUniforms {
    #[name("scalarField")]
    scalar_field: TextureUnit,
    start_block: IVec3,
    origin_offset: Vec3,
    field_scale: Vec3,
    num_boxes: IVec3,
    surface_level: f32,
    texture_sample_offset: IVec3,
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes/marching_cubes_indexer.compute")]
struct MarchingCubesIndexerUniforms {
    num_boxes: IVec3,
}

impl CubeMarcher for BlockCubeMarcher {
    fn march<'a>(&mut self, step: usize, params: &mut MarchParameters) {
         // self.dirty_area = Bounds::min_max(Vec3::ZERO, Vec3::ONE);

         assert!(step < 2);

         if params.dirty_area.is_empty() {
             return;
         }

         if step == 0 {

             self.command_buffer.update_data(0,&[IndirectElementsCommand::default()]);
             params.sync_context.force_sync_with(BufferUpdateBarrier);
     
             params.distance_field.bind_image(1, TextureAccess::Read);
     
             let triangle_buffer = static_triangle_buffer();
             params.programm_storage.access().get::<MarchingCubeProgramm>().unwrap()
             .bind().set_uniforms(MarchingCubesUniforms { 
                 scalar_field: 1.into(), 
                 start_block: params.dirty_area.min(),
                 origin_offset: Vec3::ZERO, 
                 field_scale: params.chunk_scale_factor, 
                 num_boxes: params.num_of_cubes, 
                 surface_level: params.surface_level,
                texture_sample_offset: TEXTURE_OFFSET, 
            }).unwrap()
             // .set_buffer(&params.command_buffer, 1)
             .set_buffer(&params.model_vertex_buffer, 2)
             .set_buffer(triangle_buffer, 3)
             .set_buffer(&self.counter_buffer, 4)
             .set_buffer(params.collision_field.buffer(), 5);
             
             let c = params.sync_context.dirty() | ShaderStorageBarrier;
             dispatch_compute_for(params.dirty_area.size(), WORK_GROUP);
             c.apply();
         }

         else if step == 1 {
            
             params.sync_context.sync_with(ShaderStorageBarrier);
     
             params.programm_storage.access().get::<MarchingCubeIndexerProgramm>().unwrap()
             .bind().set_uniforms(MarchingCubesIndexerUniforms {
                 num_boxes: params.num_of_cubes,
             }).unwrap()
             .set_buffer(&self.counter_buffer, 4)
             .set_buffer(&self.model_index_buffer, 2)
             .set_buffer(&self.command_buffer, 1);
     
             let c = params.sync_context.dirty() | ShaderStorageBarrier | CommandBarrier;
     
             dispatch_compute_for(params.num_of_cubes, WORK_GROUP);
             
             c.apply();
         }
 
 
     }

    fn draw(&mut self,
            params: &mut MarchParameters) {

        params.sync_context.sync_with(ShaderStorageBarrier | CommandBarrier);

        params.model_vertex_buffer.bind();
        self.model_index_buffer.bind_as_index();

        // GL!(gl::DrawArrays(gl::TRIANGLES, self.offset * 3, self.draw_count * 3));
        self.command_buffer.bind(gl::DRAW_INDIRECT_BUFFER);
        GL!(gl::DrawElementsIndirect(gl::TRIANGLES, gl::UNSIGNED_INT, (0) as *const c_void));

        params.model_vertex_buffer.unbind();
        self.model_index_buffer.unbind();
        self.command_buffer.unbind();
    }
    
    fn march_steps_count(&self) -> usize {
        2
    }
}

impl BlockCubeMarcher {
    pub fn new(params: &MarchParameters) -> Self {
        let model_index_buffer = Buffer::empty::<u32>(
            (params.num_of_cubes.x * params.num_of_cubes.y * params.num_of_cubes.z * 15) as usize, 
            Usage::dynamic_copy());
        let counter_buffer = Buffer::empty::<u32>(
            (params.num_of_cubes.x * params.num_of_cubes.y * params.num_of_cubes.z) as usize,
            Usage::dynamic_copy());

        let command_buffer = Buffer::from_data(&[IndirectElementsCommand::default()], Usage::dynamic_copy());

        Self { command_buffer, counter_buffer, model_index_buffer }

    }
}