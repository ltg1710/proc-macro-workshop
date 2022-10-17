use proc_macro::TokenStream;
use quote::{quote};
use syn::Fields;

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
                unimplemented!("derive(Builder) doesn't support tuple structs")
            }
        }
        _ => unimplemented!("derive(Builder) only support structs"),
    };
    
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
                .field(#literal, &self.#ident)
            )
        }
    });

    let output = quote! (
        impl std::fmt::Debug for #deriver_ident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.debug_struct(#deriver_literal)
                    #(#builder_fields_fmt)*
                   .finish()
            }
        }
    );

    TokenStream::from(output)
}
