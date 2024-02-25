use egui_glfw_gl::gl;

pub mod texture;
pub mod image_provider;

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureUnit(pub usize);

impl TextureUnit {
    fn gl_slot(&self) -> u32 {
        gl::TEXTURE0 + self.0 as u32
    }
}

impl From<usize> for TextureUnit {
    fn from(value: usize) -> Self {
        TextureUnit(value)
    }
}