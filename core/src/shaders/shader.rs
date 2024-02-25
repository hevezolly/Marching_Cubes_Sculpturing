use std::{ffi::{CStr, CString}, fs, path::Path, ptr::{null, null_mut}};

use egui_glfw_gl::gl::{self, types};

use crate::GL;

use super::ShaderError;

pub struct Shader {
    pub id: u32,
    pub shader_type: types::GLenum
}

pub struct ShaderBuilder {
    shader_type: types::GLenum
}

impl ShaderBuilder {
    pub fn from_source(&self, source: &str) -> Result<Shader, ShaderError> {
        Shader::new(source, self.shader_type)
    }

    pub fn from_file<P: AsRef<Path>>(&self, path: P) -> Result<Shader, ShaderError> {
        let content = fs::read_to_string(path).map_err(|o| format!("{o}"))?;
        self.from_source(&content)
    }
}

impl Shader {
    pub fn new<Source: AsRef<str>>(source: Source, shader_type: types::GLenum) -> Result<Shader, ShaderError> {
        let id = GL!(gl::CreateShader(shader_type));
        let c_string = CString::new(source.as_ref())
            .map_err(|_| "failed to create source c string")?;
        GL!(gl::ShaderSource(id, 1, &c_string.as_ptr(), null()));
        GL!(gl::CompileShader(id));

        let mut success: i32 = 0;
        GL!(gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success));

        if success == gl::FALSE as i32 {
            let mut err: [types::GLchar; 512] = [0; 512];
            GL!(gl::GetShaderInfoLog(id, 512, null_mut(), err.as_mut_ptr()));
            let shader_compilation_error = unsafe {CStr::from_ptr(err.as_ptr())}.to_str()
                .map_err(|_| "shader generation failed but message is unreadable")?;
            return Err(shader_compilation_error.to_owned());
        };
        Ok(Shader{ id, shader_type })
    }

    pub fn fragment() -> ShaderBuilder {
        ShaderBuilder { shader_type: gl::FRAGMENT_SHADER }
    }

    pub fn vertex() -> ShaderBuilder {
        ShaderBuilder { shader_type: gl::VERTEX_SHADER }
    }

    pub fn compute() -> ShaderBuilder {
        ShaderBuilder { shader_type: gl::COMPUTE_SHADER }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        GL!(gl::DeleteShader(self.id));
    }
}