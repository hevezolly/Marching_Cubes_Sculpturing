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

pub struct BitwiseContext<'a> {
    context: MutexGuard<'a, Context>,
    operation: ContextOp,
    value: GLbitfield,
    applied: bool
}

impl<'a> BitwiseContext<'a> {

    pub fn apply(mut self) {
        match self.operation {
            ContextOp::Sync => self.context.sync(self.value),
            ContextOp::SetDirty => self.context.set_dirty(self.value),
        };
    }
}

impl<'a, T: MemoryBarrier> BitOrAssign<T> for BitwiseContext<'a> {
    fn bitor_assign(&mut self, rhs: T) {
        self.value |= rhs.bit_field();
    }
}

impl<'a, T:MemoryBarrier> BitOr<T> for BitwiseContext<'a> {
    type Output = Self;

    fn bitor(mut self, rhs: T) -> Self::Output {
        self |= rhs;
        self
    }
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

    pub fn dirty(&self) -> BitwiseContext {
        BitwiseContext { context: self.context.lock().unwrap(), operation: ContextOp::SetDirty, value: 0, applied: false }
    }

    pub fn dirty_mut(&mut self) -> BitwiseContext {
        BitwiseContext { context: self.context.lock().unwrap(), operation: ContextOp::SetDirty, value: 0, applied: false }
    }

    pub fn dirty_with<T: MemoryBarrier>(&self, val: T) {
        (self.dirty() | val).apply()
    }

    pub fn sync(&self) -> BitwiseContext {
        BitwiseContext { context: self.context.lock().unwrap(), operation: ContextOp::Sync, value: 0, applied: false }
    }

    pub fn sync_with<T: MemoryBarrier>(&self, val: T) {
        (self.sync() | val).apply()
    }

    pub fn force_sync_with<T: MemoryBarrier>(&self, val: T) {
        let val = MemBarrierBits(val.bit_field());
        self.dirty_with(val);
        self.sync_with(val);
    }
}