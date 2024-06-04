use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage, VertexBuffer}, shaders::shader::Shader, GL};
use std::ops::Index;

use egui_glfw_gl::gl;
use glam::{ivec3, vec3, IVec3, Vec3};

use crate::{algorithms::{camera::Camera, Triangle}, application::{app_logick::NUM_OF_CUBES, chunk::{ModelDisplayUniform, ModelVertex}, support::shaders::ModelProgramm}};

use super::{shaders::shaders_loader::ShaderStorage, triangulation_table::{triangulate_centers, TRI_TABLE}};


const fn ceil_div(val: usize, divider: usize) -> usize {
    let div = val / divider;
    if val % divider > 0 {
        div + 1
    }
    else {
        div
    }
}

pub const COMPRESS_COLLISION: bool = true;

pub const NUM_OF_BLOCKS: usize = if !COMPRESS_COLLISION {
    (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize
} else {
    ceil_div((NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize, 4)
};

#[derive(Debug)]
pub struct CollisionShape {
    field: Box<[u32; NUM_OF_BLOCKS]>,
    buffer: Buffer
}

impl CollisionShape {
    pub fn new() -> CollisionShape {
        let field = Box::new([0u32; NUM_OF_BLOCKS]);
        let buffer = Buffer::from_data(field.as_ref(), Usage::dynamic_read());
        
        CollisionShape { 
            field, 
            buffer
        }
    }

    pub fn readback(&mut self) {
        self.buffer.read_data_from_start(self.field.as_mut());
    }
    
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn get(&self, index: IVec3) -> Vec<Triangle> {
        let position = (index.x + 
            index.y * NUM_OF_CUBES.x + 
            index.z * NUM_OF_CUBES.y * NUM_OF_CUBES.x) as usize;

        let config = if COMPRESS_COLLISION {
            let field_index = position / 4;
            let shift =  (position % 4) * 8;
            
            (self.field[field_index] >> shift) & 0xFF
        } else {
            self.field[position]
        };
        
        triangulate_centers(config as u8, index.as_vec3() + Vec3::ONE * 0.5, Vec3::ONE)
    }
}

pub struct CollisionShapeDebugView {
    buffer: VertexBuffer<ModelVertex>,
    vertex_count: i32,
    shader_storage: ShaderStorage,
}

impl CollisionShapeDebugView {
    pub fn new(shader_storage: ShaderStorage) -> CollisionShapeDebugView {
        let buffer = VertexBuffer::create_uninitialized();
        let vertex_count = 0;

        CollisionShapeDebugView { buffer, vertex_count, shader_storage }
    }

    pub fn actualize(&mut self, shape: &CollisionShape) {

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

        self.buffer.rewrite_data(&result_buffer, Usage::static_draw());
        self.vertex_count = result_buffer.len() as i32;
    }

    pub fn draw(&mut self, camera: &impl Camera) {
        
        self.buffer.bind();

        self.shader_storage.access().get::<ModelProgramm>().unwrap()
        .bind().set_uniforms(ModelDisplayUniform {
            view: camera.view_matrix(),
            projection: camera.projection_matrix(),
            light_direction: vec3(1., 1., -0.5).normalize()
        }).unwrap();

        GL!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count));

        self.buffer.unbind();
        // self.command_buffer.unbind();
    }
}

