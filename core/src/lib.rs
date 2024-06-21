use glam::{Vec3, Vec4, Quat, Vec2, IVec3, IVec4, IVec2};
use textures::{texture::Texture, TextureUnit};

pub mod shaders;
pub mod buffers;
pub mod textures;
pub mod context;

#[macro_export]
macro_rules! GL {
    ($exp:expr) => {
        {
            let res = unsafe {$exp};
            let mut opengl_error: u32 = unsafe {gl::GetError()};
            while (opengl_error != gl::NO_ERROR) {
                dbg!(opengl_error);
                // std::process::exit(1);
                panic!("opengl error occured");
                opengl_error = unsafe {gl::GetError()};
            }
            res
        }
    };
}

pub trait OpenglAlias<T> {
    const IS_ALIAS: bool = true;
    fn convert(self) -> T;
}

impl<T> OpenglAlias<T> for T {
    fn convert(self) -> T {
        self
    }
}

macro_rules! alias_ref {
    ($type: ty) => {
        impl OpenglAlias<$type> for &$type {
            fn convert(self) -> $type {
                self.clone()
            }
        }
    };
}

macro_rules! alias_copy_ref {
    ($type: ty, $val: ty) => {
        impl OpenglAlias<$val> for &$type {
            fn convert(self) -> $val {
                OpenglAlias::<$val>::convert(*self)
            }
        }
    };
}

alias_ref!([f32;2]);
alias_ref!([f32;3]);
alias_ref!([f32;4]);

alias_ref!([i32;2]);
alias_ref!([i32;3]);
alias_ref!([i32;4]);

alias_ref!([u32;2]);
alias_ref!([u32;3]);
alias_ref!([u32;4]);

alias_ref!([bool;2]);
alias_ref!([bool;3]);
alias_ref!([bool;4]);

impl OpenglAlias<[f32;3]> for Vec3 {
    fn convert(self) -> [f32;3] {
        [self.x, self.y, self.z]
    }
}

impl OpenglAlias<[f32;4]> for Vec4 {
    fn convert(self) -> [f32;4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl OpenglAlias<[f32;4]> for Quat {
    fn convert(self) -> [f32;4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl OpenglAlias<[f32;2]> for Vec2 {
    fn convert(self) -> [f32;2] {
        [self.x, self.y]
    }
}

impl OpenglAlias<[i32;3]> for IVec3 {
    fn convert(self) -> [i32;3] {
        [self.x, self.y, self.z]
    }
}

impl OpenglAlias<[i32;4]> for IVec4 {
    fn convert(self) -> [i32;4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl OpenglAlias<[i32;2]> for IVec2 {
    fn convert(self) -> [i32;2] {
        [self.x, self.y]
    }
}

alias_copy_ref!(Vec3, [f32; 3]);
alias_copy_ref!(Vec4, [f32; 4]);
alias_copy_ref!(Quat, [f32; 4]);
alias_copy_ref!(Vec2, [f32; 2]);
alias_copy_ref!(IVec3, [i32; 3]);
alias_copy_ref!(IVec4, [i32; 4]);
alias_copy_ref!(IVec2, [i32; 2]);

impl OpenglAlias<TextureUnit> for &Texture {
    fn convert(self) -> TextureUnit {
        self.slot().expect("attempt to use texture, not attached to any texture units")
    }
}