use std::{ffi::c_void, marker::PhantomData, mem::size_of, sync::{atomic::AtomicUsize, Arc, Mutex, RwLock}};

use anymap::{any::Any, AnyMap, Map};
use egui_glfw_gl::{gl::{self, types::{GLboolean, GLenum, GLint}}};
use glam::{Vec2, Vec3, Vec4, IVec2, IVec3, IVec4};

use crate::GL;


pub trait VertexDef: Sized {
    fn get_attributes() -> Vec<VertexAttrib>;
}

pub struct VertexArrayObject<V: VertexDef + Send + Sync + 'static> {
    id: u32,
    constraint: PhantomData<V>
}

pub fn apply_attributes_to_bound_buffer(attributes: &Vec<VertexAttrib>) {
    let stride = attributes.iter().map(|v| v.attribute_size).sum();
    let mut offset = 0;
    for (index, attrib) in attributes.iter().enumerate() {
        GL!(gl::EnableVertexAttribArray(index as u32));

        GL!(gl::VertexAttribPointer(
            index as u32, 
            attrib.count, 
            attrib.attribute_type, 
            attrib.normalized, 
            stride,
            offset as *const c_void));
        offset += attrib.attribute_size;
    }
}

fn get_static_map<'a>() -> &'a mut Map<dyn Any + Send + Sync> {
    static mut MAP: Mutex<Option<anymap::Map<dyn Any + Send + Sync>>> = Mutex::new(None);

    unsafe { MAP.get_mut().unwrap() }
            .get_or_insert_with(|| anymap::Map::new()) 
}

impl<V: VertexDef + Send + Sync + 'static> VertexArrayObject<V> {
    pub fn bind() {

        let mut map = get_static_map();

        if let Some(val) = map.get::<Self>() {
            GL!(gl::BindVertexArray(val.id));
        }
        else {
            let mut id = 0;
            GL!(gl::GenVertexArrays(1, &mut id));
            GL!(gl::BindVertexArray(id));

            apply_attributes_to_bound_buffer(&V::get_attributes());

            let vao = Self { id, constraint: PhantomData };
            map.insert(vao);
        };
    }

    pub fn unbind() {
        GL!(gl::BindVertexArray(0))
    }

    pub fn delete() {
        let map = get_static_map();
        if let Some(val) = map.remove::<Self>() {
            GL!(gl::DeleteVertexArrays(1, &val.id));
        }
    }
}

pub struct VertexAttrib {
    pub attribute_type: GLenum,
    pub count: GLint,
    pub normalized: GLboolean,
    pub attribute_size: i32,
}

pub trait GlAttributeType {
    fn attribute() -> VertexAttrib;
}

pub fn type_to_attribute<T: GlAttributeType>() -> VertexAttrib {
    T::attribute()
}

impl<T: GlAttributeType> VertexDef for T {
    fn get_attributes() -> Vec<VertexAttrib> {
        vec![Self::attribute()]
    }
}

macro_rules! attribute_impl {
    ($type: ty, $gl_type: expr, $count: literal, $size: expr) => {
        impl GlAttributeType for $type {
            fn attribute() -> VertexAttrib {
                VertexAttrib { 
                    attribute_type: $gl_type, 
                    count: $count, 
                    normalized: gl::FALSE, 
                    attribute_size: $size }
            }
        }
    };

    ($type: ty, $gl_type: expr) => {
        attribute_impl!($type, $gl_type, 1, size_of::<$type>() as i32);
    };
}

macro_rules! attribute_impl_array {
    ($type: ty, $gl_type: expr, $count: literal) => {
        impl GlAttributeType for [$type; $count] {
            fn attribute() -> VertexAttrib {
                VertexAttrib { 
                    attribute_type: $gl_type, 
                    count: $count, 
                    normalized: gl::FALSE, 
                    attribute_size: $count * size_of::<$type>() as i32 }
            }
        }
    };
}

attribute_impl!(f32, gl::FLOAT);
attribute_impl!(i32, gl::INT);
attribute_impl!(u32, gl::INT);
attribute_impl!(bool, gl::BOOL);
attribute_impl!(i8, gl::BYTE);
attribute_impl!(u8, gl::UNSIGNED_BYTE);
attribute_impl!(i16, gl::SHORT);
attribute_impl!(u16, gl::UNSIGNED_SHORT);
attribute_impl!(f64, gl::DOUBLE);

attribute_impl!(Vec2, gl::FLOAT, 2, 2 * size_of::<f32>() as i32);
attribute_impl!(Vec3, gl::FLOAT, 3, 3 * size_of::<f32>() as i32);
attribute_impl!(Vec4, gl::FLOAT, 4, 4 * size_of::<f32>() as i32);

attribute_impl!(IVec2, gl::INT, 2, 2 * size_of::<i32>() as i32);
attribute_impl!(IVec3, gl::INT, 3, 3 * size_of::<i32>() as i32);
attribute_impl!(IVec4, gl::INT, 4, 4 * size_of::<i32>() as i32);

attribute_impl_array!(f32, gl::FLOAT, 2);
attribute_impl_array!(i32, gl::INT, 2);
attribute_impl_array!(u32, gl::INT, 2);
attribute_impl_array!(bool, gl::BOOL, 2);
attribute_impl_array!(i8, gl::BYTE, 2);
attribute_impl_array!(u8, gl::UNSIGNED_BYTE, 2);
attribute_impl_array!(i16, gl::SHORT, 2);
attribute_impl_array!(u16, gl::UNSIGNED_SHORT, 2);
attribute_impl_array!(f64, gl::DOUBLE, 2);

attribute_impl_array!(f32, gl::FLOAT, 3);
attribute_impl_array!(i32, gl::INT, 3);
attribute_impl_array!(u32, gl::INT, 3);
attribute_impl_array!(bool, gl::BOOL, 3);
attribute_impl_array!(i8, gl::BYTE, 3);
attribute_impl_array!(u8, gl::UNSIGNED_BYTE, 3);
attribute_impl_array!(i16, gl::SHORT, 3);
attribute_impl_array!(u16, gl::UNSIGNED_SHORT, 3);
attribute_impl_array!(f64, gl::DOUBLE, 3);

attribute_impl_array!(f32, gl::FLOAT, 4);
attribute_impl_array!(i32, gl::INT, 4);
attribute_impl_array!(u32, gl::INT, 4);
attribute_impl_array!(bool, gl::BOOL, 4);
attribute_impl_array!(i8, gl::BYTE, 4);
attribute_impl_array!(u8, gl::UNSIGNED_BYTE, 4);
attribute_impl_array!(i16, gl::SHORT, 4);
attribute_impl_array!(u16, gl::UNSIGNED_SHORT, 4);
attribute_impl_array!(f64, gl::DOUBLE, 4);

