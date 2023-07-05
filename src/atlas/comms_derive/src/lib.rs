use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Shareable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    eprintln!("{:#?}", ast);

    let name = ast.ident;
    let variants = if let syn::Data::Enum(syn::DataEnum { variants, .. }) = ast.data {
        variants
    } else {
        unimplemented!();
    };

    let to_string = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! { #name::#variant => stringify!(#variant) }
    });

    let from_string = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! { stringify!(#variant) => #name::#variant }
    });

    let expanded = quote! {
        impl core::convert::Into<(wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>)> for #name {
            fn into(self) -> (wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>) {
                let ident = match self {
                    #(#to_string,)*
                };

                (wasm_bindgen::JsValue::from(ident), std::option::Option::None)
            }
        }

        impl core::convert::From<wasm_bindgen::JsValue> for #name {
            fn from(value: JsValue) -> Self {
                let ident = value.as_string().expect("Identifier should be a string.");

                match ident.as_ref() {
                    #(#from_string,)*

                    // TODO This should be a TryFrom instead.
                    _ => panic!("Invalid identifier.")
                }
            }
        }

        impl crate::port::Shareable for #name {}
    };
    expanded.into()
}