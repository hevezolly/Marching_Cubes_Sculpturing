use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage, VertexBuffer}, context::synchronization_context::{AllBarrier, BufferUpdateBarrier, CommandBarrier, ShaderImageAccessBarrier, ShaderStorageBarrier, SynchronizationContext}, shaders::{shader::Shader, shader_programm::ShaderProgramm}, textures::{image_provider::ImageFormat, texture::{FilterMode, Texture, TextureAccess, WrapMode}, TextureUnit}, GL};
use std::{ffi::c_void, mem, sync::Mutex};

use egui::epaint::image;
use egui_glfw_gl::gl;
use glam::{vec3, IVec3, Mat4, Vec3};


use crate::{algorithms::{camera::Camera, grid_line_intersection::march_grid_by_ray, raycast::{ray_triangle_intersection, Ray}}, application::support::{shaders::{FillCircleProgramm, MarchingCubeProgramm, ModelProgramm}, triangulation_table::static_triangle_buffer}};

use super::{app_logick::NUM_OF_CUBES, support::{bit_field::BitField, brush::Brush, collision_shape::{CollisionShape, CollisionShapeDebugView}, shaders::{shaders_loader::ShaderStorage, QuadProgramm}, simple_quad::SimpleQuad}};



#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes.compute")]
struct MarchingCubesUniforms {
    #[name("scalarField")]
    scalar_field: TextureUnit,
    origin_offset: Vec3,
    field_scale: Vec3,
    num_boxes: IVec3,
    surface_level: f32,
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/display_model.vert", 
              "resources/shader_sources/display_model.frag")]
pub struct ModelDisplayUniform {
    pub view: Mat4,
    pub projection: Mat4,
    pub light_direction: Vec3
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/fill_circle.compute")]
struct FillTextureUniforms {
    #[name("imgOutput")]
    img_output: TextureUnit,
    // imgOutput: TextureUnit
}   

#[repr(C)]
#[derive(Debug, VertexDef, Clone)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3
}

impl Default for Command {
    fn default() -> Self {
        Self { count: 0, primitive_count: 1, first: 0, base_instance: 0 }
    }
}

#[repr(C)]
#[derive(Debug, VertexDef, Clone)]
struct Command {
    count: u32,
    primitive_count: u32,
    first: u32,
    base_instance: u32,
}

pub struct Chunk {
    model_vertex_buffer: VertexBuffer<ModelVertex>,
    command_buffer: Buffer,
    bit_field: BitField,
    collision_field: CollisionShape,
    quad: SimpleQuad,
    texture: Texture,
    swap_buffer: Texture,
    swap_buffer_is_actual: bool,
    is_bitfield_dirty: bool,
    sync_context: SynchronizationContext,
    programm_storage: ShaderStorage,
    collider_drawer: Option<CollisionShapeDebugView>
}

const TEXTURE_DIM: IVec3 = IVec3 { 
    x: NUM_OF_CUBES.x + 3, 
    y: NUM_OF_CUBES.y + 3, 
    z: NUM_OF_CUBES.z + 3 
};

impl Chunk {
    pub fn empty(sync_context: SynchronizationContext, programm_storage: ShaderStorage) -> Chunk {
        let model_vertex_buffer = VertexBuffer::empty(
            (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z * 15) as usize,
            Usage::dynamic_copy());

        let bit_field = BitField::new();
        let collision_field = CollisionShape::new();
        let command_buffer = Buffer::from_data(&[Command::default()], Usage::dynamic_read());
        

        let img_format = ImageFormat {
            lod: 0,
            format: gl::RED,
            internal_format: gl::R32F,
            data_type: gl::FLOAT,
        };
        let texture = Texture::new_3d()
            .magnification_filter(FilterMode::Linear)
            .minification_filter(FilterMode::Linear)
            .wrap_mode_x(WrapMode::Repeat)
            .wrap_mode_y(WrapMode::Repeat)
            // .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, ImageFormat {
            //     lod: 0,
            //     format: gl::RED,
            //     internal_format: gl::R32F,
            //     data_type: gl::FLOAT
            // });
            .wrap_mode_z(WrapMode::Repeat)
            .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, TEXTURE_DIM.z, img_format);

        let swap_buffer = Texture::new_3d()
            .magnification_filter(FilterMode::Linear)
            .minification_filter(FilterMode::Linear)
            .wrap_mode_x(WrapMode::Repeat)
            .wrap_mode_y(WrapMode::Repeat)
            // .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, ImageFormat {
            //     lod: 0,
            //     format: gl::RED,
            //     internal_format: gl::R32F,
            //     data_type: gl::FLOAT
            // });
            .wrap_mode_z(WrapMode::Repeat)
            .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, TEXTURE_DIM.z, img_format);

        sync_context.force_sync_with(AllBarrier);
        let quad = SimpleQuad::new(programm_storage.clone()).unwrap();

        Chunk { 
            model_vertex_buffer, 
            bit_field, 
            texture, 
            sync_context, 
            collision_field,
            swap_buffer,
            swap_buffer_is_actual: false,
            command_buffer, 
            programm_storage,
            quad, 
            collider_drawer: None,
            is_bitfield_dirty: false, 
        }
    }

    pub fn fill_sphere(&mut self) {
        self.texture.bind_image(1, TextureAccess::Write);

        self.programm_storage.access().get::<FillCircleProgramm>().unwrap()
        .bind().set_uniforms(FillTextureUniforms {
            img_output: 1.into(),
        }).unwrap();

        let c = self.sync_context.dirty() | ShaderImageAccessBarrier;
        GL!(gl::DispatchCompute((TEXTURE_DIM.x) as u32, 
            (TEXTURE_DIM.y) as u32, 
            (TEXTURE_DIM.z) as u32));
        c.apply();

        // self.texture.unbind();        
    }

    pub fn march(&mut self, update: bool) {

        self.actualise_texture();

        self.command_buffer.update_data(0,&[Command::default()]);
        self.sync_context.force_sync_with(BufferUpdateBarrier);

        self.texture.bind_image(1, TextureAccess::Read);

        let triangle_buffer = static_triangle_buffer();
        self.programm_storage.access().get::<MarchingCubeProgramm>().unwrap()
        .bind().set_uniforms(MarchingCubesUniforms { 
            scalar_field: 1.into(), 
            origin_offset: Vec3::ZERO, 
            field_scale: Vec3::ONE,
                // / vec3(NUM_OF_CUBES.x as f32, 
                // NUM_OF_CUBES.y as f32,
                // NUM_OF_CUBES.z as f32), 
            num_boxes: NUM_OF_CUBES, 
            surface_level: 0.3 }
        ).unwrap()
        .set_buffer(&self.command_buffer, 1)
        .set_buffer(&self.model_vertex_buffer, 2)
        .set_buffer(triangle_buffer, 3)
        .set_buffer(self.bit_field.buffer(), 4)
        .set_buffer(self.collision_field.buffer(), 5);
        
        let c = self.sync_context.dirty() | ShaderStorageBarrier | CommandBarrier;
        self.is_bitfield_dirty = update;
        GL!(gl::DispatchCompute((NUM_OF_CUBES.x) as u32, (NUM_OF_CUBES.y) as u32, (NUM_OF_CUBES.z) as u32));
        c.apply();

        self.sync_context.force_sync_with(AllBarrier);
        // self.texture.unbind();
    }

    pub fn draw(&mut self, camera: &impl Camera) {

        self.sync_context.sync_with(ShaderStorageBarrier | CommandBarrier);

        self.model_vertex_buffer.bind();

        self.programm_storage.access().get::<ModelProgramm>().unwrap()
        .bind().set_uniforms(ModelDisplayUniform {
            view: camera.view_matrix(),
            projection: camera.projection_matrix(),
            light_direction: vec3(1., 1., -0.5).normalize()
        }).unwrap();

        // GL!(gl::DrawArrays(gl::TRIANGLES, self.offset * 3, self.draw_count * 3));
        self.command_buffer.bind(gl::DRAW_INDIRECT_BUFFER);
        GL!(gl::DrawArraysIndirect(gl::TRIANGLES, (0) as *const c_void));

        self.model_vertex_buffer.unbind();
        self.command_buffer.unbind();
    }

    pub fn draw_debug_vew(&mut self, camera: &impl Camera) {
        
        let mut actualize = false;

        let drawer = self.collider_drawer.get_or_insert_with(|| {
            actualize = true;
            CollisionShapeDebugView::new(self.programm_storage.clone())
        });
        if self.is_bitfield_dirty {
            actualize = true;
            self.sync_context.sync_with(ShaderStorageBarrier);
            // self.bit_field.readback();
            self.collision_field.readback();
            self.is_bitfield_dirty = false;
            drawer.actualize(&self.collision_field);
        };
        
        if actualize {
            drawer.actualize(&self.collision_field);
        }
        
        drawer.draw(camera);
    }

    pub fn draw_3d_texture(&mut self, camera: &impl Camera, slice: f32) {

        self.actualise_texture();

        // self.swap_buffer.bind(10);
        self.texture.bind(1);
        self.quad.draw(camera, 1.into(), slice);
        // self.texture.unbind();

        // self.swap_buffer.bind(1);
        // self.quad.draw_offset(Vec3::Y * 2., camera, 1.into(), slice);
        // self.swap_buffer.unbind();
    }

    pub fn raycast(&mut self, ray: Ray) -> Option<Vec3> {

        if self.is_bitfield_dirty {
            self.sync_context.sync_with(ShaderStorageBarrier);
            // self.bit_field.readback();
            self.collision_field.readback();
            self.is_bitfield_dirty = false;
        };


        for c in march_grid_by_ray(
            ray.origin, ray.direction, IVec3::ZERO, NUM_OF_CUBES - IVec3::ONE)? {
            for triangle in self.collision_field.get(c) {
                let intersection: Option<Vec3> = ray_triangle_intersection(ray, &triangle);
                if intersection.is_some() {
                    return intersection;
                }
            }
        }

        None
    }

    fn actualise_texture(&mut self) {
        if self.swap_buffer_is_actual {
            self.swap_buffer_is_actual = false;
            mem::swap(&mut self.texture, &mut self.swap_buffer);
        }
        self.sync_context.sync_with(AllBarrier);
    }

    pub fn apply_brush(&mut self, brush: &dyn Brush) {
        self.actualise_texture();
        
        
        let c = self.sync_context.dirty() | AllBarrier;
        
        self.swap_buffer_is_actual = true;
        brush.apply(&mut self.texture, &mut self.swap_buffer);

        c.apply();
        // self.sync_context.sync_with(AllBarrier);
    }
}