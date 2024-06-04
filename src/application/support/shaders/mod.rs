use shaders_loader::ShaderReference;

pub mod shaders_loader;

use shaders_loader::ShaderType::*;
use shaders_loader::ShaderType;

use crate::application::app_logick::BLOCKY;

use super::collision_shape::COMPRESS_COLLISION;

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

            fn preprocessors() -> Vec<&'static str> {
                vec![$($defines),+]
            }
        }
    };
}


shader_ref!(FillCircleProgramm, Compute("resources/shader_sources/fill_circle.compute"));

shader_ref!(MarchingCubeProgramm, Compute("resources/shader_sources/marching_cubes.compute"), 
    if COMPRESS_COLLISION {"COMPRESS_COLLISION"} else {""},
    if BLOCKY {"BLOCKY"} else {""});

shader_ref!(ModelProgramm, Model { vertex: "resources/shader_sources/display_model.vert", 
                                   fragment: "resources/shader_sources/display_model.frag" });

shader_ref!(QuadProgramm, Model { vertex: "resources/shader_sources/display_quad.vert", 
                                  fragment: "resources/shader_sources/display_quad.frag" });