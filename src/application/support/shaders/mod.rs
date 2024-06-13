use core::GL;

use egui_glfw_gl::gl;
use glam::IVec3;
use shaders_loader::ShaderReference;

pub mod shaders_loader;

use shaders_loader::ShaderType::*;

#[macro_export]
macro_rules! shader_ref {
    ($i:ident, $def:expr) => {
        pub struct $i;

        impl crate::application::support::shaders::shaders_loader::ShaderReference for $i {
            fn defenition() -> crate::application::support::shaders::shaders_loader::ShaderType {
                $def
            }
        }
    };

    ($i:ident, $def:expr, $($defines:expr),+) => {
        pub struct $i;

        impl crate::application::support::shaders::shaders_loader::ShaderReference for $i {
            fn defenition() -> crate::application::support::shaders::shaders_loader::ShaderType {
                $def
            }

            fn preprocessors() -> Vec<String> {
                vec![$(Into::<String>::into($defines)),+]
            }
        }
    };
}


shader_ref!(FillCircleProgramm, Compute("resources/shader_sources/marching_cubes/fill_circle.compute"));
shader_ref!(ZeroFieldProgramm, Compute("resources/shader_sources/marching_cubes/zero_field.compute"));

shader_ref!(ModelProgramm, Model { vertex: "resources/shader_sources/display_model.vert", 
                                   fragment: "resources/shader_sources/display_model.frag" });

shader_ref!(QuadProgramm, Model { vertex: "resources/shader_sources/display_quad.vert", 
                                  fragment: "resources/shader_sources/display_quad.frag" });

pub fn dispatch_compute_for(total_size: IVec3, work_group: IVec3) {
    let dispatch = total_size.as_vec3() / work_group.as_vec3();
    let res = dispatch.ceil();
    GL!(gl::DispatchCompute((res.x) as u32, (res.y) as u32, (res.z) as u32));
}
                                