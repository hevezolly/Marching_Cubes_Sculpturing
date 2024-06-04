use std::{ffi::{CStr, CString}, fs, path::Path, ptr::{null, null_mut}};

use egui_glfw_gl::gl::{self, types};

use crate::GL;

use super::ShaderError;

pub struct Shader {
    pub id: u32,
    pub shader_type: types::GLenum
}

pub struct ShaderBuilder {
    shader_type: types::GLenum,
    defines: Vec<String>
}

impl ShaderBuilder {

    pub fn define(mut self, def: &str) -> ShaderBuilder {
        self.define_ref(def);
        self
    }

    pub fn define_ref(&mut self, def: &str) {
        if def.is_empty() {
            return;
        }
        self.defines.push(def.to_owned());
    }

    pub fn from_source(&self, source: &str) -> Result<Shader, ShaderError> {
        let mut result_source = String::new();
        for d in &self.defines {
            result_source += &format!("#define {}\n", d);
        }

        let version = source.lines().find(|l| l.contains("#version")).ok_or("#version directive missing")?;

        result_source = version.to_owned() + "\n\n" + &result_source;
        
        for l in source.lines() {
            if l.contains("#version") {
                continue;
            }
            result_source += l;
            result_source += "\n";
        }

        // dbg!(&result_source);

        // result_source += source;
        Shader::new(result_source, self.shader_type)
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
        ShaderBuilder { shader_type: gl::FRAGMENT_SHADER, defines: Vec::new() }
    }

    pub fn vertex() -> ShaderBuilder {
        ShaderBuilder { shader_type: gl::VERTEX_SHADER, defines: Vec::new() }
    }

    pub fn compute() -> ShaderBuilder {
        ShaderBuilder { shader_type: gl::COMPUTE_SHADER, defines: Vec::new() }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        GL!(gl::DeleteShader(self.id));
    }
}