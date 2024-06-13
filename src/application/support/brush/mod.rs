use core::{textures::texture::Texture, GL};

use egui_glfw_gl::{egui::plot::Text, gl};
use glam::{IVec3, Mat4, Quat, Vec3};

use crate::algorithms::cordinates::RoundableToIVec3;

use super::bounds::Bounds;

pub mod circle_bruhs;

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
    let chunk_dim = tex_dim - Vec3::ONE * 3.;

    ((chunk_pos - Vec3::ONE * 0.5) * (chunk_dim)) / (tex_dim - Vec3::ONE) + Vec3::ONE * 0.5
}

fn chunk_size_to_texture_size(chunk_size: Vec3, tex_dim: IVec3) -> Vec3 {
    let tex_dim = tex_dim.as_vec3();
    let chunk_dim = tex_dim - Vec3::ONE * 3.;

    chunk_size * (chunk_dim) / (tex_dim - Vec3::ONE)
}

pub trait Brush {
    fn bounds(&self) -> Bounds<Vec3>;
    fn apply(&self, source: &mut Texture);

    fn transformed(&self, offset: Vec3, scale: f32 ) -> Self;

    fn texture_space_cords(&self, tex_dim: IVec3) -> Bounds<IVec3> {

        let bounds = self.bounds();

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

    fn chunk_space_cords(&self, chunk_size: IVec3) -> Bounds<IVec3> {
        let bounds = self.bounds();
        let min = IVec3::max((bounds.min() * chunk_size.as_vec3()).floor_to_ivec() - IVec3::ONE * 2, IVec3::ZERO);
        let max = IVec3::min(min + 
            (bounds.size() * chunk_size.as_vec3()).ceil_to_ivec() + IVec3::ONE * 2, chunk_size);
            
        Bounds::min_max(min, max)
    }
}

