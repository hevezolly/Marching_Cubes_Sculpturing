use core::{buffers::buffer::{Usage, VertexBuffer}, context::synchronization_context::{AllBarrier, ShaderImageAccessBarrier, ShaderStorageBarrier, SynchronizationContext}, textures::{image_provider::ImageFormat, texture::{FilterMode, MipMapFilterMode, Texture, TextureAccess, WrapMode}, TextureUnit}, GL};

use egui_glfw_gl::{egui::Color32, gl};
use glam::{ivec3, vec3, IVec3, Mat4, Vec3};

use crate::{algorithms::{camera::Camera, grid_line_intersection::march_grid_by_ray, raycast::{ray_box_intersection, ray_triangle_intersection, IntersectionResult, Ray}}, application::{app_logick::NUM_OF_CUBES, support::{bounds::Bounds, brush::{chunk_to_texture_position, Brush}, debugger::{DebugPrimitive, Debugger}, shaders::{shaders_loader::ShaderStorage, FillCircleProgramm, ModelProgramm, ShadedModelProgramm, ZeroFieldProgramm}, simple_quad::SimpleQuad}}};

use super::{collision_shape::{CollisionShape, CollisionShapeDebugView}, field::CHUNK_SCALE_FACTOR, marching_cubes::{block_marcher::BlockCubeMarcher, full_marcher::FullCubeMarcher, CubeMarcher, MarchParameters}, DrawParameters, ModelVertex};

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/marching_cubes/fill_circle.compute")]
struct FillTextureUniforms {
    #[name("imgOutput")]
    img_output: TextureUnit,
    center_uvw: Vec3,
    // imgOutput: TextureUnit
}  


pub struct Chunk {
    marcher: Box<dyn CubeMarcher>,
    march_parameters: MarchParameters,
    debugger: Debugger,
    quad: Option<SimpleQuad>,
    is_collision_shape_dirty: bool,
    is_sdf_top_level_dirty: bool,
    collider_drawer: Option<CollisionShapeDebugView>
}

pub const TEXTURE_OFFSET: IVec3 = IVec3 {
    x: 8,
    y: 8,
    z: 8
};

pub const TEXTURE_SIZE_DELTA: IVec3 = IVec3 {
    x: TEXTURE_OFFSET.x * 2 + 1,
    y: TEXTURE_OFFSET.y * 2 + 1,
    z: TEXTURE_OFFSET.z * 2 + 1,
};

pub const TEXTURE_SIZE_DELTA_HALVED: Vec3 = Vec3 {
    x: TEXTURE_SIZE_DELTA.x as f32 * 0.5,
    y: TEXTURE_SIZE_DELTA.y as f32 * 0.5,
    z: TEXTURE_SIZE_DELTA.z as f32 * 0.5,
};

pub const TEXTURE_DIM: IVec3 = IVec3 { 
    x: NUM_OF_CUBES.x + TEXTURE_SIZE_DELTA.x, 
    y: NUM_OF_CUBES.y + TEXTURE_SIZE_DELTA.y, 
    z: NUM_OF_CUBES.z + TEXTURE_SIZE_DELTA.z 
};


const BLOCK_WRITE: bool = true;

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/drawing/display_model.vert", 
              "resources/shader_sources/drawing/display_model.frag")]
struct ModelDisplayUniform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
    light_direction: Vec3
}

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/drawing/shaded_model.vert", 
              "resources/shader_sources/drawing/shaded_model.frag")]
struct ShadedModelDisplayUniform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
    chunk_scale_factor: Vec3,
    light_direction: Vec3,
    scalar_field: TextureUnit,
    field_chunk_size_diff: IVec3,
    // ao_upper_edge: f32,
    surface_level: f32,
    ao_max_dist: f32,
}


fn fill_sphere(chunk: &mut Chunk, center_uvw: Vec3) {
    chunk.march_parameters.distance_field.bind_image(1, TextureAccess::Write);

    chunk.march_parameters.programm_storage.access().get::<FillCircleProgramm>().unwrap()
    .bind().set_uniforms(FillTextureUniforms {
        img_output: 1.into(),
        center_uvw: chunk_to_texture_position(center_uvw, TEXTURE_DIM),
    }).unwrap();

    let c = chunk.march_parameters.sync_context.dirty() | ShaderImageAccessBarrier;
    GL!(gl::DispatchCompute((TEXTURE_DIM.x) as u32, 
        (TEXTURE_DIM.y) as u32, 
        (TEXTURE_DIM.z) as u32));
    c.apply();

    chunk.is_sdf_top_level_dirty = true;

    chunk.march_parameters.dirty_area = Bounds::min_max(IVec3::ZERO, NUM_OF_CUBES);     
}

impl Chunk {
    fn uninitialized(sync_context: SynchronizationContext, programm_storage: ShaderStorage, debugger: Debugger) -> Chunk {
        let model_vertex_buffer = VertexBuffer::from_data(
            &vec![ModelVertex::default(); (NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z * 15) as usize],
            Usage::dynamic_copy());

        let collision_field = CollisionShape::new();

        let img_format = ImageFormat {
            lod: 0,
            format: gl::RED,
            internal_format: gl::R32F,
            data_type: gl::FLOAT,
        };
        let texture = Texture::new_3d()
            .magnification_filter(FilterMode::Linear)
            .minification_filter(MipMapFilterMode::TRILINEAR )
            .wrap_mode_x(WrapMode::ClampToEdge)
            .wrap_mode_y(WrapMode::ClampToEdge)
            // .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, ImageFormat {
            //     lod: 0,
            //     format: gl::RED,
            //     internal_format: gl::R32F,
            //     data_type: gl::FLOAT
            // });
            .wrap_mode_z(WrapMode::ClampToEdge)
            .empty(TEXTURE_DIM.x, TEXTURE_DIM.y, TEXTURE_DIM.z, img_format);

        sync_context.force_sync_with(AllBarrier);

        let march_parameters = MarchParameters {
            sync_context,
            distance_field: texture,
            programm_storage,
            chunk_scale_factor: Vec3::ONE,
            num_of_cubes: NUM_OF_CUBES,
            model_vertex_buffer,
            collision_field,
            surface_level: 0.3,
            dirty_area: Bounds::empty(),
        };

        let marcher: Box<dyn CubeMarcher> = if BLOCK_WRITE { 
            Box::new(BlockCubeMarcher::new(&march_parameters))
        } else { 
            Box::new(FullCubeMarcher::new())
        };

        Chunk { 
            quad: None,
            marcher,
            is_sdf_top_level_dirty: false,
            debugger,
            march_parameters,
            collider_drawer: None,
            is_collision_shape_dirty: false, 
        }
    }

    pub fn sphere(sync_context: SynchronizationContext, programm_storage: ShaderStorage, debugger: Debugger, center: Vec3) -> Chunk {
        let mut c = Chunk::uninitialized(sync_context, programm_storage, debugger);
        fill_sphere(&mut c, center);
        c.march();
        c
    }

    // pub fn empty(sync_context: SynchronizationContext, programm_storage: ShaderStorage) -> Chunk {
    //     let mut c = Chunk::uninitialized(sync_context, programm_storage);
    //     fill_empty(&mut c);
    //     // c.march();
    //     c
    // }

    pub fn march(&mut self) {
        self.before_march();
        for step in 0..self.march_steps_count() {
            self.march_step(step);
        }
        self.after_march();
    }

    pub fn march_steps_count(&self) -> usize {
        self.marcher.march_steps_count()
    } 

    pub fn march_step(&mut self, step: usize) {
        self.marcher.march(step, &mut self.march_parameters);
    }

    pub fn before_march(&mut self) {
        self.march_parameters.sync_context.sync_with(ShaderImageAccessBarrier);
    }

    pub fn after_march(&mut self) {
        self.is_collision_shape_dirty = true;
        self.march_parameters.dirty_area = Bounds::empty();
    }

    pub fn draw(&mut self, 
        draw_parameters: DrawParameters<'_>, 
        ao_max_dist: f32) {
        self.actualise_texture();

        self.march_parameters.distance_field.bind(1);
        self.march_parameters.programm_storage.access().get::<ShadedModelProgramm>().unwrap()
        .bind().set_uniforms(ShadedModelDisplayUniform {
            model: *draw_parameters.model,
            view: draw_parameters.camera.view_matrix(),
            projection: draw_parameters.camera.projection_matrix(),
            chunk_scale_factor: CHUNK_SCALE_FACTOR,
            scalar_field: 1.into(),
            field_chunk_size_diff: TEXTURE_SIZE_DELTA,
            surface_level: self.march_parameters.surface_level,
            light_direction: vec3(1., 1., -0.5).normalize(),
            ao_max_dist,
            // ao_upper_edge
        }).unwrap();

        self.marcher.draw(&mut self.march_parameters);
    }

    pub fn draw_distance_field(&mut self, params: DrawParameters<'_>, slice: f32) {
        if self.quad.is_none() {
            self.quad = Some(SimpleQuad::new(self.march_parameters.programm_storage.clone()).unwrap());
        }

        self.actualise_texture();

        self.march_parameters.distance_field.bind(1);
        self.quad.as_mut().unwrap().draw(params, TextureUnit(1), slice);
    }

    pub fn draw_debug_vew(&mut self, params: DrawParameters<'_>) {
        
        let mut actualize = false;

        let drawer = self.collider_drawer.get_or_insert_with(|| {
            actualize = true;
            CollisionShapeDebugView::new(self.march_parameters.programm_storage.clone())
        });
        if self.is_collision_shape_dirty {
            actualize = true;
            self.march_parameters.sync_context.sync_with(ShaderStorageBarrier);
            // self.bit_field.readback();
            self.march_parameters.collision_field.readback();
            self.is_collision_shape_dirty = false;
            drawer.actualize(&mut self.march_parameters.collision_field);
        };
        
        if actualize {
            drawer.actualize(&mut self.march_parameters.collision_field);
        }
        
        drawer.draw(params);
    }

    // pub fn raycast(&mut self, ray: Ray, draw: impl Fn(Vec3, Vec3)) -> Option<Vec3> {
    pub fn raycast(&mut self, ray: Ray) -> Option<Vec3> {

        if self.is_collision_shape_dirty {
            self.march_parameters.sync_context.sync_with(ShaderStorageBarrier);
            // self.bit_field.readback();
            self.march_parameters.collision_field.readback();
            self.is_collision_shape_dirty = false;
        };


        for c in march_grid_by_ray(
            ray.origin, ray.direction, IVec3::ZERO, NUM_OF_CUBES - IVec3::ONE)? {
            // let center = (c.as_vec3() + Vec3::ONE * 0.5) * CHUNK_SCALE_FACTOR;
            // let size = CHUNK_SCALE_FACTOR;
            // draw(center, size);

            // let bounds = Bounds::min_max(c.as_vec3(), Vec3::ONE);
                
            let triangles = self.march_parameters.collision_field.get(c);

            if triangles.len() > 0 {
                self.debugger.draw_width(DebugPrimitive::Box { corner: c.as_vec3(), size: Vec3::ONE }, Color32::RED, 2.);
            }
            else {
                self.debugger.draw_width(DebugPrimitive::Box { corner: c.as_vec3(), size: Vec3::ONE }, Color32::RED, 0.1);
            }

            for triangle in triangles {
                let intersection: Option<Vec3> = ray_triangle_intersection(ray, &triangle);
                let debug_enabled = self.debugger.is_debug_enabled();
                if debug_enabled {
                    self.debugger.draw(DebugPrimitive::Triangle(triangle.clone()), Color32::GOLD);
                }
                if intersection.is_some() {
                    if debug_enabled {
                        self.debugger.draw_width(DebugPrimitive::Triangle(triangle.clone()), Color32::BLUE, 2.);
                    }
                    return intersection;
                }
            }
        }

        None
    }

    fn actualise_texture(&mut self) {
        self.march_parameters.sync_context.sync_with(ShaderImageAccessBarrier);
        if self.is_sdf_top_level_dirty {
            self.march_parameters.distance_field.generate_mips();
            self.is_sdf_top_level_dirty = false;
        }
    }

    pub fn before_brush(&mut self) {
        self.march_parameters.sync_context.sync_with(ShaderImageAccessBarrier);
    }

    pub fn apply_brush(&mut self, brush: &impl Brush) {      
        
        let c = self.march_parameters.sync_context.dirty() | ShaderImageAccessBarrier;
        
        self.march_parameters.dirty_area.encapsulate_other(&brush.chunk_space_cords(NUM_OF_CUBES));
        // self.swap_buffer_is_actual = true;
        brush.apply(&mut self.march_parameters.distance_field);

        self.is_sdf_top_level_dirty = !self.march_parameters.dirty_area.is_empty();
        
        c.apply();
        // self.sync_context.sync_with(AllBarrier);
    }
}