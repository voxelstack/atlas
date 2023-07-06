use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(PartialEq, Eq, Debug)]
enum Attribute {
    Transfer,
}

fn parse_attributes(field: &syn::Field) -> Option<Vec<Attribute>> {
    let attrs: Vec<Attribute> = field
        .attrs
        .iter()
        .filter_map(|a| {
            if let syn::Attribute {
                meta: syn::Meta::List(syn::MetaList { path, tokens, .. }),
                ..
            } = a
            {
                if path.segments.len() == 1 && path.segments.last().unwrap().ident == "shareable" {
                    return Some(
                        tokens
                            .clone()
                            .into_iter()
                            .filter_map(|t| {
                                if let proc_macro2::TokenTree::Ident(ident) = t {
                                    if ident == "transfer" {
                                        return Some(Attribute::Transfer);
                                    }
                                }

                                None
                            })
                            .collect::<Vec<Attribute>>(),
                    );
                }
            };

            None
        })
        .flatten()
        .collect();

    if attrs.len() > 0 {
        Some(attrs)
    } else {
        None
    }
}

#[proc_macro_derive(Shareable, attributes(shareable))]
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
            syn::Fields::Named(ref fields) => {
                let name_fields = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote! { #field_name }
                });

                let store_fields = fields.named.iter().map(|f| {
                    let field_name = &f.ident;

                    let store_data = quote! {
                        payload.push(&stringify!(#field_name).into());
                        payload.push(&#field_name.clone().into());
                    };

                    if let Some(attrs) = parse_attributes(f) {
                        let process_attrs = attrs.iter().map(|a| match a {
                            Attribute::Transfer => {
                                quote! { transfer.push(#field_name.clone().into()); }
                            }
                        });

                        quote! {
                            #store_data
                            #(#process_attrs)*
                        }
                    } else {
                        quote! {
                            #store_data
                        }
                    }
                });

                quote! {
                    #enum_ident::#entry_name { #(#name_fields,)* } => {
                        payload.push(&stringify!(#entry_name).into());
                        #(#store_fields)*
                    }
                }
            }
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

                    let store_data = quote! {
                        payload.push(&#field.clone().into());
                    };

                    if let Some(attrs) = parse_attributes(f) {
                        let process_attrs = attrs.iter().map(|a| match a {
                            Attribute::Transfer => quote! { transfer.push(#field.clone().into()); },
                        });

                        quote! {
                            #store_data
                            #(#process_attrs)*
                        }
                    } else {
                        quote! {
                            #store_data
                        }
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
            syn::Fields::Named(ref fields) => {
                let field_count = fields.named.len();
                let name_fields = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote! {
                        #field_name: fields.remove(stringify!(#field_name)).unwrap().into()
                    }
                });

                quote! {
                    stringify!(#entry_name) => {
                        let mut fields = std::collections::HashMap::<String, JsValue>::new();
                        for _ in 0..#field_count {
                            let field_name = payload.shift().as_string().unwrap();
                            fields.insert(field_name, payload.shift());
                        }

                        std::result::Result::Ok(#enum_ident::#entry_name {
                            #(#name_fields,)*
                        })
                    }
                }
            }
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
                let mut transfer: std::vec::Vec<JsValue> = std::vec::Vec::new();

                match self {
                    #(#store,)*
                };

                let transfer = if transfer.len() > 0 {
                    let js_transfer = js_sys::Array::new_with_length(transfer.len().try_into().unwrap());
                    for (i, t) in transfer.into_iter().enumerate() {
                        js_transfer.set(i.try_into().unwrap(), t);
                    }

                    std::option::Option::Some(js_transfer.into())
                } else {
                    std::option::Option::None
                };

                (payload.into(), transfer)
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
