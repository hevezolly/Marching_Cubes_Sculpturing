use core::textures::texture::Texture;

use egui_glfw_gl::egui::plot::Text;
use glam::{Mat4, Quat, Vec3};

use super::bounds::Bounds;

pub mod circle_bruhs;

pub trait Brush {

    fn apply(&self, source: &mut Texture, dest: &mut Texture);
}

