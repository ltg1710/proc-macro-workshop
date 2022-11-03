#![feature(trace_macros)]

use proc_macro::TokenStream;
use quote::{quote};
use syn::{Fields, parse_quote, Type, TypePath};

fn get_name_value(name:&str, attr:&syn::Attribute) -> Result<String, syn::__private::TokenStream2>{
    match attr.parse_meta() {
        Ok(syn::Meta::NameValue(nv)) => {
            if !nv.path.is_ident(name) {
                Err(syn::Error::new_spanned(nv, "expected `debug = \"...\")`").to_compile_error())
            } else {
                match &nv.lit {
                    syn::Lit::Str(val) => {
                        Ok(val.value())
                    },
                    _ => {
                        Err(syn::Error::new_spanned(nv, "expected `debug = \"...\")`").to_compile_error())
                    }
                }
            }
        }
        _ => {
            Err(syn::Error::new_spanned(attr, "expected `builder(each = \"...\")`").to_compile_error())
        }
    }
}

// #[debug="0b{:08b}"]
fn get_attr_debug(field:& syn::Field) ->Option<&syn::Attribute> {
    let attrs = &field.attrs;
    attrs.iter().find(|&attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "debug")
}

fn get_phantomdata_generic_type_name(field:&syn::Field) -> syn::Result<Option<String>>{
    if let Type::Path(TypePath{path: syn::Path { ref segments,..}, ..}) = field.ty {
        if let Some(syn::PathSegment{ref ident, ref arguments}) = segments.first() {
            if ident == "PhantomData" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{args, ..}) = arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(ref gp))) = args.first() {
                        if let Some(generic_ident) = gp.path.segments.first() {
                            return Ok(Some(generic_ident.ident.to_string()));
                        }
                    }
                }
            }
        }        
    }
    Ok(None)
}

fn get_field_type(field:&syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath{path: syn::Path{ref segments, ..}, ..}) = field.ty {
        if let Some(syn::PathSegment{ref ident,..}) = segments.last() {
            return Ok(Some(ident.to_string()))
        }
    }
    Ok(None)
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let deriver_ident = &ast.ident;
    let deriver_literal = deriver_ident.to_string();
    
    let fields = match ast.data {
        syn::Data::Struct(ds) => {
            if let Fields::Named(fs) = ds.fields {
                fs.named
            } else {
                unimplemented!("derive(debug) doesn't support tuple structs")
            }
        }
        _ => unimplemented!("derive(debug) only support structs"),
    };

    let mut field_type_names = vec![];
    let mut phantomdata_type_names = vec![];

    for field in &fields {
        if let Ok(Some(n)) = get_phantomdata_generic_type_name(field) {
            phantomdata_type_names.push(n);
        } else if let Ok(Some(n)) = get_field_type(field) {
            field_type_names.push(n);
        }
    }

    // modify generics
    let mut generics_new = ast.generics.clone();
    for g in generics_new.params.iter_mut() {
        if let syn::GenericParam::Type(t) = g {
            let type_name = t.ident.to_string();
            // add debug trait
            if phantomdata_type_names.contains(&type_name)  && !field_type_names.contains(&type_name) {
                continue;
            } else {
                t.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics_new.split_for_impl();
    
    let builder_fields_fmt = fields.iter().map(|f| {
        let ident = &f.ident;
        let literal = ident.as_ref().unwrap().to_string();

        if let Some(attr) = get_attr_debug(f) {
            let fmt = get_name_value("debug", attr).unwrap();
            quote!(
                .field(#literal, &format_args!(#fmt, &self.#ident))
            )
        } else {
            quote!(
                .field(#literal, &format_args!("{:?}", &self.#ident))
            )
        }
    });

    let output = quote! (
        impl #impl_generics std::fmt::Debug for #deriver_ident #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.debug_struct(#deriver_literal)
                    #(#builder_fields_fmt)*
                   .finish()
            }
        }
    );

    // when an error occurs, `str` used to show macro expansion 
    // let str = output.to_string();

    // let res = quote!(
    //     fn test () {
    //         let str = #str;
    //     }
    // );

    TokenStream::from(output)
}
