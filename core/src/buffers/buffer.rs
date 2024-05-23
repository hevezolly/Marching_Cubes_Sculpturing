use std::{ffi::c_void, marker::PhantomData, mem::size_of, ops::Index, ptr::null};

use egui::Context;
use egui_glfw_gl::gl::{self, types::{GLenum, GLboolean, GLint}};

use crate::{buffers::vertex_attributes::{apply_attributes_to_bound_buffer, VertexArrayObject}, GL};

use super::vertex_attributes::{VertexAttrib, GlAttributeType, VertexDef};


pub trait BoundBufferContext<'a, B: Buffer> {
    fn new(buffer: &'a B) -> Self;
    fn unbind(self);
}

pub trait Buffer: Sized {
    type Context<'a>: BoundBufferContext<'a, Self> where Self: 'a;

    fn id(&self) -> u32;
    fn buffer_type(&self) -> GLenum;

    fn unbind(&self) {
        GL!(gl::BindBuffer(self.buffer_type(), 0));
    } 

    fn bind<'a>(&'a self) -> Self::Context<'a> {
        GL!(gl::BindBuffer(self.buffer_type(), self.id()));
        self.already_bound()
    }

    fn already_bound<'a>(&'a self) -> Self::Context<'a> {
        Self::Context::new(self)
    }

    fn create_from_existing(other_id: u32) -> Self;

    fn rebind<Other: Buffer>(self) -> Other {
        Other::create_from_existing(self.id())
    }
}

pub struct DefaultBoundBufferContext<'a, B: Buffer> {
    buffer: &'a B
}

impl<'a, B: Buffer> BoundBufferContext<'a, B> for DefaultBoundBufferContext<'a, B>  {
    fn new(buffer: &'a B) -> Self {
        DefaultBoundBufferContext { buffer }
    }

    fn unbind(self) {
        self.buffer.unbind()
    }
}

impl<'a, B: Buffer> DefaultBoundBufferContext<'a, B> {
    pub fn new_data<Data: Sized>(self, data: &[Data], hint: Usage) -> Self {
        new_data(self.buffer.buffer_type(), data, hint);
        self
    }

    pub fn read_to<Data: Sized>(self, destination: &mut [Data], offset: usize) -> Self {
        get_data(self.buffer.buffer_type(), offset, destination);
        self
    }

    pub fn read_from_start_to<Data: Sized>(self, destination: &mut [Data]) -> Self {
        self.read_to(destination, 0)
    }
    
    pub fn empty<Data: Sized>(self, elements_count: usize, hint: Usage) -> Self {
        empty::<Data>(self.buffer.buffer_type(), elements_count, hint);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UsageFrequency {
    // The data store contents will be modified once and used at most a few times.
    Stream,

    // The data store contents will be modified once and used many times.
    Static,

    // The data store contents will be modified repeatedly and used many times.
    Dynamic
}

#[derive(Debug, Clone, Copy)]
pub enum UsagePattern {
    // The data store contents are modified by the application, 
    // and used as the source for GL drawing and image specification commands.
    Draw,

    // The data store contents are modified by reading data from the GL, 
    // and used to return that data when queried by the application.
    Read,

    // The data store contents are modified by reading data from the GL, 
    // and used as the source for GL drawing and image specification commands.
    Copy,
}

#[derive(Debug, Clone, Copy)]
pub struct Usage(pub UsageFrequency, pub UsagePattern);

impl Usage {
    pub fn gl_usage(&self) -> u32 {
        match (self.0, self.1) {
            (UsageFrequency::Stream, UsagePattern::Draw) => gl::STREAM_DRAW,
            (UsageFrequency::Stream, UsagePattern::Read) => gl::STREAM_READ,
            (UsageFrequency::Stream, UsagePattern::Copy) => gl::STREAM_COPY,
            (UsageFrequency::Static, UsagePattern::Draw) => gl::STATIC_DRAW,
            (UsageFrequency::Static, UsagePattern::Read) => gl::STATIC_READ,
            (UsageFrequency::Static, UsagePattern::Copy) => gl::STATIC_COPY,
            (UsageFrequency::Dynamic, UsagePattern::Draw) => gl::DYNAMIC_DRAW,
            (UsageFrequency::Dynamic, UsagePattern::Read) => gl::DYNAMIC_READ,
            (UsageFrequency::Dynamic, UsagePattern::Copy) => gl::DYNAMIC_COPY,
        }
    }

    pub fn static_draw() -> Usage {
        Usage(UsageFrequency::Static, UsagePattern::Draw)
    }

    pub fn dynamic_read() -> Usage {
        Usage(UsageFrequency::Dynamic, UsagePattern::Read)
    }

    pub fn dynamic_copy() -> Usage {
        Usage(UsageFrequency::Dynamic, UsagePattern::Copy)
    }
}

pub struct VertexBuffer<V: VertexDef + Send + Sync + 'static> {
    id: u32,
    constraint: PhantomData<V>
}

pub struct VertexBufferBoundContext<'a, V: VertexDef + Send + Sync + 'static> {
    buffer: &'a VertexBuffer<V>
}



impl<'a, V: VertexDef + Send + Sync + 'static> BoundBufferContext<'a, VertexBuffer<V>> for VertexBufferBoundContext<'a, V> {
    
    fn new(buffer: &'a VertexBuffer<V>) -> Self {
        VertexBufferBoundContext { buffer }
    }

    fn unbind(self) {
        self.buffer.unbind()
    }
}

impl<'a, V: VertexDef + Send + Sync + 'static> VertexBufferBoundContext<'a, V> {
    pub fn new_data(self, data: &[V], hint: Usage) -> Self {
        new_data(self.buffer.buffer_type(), data, hint);
        self
    }

    pub fn empty(self, elements_count: usize, hint: Usage) -> Self {
        empty::<V>(self.buffer.buffer_type(), elements_count, hint);
        self
    }

    pub fn read_to(self, destination: &mut [V], offset: usize) -> Self {
        get_data(self.buffer.buffer_type(), offset, destination);
        self
    }

    pub fn read_from_start_to(self, destination: &mut [V]) -> Self {
        self.read_to(destination, 0)
    }
}

impl<V: VertexDef + Send + Sync + 'static> Buffer for VertexBuffer<V> {
    type Context<'a> = VertexBufferBoundContext<'a, V> where Self: 'a;

    fn bind<'a>(&'a self) -> Self::Context<'a> {
        GL!(gl::BindBuffer(self.buffer_type(), self.id()));
        VertexArrayObject::<V>::bind();
        Self::Context::new(self)
    }

    fn unbind(&self) {
        VertexArrayObject::<V>::unbind();
        GL!(gl::BindBuffer(self.buffer_type(), 0));
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn buffer_type(&self) -> GLenum {
        gl::ARRAY_BUFFER
    }

    fn create_from_existing(id: u32) -> Self {
        VertexBuffer { id, constraint: PhantomData }
    }
}

impl<V: VertexDef + Send + Sync + 'static> VertexBuffer<V> {

    pub fn new() -> VertexBuffer<V> {
        create_buffer()
    }
}

impl<V: VertexDef + Send + Sync + 'static> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        drop_buffer(self);
    }
}

pub fn create_buffer<B: Buffer>() -> B {
    let mut buffer_id: u32 = 0;
    GL!(gl::GenBuffers(1, &mut buffer_id));
    B::create_from_existing(buffer_id)
}

fn drop_buffer<B: Buffer>(buffer: &B) {
    buffer.unbind();
    GL!(gl::DeleteBuffers(1, &buffer.id()));
}

fn new_data<Data: Sized>(buffer_type: u32, data: &[Data], hint: Usage) {
    let size = (data.len() * size_of::<Data>()) as isize;
        GL!(gl::BufferData(
            buffer_type, 
            size,
            data.as_ptr() as *const c_void, 
            hint.gl_usage()));
}

fn get_data<Data: Sized>(buffer_type: u32, offset: usize, destination: &mut [Data]) {

    let byte_offset = (offset * size_of::<Data>()) as isize;
    let byte_size = (destination.len() * size_of::<Data>()) as isize;

    GL!(gl::GetBufferSubData(buffer_type, byte_offset, byte_size, destination.as_mut_ptr() as *mut c_void));
}

fn empty<Data: Sized>(buffer_type: u32, elements_count: usize, hint: Usage) {
    let size = (elements_count * size_of::<Data>()) as isize;
    GL!(gl::BufferData(
        buffer_type, 
        size,
        null(), 
        hint.gl_usage()));
}

pub type IndexBuffer = GenericBuffer<{gl::ELEMENT_ARRAY_BUFFER}>;
pub type ShaderStorageBuffer = GenericBuffer<{gl::SHADER_STORAGE_BUFFER}>;
pub type DrawIndirectBuffer = GenericBuffer<{gl::DRAW_INDIRECT_BUFFER}>;

pub struct GenericBuffer<const BUFFER_TYPE: u32> {
    id: u32
}

impl<const BUFFER_TYPE: u32> Buffer for GenericBuffer<BUFFER_TYPE> {
    type Context<'a> = DefaultBoundBufferContext<'a, Self> where Self: 'a;

    fn id(&self) -> u32 {
        self.id
    }

    fn buffer_type(&self) -> GLenum {
        BUFFER_TYPE
    }

    fn create_from_existing(id: u32) -> Self {
        GenericBuffer { id }
    }
}

impl<const BUFFER_TYPE: u32> GenericBuffer<BUFFER_TYPE> {
    pub fn new() -> Self {
        create_buffer()
    }    
}

