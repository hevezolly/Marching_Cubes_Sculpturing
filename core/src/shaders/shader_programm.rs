use std::{collections::HashMap, ffi::{CString, CStr}, fs, path::Path, ptr::{null, null_mut}};

use egui_glfw_gl::gl::{self, types::{self, GLint}};


use crate::{buffers::buffer::{Buffer}, GL};

use super::{shader::Shader, uniforms::{Uniforms, UniformCompatableType}, ShaderError};


#[derive(Debug)]
pub struct ShaderProgramm {
    pub id: u32,
    uniforms_mapping: HashMap<String, GLint>,
}

pub struct ShaderProgrammBuilder {
    id: u32,
}


// impl Drop for ShaderProgramm {
//     fn drop(&mut self) {
//         GL!(gl::DeleteProgram(self.id));
//     }
// }


impl ShaderProgramm {

    pub fn new() -> ShaderProgrammBuilder {
        let id = GL!(gl::CreateProgram());

        ShaderProgrammBuilder { id }
    }

    pub fn bind(&mut self) -> ShaderProgrammContext {
        GL!(gl::UseProgram(self.id));
        ShaderProgrammContext { programm: self }
    }

    pub fn unbind(&self) {
        GL!(gl::UseProgram(0));
    }

    fn get_uniform_location<N: Into<String>>(&mut self, name: N) -> Result<Option<i32>, ShaderError> {
        let name = name.into();
        
        let mut location = self.uniforms_mapping.get(&name).copied();

        if let None = location {
            let c_name = CString::new(name.as_str())
                .map_err(|_| format!("error: failed to create c_string from name {}", name))?;

            let uniform_id = GL!(gl::GetUniformLocation(self.id, c_name.as_ptr()));

            if uniform_id != -1 {
                self.uniforms_mapping.insert(name.to_owned(), uniform_id);
                location = Some(uniform_id);
            }

        };
        Ok(location)
    }
}

pub struct ShaderProgrammContext<'a> {
    programm: &'a mut ShaderProgramm,
}

impl<'a> ShaderProgrammContext<'a> {
    pub fn set_uniforms<U: Uniforms>(self, uniforms: U) -> Result<Self, ShaderError> {

        for name in U::defenition() {
            self.programm.get_uniform_location(name)?;
        };

        uniforms.apply_uniforms(&self.programm.uniforms_mapping);
        Ok(self)
    }

    pub fn set_uniform<N: Into<String>, U: UniformCompatableType>(self, name: N, value: U) -> Result<Self, ShaderError> {
        if let Some(location) = self.programm.get_uniform_location(name)? {
            value.apply_by_location(location);  
        }
        Ok(self)
    }

    pub fn set_buffer<T: AsRef<Buffer>>(self, buffer: T, binding: u32) -> Self {
        GL!(gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, buffer.as_ref().id()));
        self
    }
}

impl ShaderProgrammBuilder {

    pub fn attach_shader(self, shader: Shader) -> ShaderProgrammBuilder {
        GL!(gl::AttachShader(self.id, shader.id));
        self
    }

    pub fn build(self) -> Result<ShaderProgramm, ShaderError> {
        GL!(gl::LinkProgram(self.id));

        let mut success: i32 = 0;
        GL!(gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success));

        if success == gl::FALSE as i32 {
            let mut err: [types::GLchar; 512] = [0; 512];
            GL!(gl::GetProgramInfoLog(self.id, 512, null_mut(), err.as_mut_ptr()));
            let programm_linking_error = unsafe {CStr::from_ptr(err.as_ptr())}.to_str()
                .map_err(|_| "programm linking failed but message is unreadable")?;
            dbg!(programm_linking_error);
            return Err(programm_linking_error.to_owned());
        };

        Ok(ShaderProgramm { id: self.id, uniforms_mapping: HashMap::new() })
    }
}