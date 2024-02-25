
use std::{marker::PhantomData, ptr::null};

use egui_glfw_gl::{egui::Image, gl};

use crate::{GL, OpenglAlias};

use super::{image_provider::{ImageFormat, ImageProvider2D}, TextureUnit};



pub struct Texture {
    id: u32,
    format: ImageFormat,
    texture_target: u32,
    slot: Option<TextureUnit> 
}

pub trait TextureFilterMode{
    fn param_value(&self) -> gl::types::GLenum;
    fn require_mips(&self) -> bool;
}


#[derive(Debug, Clone, Copy)]
pub enum FilterMode {
    Nearest,
    Linear
}

#[derive(Debug, Clone, Copy)]
pub enum WrapMode {
    Repeat,
    MirroredRepeat,
    MirrorClampToEdge,
    ClampToEdge,
    ClampToBorder([f32; 4])
}

impl WrapMode {
    pub fn param_value(&self) -> gl::types::GLenum {
        match &self {
            WrapMode::Repeat => gl::REPEAT,
            WrapMode::MirroredRepeat => gl::MIRRORED_REPEAT,
            WrapMode::MirrorClampToEdge => gl::MIRROR_CLAMP_TO_EDGE,
            WrapMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrapMode::ClampToBorder(_) => gl::CLAMP_TO_BORDER,
        }
    }
}

impl TextureFilterMode for FilterMode {
    fn param_value(&self) -> gl::types::GLenum {
        match self {
            FilterMode::Nearest => gl::NEAREST,
            FilterMode::Linear => gl::LINEAR,
        }
    }

    fn require_mips(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MipMapFilterMode {
    pub tex_filter: FilterMode,
    pub mip_filter: FilterMode
}

impl TextureFilterMode for MipMapFilterMode {
    fn param_value(&self) -> gl::types::GLenum {
        match (&self.tex_filter, &self.mip_filter) {
            (FilterMode::Nearest, FilterMode::Nearest) => gl::NEAREST_MIPMAP_NEAREST,
            (FilterMode::Nearest, FilterMode::Linear) => gl::NEAREST_MIPMAP_LINEAR,
            (FilterMode::Linear, FilterMode::Nearest) => gl::LINEAR_MIPMAP_NEAREST,
            (FilterMode::Linear, FilterMode::Linear) => gl::LINEAR_MIPMAP_LINEAR,
        }
    }

    fn require_mips(&self) -> bool {
        true
    }
}

pub trait TextureType {}

pub struct DimTexture<const DIM: usize>;
pub type TexType1d = DimTexture<1>;
pub type TexType2d = DimTexture<2>;
pub type TexType3d = DimTexture<3>;
impl<const DIM: usize> TextureType for DimTexture<DIM> {}

pub struct TextureBuilder<TextureType> {
    tex: Texture,
    generate_mips: bool,
    border_color: Option<[f32; 4]>,
    constraint: PhantomData<TextureType>
}

fn new_texture(image_type: u32) -> Texture {
    let mut id = 0;
    GL!(gl::GenTextures(1, &mut id));
    let tex = Texture { id, texture_target: image_type, slot: None, format: Default::default() };
    GL!(gl::BindTexture(tex.texture_target, tex.id));
    tex
}

#[derive(Debug, Clone, Copy)]
pub enum TextureAccess {
    Read,
    Write,
    ReadWrite
}

impl TextureAccess {
    pub fn gl_access(&self) -> gl::types::GLenum {
        match self {
            TextureAccess::Read => gl::READ_ONLY,
            TextureAccess::Write => gl::WRITE_ONLY,
            TextureAccess::ReadWrite => gl::READ_WRITE,
        }
    }
}

impl Texture {

    pub fn bind<T: Into<TextureUnit>>(&mut self, unit: T) {
        let unit = unit.into();

        self.unbind();

        self.slot = Some(unit);
        GL!(gl::ActiveTexture(unit.gl_slot()));
        GL!(gl::BindTexture(self.texture_target, self.id));
    }

    pub fn bind_image_lod<T: Into<TextureUnit>>(&mut self, lod: i32, unit: T, access: TextureAccess) {
        let unit = unit.into();
        self.unbind();

        self.slot = Some(unit);
        GL!(gl::ActiveTexture(unit.gl_slot()));
        GL!(gl::BindImageTexture(
            unit.0 as u32, 
            self.id, 
            lod, 
            gl::FALSE, 
            0, 
            access.gl_access(), 
            self.format.internal_format))
    }

    pub fn bind_image<T: Into<TextureUnit>>(&mut self, unit: T, access: TextureAccess) {
        self.bind_image_lod(self.format.lod, unit, access)
    }

    pub fn unbind(&mut self) {
        if let Some(slot) = self.slot {
            GL!(gl::ActiveTexture(slot.gl_slot()));
            GL!(gl::BindTexture(self.texture_target, 0));
            self.slot = None;
        }
    }

    pub fn slot(&self) -> Option<TextureUnit> {
        self.slot
    }

    pub fn new_1d() -> TextureBuilder<TexType1d> {
        TextureBuilder {
            tex: new_texture(gl::TEXTURE_1D), 
            border_color: None, 
            constraint: PhantomData,
            generate_mips: false 
        }
    }

    pub fn new_2d() -> TextureBuilder<TexType2d> {
        TextureBuilder {
            tex: new_texture(gl::TEXTURE_2D), 
            border_color: None, 
            constraint: PhantomData,
            generate_mips: false
        }

    }

    pub fn new_3d() -> TextureBuilder<TexType3d> {
        TextureBuilder {
            tex: new_texture(gl::TEXTURE_3D), 
            border_color: None, 
            constraint: PhantomData,
            generate_mips: false
        }
    }
}

impl<T: TextureType> TextureBuilder<T> {
    pub fn generate_mipmap(self) -> TextureBuilder<T> {
        TextureBuilder { 
            tex: self.tex, 
            border_color: self.border_color, 
            constraint: PhantomData,
            generate_mips: true
        }
    }
}

impl<T: TextureType> TextureBuilder<T> {
    pub fn minification_filter<Filter: TextureFilterMode>(mut self, filter: Filter) -> TextureBuilder<T> {
        GL!(gl::TexParameteri(self.tex.texture_target, gl::TEXTURE_MIN_FILTER, filter.param_value() as i32));
        self.generate_mips = self.generate_mips || filter.require_mips();
        self
    }
}

fn wrap_x<T: TextureType>(mut b: TextureBuilder<T>, wrap_mode: WrapMode) -> TextureBuilder<T> {
    b.set_wrap_mode(wrap_mode, gl::TEXTURE_WRAP_S);
    b
}

fn wrap_y<T: TextureType>(mut b: TextureBuilder<T>, wrap_mode: WrapMode) -> TextureBuilder<T> {
    b.set_wrap_mode(wrap_mode, gl::TEXTURE_WRAP_T);
    b
}

fn wrap_z<T: TextureType>(mut b: TextureBuilder<T>, wrap_mode: WrapMode) -> TextureBuilder<T> {
    b.set_wrap_mode(wrap_mode, gl::TEXTURE_WRAP_R);
    b
}

impl TextureBuilder<TexType1d> {
    pub fn wrap_mode_x(self, wrap_mode: WrapMode) -> TextureBuilder<TexType1d> {
        wrap_x(self, wrap_mode)
    }
}

impl TextureBuilder<TexType2d> {
    pub fn wrap_mode_x(self, wrap_mode: WrapMode) -> TextureBuilder<TexType2d> {
        wrap_x(self, wrap_mode)
    }

    pub fn wrap_mode_y(self, wrap_mode: WrapMode) -> TextureBuilder<TexType2d> {
        wrap_y(self, wrap_mode)
    }
}

impl TextureBuilder<TexType3d> {
    pub fn wrap_mode_x(self, wrap_mode: WrapMode) -> TextureBuilder<TexType3d> {
        wrap_x(self, wrap_mode)
    }

    pub fn wrap_mode_y(self, wrap_mode: WrapMode) -> TextureBuilder<TexType3d> {
        wrap_y(self, wrap_mode)
    }

    pub fn wrap_mode_z(self, wrap_mode: WrapMode) -> TextureBuilder<TexType3d> {
        wrap_z(self, wrap_mode)
    }
}


impl<T: TextureType> TextureBuilder<T> {
    pub fn magnification_filter(self, filter: FilterMode) -> TextureBuilder<T> {
        GL!(gl::TexParameteri(self.tex.texture_target, gl::TEXTURE_MAG_FILTER, filter.param_value() as i32));
        self
    }
    fn set_wrap_mode(&mut self, wrap_mode: WrapMode, param_type: gl::types::GLenum) {
        GL!(gl::TexParameteri(self.tex.texture_target, param_type, wrap_mode.param_value() as i32));

        if let WrapMode::ClampToBorder(border) = wrap_mode {
            self.border_color = Some(border.convert());
        };
    }
}

fn build<T: TextureType> (builder: TextureBuilder<T>) -> Texture {
    if let Some(border) = &builder.border_color {
        GL!(gl::TexParameterfv(builder.tex.texture_target, gl::TEXTURE_BORDER_COLOR, border.as_ptr() as *const f32));
    };
    if builder.generate_mips {
        GL!(gl::GenerateTextureMipmap(builder.tex.id));
    };
    GL!(gl::BindTexture(builder.tex.texture_target, 0));
    builder.tex
}

impl TextureBuilder<TexType2d>  {
    pub fn with_data<T: ImageProvider2D>(mut self, provider: &T) -> Texture {
        let description = provider.description();
        self.tex.format = description;
        GL!(gl::TexImage2D(
            self.tex.texture_target, 
            description.lod,
            description.internal_format as i32,
            provider.width(),
            provider.height(),
            0,
            description.format,
            description.data_type,
            provider.data()
        ));
        build(self)
    }

    pub fn empty(mut self, width: i32, height: i32, description: ImageFormat) -> Texture {
        self.tex.format = description;
        GL!(gl::TexImage2D(
            self.tex.texture_target, 
            description.lod,
            description.internal_format as i32,
            width,
            height,
            0,
            description.format,
            description.data_type,
            null()
        ));
        build(self)
    }
}

impl TextureBuilder<TexType3d> {
    pub fn empty(mut self, width: i32, height: i32, depth: i32, description: ImageFormat) -> Texture {
        self.tex.format = description;
        GL!(gl::TexImage3D(
            self.tex.texture_target, 
            description.lod,
            description.internal_format as i32,
            width,
            height,
            depth,
            0,
            description.format,
            description.data_type,
            null()
        ));
        build(self)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.unbind();
        GL!(gl::DeleteTextures(1, &self.id))
    }
}
