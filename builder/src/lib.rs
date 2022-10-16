use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields};

// Option<String> => Some(String)
fn inner_type_option<'a>(ty:&'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, ref path,}) = ty {
        if path.segments.len() != 1 || path.segments[0].ident != *"Option" {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_type) = path.segments[0].arguments {
            if inner_type.args.len() != 1 {
                return None;
            }

            if let syn::GenericArgument::Type(ref ty) = inner_type.args[0] {
                return Some(ty);
            }
        }
    }
    None
}

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
        let ty:&syn::Type = &f.ty;
        if inner_type_option(ty).is_some() {
            quote!(#ident: #ty)
        } else {
            quote!(#ident: std::option::Option<#ty>)
        }
    });

    let builder_fields_default = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident: std::option::Option::None)
    });

    let builder_fields_setters = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &&f.ty;
        let inner_ty = inner_type_option(ty);
        if let Some(inner_ty) = inner_ty {
            quote!(
                pub fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            )
        } else {
            quote!(
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            )
        }
    });

    let builder_fields_clauses = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &&f.ty;
        if inner_type_option(ty).is_some() {
            quote!(
                #ident: self.#ident.clone()
            )
        } else {
            quote!(
                #ident: self.#ident.clone().ok_or(format!("{} field is missing", stringify!(#ident)))?
            )
        }
    });

    let output = quote!(
        pub struct #derived_obj_builder_ident {
            #(#builder_fields_declare),
            *
        }

        impl #derived_obj_builder_ident {
            #(#builder_fields_setters)*
            pub fn build(&mut self) -> std::result::Result<#derived_obj_ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                std::result::Result::Ok(
                    #derived_obj_ident {
                        #(#builder_fields_clauses),*
                    }
                )
            }
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

    // eprintln!("{:?}", output.to_string());

    TokenStream::from(output)
}
