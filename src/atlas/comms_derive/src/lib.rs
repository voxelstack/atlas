use proc_macro::TokenStream;
use quote::quote;
use std::iter;
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

    // Might wanna write this to the payload too so we can check if we're
    // deserializing the right type.
    let shareable_ident = &ast.ident;

    let (write, read) = match &ast.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let write_fields: Box<dyn Iterator<Item = proc_macro2::TokenStream>> = match &fields {
                syn::Fields::Named(ref fields_named) => {
                    Box::new(fields_named.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_attrs = parse_attributes(f);
                        let is_transfer =
                            field_attrs.is_some_and(|attrs| attrs.contains(&Attribute::Transfer));

                        let transfer = if is_transfer {
                            quote! { transfer.push(self.#field_name.clone().into()); }
                        } else {
                            quote! {}
                        };

                        quote! {
                            #transfer
                            payload.push(&stringify!(#field_name).into());
                            payload.push(&self.#field_name.into());
                        }
                    }))
                }
                syn::Fields::Unnamed(ref fields_unnamed) => {
                    Box::new(fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                        let field_index = syn::Index::from(i);
                        let field_attrs = parse_attributes(f);
                        let is_transfer =
                            field_attrs.is_some_and(|attrs| attrs.contains(&Attribute::Transfer));

                        let transfer = if is_transfer {
                            quote! { transfer.push(self.#field_index.clone().into()); }
                        } else {
                            quote! {}
                        };

                        quote! {
                            #transfer
                            payload.push(&self.#field_index.into());
                        }
                    }))
                }
                syn::Fields::Unit => Box::new(iter::empty()),
            };

            let write = quote! {
                let payload = js_sys::Array::new();
                let mut transfer: std::vec::Vec<JsValue> = std::vec::Vec::new();

                #(#write_fields)*

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
            };

            let make_struct = match &fields {
                syn::Fields::Named(ref named_fields) => {
                    let field_count = named_fields.named.len();
                    let read_fields = named_fields.named.iter().map(|f| {
                        let field_name = &f.ident;

                        quote! {#field_name: fields.remove(stringify!(#field_name)).unwrap().into()}
                    });
                    quote! {{
                        let mut fields = std::collections::HashMap::<String, JsValue>::new();
                        for _ in 0..#field_count {
                            let field_name = payload.shift().as_string().unwrap();
                            fields.insert(field_name, payload.shift());
                        }

                        #shareable_ident { #(#read_fields,)* }
                    }}
                }
                syn::Fields::Unnamed(ref unnamed_fields) => {
                    let read_fields = unnamed_fields
                        .unnamed
                        .iter()
                        .map(|_| quote! {payload.shift().into()});
                    quote! { #shareable_ident(#(#read_fields,)*) }
                }
                syn::Fields::Unit => quote! { #shareable_ident },
            };

            let read = quote! {
                let payload: js_sys::Array = value.into();
                std::result::Result::Ok(#make_struct)
            };

            (write, read)
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let write_variants = variants.iter().map(|v| {
                let variant_ident = &v.ident;

                fn unnamed_ident(field: &syn::Field, index: usize) -> syn::Ident {
                    let span = if let syn::Type::Path(path) = &field.ty {
                        path.path.segments.last().unwrap().ident.span()
                    } else {
                        todo!("take span from other types that are not a path");
                    };
                    syn::Ident::new(&format!("field{}", index), span)
                }

                let list_fields: Box<dyn Iterator<Item = proc_macro2::TokenStream>> =
                    match &v.fields {
                        syn::Fields::Named(ref fields_named) => {
                            Box::new(fields_named.named.iter().map(|f| {
                                let field_name = &f.ident;
                                quote! {#field_name}
                            }))
                        }
                        syn::Fields::Unnamed(ref fields_unnamed) => {
                            Box::new(fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                                let field_name = unnamed_ident(f, i);
                                quote! {#field_name}
                            }))
                        }
                        syn::Fields::Unit => Box::new(iter::empty()),
                    };

                match &v.fields {
                    syn::Fields::Named(ref fields_named) => {
                        let write_fields = fields_named.named.iter().map(|f| {
                            let field_name = &f.ident;

                            let field_attrs = parse_attributes(f);
                            let is_transfer = field_attrs
                                .is_some_and(|attrs| attrs.contains(&Attribute::Transfer));

                            let transfer = if is_transfer {
                                quote! { transfer.push(#field_name.clone().into()); }
                            } else {
                                quote! {}
                            };

                            quote! {
                                #transfer
                                payload.push(&stringify!(#field_name).into());
                                payload.push(&#field_name.into());
                            }
                        });

                        quote! {
                            #shareable_ident::#variant_ident{#(#list_fields,)*} => {
                                payload.push(&stringify!(#variant_ident).into());
                                #(#write_fields)*
                            }
                        }
                    }
                    syn::Fields::Unnamed(ref fields_unnamed) => {
                        let write_fields =
                            fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                                let field_name = unnamed_ident(f, i);

                                let field_attrs = parse_attributes(f);
                                let is_transfer = field_attrs
                                    .is_some_and(|attrs| attrs.contains(&Attribute::Transfer));

                                let transfer = if is_transfer {
                                    quote! { transfer.push(#field_name.clone().into()); }
                                } else {
                                    quote! {}
                                };

                                quote! {
                                    #transfer
                                    payload.push(&#field_name.into());
                                }
                            });

                        quote! {
                            #shareable_ident::#variant_ident(#(#list_fields,)*) => {
                                payload.push(&stringify!(#variant_ident).into());
                                #(#write_fields)*
                            }
                        }
                    }
                    syn::Fields::Unit => quote! {
                        #shareable_ident::#variant_ident => {
                            payload.push(&stringify!(#variant_ident).into());
                        }
                    },
                }
            });

            let write = quote! {
                let payload = js_sys::Array::new();
                let mut transfer: std::vec::Vec<JsValue> = std::vec::Vec::new();

                match self {
                    #(#write_variants,)*
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
            };

            let read_variants = variants.iter().map(|v| {
                let variant_ident = &v.ident;

                match &v.fields {
                    syn::Fields::Named(ref fields_named) => {
                        let field_count = fields_named.named.len();
                        let name_fields = fields_named.named.iter().map(|f| {
                            let field_name = &f.ident;
                            quote! {
                                #field_name: fields.remove(stringify!(#field_name)).unwrap().into()
                            }
                        });

                        quote! {
                            stringify!(#variant_ident) => {
                                let mut fields = std::collections::HashMap::<String, JsValue>::new();
                                for _ in 0..#field_count {
                                    let field_name = payload.shift().as_string().unwrap();
                                    fields.insert(field_name, payload.shift());
                                }

                                std::result::Result::Ok(#shareable_ident::#variant_ident {
                                    #(#name_fields,)*
                                })
                            }
                        }
                    },
                    syn::Fields::Unnamed(ref fields_unnamed) => {
                        let read_fields = fields_unnamed.unnamed.iter().map(|_| quote!{payload.shift().into()});
                        quote! {
                            stringify!(#variant_ident) => std::result::Result::Ok(#shareable_ident::#variant_ident(#(#read_fields,)*))
                        }
                    },
                    syn::Fields::Unit => quote! {
                        stringify!(#variant_ident) => std::result::Result::Ok(#shareable_ident::#variant_ident)
                    },
                }
            });

            let read = quote! {
                let payload: js_sys::Array = value.into();
                let variant_ident = payload.shift().as_string().unwrap();

                match variant_ident.as_ref() {
                    #(#read_variants,)*
                    _ => std::result::Result::Err(crate::port::ShareableError::InvalidIdentifier(variant_ident))
                }

            };

            (write, read)
        }
        syn::Data::Union(_) => todo!("not supported, generate a compile error"),
    };

    let expanded = quote! {
        impl core::convert::Into<(wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>)> for #shareable_ident {
            fn into(self) -> (wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>) {
                #write
            }
        }

        impl core::convert::TryFrom<wasm_bindgen::JsValue> for #shareable_ident {
            type Error = crate::port::ShareableError;

            fn try_from(value: JsValue) -> Result<Self, Self::Error> {
                #read
            }
        }

        impl crate::port::Shareable for #shareable_ident {}
    };
    expanded.into()
}
