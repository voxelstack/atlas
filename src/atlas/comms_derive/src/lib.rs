use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Shareable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    eprintln!("{:#?}", ast);

    let enum_ident = ast.ident;
    let variants = if let syn::Data::Enum(syn::DataEnum { variants, .. }) = ast.data {
        variants
    } else {
        unimplemented!();
    };

    let store = variants.iter().map(|v| {
        let entry_name = &v.ident;

        match v.fields {
            syn::Fields::Named(_) => todo!(),
            syn::Fields::Unnamed(ref fields) => {
                let mut index = 0;
                let name_fields = fields.unnamed.iter().map(|f| {
                    let field = format!("field{}", index);
                    let span = if let syn::Type::Path(path) = &f.ty {
                        path.path.segments.last().unwrap().ident.span()
                    } else {
                        unimplemented!();
                    };
                    let field = syn::Ident::new(&field, span);
                    index += 1;
                    quote! { #field }
                });

                let mut index = 0;
                let store_fields = fields.unnamed.iter().map(|f| {
                    let field = format!("field{}", index);
                    let span = if let syn::Type::Path(path) = &f.ty {
                        path.path.segments.last().unwrap().ident.span()
                    } else {
                        unimplemented!();
                    };
                    let field = syn::Ident::new(&field, span);
                    index += 1;
                    quote! {
                        payload.push(&#field.into());
                    }
                });

                quote! {
                    #enum_ident::#entry_name(#(#name_fields,)*) => {
                        payload.push(&stringify!(#entry_name).into());
                        #(#store_fields)*
                    }
                }
            }
            syn::Fields::Unit => quote! {
                #enum_ident::#entry_name => {
                    payload.push(&stringify!(#entry_name).into());
                }
            },
        }
    });

    let load = variants.iter().map(|v| {
        let entry_name = &v.ident;

        match v.fields {
            syn::Fields::Named(_) => todo!(),
            syn::Fields::Unnamed(ref fields) => {
                let load_fields = fields.unnamed.iter().map(|_| {
                    // TODO Detect and handle values that don't impl From<JsValue>.
                    // Start with (bools, numbers and strings).
                    quote! {
                        payload.shift().into()
                    }
                });

                quote! {
                    stringify!(#entry_name) => {
                        std::result::Result::Ok(#enum_ident::#entry_name(
                            #(#load_fields,)*
                        ))
                    }
                }
            }
            syn::Fields::Unit => quote! {
                stringify!(#entry_name) => std::result::Result::Ok(#enum_ident::#entry_name)
            },
        }
    });

    let expanded = quote! {
        impl core::convert::Into<(wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>)> for #enum_ident {
            fn into(self) -> (wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>) {
                let payload = js_sys::Array::new();

                match self {
                    #(#store,)*
                };

                (payload.into(), std::option::Option::None)
            }
        }

        impl core::convert::TryFrom<wasm_bindgen::JsValue> for #enum_ident {
            type Error = crate::port::ShareableError;

            fn try_from(value: JsValue) -> Result<Self, Self::Error> {
                let payload: js_sys::Array = value.into();
                let ident = payload.shift().as_string().unwrap();

                match ident.as_ref() {
                    #(#load,)*
                    _ => std::result::Result::Err(crate::port::ShareableError::InvalidIdentifier(ident))
                }
            }
        }

        impl crate::port::Shareable for #enum_ident {}
    };
    expanded.into()
}
