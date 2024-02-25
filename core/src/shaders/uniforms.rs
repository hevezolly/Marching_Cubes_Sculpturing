use std::collections::HashMap;

use egui_glfw_gl::gl::{types::GLint, self};
use glam::{IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{textures::TextureUnit, OpenglAlias, GL};

trait ToU32 {
    fn to_u32(&self) -> u32;
}

impl ToU32 for bool {
    fn to_u32(&self) -> u32 {
        if *self {1} else {0}
    }
}

pub trait UniformCompatableType: Default {
    const IS_COMPATABLE: bool = true;
    
    fn apply_by_location(&self, location: GLint);

    fn apply_by_name(&self, name: &str, names_mapping: &HashMap<String, GLint>) {
        if let Some(location) = names_mapping.get(name) {
            self.apply_by_location(*location);
        }
    }
}


impl UniformCompatableType for f32 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform1f(location, *self))
    }
}

impl UniformCompatableType for [f32;2] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform2f(location, self[0], self[1]))
    }
}

impl UniformCompatableType for [f32;3] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform3f(location, self[0], self[1], self[2]))
    }
}

impl UniformCompatableType for [f32;4] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform4f(location, self[0], self[1], self[2], self[3]))
    }
}

impl UniformCompatableType for i32 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform1i(location, *self))
    }
}

impl UniformCompatableType for [i32;2] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform2i(location, self[0], self[1]))
    }
}

impl UniformCompatableType for [i32;3] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform3i(location, self[0], self[1], self[2]))
    }
}

impl UniformCompatableType for [i32;4] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform4i(location, self[0], self[1], self[2], self[3]))
    }
}

impl UniformCompatableType for u32 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform1ui(location, *self))
    }
}

impl UniformCompatableType for [u32;2] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform2ui(location, self[0], self[1]))
    }
}

impl UniformCompatableType for [u32;3] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform3ui(location, self[0], self[1], self[2]))
    }
}

impl UniformCompatableType for [u32;4] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform4ui(location, self[0], self[1], self[2], self[3]))
    }
}

impl UniformCompatableType for bool {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform1ui(location, self.to_u32()))
    }
}

impl UniformCompatableType for [bool;2] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform2ui(location, self[0].to_u32(), self[1].to_u32()))
    }
}

impl UniformCompatableType for [bool;3] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform3ui(location, self[0].to_u32(), self[1].to_u32(), self[2].to_u32()))
    }
}

impl UniformCompatableType for [bool;4] {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform4ui(location, self[0].to_u32(), self[1].to_u32(), self[2].to_u32(), self[3].to_u32()))
    }
}

impl UniformCompatableType for Mat4 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::UniformMatrix4fv(location, 1, gl::FALSE, &self.to_cols_array()[0]));
    }
}

impl UniformCompatableType for Mat3 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::UniformMatrix3fv(location, 1, gl::FALSE, &self.to_cols_array()[0]));
    }
}

impl UniformCompatableType for Mat2 {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::UniformMatrix2fv(location, 1, gl::FALSE, &self.to_cols_array()[0]));
    }
}

impl UniformCompatableType for TextureUnit {
    fn apply_by_location(&self, location: GLint) {
        GL!(gl::Uniform1i(location, self.0 as i32))
    }
}

macro_rules! uniform_aleas {
    ($aleas:ty, $type:ty) => {
        impl UniformCompatableType for $aleas {
            fn apply_by_location(&self, location: GLint) {
                OpenglAlias::<$type>::convert(self).apply_by_location(location)
            }
        }
    };
}

uniform_aleas!(Vec2, [f32; 2]);
uniform_aleas!(Vec3, [f32; 3]);
uniform_aleas!(Vec4, [f32; 4]);
uniform_aleas!(IVec2, [i32; 2]);
uniform_aleas!(IVec3, [i32; 3]);
uniform_aleas!(IVec4, [i32; 4]);
uniform_aleas!(Quat, [f32; 4]);

pub trait Uniforms {
    fn apply_uniforms(&self, names_mapping: &HashMap<String, GLint>);
    fn defenition() -> Vec<String>;
}