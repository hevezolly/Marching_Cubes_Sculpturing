use core::{buffers::buffer::{Usage, VertexBuffer}, context::synchronization_context::{AllBarrier, ShaderImageAccessBarrier, ShaderStorageBarrier, SynchronizationContext}, textures::{image_provider::ImageFormat, texture::{FilterMode, Texture, TextureAccess, WrapMode}, TextureUnit}, GL};

use egui_glfw_gl::{egui::Color32, gl};
use glam::{ivec3, vec3, IVec3, Mat4, Vec3};

use crate::{algorithms::{camera::Camera, grid_line_intersection::march_grid_by_ray, raycast::{ray_box_intersection, ray_triangle_intersection, IntersectionResult, Ray}}, application::{app_logick::NUM_OF_CUBES, support::{bounds::Bounds, brush::{chunk_to_texture_position, Brush}, debugger::{DebugPrimitive, Debugger}, shaders::{shaders_loader::ShaderStorage, FillCircleProgramm, ZeroFieldProgramm}, simple_quad::SimpleQuad}}};

use super::{collision_shape::{CollisionShape, CollisionShapeDebugView}, marching_cubes::{block_marcher::BlockCubeMarcher, full_marcher::FullCubeMarcher, CubeMarcher, MarchParameters}, DrawParameters, ModelVertex};

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
    is_collision_shape_dirty: bool,
    collider_drawer: Option<CollisionShapeDebugView>
}

pub const TEXTURE_DIM: IVec3 = IVec3 { 
    x: NUM_OF_CUBES.x + 3, 
    y: NUM_OF_CUBES.y + 3, 
    z: NUM_OF_CUBES.z + 3 
};


const BLOCK_WRITE: bool = true;


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

    chunk.march_parameters.dirty_area = Bounds::min_max(IVec3::ZERO, NUM_OF_CUBES);     
}

// #[derive(Uniforms)]
// #[for_shaders("resources/shader_sources/marching_cubes/zero_field.compute")]
// struct ZeroFieldUniforms {
//     #[name("imgOutput")]
//     img_output: TextureUnit,
//     // imgOutput: TextureUnit
// }  

// fn fill_empty(chunk: &mut Chunk) {
//     chunk.march_parameters.distance_field.bind_image(1, TextureAccess::Write);

//     chunk.march_parameters.programm_storage.access().get::<ZeroFieldProgramm>().unwrap()
//     .bind().set_uniforms(ZeroFieldUniforms {
//         img_output: 1.into(),
//     }).unwrap();

//     let c = chunk.march_parameters.sync_context.dirty() | ShaderImageAccessBarrier;
//     GL!(gl::DispatchCompute((TEXTURE_DIM.x) as u32, 
//         (TEXTURE_DIM.y) as u32, 
//         (TEXTURE_DIM.z) as u32));
//     c.apply();

//     // chunk.march_parameters.dirty_area = Bounds::min_max(IVec3::ZERO, NUM_OF_CUBES);     
// }

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

        let march_parameters = MarchParameters {
            sync_context,
            distance_field: texture,
            programm_storage,
            chunk_scale_factor: Vec3::ONE,
            num_of_cubes: NUM_OF_CUBES,
            model_vertex_buffer,
            collision_field,
            dirty_area: Bounds::empty(),
        };

        let marcher: Box<dyn CubeMarcher> = if BLOCK_WRITE { 
            Box::new(BlockCubeMarcher::new(&march_parameters))
        } else { 
            Box::new(FullCubeMarcher::new())
        };

        Chunk { 
            marcher,
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
        self.actualise_texture();
        self.marcher.march(&mut self.march_parameters);
        self.is_collision_shape_dirty = true;
        self.march_parameters.dirty_area = Bounds::empty();
    }

    pub fn draw(&mut self, draw_parameters: DrawParameters<'_>) {

        // for x in 0..=2 {
        //     for y in 0..=2 {
        //         for z in 0..=2 {
        //             self.debugger.draw(DebugPrimitive::Box { corner: (ivec3(x, y, z) * (NUM_OF_CUBES - IVec3::ONE)).as_vec3() * 0.5, 
        //                 size: Vec3::ONE }, Color32::YELLOW);
        //         }
        //     }
        // }
        

        self.marcher.draw(draw_parameters, &mut self.march_parameters);
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
            drawer.actualize(&self.march_parameters.collision_field);
        };
        
        if actualize {
            drawer.actualize(&self.march_parameters.collision_field);
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

            let result = ray_box_intersection(c.as_vec3(), c.as_vec3() + Vec3::ONE, ray);
            if let IntersectionResult::None = result {
                self.debugger.draw(DebugPrimitive::Point(c.as_vec3() + Vec3::ONE * 0.5), Color32::BLUE);
                // panic!("incorrect intersection! {:?} {:?}", c, ray);
            }
                
            let triangles = self.march_parameters.collision_field.get(c);

            if triangles.len() > 0 {
                self.debugger.draw(DebugPrimitive::Box { corner: c.as_vec3(), size: Vec3::ONE }, Color32::RED);
            }

            for triangle in triangles {
                self.debugger.draw(DebugPrimitive::Triangle(triangle), Color32::GOLD);
                let intersection: Option<Vec3> = ray_triangle_intersection(ray, &triangle);
                if intersection.is_some() {
                    return intersection;
                }
            }
        }

        None
    }

    fn actualise_texture(&mut self) {
        self.march_parameters.sync_context.sync_with(ShaderImageAccessBarrier);
    }

    pub fn apply_brush(&mut self, brush: &impl Brush) {
        self.actualise_texture();
        
        
        let c = self.march_parameters.sync_context.dirty() | ShaderImageAccessBarrier;
        
        self.march_parameters.dirty_area.encapsulate_other(&brush.chunk_space_cords(NUM_OF_CUBES));
        // self.swap_buffer_is_actual = true;
        brush.apply(&mut self.march_parameters.distance_field);

        c.apply();
        // self.sync_context.sync_with(AllBarrier);
    }
}