use core::{textures::{texture::{Texture, TextureAccess}, TextureUnit}, GL};
use std::cmp::{max, min};

use egui_glfw_gl::gl;
use glam::{ivec3, vec2, IVec3, Mat4, Quat, Vec2, Vec3};

use crate::{algorithms::{cordinates::RoundableToIVec3, transform}, application::support::{bounds::Bounds, brush::WORK_GROUP_FOR_BRUSH, shaders::shaders_loader::{ShaderStorage, ShaderType}}, shader_ref};

use super::{dispatch_compute_for, Brush};


shader_ref!(CirckleBrushProgramm, ShaderType::Compute("resources/shader_sources/brushes/circle_brush.compute"),
    format!("DISPATCH_SIZE local_size_x = {}, local_size_y = {}, local_size_z = {}",
    WORK_GROUP_FOR_BRUSH.x,
    WORK_GROUP_FOR_BRUSH.y,
    WORK_GROUP_FOR_BRUSH.z));

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/brushes/circle_brush.compute")]
struct CircleBrushUniforms {
    source: TextureUnit,
    // start_cell: IVec3,
    destination: TextureUnit,
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

    pub fn clone_at(&self, center: Vec3, radius: f32) -> CircleBrush {
        let mut clone = self.clone();
        clone.center = center;
        clone.radius = radius;
        clone
    }
}

impl Brush for CircleBrush {
    fn apply(&self, source: &mut Texture, destination: &mut Texture) {
        let tex_dim = source.size();

        // let bounds = self.bounds();
        // let start_cell = (bounds.min() * tex_dim.as_vec3()).floor_to_ivec();
        // let dispatch_size = IVec3::min(start_cell + 
        //     (bounds.size() * tex_dim.as_vec3()).ceil_to_ivec(), tex_dim) - start_cell;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.radius,
            Quat::IDENTITY, 
        self.center).inverse();
        source.bind_image(2, TextureAccess::Read);
        destination.bind_image(1, TextureAccess::Write);
        self.shader_storage.access().get::<CirckleBrushProgramm>().unwrap()
            .bind()
            .set_uniforms(CircleBrushUniforms {
                source: 2.into(),
                // start_cell,
                destination: 1.into(),
                transform,
                falloff_strength: vec2(self.falloff, -self.strength),
            }).unwrap();

        dispatch_compute_for(tex_dim);
        // GL!(gl::DispatchCompute(tex_dim.x as u32, tex_dim.y as u32, tex_dim.z as u32));

        // source.unbind();
        // destination.unbind();
    }
    
    fn bounds(&self) -> crate::application::support::bounds::Bounds<Vec3> {
        let h_size = Vec3::ONE * self.radius * 0.5;
        Bounds::min_max(self.center - h_size, self.center + h_size)
    }
}