use core::{buffers::buffer::{Buffer, BufferDataInterface, Usage}, context::synchronization_context::{BufferUpdateBarrier, ShaderImageAccessBarrier, ShaderStorageBarrier, SynchronizationContext}, textures::{texture::TextureAccess, TextureUnit}, GL};

use glam::{IVec3, Mat4, Vec3};

use crate::{application::support::{bounds::Bounds, brush::WORK_GROUP_FOR_BRUSH, shaders::shaders_loader::{ShaderStorage, ShaderType}}, dispatch_size, shader_ref};

use super::{chunk_size_to_texture_size, chunk_to_texture_position, dispatch_compute_for, texture_space_cords, BrushSettings};
use egui_glfw_gl::{egui, gl};

shader_ref!(FlattenCountProgramm, 
    ShaderType::Compute("resources/shader_sources/brushes/flatten_brush_count.compute"),
    dispatch_size!(WORK_GROUP_FOR_BRUSH));

shader_ref!(FlattenApplyProgramm, 
    ShaderType::Compute("resources/shader_sources/brushes/flatten_brush_apply.compute"),
    dispatch_size!(WORK_GROUP_FOR_BRUSH));

#[derive(Uniforms)]
#[for_shaders("resources/shader_sources/brushes/flatten_brush_apply.compute")]
struct FlattenUniforms {
    source: TextureUnit,
    transform: Mat4,
    start_cell: IVec3,
    strength: f32,
    tex_space_center: Vec3
}

pub struct FlattenBrush {
    sync_context: SynchronizationContext,
    shader_storage: ShaderStorage,
    radius: f32,
    strenght: f32,
}

impl FlattenBrush {
    pub fn new(
        sync_context: SynchronizationContext, 
        shader_storage: ShaderStorage
    ) -> FlattenBrush {
        let counter_buffer = Buffer::from_data(&[0u32;4], Usage::dynamic_copy());
        FlattenBrush { 
            sync_context, 
            shader_storage, 
            radius: 0.1, 
            strenght: 0.1, 
        }
    }
}


impl BrushSettings for FlattenBrush {
    fn bounds(&self, parameters: &super::BrushApplicationParameters
    ) -> Bounds<Vec3> {
        let center = parameters.transform.position();
        let h_size = parameters.transform.scale() * self.radius;

        Bounds::min_max(center - h_size, center + h_size)
    }

    fn apply(&mut self, source: &mut core::textures::texture::Texture, parameters: &super::BrushApplicationParameters) {
        
        // self.counter_buffer.update_data(0, &[0u32;4]);
        // self.sync_context.force_sync(BufferUpdateBarrier);

        let tex_dim = source.size();

        let bounds = self.bounds(parameters);
        let dispatch_bounds = texture_space_cords(bounds, tex_dim);

        let tex_space_center = chunk_to_texture_position(parameters.transform.position(), tex_dim);
        let transform = Mat4::from_scale_rotation_translation(
            chunk_size_to_texture_size(
            parameters.transform.scale() * self.radius, tex_dim),
            parameters.transform.rotation(), 
            tex_space_center
        );

        source.bind_image(1, TextureAccess::ReadWrite);

        self.sync_context.force_sync(ShaderStorageBarrier);
        
        // source.bind_image(1, TextureAccess::ReadWrite);
        self.shader_storage.access().get::<FlattenApplyProgramm>().unwrap()
            .bind()
            .set_uniforms(FlattenUniforms {
                source: 1.into(),
                start_cell: dispatch_bounds.min(),
                transform: transform.inverse(),
                strength: self.strenght,
                tex_space_center
            }).unwrap();

        dispatch_compute_for(dispatch_bounds.size());
    }

    fn display_ui(&mut self, ui: &mut egui_glfw_gl::egui::Ui) {
        ui.add(egui::Slider::new(&mut self.radius, 0.05..=0.2).text("radius"));
        ui.add(egui::Slider::new(&mut self.strenght, 0.0..=0.2).text("strength"));
        // ui.add(egui::Slider::new(&mut self.bias, 0.0..=5.).text("bias"));

    }

    fn brush_name(&self) -> &'static str {
        "flatten"
    }
}