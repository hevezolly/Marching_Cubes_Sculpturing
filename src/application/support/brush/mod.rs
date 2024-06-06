use core::{textures::texture::Texture, GL};

use egui_glfw_gl::{egui::plot::Text, gl};
use glam::{IVec3, Mat4, Quat, Vec3};

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

pub trait Brush {
    fn bounds(&self) -> Bounds<Vec3>;
    fn apply(&self, source: &mut Texture, dest: &mut Texture);
}

