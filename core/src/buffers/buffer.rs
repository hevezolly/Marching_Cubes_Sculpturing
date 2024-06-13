use std::{collections::{HashMap, HashSet}, ffi::c_void, fmt::Debug, marker::PhantomData, mem::size_of, ops::{Deref, Index}, ptr::null, result, sync::Mutex};

use egui_glfw_gl::gl::{self, types::{GLboolean, GLenum, GLint}, BufferData};

use crate::{buffers::vertex_attributes::{apply_attributes_to_bound_buffer}, GL};

use super::vertex_attributes::{bind_vertex_array_object, unbind_vertex_array_object, GlAttributeType, VertexAttrib, VertexDef};

struct BuffersContext {
    bounds_map: BoundMap,
    vertex_buffer_to_vao: HashMap<u32, u32>
}

struct BoundMap([Option<u32>;14]);

impl BoundMap {
    fn new() -> BoundMap {
        BoundMap([None;14])
    }

    fn id_of_target(target: GLenum) -> usize {
        match target {
            gl::ARRAY_BUFFER => 0,	
            gl::ATOMIC_COUNTER_BUFFER => 1,	
            gl::COPY_READ_BUFFER => 2,	
            gl::COPY_WRITE_BUFFER => 3,	
            gl::DISPATCH_INDIRECT_BUFFER => 4,
            gl::DRAW_INDIRECT_BUFFER => 5,	
            gl::ELEMENT_ARRAY_BUFFER => 6,	
            gl::PIXEL_PACK_BUFFER => 7,	
            gl::PIXEL_UNPACK_BUFFER => 8,	
            gl::QUERY_BUFFER => 9,	
            gl::SHADER_STORAGE_BUFFER => 10,
            gl::TEXTURE_BUFFER => 11,	
            gl::TRANSFORM_FEEDBACK_BUFFER => 12,	
            gl::UNIFORM_BUFFER => 13,
            _ => panic!("Unsupported buffer target")
        }
    }

    fn target_by_index(id: usize) -> GLenum {
        match id {
            0 => gl::ARRAY_BUFFER,	
            1 => gl::ATOMIC_COUNTER_BUFFER,	
            2 => gl::COPY_READ_BUFFER,	
            3 => gl::COPY_WRITE_BUFFER,	
            4 => gl::DISPATCH_INDIRECT_BUFFER,
            5 => gl::DRAW_INDIRECT_BUFFER,	
            6 => gl::ELEMENT_ARRAY_BUFFER,	
            7 => gl::PIXEL_PACK_BUFFER,	
            8 => gl::PIXEL_UNPACK_BUFFER,	
            9 => gl::QUERY_BUFFER,	
            10 => gl::SHADER_STORAGE_BUFFER,
            11 => gl::TEXTURE_BUFFER,	
            12 => gl::TRANSFORM_FEEDBACK_BUFFER,	
            13 => gl::UNIFORM_BUFFER,
            _ => panic!("Unsupported buffer target")
        }
    }

    fn get_bound_id(&self, target: GLenum) -> Option<u32> {
        self.0[BoundMap::id_of_target(target)]
    }

    fn insert(&mut self, id: u32, target: GLenum) -> Option<u32> {
        let key = BoundMap::id_of_target(target);
        let prev = self.0[key];
        self.0[key] = Some(id);
        prev
    }

    fn remove(&mut self, target: GLenum) -> Option<u32> {
        let key = BoundMap::id_of_target(target);
        let prev = self.0[key];
        self.0[key] = None;
        prev
    }

    fn target_by_id(&self, id: u32) -> Option<GLenum> {
        self.0.iter().enumerate()
            .find(|(_, s)| s.is_some_and(|v| v == id))
            .unzip().0.map(BoundMap::target_by_index)
    }
}



impl BuffersContext {
    fn try_bind(&mut self, id: u32, target: GLenum) {
        match self.bound_target_by_id(id) {
            Some(v) if v == target => (),
            _ => {
                GL!(gl::BindBuffer(target, id));
                self.bounds_map.insert(id, target);
            },
        };
    }

    fn try_bind_vertex<V: VertexDef + 'static>(&mut self, id: u32) {
        self.try_bind(id, gl::ARRAY_BUFFER);
        let bound_vao = bind_vertex_array_object::<V>();
        self.vertex_buffer_to_vao.insert(id, bound_vao);
    }

    fn try_unbind(&mut self, id: u32) {
        if let Some(bound_target) = self.bound_target_by_id(id) {
            self.bounds_map.remove(bound_target);
            GL!(gl::BindBuffer(bound_target, 0));
        }
        
        if let Some(_) = self.vertex_buffer_to_vao.remove(&id) {
            unbind_vertex_array_object()
        }
    }

    fn bound_target_by_id(&self, id: u32) -> Option<GLenum> {
        self.bounds_map.target_by_id(id)
    }
}

fn get_static_context<'a>() -> &'a mut BuffersContext {
    static mut MAP: Mutex<Option<BuffersContext>> = Mutex::new(None);

    unsafe { MAP.get_mut().unwrap() }
            .get_or_insert_with(|| BuffersContext { bounds_map: BoundMap::new(), vertex_buffer_to_vao: HashMap::new() }) 
}

#[derive(Debug)]
pub struct Buffer {
    id: u32
}

pub trait BufferDataInterface<Data: Sized> {
    fn rewrite_empty(&mut self, elements_count: usize, hint: Usage) {
        let size = (elements_count * size_of::<Data>()) as isize;
        self.rewrite_data_by_pointer(null(), size, hint)
    }

    fn read_data(&self, offset: usize, destination: &mut [Data]);

    fn read_data_from_start(&self, destination: &mut [Data]) {
        self.read_data(0, destination)
    }

    fn size_in_bytes(&self) -> usize;
    
    fn len(&self) -> usize {
        self.size_in_bytes() / size_of::<Data>()
    }

    fn get_data_slice(&self, offset: usize, count: usize) -> Vec<Data> {
        assert!(offset + count <= self.len());
        let mut out = Vec::<Data>::with_capacity(count);
        unsafe {
            out.set_len(count);
        }
        self.read_data(offset, &mut out);
        out
    }

    fn get_data_slice_from_start(&self, count: usize) -> Vec<Data> { self.get_data_slice(0, count) }
    fn get_data_slice_to_end(&self, offset: usize) -> Vec<Data> {self.get_data_slice(offset, self.len() - offset)}
    fn get_all_data(&self) -> Vec<Data> {self.get_data_slice(0, self.len())}

    fn rewrite_data_by_pointer(&mut self, data: *const c_void, size: isize, hint: Usage);
    fn update_data(&mut self, offset: usize, data: &[Data]);

    fn rewrite_data(&mut self, data: &[Data], hint: Usage) {
        let size = (data.len() * size_of::<Data>()) as isize;
        self.rewrite_data_by_pointer(data.as_ptr() as *const c_void, size, hint)
    }
}

impl Buffer {
    pub fn create_uninitialized() -> Buffer {
        let mut id: u32 = 0;
        GL!(gl::GenBuffers(1, &mut id));
        Buffer { id }
    }

    pub fn empty<Data: Sized>(elements_count: usize, hint: Usage) -> Buffer {
        let mut buffer = Buffer::create_uninitialized();
        buffer.rewrite_empty::<Data>(elements_count, hint);
        buffer
    }

    pub fn from_data<Data: Sized>(data: &[Data], hint: Usage) -> Buffer {
        let mut buffer = Buffer::create_uninitialized();
        buffer.rewrite_data(data, hint);
        buffer
    }

    pub fn unbind(&self) {
        get_static_context().try_unbind(self.id)
    }

    pub fn bind(&self, target: GLenum) {
        get_static_context().try_bind(self.id, target);
    }

    pub fn bind_as_index(&self) {
        get_static_context().try_bind(self.id(), gl::ELEMENT_ARRAY_BUFFER);
    }

    pub fn bind_as_vertex<V: VertexDef + 'static>(&self) {
        get_static_context().try_bind_vertex::<V>(self.id);
    }

    pub fn rewrite_empty<Data: Sized>(&mut self, elements_count: usize, hint: Usage) {
        BufferDataInterface::<Data>::rewrite_empty(self, elements_count, hint)
    }

    pub fn get_all_data<Data: Sized>(&self) -> Vec<Data> {
        BufferDataInterface::<Data>::get_all_data(self)
    }
    
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn delete(&mut self) {
        self.unbind();
        GL!(gl::DeleteBuffers(1, &self.id()));
    } 
}

impl AsRef<Buffer> for Buffer {
    fn as_ref(&self) -> &Buffer {
        self
    }
}

impl<Data: Sized> BufferDataInterface<Data> for Buffer {
    fn read_data(&self, offset: usize, destination: &mut [Data]) {

        let byte_offset = (offset * size_of::<Data>()) as isize;
        let byte_size = (destination.len() * size_of::<Data>()) as isize;
    
        GL!(gl::GetNamedBufferSubData(self.id, byte_offset, byte_size, destination.as_mut_ptr() as *mut c_void));
    }
    
    fn rewrite_data_by_pointer(&mut self, data: *const c_void, size: isize, hint: Usage) {
        GL!(gl::NamedBufferData(
            self.id, 
            size,
            data, 
            hint.gl_usage()));
    }
    
    fn size_in_bytes(&self) -> usize {
        let mut size_bytes = 0;
        GL!(gl::GetNamedBufferParameteriv(self.id, gl::BUFFER_SIZE, &mut size_bytes));
        size_bytes as usize
    }
    
    fn update_data(&mut self, offset: usize, data: &[Data]) {
        GL!(gl::NamedBufferSubData(self.id, offset as isize, (data.len() * size_of::<Data>()) as isize, data.as_ptr() as *const c_void));
    }
}

// impl Drop for Buffer {
//     fn drop(&mut self) {
//         self.unbind();
//         GL!(gl::DeleteBuffers(1, &self.id()));
//     }
// }

pub struct VertexBuffer<V: VertexDef + Sized + 'static> {
    buffer: Buffer,
    phantom: PhantomData<V>
}

impl<V: VertexDef + Sized + 'static> VertexBuffer<V> {
    pub fn create_uninitialized() -> VertexBuffer<V> {
        VertexBuffer { buffer: Buffer::create_uninitialized(), phantom: PhantomData }
    }

    pub fn empty(elements_count: usize, hint: Usage) -> VertexBuffer<V> {
        let buffer = Buffer::empty::<V>(elements_count, hint);
        VertexBuffer { buffer, phantom: PhantomData }
    }

    pub fn from_data(data: &[V], hint: Usage) -> VertexBuffer<V> {
        let buffer = Buffer::from_data(data, hint);
        VertexBuffer { buffer: buffer, phantom: PhantomData }
    }

    pub fn unbind(&mut self) {
        self.buffer.unbind()
    }

    pub fn bind(&mut self) {
        self.buffer.bind_as_vertex::<V>()
    }

    pub fn rewrite_empty(&mut self, elements_count: usize, hint: Usage) {
        self.buffer.rewrite_empty::<V>(elements_count, hint)
    }

    pub fn read_data(&self, offset: usize, destination: &mut [V]) {
        self.buffer.read_data(offset, destination)
    }

    pub fn read_data_from_start(&self, destination: &mut [V]) {
        self.buffer.read_data_from_start(destination)
    }

    pub fn rewrite_data(&mut self, data: &[V], hint: Usage) {
        self.buffer.rewrite_data(data, hint)
    }
    
    pub fn id(&self) -> u32 {
        self.buffer.id()
    }
    
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl<V: VertexDef + Sized + 'static> AsRef<Buffer> for VertexBuffer<V> {
    fn as_ref(&self) -> &Buffer {
        self.buffer()
    }
}

impl<V: VertexDef + Sized + 'static> BufferDataInterface<V> for VertexBuffer<V> {
    fn rewrite_empty(&mut self, elements_count: usize, hint: Usage) {
        self.buffer.rewrite_empty::<V>(elements_count, hint)
    }

    fn read_data(&self, offset: usize, destination: &mut [V]) {
        self.buffer.read_data(offset, destination)
    }

    fn rewrite_data(&mut self, data: &[V], hint: Usage) {
        self.buffer.rewrite_data(data, hint)
    }
    
    fn rewrite_data_by_pointer(&mut self, data: *const c_void, size: isize, hint: Usage) {
        BufferDataInterface::<V>::rewrite_data_by_pointer(&mut self.buffer, data, size, hint)
    }
    
    fn size_in_bytes(&self) -> usize {
        BufferDataInterface::<V>::size_in_bytes(self.buffer())
    }
    
    fn update_data(&mut self, offset: usize, data: &[V]) {
        BufferDataInterface::<V>::update_data(&mut self.buffer, offset, data);
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