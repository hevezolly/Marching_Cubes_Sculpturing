use std::{fs, path::PathBuf};

use glsl::{parser::Parse, syntax::{SingleDeclaration, StorageQualifier, TypeQualifierSpec, TypeSpecifierNonArray}};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, Field, ItemStruct, LitStr, Token, Type};


const OPTIONAL_PATTERN: &'static str = "//!OPTIONAL";
const FILE_PATHS_ATTRIBUTE: &'static str = "for_shaders";
const NAME_ATTRIBUTE: &'static str = "name";

fn glsl_type_to_rust_type(glsl_type: &TypeSpecifierNonArray, shader_name: &str) -> proc_macro2::TokenStream {
    match glsl_type {
        TypeSpecifierNonArray::Bool => quote!(bool),
        TypeSpecifierNonArray::Int => quote!(i32),
        TypeSpecifierNonArray::UInt => quote!(u32),
        TypeSpecifierNonArray::Float => quote!(f32),
        TypeSpecifierNonArray::Vec2 => quote!([f32;2]),
        TypeSpecifierNonArray::Vec3 => quote!([f32;3]),
        TypeSpecifierNonArray::Vec4 => quote!([f32;4]),
        TypeSpecifierNonArray::BVec2 => quote!([bool;2]),
        TypeSpecifierNonArray::BVec3 => quote!([bool;3]),
        TypeSpecifierNonArray::BVec4 => quote!([bool;4]),
        TypeSpecifierNonArray::IVec2 => quote!([i32;2]),
        TypeSpecifierNonArray::IVec3 => quote!([i32;3]),
        TypeSpecifierNonArray::IVec4 => quote!([i32;4]),
        TypeSpecifierNonArray::UVec2 => quote!([u32;2]),
        TypeSpecifierNonArray::UVec3 => quote!([u32;3]),
        TypeSpecifierNonArray::UVec4 => quote!([u32;4]),
        TypeSpecifierNonArray::Sampler1D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Sampler2D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Sampler3D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Image1D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Image2D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Image3D => quote!(core::textures::TextureUnit),
        TypeSpecifierNonArray::Mat2 => quote!(glam::Mat2),
        TypeSpecifierNonArray::Mat3 => quote!(glam::Mat3),
        TypeSpecifierNonArray::Mat4 => quote!(glam::Mat4),
        other => panic!("shader '{}' error: glsl type {:?} not supported for uniforms", shader_name, other)
    }
}

struct UniformDefinition {
    name: String,
    rust_type: proc_macro2::TokenStream,
    optional: bool,
}

impl UniformDefinition {
    fn try_from_line(line: &str, shader_name: &str) -> Option<UniformDefinition> {
        let optional = line.contains(OPTIONAL_PATTERN);
        let declaration = SingleDeclaration::parse(line).ok()?;
        let has_uniform = declaration.ty.qualifier?.qualifiers.0.iter()
            .any(|q| match q {
                TypeQualifierSpec::Storage(StorageQualifier::Uniform) => true,
                _ => false,
            });
        
        if !has_uniform {return None;};
        
        let name = declaration.name?.0.clone();

        let uni_type_decl = declaration.ty.ty;

        if uni_type_decl.array_specifier.is_some() {
            panic!("\"{}\" shader error: array uniforms not supported", shader_name);
        };

        let rust_type = glsl_type_to_rust_type(&uni_type_decl.ty, shader_name);


        Some(UniformDefinition { name, rust_type, optional })
    }
}

pub fn derive_uniforms_internal(item: TokenStream) -> TokenStream {
    let parsed_item = parse_macro_input!(item as ItemStruct);

    let self_type = &parsed_item.ident;
    
    let (field_names, field_types): (Vec<_>, Vec<_>) = 
        parsed_item.fields.iter().map(|f| (&f.ident, &f.ty)).unzip();

    let uniform_names: Vec<_> = parsed_item.fields
        .iter()
        .map(field_name)
        .collect();

    let uniform_types = get_uniform_types_from_file(&parsed_item, &uniform_names);


    let result = quote!(
        impl core::shaders::uniforms::Uniforms for #self_type {
            fn apply_uniforms(&self, names_mapping: &std::collections::HashMap<String, i32>) {
                #(core::shaders::uniforms::UniformCompatableType::apply_by_name(&self.#field_names, #uniform_names, names_mapping);)*
            }
            
            fn defenition() -> Vec<String> {
                #(let compatable = core::shaders::uniforms::check_uniform_compatable::<#field_types, #uniform_types>();)*
                // #(let compatable = <#fied_types as core::shaders::uniforms::UniformCompatableType<Target = #uniform_types>>::IS_COMPATABLE;)*
                // #(let alias = <#fied_types as core::OpenglAlias<#uniform_types>>::IS_ALIAS;)*
                vec![#(#uniform_names.to_owned(),)*]
            }
        }
    );

    result.into()
}

fn field_name(field: &Field) -> String {
    let attribute = field.attrs.iter().find(|a| a.path().is_ident(NAME_ATTRIBUTE));
    
    if let Some(attribute) = attribute {
        attribute
            .parse_args::<LitStr>()
            .expect("attribute name should contain string literal")
            .value()
    }
    else {
        field.ident
            .to_token_stream()
            .to_string()
    }
}

fn get_uniform_types_from_file(
    parsed_item: &ItemStruct,
    uniform_names: &Vec<String>
) -> Vec<Type> {
    let mut uniform_types = Vec::new();

    if let Some(uniforms) = get_uniforms_from_file(&parsed_item) {
        let mut used_uniforms = Vec::with_capacity(uniforms.len());
        for field_name in uniform_names.iter() {
            if let Some(uniform) = uniforms.iter().find(|u| &u.name == field_name) {
                if used_uniforms.contains(&uniform.name) {
                    continue;
                    // panic!("uniform '{}' is defined multiple times", uniform.name)
                }


                uniform_types.push(Type::Verbatim(uniform.rust_type.clone()));
                used_uniforms.push(uniform.name.to_owned());
            }
            else {
                panic!("uniform with name {} not found in shader sources", field_name)
            }
        }

        let unused_uniforms: Vec<_> = uniforms
            .into_iter()
            .filter(|v| !used_uniforms.contains(&v.name))
            .filter(|v| !v.optional)
            .map(|v| format!("'{}'", v.name))
            .collect();
        if unused_uniforms.len() != 0 {
            panic!("unused uniforms found in shader sources: {}", unused_uniforms.join(", "));
        }
    }

    uniform_types
}

fn get_uniforms_from_file(parsed_item: &ItemStruct) -> Option<Vec<UniformDefinition>> {
    let file_paths_source = parsed_item.attrs.iter().find(|a| a.path().is_ident(FILE_PATHS_ATTRIBUTE));
    
    let parser = Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;

    if let Some(p) = file_paths_source {
        let mut result = Vec::<UniformDefinition>::new();

        let paths = p.parse_args_with(parser).unwrap();
        for literal in paths {
            let path_str = literal.value();
            let path = PathBuf::from(&path_str)
                .canonicalize()
                .expect(&format!("path {} does not exist", &path_str));
            
            if !path.is_file() {
                panic!("path {} should be a file", path_str);
            }
    
    
            let name = &path.file_name()
                .expect("shader path should be a file").to_str()
                .expect("unreadable ").to_owned();
    
            let source = fs::read_to_string(path).unwrap();
    
            for uniform in source.lines()
                .filter_map(|l| UniformDefinition::try_from_line(l, name)) 
            {
                if let Some(UniformDefinition { name, rust_type, optional: _ }) = result.iter()
                    .find(|f| f.name == uniform.name) 
                {
                    if uniform.rust_type.to_string() != rust_type.to_string() {
                        panic!("uniform '{}' is declared multiple times with different types", name);
                    }
                    continue;
                }
                result.push(uniform);
            }
        }
        Some(result)
    }
    else {
        None
    }
}