use core::{textures::{texture::{Texture, TextureAccess}, TextureUnit}, GL};

use egui_glfw_gl::gl;
use glam::{vec2, Mat4, Quat, Vec2, Vec3};

use crate::{algorithms::transform, application::support::shaders::shaders_loader::{ShaderStorage, ShaderType}, shader_ref};

use super::Brush;


shader_ref!(CirckleBrushProgramm, ShaderType::Compute("resources/shader_sources/brushes/circle_brush.compute"));

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/brushes/circle_brush.compute")]
struct CircleBrushUniforms {
    source: TextureUnit,
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
                destination: 1.into(),
                transform,
                falloff_strength: vec2(self.falloff, -self.strength),
            }).unwrap();
        let tex_dim = source.size();
        GL!(gl::DispatchCompute(tex_dim.x as u32, tex_dim.y as u32, tex_dim.z as u32));

        // source.unbind();
        // destination.unbind();
    }
}