use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let derived_obj_ident = ast.ident;
    let derived_obj_builder_ident = format_ident!("{}Builder", derived_obj_ident);

    let fields = match ast.data {
        syn::Data::Struct(ds) => {
            if let Fields::Named(fs) = ds.fields {
                fs.named
            } else {
                unimplemented!("derive(Builder) doesn't support tuple structs")
            }
        }
        _ => unimplemented!("derive(Builder) only support structs"),
    };

    let builder_fields_declare = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!(#ident: std::option::Option<#ty>)
    });

    let builder_fields_default = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident: std::option::Option::None)
    });

    let output = quote!(
        pub struct #derived_obj_builder_ident {
            #(#builder_fields_declare),
            *
        }
        
        impl #derived_obj_ident {
            pub fn builder() -> #derived_obj_builder_ident {
                #derived_obj_builder_ident {
                    #(#builder_fields_default),
                    *
                }
            }
        }
    );

    TokenStream::from(output)
}
