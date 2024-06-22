use core::context::synchronization_context::{ShaderImageAccessBarrier, SynchronizationContext};
use std::{collections::HashMap};

use egui_glfw_gl::egui::Color32;
use glam::{vec3, BVec3, IVec3, Mat4, Quat, Vec3};

use crate::{algorithms::{camera::Camera, cordinates::{to_vec3_const, RoundableToIVec3}, grid_line_intersection::march_grid_by_ray, raycast::Ray}, application::{app_logick::NUM_OF_CUBES, support::{bounds::{Bounds, Cord3D}, brush::Brush, debugger::{DebugPrimitive, Debugger}, shaders::shaders_loader::ShaderStorage}}};

use super::{chunk::{self, Chunk, TEXTURE_OFFSET, TEXTURE_SIZE_DELTA, TEXTURE_SIZE_DELTA_HALVED}, DrawParameters};



pub struct Field {
    chunks: HashMap<IVec3, Chunk>,
    chunk_bounds: Bounds<IVec3>,
    sync_context: SynchronizationContext,
    shader_storage: ShaderStorage,
    debugger: Debugger,
}

fn chunk_position(cord: IVec3) -> Vec3 {
    cord.as_vec3() * CHUNK_SIZE
}

fn chunk_matrix(cord: IVec3) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        CHUNK_SCALE_FACTOR, 
        Quat::IDENTITY, 
        chunk_position(cord))
}

pub const CHUNK_SCALE_FACTOR: Vec3 = vec3(1. / (NUM_OF_CUBES.x as f32), 
    1. / (NUM_OF_CUBES.y as f32),
    1. / (NUM_OF_CUBES.z as f32));

const CHUNK_SIZE: Vec3 = Vec3 {
    x: CHUNK_SCALE_FACTOR.x * NUM_OF_CUBES.x as f32,
    y: CHUNK_SCALE_FACTOR.y * NUM_OF_CUBES.y as f32,
    z: CHUNK_SCALE_FACTOR.z * NUM_OF_CUBES.z as f32,
};

const MARCH_BY_STEPS: bool = true;



impl Field {
    pub fn new(sync_context: SynchronizationContext, shader_storage: ShaderStorage, debugger: Debugger) -> Field {
        let chunks = HashMap::new();

        let mut f = Field { 
            debugger,
            chunks, 
            sync_context, 
            shader_storage, 
            chunk_bounds: Bounds::empty() };

        f.insert_chunk_at(IVec3::ZERO);
        f
    }

    fn insert_chunk_at(&mut self, cord: IVec3) {
        let c = Chunk::sphere(
            self.sync_context.clone(), 
            self.shader_storage.clone(), 
            // self.debugger.clone(),
            self.debugger.clone_with_matrix(chunk_matrix(cord)),
            -cord.as_vec3() + Vec3::ONE * 0.5);
        self.chunks.insert(cord, c);
        self.chunk_bounds.encapsulate(cord);
    }

    pub fn draw_distance_field(&mut self, camera: &impl Camera, slice: f32, parity: BVec3) {
        for (cord, chunk) in self.chunks.iter_mut() {

            if (cord.x % 2 == 0) != parity.x ||
               (cord.y % 2 == 0) != parity.y ||
               (cord.z % 2 == 0) != parity.z {
                continue;
            }
            
            let chunk_position = chunk_position(*cord);
            // self.debugger.draw(DebugPrimitive::Box { 
            //     corner: chunk_position - 1.5 * CHUNK_SCALE_FACTOR, 
            //     size: CHUNK_SIZE + 3. * CHUNK_SCALE_FACTOR 
            // }, Color32::YELLOW);

            let position = chunk_position - 
                TEXTURE_SIZE_DELTA_HALVED.x * CHUNK_SCALE_FACTOR.x * Vec3::X - 
                TEXTURE_SIZE_DELTA_HALVED.y * CHUNK_SCALE_FACTOR.y * Vec3::Y;
            let offset = -TEXTURE_SIZE_DELTA_HALVED.z * CHUNK_SCALE_FACTOR.z + 
                (1. + TEXTURE_SIZE_DELTA.as_vec3().z * CHUNK_SCALE_FACTOR.z) * slice;
            let model = Mat4::from_scale_rotation_translation(
                Vec3::ONE + TEXTURE_SIZE_DELTA.as_vec3() * CHUNK_SCALE_FACTOR, 
                Quat::IDENTITY, 
                position + Vec3::Z * offset);
            chunk.draw_distance_field(DrawParameters { camera: camera, model: &model }, slice);
        }
    }


    pub fn raycast(&mut self, ray: Ray) -> Option<Vec3> {
        
        for chunk_cord in march_grid_by_ray(
                ray.origin / CHUNK_SIZE, 
                ray.direction / CHUNK_SIZE, 
                self.chunk_bounds.min(),
                self.chunk_bounds.max() + IVec3::ONE)? {

            let chunk = self.chunks.get_mut(&chunk_cord);
            if chunk.is_none() {continue;}

            
            let chunk_position = chunk_position(chunk_cord);
            // self.debugger.draw(DebugPrimitive::Box { corner: chunk_position, size: CHUNK_SIZE }, Color32::GREEN);
            // self.debugger.draw(DebugPrimitive::Box { corner: chunk_position + CHUNK_SIZE * 0.25, size: CHUNK_SIZE * 0.5 }, Color32::YELLOW);

            let adjusted_ray = Ray::new((ray.origin - chunk_position) / CHUNK_SCALE_FACTOR, ray.direction / CHUNK_SCALE_FACTOR);

            if let Some(hit) = chunk.unwrap().raycast(adjusted_ray) {
                return Some(hit * CHUNK_SCALE_FACTOR + chunk_position);
            }
        };
        None
    }

    pub fn draw(&mut self, camera: &impl Camera, 
        ao_max_dist: f32, 
        ao_falloff: f32,
        ao_upper_edge: f32) {
        for (cord, chunk) in self.chunks.iter_mut() {

            
            let chunk_position = chunk_position(*cord);
            self.debugger.draw(DebugPrimitive::Box { corner: chunk_position, size: CHUNK_SIZE }, Color32::GREEN);
            // self.debugger.draw(DebugPrimitive::Box { 
            //     corner: chunk_position - 1.5 * CHUNK_SCALE_FACTOR, 
            //     size: CHUNK_SIZE + 3. * CHUNK_SCALE_FACTOR 
            // }, Color32::YELLOW);
            let model = chunk_matrix(*cord);
                
                chunk.draw(DrawParameters { camera: camera, model: &model }, 
                    TEXTURE_OFFSET.x as f32 * CHUNK_SCALE_FACTOR.x, 
                    ao_falloff, 
                ao_upper_edge);
        }
    }

    pub fn debug(&mut self, camera: &impl Camera) {
        for (cord, chunk) in self.chunks.iter_mut() {
            let chunk_position = chunk_position(*cord);
            // self.debugger.draw(DebugPrimitive::Box { 
            //     corner: chunk_position - 1.5 * CHUNK_SCALE_FACTOR, 
            //     size: CHUNK_SIZE + 3. * CHUNK_SCALE_FACTOR 
            // }, Color32::YELLOW);
            let model = chunk_matrix(*cord);
                
                chunk.draw_debug_vew(DrawParameters { camera: camera, model: &model })
        }
    }

    pub fn apply_brush(&mut self, brush: &impl Brush) {

        let bounds = brush.bounds();


        // self.debugger.draw(DebugPrimitive::Bounds(&bounds), Color32::RED);

        let chunk_bounds = Bounds::min_max(
            ((bounds.min() - TEXTURE_SIZE_DELTA_HALVED * CHUNK_SCALE_FACTOR) / CHUNK_SIZE).floor_to_ivec(),
            ((bounds.max() + TEXTURE_SIZE_DELTA_HALVED * CHUNK_SCALE_FACTOR) / CHUNK_SIZE).floor_to_ivec()
        );

        if MARCH_BY_STEPS {
            let cords: Vec<_> = chunk_bounds.iterate_cords().collect();
    
            for cord in &cords {
                if !self.chunks.contains_key(&cord) {
                    self.insert_chunk_at(*cord);
                    // dbg!("insert");
                }
            }
    
    
            for cord in &cords {
                let chunk = self.chunks.get_mut(cord).unwrap();
                let chunk_pos = chunk_position(*cord);
                let chunk_local_brush = brush.transformed(-chunk_pos, 1.);
                chunk.apply_brush(&chunk_local_brush)
            }
    
            let mut max_march_steps = 0;
            for cord in &cords {
                let chunk = self.chunks.get_mut(cord).unwrap();
                chunk.before_march();
                max_march_steps = usize::max(max_march_steps, chunk.march_steps_count());
            }
    
            for step in 0..max_march_steps {
                for cord in &cords {
                    let chunk = self.chunks.get_mut(cord).unwrap();
                    if step >= chunk.march_steps_count() {
                        continue;
                    }
                    chunk.march_step(step);
                }
            }
    
            for cord in &cords {
                let chunk = self.chunks.get_mut(cord).unwrap();
                chunk.after_march();
            }
        }
        else {
            for cord in chunk_bounds.iterate_cords() {
    
                // dbg!(cord);
    
                if !self.chunks.contains_key(&cord) {
                    self.insert_chunk_at(cord);
                    // dbg!("insert");
                }
    
                if let Some(chunk) = self.chunks.get_mut(&cord) {
                    let chunk_pos = chunk_position(cord);
                    let chunk_local_brush = brush.transformed(-chunk_pos, 1.);
                    chunk.before_brush();
                    chunk.apply_brush(&chunk_local_brush);
    
                    chunk.march();
                }
            }
        }



    }
}