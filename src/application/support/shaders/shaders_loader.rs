use core::shaders::{shader::Shader, shader_programm::ShaderProgramm, ShaderError};
use std::{any::{type_name, TypeId}, collections::HashMap, fmt::Debug, sync::Arc};

use egui::mutex::{Mutex, MutexGuard};


pub enum ShaderType {
    Compute(&'static str),
    Model{
        vertex: &'static str,
        fragment: &'static str
    }
}

pub trait ShaderReference {
    fn defenition() -> ShaderType;
    fn preprocessors() -> Vec<String> {
        vec![]
    }
}

struct ShaderStorageBase {
    programms: HashMap<TypeId, ShaderProgramm>
}

impl ShaderStorageBase {
    pub fn load<T: ShaderReference + 'static>(&mut self) -> Result<(), ShaderError> {
        let def = T::defenition();
        let shader = match def {
            ShaderType::Compute(path) =>{
                let mut shader = Shader::compute();
                for p in T::preprocessors() {
                    shader.define_ref(&p);
                }
                ShaderProgramm::new()
                .attach_shader(shader
                    .from_file(path)?)
                .build()?
            } ,
            ShaderType::Model { vertex, fragment } => {
                let mut frag = Shader::fragment();
                for p in T::preprocessors() {
                    frag.define_ref(&p);
                };
                let mut vert = Shader::vertex();
                for p in T::preprocessors() {
                    vert.define_ref(&p);
                };
                ShaderProgramm::new()
                    .attach_shader(vert
                        .from_file(vertex)?)
                    .attach_shader(frag
                        .from_file(fragment)?)
                    .build()?
            } ,
        };

        let key = TypeId::of::<T>();
        self.programms.insert(key, shader);

        Ok(())
    }
}

pub struct ShaderStorageAccess<'a>(MutexGuard<'a, ShaderStorageBase>);

impl<'a> ShaderStorageAccess<'a> {
    pub fn get<T: ShaderReference + 'static>(&mut self) -> Result<&mut ShaderProgramm, ShaderError> {
        let t = TypeId::of::<T>();

        self.preload::<T>()?;

        self.0.programms.get_mut(&t).ok_or(format!("shader {} not found!", type_name::<T>()))
    }

    pub fn preload<T: ShaderReference + 'static>(&mut self) -> Result<(), ShaderError> {
        let t = TypeId::of::<T>();

        if !self.0.programms.contains_key(&t) {
            self.0.load::<T>()?
        };

        Ok(())
    }
}

#[derive(Clone)]
pub struct ShaderStorage {
    reference: Arc<Mutex<ShaderStorageBase>>
}

impl Debug for ShaderStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderStorage").field("data", &self.access().0.programms).finish()
    }
}

impl ShaderStorage {

    pub fn new() -> ShaderStorage {
        ShaderStorage { reference: Arc::new(Mutex::new(ShaderStorageBase { programms: HashMap::new() })) }
    }

    pub fn access(&self) -> ShaderStorageAccess<'_> {
        ShaderStorageAccess(self.reference.lock())
    }

    pub fn load<T: ShaderReference + 'static>(&mut self) -> Result<(), ShaderError> {
        self.access().0.load::<T>()
    }
}