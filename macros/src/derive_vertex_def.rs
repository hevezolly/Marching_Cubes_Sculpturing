use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};


pub fn derive_vertex_def_internal(item: TokenStream) -> TokenStream {
    let parsed_item = parse_macro_input!(item as ItemStruct);

    let self_type = &parsed_item.ident;

    let field_types: Vec<_> = parsed_item.fields.iter().map(|f| &f.ty).collect();

    let result = quote!(
        impl core::buffers::vertex_attributes::VertexDef for #self_type {
            fn get_attributes() -> Vec<core::buffers::vertex_attributes::VertexAttrib> {
                vec![
                    #(core::buffers::vertex_attributes::type_to_attribute::<#field_types>(),)*
                ]
            }
        }
    );

    result.into()
}