use std::{cell::RefCell, collections::HashMap, marker::PhantomData, ops::{BitOr, BitOrAssign}, rc::Rc, sync::{Arc, Mutex, MutexGuard}};
use egui::{ahash::HashSet, util::id_type_map::TypeId};
use egui_glfw_gl::{egui::Output, gl::{self, types::GLbitfield}};

use crate::GL;

pub trait MemoryBarrier {
    fn bit_field(&self) -> gl::types::GLbitfield;
}

#[derive(Debug, Clone, Copy)]
pub struct MemBarrierBits(GLbitfield);

impl MemoryBarrier for MemBarrierBits {
    fn bit_field(&self) -> gl::types::GLbitfield {
        self.0
    }
}

impl<T: MemoryBarrier> BitOr<T> for MemBarrierBits {
    type Output = Self;
    
    fn bitor(self, rhs: T) -> Self::Output {
        MemBarrierBits(self.bit_field() | rhs.bit_field())
    }
}


macro_rules! barrier {
    ($t:ident, $e:expr) => {
        #[derive(Clone, Copy)]
        pub struct $t;
        
        impl MemoryBarrier for $t {
            fn bit_field(&self) -> GLbitfield {
                $e
            }
        }

        impl<T: MemoryBarrier> BitOr<T> for $t {
            type Output = MemBarrierBits;
            
            fn bitor(self, rhs: T) -> Self::Output {
                MemBarrierBits(self.bit_field() | rhs.bit_field())
            }
        }
    };
}

barrier!(ShaderStorageBarrier, gl::SHADER_STORAGE_BARRIER_BIT);
barrier!(ShaderImageAccessBarrier, gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
barrier!(CommandBarrier, gl::COMMAND_BARRIER_BIT);
barrier!(BufferUpdateBarrier, gl::BUFFER_UPDATE_BARRIER_BIT);
barrier!(AllBarrier, gl::ALL_BARRIER_BITS);

pub struct Context {
    value: GLbitfield
}

enum ContextOp {
    Sync,
    SetDirty
}

impl Context {
    pub fn sync(&mut self, bits: GLbitfield) {
        let to_sync = self.value & bits;
        if to_sync > 0 {
            GL!(gl::MemoryBarrier(to_sync));
            self.value &= !to_sync
        }
    }

    pub fn set_dirty(&mut self, bits: GLbitfield) {
        self.value |= bits;
    }
}

#[derive(Clone)]
pub struct SynchronizationContext {
    context: Arc<Mutex<Context>>
}

impl SynchronizationContext {
    pub fn new() -> Self {
        SynchronizationContext {context: Arc::new(Mutex::new(Context { value: 0 }))}
    }

    pub fn dirty<T: MemoryBarrier>(&self, val: T) {
        self.context.lock().unwrap().set_dirty(val.bit_field());
    }

    pub fn sync<T: MemoryBarrier>(&self, val: T) {
        self.context.lock().unwrap().sync(val.bit_field());
    }

    pub fn force_sync<T: MemoryBarrier>(&self, val: T) {
        let mut context = self.context.lock().unwrap();
        let bits = val.bit_field();
        context.set_dirty(bits);
        context.sync(bits);
    }
}