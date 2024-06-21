use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage, VertexBuffer}, shaders::shader::Shader, GL};
use std::{fmt::Debug, ops::Index};

use egui_glfw_gl::gl;
use glam::{ivec3, vec3, IVec3, Mat4, Vec3};

use crate::{algorithms::{camera::Camera, transform::Transform, Triangle}, application::{app_logick::{ceil_div, NUM_OF_CUBES}, support::{shaders::{shaders_loader::ShaderStorage, ModelProgramm}, triangulation_table::triangulate_centers}}};

use super::{DrawParameters, ModelVertex};


pub const COMPRESS_COLLISION: bool = true;

const NUM_OF_BLOCKS: usize = if !COMPRESS_COLLISION {
    (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize
} else {
    ceil_div((NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize, 4)
};

const TOTAL_NUM_OF_CUBES: usize = (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize; 


#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/drawing/display_model.vert", 
              "resources/shader_sources/drawing/display_model.frag")]
pub struct ModelDisplayUniform {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub light_direction: Vec3
}

#[derive(Default, Clone)]
struct BlockTriangles {
    triangles: [Triangle; 5],
    triangles_count: usize,
    update_iteration: u64
}

impl BlockTriangles {
    fn new() -> Self {
        Self::default()
    }

    fn get(&self) -> &[Triangle] {
        &self.triangles[0..self.triangles_count]
    }

    fn fill(&mut self, config_index: u8, cube_center: Vec3, cube_size: Vec3, offset: f32, update_parity: u64) {
        self.triangles_count = triangulate_centers(&mut self.triangles, config_index, cube_center, cube_size, offset);
        self.update_iteration = update_parity;
    }
}

impl Debug for BlockTriangles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockTriangles").field("triangles", &self.get()).finish()
    }
}

#[derive(Debug)]
pub struct CollisionShape {
    raw_field: Box<[u32; NUM_OF_BLOCKS]>,
    buffer: Buffer,
    triangles: Vec<BlockTriangles>,
    update_iteration: u64
}

fn position(index: IVec3) -> usize {
    (index.x + 
        index.y * NUM_OF_CUBES.x + 
        index.z * NUM_OF_CUBES.y * NUM_OF_CUBES.x) as usize
}

impl CollisionShape {
    pub fn new() -> CollisionShape {
        let field = Box::new([0u32; NUM_OF_BLOCKS]);
        let buffer = Buffer::from_data(field.as_ref(), Usage::dynamic_read());
        let triangles = vec![BlockTriangles::new(); TOTAL_NUM_OF_CUBES];

        CollisionShape { 
            raw_field: field,
            update_iteration: 0, 
            buffer,
            triangles
        }
    }

    pub fn readback(&mut self) {
        self.buffer.read_data_from_start(self.raw_field.as_mut());
        self.update_iteration += 1;
    }
    
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    fn fill(&mut self, index: IVec3) {
        let position = position(index);

        let config = if COMPRESS_COLLISION {
            let field_index = position / 4;
            let shift =  (position % 4) * 8;
            
            (self.raw_field[field_index] >> shift) & 0xFF
        } else {
            self.raw_field[position]
        };

        self.triangles[position].fill(config as u8, index.as_vec3() + Vec3::ONE * 0.5, Vec3::ONE, 0.01, self.update_iteration);
    }

    pub fn get(&mut self, index: IVec3) -> &[Triangle] {
        let position = position(index);

        if self.triangles[position].update_iteration != self.update_iteration {
            self.fill(index);
        }

        self.triangles[position].get()
    }
}

pub struct CollisionShapeDebugView {
    buffer: VertexBuffer<ModelVertex>,
    vertex_count: i32,
    shader_storage: ShaderStorage,
}

impl CollisionShapeDebugView {
    pub fn new(shader_storage: ShaderStorage) -> CollisionShapeDebugView {
        let buffer = VertexBuffer::empty(
            (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z * 15) as usize, Usage::dynamic_copy());
        let vertex_count = 0;

        CollisionShapeDebugView { buffer, vertex_count, shader_storage }
    }

    pub fn actualize(&mut self, shape: &mut CollisionShape) {

        let mut result_buffer = Vec::new();

        for z in (0..NUM_OF_CUBES.z) {
            for y in (0..NUM_OF_CUBES.y) {
                for x in (0..NUM_OF_CUBES.x) {
                    for triangle in shape.get(ivec3(x, y, z)) {
                        let normal = triangle.normal();
                        result_buffer.push(ModelVertex {
                            position: triangle.a(),
                            normal: normal
                        });
                        result_buffer.push(ModelVertex {
                            position: triangle.b(),
                            normal: normal
                        });
                        result_buffer.push(ModelVertex {
                            position: triangle.c(),
                            normal: normal
                        });
                    }
                }
            }
        };

        self.buffer.update_data(0, &result_buffer);
        self.vertex_count = result_buffer.len() as i32;
    }

    pub fn draw(&mut self, params: DrawParameters<'_>) {
        
        self.buffer.bind();

        self.shader_storage.access().get::<ModelProgramm>().unwrap()
        .bind().set_uniforms(ModelDisplayUniform {
            model: *params.model,
            view: params.camera.view_matrix(),
            projection: params.camera.projection_matrix(),
            light_direction: vec3(1., 1., -0.5).normalize()
        }).unwrap();

        GL!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count));

        self.buffer.unbind();
        // self.command_buffer.unbind();
    }
}

