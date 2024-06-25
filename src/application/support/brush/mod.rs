use core::{textures::texture::Texture, GL};
use std::sync::{Arc, Mutex};

use egui_glfw_gl::{egui::Ui, gl};
use glam::{IVec3, Mat4, Quat, Vec3};

use crate::{algorithms::{cordinates::RoundableToIVec3, transform::Transform}, application::cunks::chunk::TEXTURE_SIZE_DELTA};

use super::bounds::Bounds;

pub mod circle_bruhs;
pub mod flatten_brush;

const WORK_GROUP_FOR_BRUSH: IVec3 = IVec3 {
    x: 8,
    y: 8,
    z: 8,
};

fn dispatch_compute_for(total_size: IVec3) {
    let dispatch = total_size.as_vec3() / WORK_GROUP_FOR_BRUSH.as_vec3();
    let res = dispatch.ceil();
    GL!(gl::DispatchCompute((res.x) as u32, (res.y) as u32, (res.z) as u32));
}

pub fn chunk_to_texture_position(chunk_pos: Vec3, tex_dim: IVec3) -> Vec3 {
    let tex_dim = tex_dim.as_vec3();
    let chunk_dim = tex_dim - TEXTURE_SIZE_DELTA.as_vec3();

    ((chunk_pos - Vec3::ONE * 0.5) * (chunk_dim)) / (tex_dim - Vec3::ONE) + Vec3::ONE * 0.5
}

fn chunk_size_to_texture_size(chunk_size: Vec3, tex_dim: IVec3) -> Vec3 {
    let tex_dim = tex_dim.as_vec3();
    let chunk_dim = tex_dim - TEXTURE_SIZE_DELTA.as_vec3();

    chunk_size * (chunk_dim) / (tex_dim - Vec3::ONE)
}

pub struct BrushApplicationParameters {
    transform: Transform,
    frame_rate_coefficient: f32,
}

impl BrushApplicationParameters {
    pub fn new(position: Vec3, frame_rate_coefficient: f32) -> Self {
        Self { transform: Transform::from_position(position), frame_rate_coefficient }
    }
}

pub struct Brush {
    settings: Arc<Mutex<dyn BrushSettings>>,
    parameters: BrushApplicationParameters,
}

impl Brush {

    pub fn new(settings: Arc<Mutex<dyn BrushSettings>>, 
        parameters: BrushApplicationParameters
    ) -> Brush {
        Brush { settings, parameters }
    }

    pub fn apply(&mut self, source: &mut Texture) {
        self.settings.lock().unwrap().apply(source, &self.parameters);
    }

    pub fn chunk_space_cords(&self, chunk_size: IVec3) -> Bounds<IVec3> {
        let bounds = self.settings.lock().unwrap().bounds(&self.parameters);
        chunk_space_cords(bounds, chunk_size)
    }

    pub fn bounds(&self) -> Bounds<Vec3> {
        self.settings.lock().unwrap().bounds(&self.parameters)
    }

    pub fn transformed(&self, offset: Vec3, scale: Vec3) -> Brush {

        let mut transform = self.parameters.transform.clone();
        transform.set_scale(transform.scale() * scale);
        transform.set_position(transform.position() + offset);

        Brush { settings: self.settings.clone(), parameters: BrushApplicationParameters {
            transform,
            frame_rate_coefficient: self.parameters.frame_rate_coefficient
        } }
    }
}

fn texture_space_cords(bounds: Bounds<Vec3>, tex_dim: IVec3) -> Bounds<IVec3> {
    let texture_min = chunk_to_texture_position(bounds.min(), tex_dim) * tex_dim.as_vec3();
        let texture_max = chunk_to_texture_position(bounds.max(), tex_dim) * tex_dim.as_vec3();

        // let chunk_min = bounds.min() * tex_dim.as_vec3();
        // let chunk_max = bounds.max() * tex_dim.as_vec3();


        let min = IVec3::max(texture_min.floor_to_ivec(), IVec3::ZERO);
        let max = IVec3::min(texture_max.ceil_to_ivec(), tex_dim);

        // let min = IVec3::max(chunk_min.floor_to_ivec(), IVec3::ZERO);
        // let max = IVec3::min(chunk_max.ceil_to_ivec(), tex_dim);

        Bounds::min_max(min, max)
}

fn chunk_space_cords(bounds: Bounds<Vec3>, chunk_size: IVec3) -> Bounds<IVec3> {
    let min = (bounds.min() * chunk_size.as_vec3()).floor_to_ivec() - IVec3::ONE * 3;
    let max = min + 
        (bounds.size() * chunk_size.as_vec3()).ceil_to_ivec() + IVec3::ONE * 3;

    if (min.x < 0 && max.x < 0) || (min.x > chunk_size.x && max.x > chunk_size.x) || 
        (min.y < 0 && max.y < 0) || (min.y > chunk_size.y && max.y > chunk_size.y) || 
        (min.z < 0 && max.z < 0) || (min.z > chunk_size.z && max.z > chunk_size.z) {

        Bounds::empty()
    } 
    else {
        Bounds::min_max(IVec3::max(min, IVec3::ZERO), IVec3::min(max, chunk_size))
    }
}

pub trait BrushSettings {
    fn bounds(&self, parameters: &BrushApplicationParameters) -> Bounds<Vec3>;
    fn apply(&mut self, source: &mut Texture, parameters: &BrushApplicationParameters);
    fn display_ui(&mut self, ui: &mut Ui);
    fn brush_name(&self) -> &'static str;
}

