use core::{textures::{texture::{Texture, TextureAccess}, TextureUnit}, GL};
use std::cmp::{max, min};

use egui_glfw_gl::gl;
use glam::{ivec3, vec2, IVec3, Mat4, Quat, Vec2, Vec3};

use crate::{algorithms::{cordinates::RoundableToIVec3, transform}, application::{cunks::field::CHUNK_SCALE_FACTOR, support::{bounds::Bounds, brush::WORK_GROUP_FOR_BRUSH, shaders::shaders_loader::{ShaderStorage, ShaderType}}}, shader_ref};

use super::{chunk_size_to_texture_size, chunk_to_texture_position, dispatch_compute_for, Brush};


shader_ref!(CirckleBrushProgramm, ShaderType::Compute("resources/shader_sources/brushes/circle_brush.compute"),
    format!("DISPATCH_SIZE local_size_x = {}, local_size_y = {}, local_size_z = {}",
    WORK_GROUP_FOR_BRUSH.x,
    WORK_GROUP_FOR_BRUSH.y,
    WORK_GROUP_FOR_BRUSH.z));

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/brushes/circle_brush.compute")]
struct CircleBrushUniforms {
    source: TextureUnit,
    start_cell: IVec3,
    // destination: TextureUnit,
    transform: Mat4,
    falloff_strength: Vec2
}

#[derive(Debug, Clone)]
pub struct CircleBrush {
    center: Vec3,
    radius: f32,
    strength: f32,
    falloff: f32,
    shader_storage: ShaderStorage
}

impl CircleBrush {
    pub fn new(
        shader_storage: ShaderStorage, 
        center: Vec3,
        radius: f32,
        strength: f32,
        falloff: f32
    ) -> CircleBrush {
        CircleBrush { center, radius, strength, falloff, shader_storage }
    }
}

impl Brush for CircleBrush {
    fn apply(&self, source: &mut Texture) {
        let tex_dim = source.size();

        let dispatch_bounds = self.texture_space_cords(tex_dim);

        let transform = 
            Mat4::from_scale_rotation_translation(
            chunk_size_to_texture_size(Vec3::ONE * self.radius, tex_dim),
            Quat::IDENTITY, 
            chunk_to_texture_position(self.center, tex_dim));

        source.bind_image(1, TextureAccess::ReadWrite);
        self.shader_storage.access().get::<CirckleBrushProgramm>().unwrap()
            .bind()
            .set_uniforms(CircleBrushUniforms {
                source: 1.into(),
                start_cell: dispatch_bounds.min(),
                transform: transform.inverse(),
                falloff_strength: vec2(self.falloff, -self.strength),
            }).unwrap();

        dispatch_compute_for(dispatch_bounds.size());
        // GL!(gl::DispatchCompute(tex_dim.x as u32, tex_dim.y as u32, tex_dim.z as u32));

        // source.unbind();
        // destination.unbind();
    }
    
    fn bounds(&self) -> crate::application::support::bounds::Bounds<Vec3> {
        let h_size = Vec3::ONE * self.radius + CHUNK_SCALE_FACTOR;
        Bounds::min_max(self.center - h_size, self.center + h_size)
    }
    
    fn transformed(&self, offset: Vec3, scale: f32 ) -> Self {
        Self { center: self.center + offset, 
               radius: self.radius * scale, 
               strength: self.strength, 
               falloff: self.falloff, 
               shader_storage: self.shader_storage.clone() }
    }
}