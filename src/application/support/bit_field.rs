use core::buffers::buffer::{Buffer, BufferDataInterface, Usage};
use std::ops::Index;

use glam::IVec3;

use crate::application::app_logick::{ceil_div, NUM_OF_CUBES};

pub const NUM_OF_BITMASK_VALUES: usize = ceil_div((NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize, 32);

#[derive(Debug)]
pub struct BitField {
    field: Box<[u32; NUM_OF_BITMASK_VALUES]>,
    buffer: Buffer
}

impl BitField {
    pub fn new() -> BitField {
        let field = Box::new([0u32; NUM_OF_BITMASK_VALUES]);
        let buffer = Buffer::from_data(field.as_ref(), Usage::dynamic_read());
        
        BitField { 
            field, 
            buffer
        }
    }

    pub fn readback(&mut self) {
        self.buffer.read_data_from_start(self.field.as_mut());
    }
    
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn get(&self, index: IVec3) -> bool {
        let position = (index.x + 
            index.y * NUM_OF_CUBES.x + 
            index.z * NUM_OF_CUBES.y * NUM_OF_CUBES.x) as usize;

        let block_id = position / 32;
        let mask: u32 = 1 << (position % 32);
        self.field[block_id] & mask > 0
    }
}