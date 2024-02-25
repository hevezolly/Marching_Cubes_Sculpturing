use derive_uniforms::derive_uniforms_internal;
use derive_vertex_def::derive_vertex_def_internal;
// use import_shader::import_shader_macro_execution;
use proc_macro::TokenStream;
#[macro_use]
extern crate quote;

// mod import_shader;
mod derive_vertex_def;
mod derive_uniforms;

#[proc_macro_derive(VertexDef)]
pub fn derive_vertex_def(item: TokenStream) -> TokenStream {
    derive_vertex_def_internal(item)
}

// #[proc_macro_attribute]
// pub fn import_shader(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
//     import_shader_macro_execution(input, annotated_item)
// }


#[proc_macro_derive(Uniforms, attributes(for_shaders, name))]
pub fn derive_uniforms(item: TokenStream) -> TokenStream {
    derive_uniforms_internal(item)
}