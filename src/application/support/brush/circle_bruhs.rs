use core::{textures::{texture::{Texture, TextureAccess}, TextureUnit}, GL};
use std::cmp::{max, min};

use egui_glfw_gl::egui::Ui;
use egui_glfw_gl::egui;
use glam::{ivec3, vec2, IVec3, Mat4, Quat, Vec2, Vec3};

use crate::{algorithms::{cordinates::RoundableToIVec3, transform}, application::{cunks::field::CHUNK_SCALE_FACTOR, support::{bounds::Bounds, brush::WORK_GROUP_FOR_BRUSH, shaders::shaders_loader::{ShaderStorage, ShaderType}}}, shader_ref};

use super::{chunk_size_to_texture_size, chunk_to_texture_position, dispatch_compute_for, texture_space_cords, Brush, BrushApplicationParameters, BrushSettings};


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
    radius: f32,
    strength: f32,
    falloff: f32,
    shader_storage: ShaderStorage
}

impl CircleBrush {
    pub fn new(
        shader_storage: ShaderStorage,
    ) -> CircleBrush {
        CircleBrush { radius: 0.1, strength: 0.01, falloff: 1., shader_storage }
    }
}

impl BrushSettings for CircleBrush {
    fn apply(&mut self, source: &mut Texture, parameters: &BrushApplicationParameters) {
        let tex_dim = source.size();

        let bounds = self.bounds(parameters);
        let dispatch_bounds = texture_space_cords(bounds, tex_dim);

        let transform = 
            Mat4::from_scale_rotation_translation(
            chunk_size_to_texture_size(
                parameters.transform.scale() * self.radius, tex_dim),
            parameters.transform.rotation(), 
            chunk_to_texture_position(parameters.transform.position(), tex_dim));

        source.bind_image(1, TextureAccess::ReadWrite);
        self.shader_storage.access().get::<CirckleBrushProgramm>().unwrap()
            .bind()
            .set_uniforms(CircleBrushUniforms {
                source: 1.into(),
                start_cell: dispatch_bounds.min(),
                transform: transform.inverse(),
                falloff_strength: vec2(self.falloff, 
                    -self.strength * parameters.frame_rate_coefficient),
            }).unwrap();

        dispatch_compute_for(dispatch_bounds.size());
        // GL!(gl::DispatchCompute(tex_dim.x as u32, tex_dim.y as u32, tex_dim.z as u32));

        // source.unbind();
        // destination.unbind();
    }
    
    fn bounds(&self, parameters: &BrushApplicationParameters) -> crate::application::support::bounds::Bounds<Vec3> {
        let h_size = parameters.transform.scale() * self.radius;
        Bounds::min_max(
            parameters.transform.position() - h_size, 
            parameters.transform.position() + h_size)
    }
    
    fn display_ui(&mut self, ui: &mut Ui) {
        ui.add(egui::Slider::new(&mut self.radius, 0.05..=0.2).text("radius"));
        ui.add(egui::Slider::new(&mut self.strength, 0.0..=0.1).text("strength"));
        ui.add(egui::Slider::new(&mut self.falloff, 0.1..=5.).text("sharpness"));
    }
    
    fn brush_name(&self) -> &'static str {
        "add"
    }
}

pub struct InverseCircleBrush(CircleBrush);

impl InverseCircleBrush {
    pub fn new(
        shader_storage: ShaderStorage,
    ) -> InverseCircleBrush {

        let mut c = CircleBrush::new(shader_storage);
        c.strength = -c.strength;

        InverseCircleBrush(c)
    }
}

impl BrushSettings for InverseCircleBrush {
    fn bounds(&self, parameters: &BrushApplicationParameters) -> Bounds<Vec3> {
        self.0.bounds(parameters)
    }

    fn apply(&mut self, source: &mut Texture, parameters: &BrushApplicationParameters) {
        self.0.apply(source, parameters)
    }

    fn display_ui(&mut self, ui: &mut Ui) {
        ui.add(egui::Slider::new(&mut self.0.radius, 0.05..=0.2).text("radius"));
        
        let mut strength = - self.0.strength;
        ui.add(egui::Slider::new(&mut strength, 0.0..=0.1).text("strength"));
        self.0.strength = -strength;

        ui.add(egui::Slider::new(&mut self.0.falloff, 0.1..=5.).text("sharpness"));
    }
    
    fn brush_name(&self) -> &'static str {
        "remove"
    }
}