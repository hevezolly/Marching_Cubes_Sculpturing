use core::buffers::buffer::{BoundBufferContext, Buffer, ShaderStorageBuffer, Usage};
use std::ops::Index;

use glam::IVec3;

use super::app_logick::NUM_OF_CUBES;

const fn ceil_div(val: usize, divider: usize) -> usize {
    let div = val / divider;
    if val % divider > 0 {
        div + 1
    }
    else {
        div
    }
}

pub const NUM_OF_BITMASK_VALUES: usize = ceil_div((NUM_OF_CUBES.x * NUM_OF_CUBES.y * NUM_OF_CUBES.z) as usize, 32);


pub struct BitField {
    field: Box<[[[bool;NUM_OF_CUBES.z as usize]; 
        NUM_OF_CUBES.y as usize]; 
        NUM_OF_CUBES.x as usize]>,
    buffer: ShaderStorageBuffer
}

impl BitField {
    pub fn new() -> BitField {
        let buffer = ShaderStorageBuffer::new();
        buffer
        .bind()
            .new_data(&[0u32; NUM_OF_BITMASK_VALUES], Usage::dynamic_read())
            .unbind();
        
        BitField { 
            field: Box::new([[[false;NUM_OF_CUBES.x as usize]; 
                NUM_OF_CUBES.y as usize]; 
                NUM_OF_CUBES.z as usize]), 
            buffer
        }
    }

    pub fn readback(&mut self) {
        let mut temp = [0i32; NUM_OF_BITMASK_VALUES as usize];
        self.buffer.bind()
            .read_to(&mut temp, 0)
            .unbind();
        
        for x in 0..NUM_OF_CUBES.x {
            for y in 0..NUM_OF_CUBES.y {
                for z in 0..NUM_OF_CUBES.z {
                    let index = x + y * NUM_OF_CUBES.y + z * NUM_OF_CUBES.y * NUM_OF_CUBES.x;
                    let offset = index / 32;
                    let mask = 1 << (index % 32);

                    self.field[x as usize][y as usize][z as usize] = 
                        (temp[offset as usize] & mask) > 0;
                }
            }
        }
    }
    
    pub fn buffer(&self) -> &ShaderStorageBuffer {
        &self.buffer
    }
}

impl Index<IVec3> for BitField {
    type Output = bool;

    fn index(&self, index: IVec3) -> &Self::Output {
        &self.field[index.x as usize][index.y as usize][index.z as usize]
    }
}