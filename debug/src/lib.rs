use proc_macro::TokenStream;
use quote::quote;
use syn::Fields;

#[proc_macro_derive(CustomDebug)]
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
        quote!(
            .field(#literal, &self.#ident)
        ) 
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
