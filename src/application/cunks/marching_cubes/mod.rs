use core::{buffers::buffer::{Buffer, VertexBuffer}, context::synchronization_context::SynchronizationContext, textures::texture::Texture};

use glam::{IVec3, Mat4, Vec3};

use crate::{algorithms::camera::Camera, application::support::{bounds::Bounds, shaders::shaders_loader::ShaderStorage}};

use super::{collision_shape::CollisionShape, DrawParameters, ModelVertex};


pub mod full_marcher;
pub mod block_marcher;

pub const WORK_GROUP: IVec3 = IVec3 {
    x: 8,
    y: 8,
    z: 8,
};

pub struct MarchParameters {
    pub sync_context: SynchronizationContext,
    pub distance_field: Texture,
    pub programm_storage: ShaderStorage,
    pub chunk_scale_factor: Vec3,
    pub num_of_cubes: IVec3,
    pub model_vertex_buffer: VertexBuffer<ModelVertex>,
    pub collision_field: CollisionShape,
    pub dirty_area: Bounds<IVec3>,
}

pub trait CubeMarcher {

    fn march(&mut self, params: &mut MarchParameters);
    fn draw<'a>(&mut self, draw_params: DrawParameters<'a>, params: &mut MarchParameters);

    // fn new(params: &MarchParameters) -> Self;
}