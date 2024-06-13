use glam::{Mat4, Vec3};

use crate::algorithms::camera::Camera;

pub mod chunk;
pub mod marching_cubes;
pub mod collision_shape;
pub mod field;


#[repr(C)]
#[derive(Default, Debug, VertexDef, Clone)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3
}

pub struct DrawParameters<'a> {
    pub camera: &'a dyn Camera,
    pub model: &'a Mat4,
}