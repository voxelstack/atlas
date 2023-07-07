use proc_macro::TokenStream;
use quote::quote;
use std::iter;
use syn::{parse_macro_input, DeriveInput};

#[derive(PartialEq, Eq)]
enum Repr {
    Raw,
    Serde,
    Shareable,
}

struct Attributes {
    repr: Repr,
    transfer: bool,
}

fn parse_attributes(field: &syn::Field) -> Attributes {
    let mut field_attrs = Attributes {
        repr: Repr::Shareable,
        transfer: false,
    };

    // TODO Make this function return a syn::Error and fix this nesting mess.
    field.attrs.iter().for_each(|attr| {
        if let syn::Attribute {
            meta: syn::Meta::List(syn::MetaList { path, tokens, .. }),
            ..
        } = attr
        {
            if path
                .get_ident()
                .is_some_and(|ident| ident.to_string() == "shareable")
            {
                let mut list_tokens = tokens.clone().into_iter();
                while let Some(token) = list_tokens.next() {
                    match token {
                        proc_macro2::TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                            "transfer" => field_attrs.transfer = true,
                            "repr" => {
                                let eq_sign = list_tokens.next();

                                if let Some(proc_macro2::TokenTree::Punct(punct)) = eq_sign {
                                    if punct.as_char() == '=' {
                                        let repr = list_tokens.next();
                                        if let Some(proc_macro2::TokenTree::Literal(lit)) = repr {
                                            match lit.to_string().as_ref() {
                                                "\"raw\"" => field_attrs.repr = Repr::Raw,
                                                "\"serde\"" => field_attrs.repr = Repr::Serde,
                                                "\"shareable\"" => {
                                                    field_attrs.repr = Repr::Shareable
                                                }
                                                _ => todo!("compiler error: invalid repr"),
                                            }
                                        }
                                    } else {
                                        todo!("compiler error: expected '='")
                                    }
                                } else {
                                    todo!("compiler error: expected '='")
                                }
                            }
                            _ => todo!("compiler error: unexpected attr"),
                        },
                        proc_macro2::TokenTree::Punct(punct) => {
                            if punct.as_char() == ',' {
                                continue;
                            }
                        }
                        _ => todo!("compiler error: unexpected token"),
                    }
                }
            }
        };
    });

    field_attrs
}

fn unnamed_ident(i: usize, f: &syn::Field) -> syn::Ident {
    let span = if let syn::Type::Path(path) = &f.ty {
        path.path.segments.last().unwrap().ident.span()
    } else {
        todo!("might need to support other types that aren't paths");
    };
    syn::Ident::new(&format!("field{}", i), span)
}

fn list_fields(fields: &syn::Fields) -> proc_macro2::TokenStream {
    // TODO I'm returning a quote!, this doen't have to be a box.
    let field_names: Box<dyn Iterator<Item = proc_macro2::TokenStream>> = match &fields {
        syn::Fields::Named(ref fields_named) => Box::new(fields_named.named.iter().map(|f| {
            let field_name = &f.ident;
            quote! {#field_name}
        })),
        syn::Fields::Unnamed(ref fields_unnamed) => {
            Box::new(fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                let field_name = unnamed_ident(i, f);
                quote! {#field_name}
            }))
        }
        syn::Fields::Unit => Box::new(iter::empty()),
    };

    quote! { #(#field_names,)* }
}

fn write_fields_named(fields_named: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let write_fields = fields_named.named.iter().map(|f| {
        let field_ident = &f.ident;
        let field_attrs = parse_attributes(f);

        match field_attrs.repr {
            Repr::Raw | Repr::Shareable => {
                let write = quote! {
                    payload.push(&stringify!(#field_ident).into());
                    payload.push(&#field_ident.into());
                };

                if field_attrs.transfer {
                    quote! {
                        transfer.push(#field_ident.clone().into());
                        #write
                    }
                } else {
                    write
                }
            }
            Repr::Serde => quote! {
                payload.push(&stringify!(#field_ident).into());
                payload.push(&serde_wasm_bindgen::to_value(&#field_ident).unwrap());
            },
            // Repr::Shareable => todo!(),
        }
    });

    quote! { #(#write_fields)* }
}

fn write_fields_unnamed(fields_unnamed: &syn::FieldsUnnamed) -> proc_macro2::TokenStream {
    let write_fields = fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
        let field_ident = unnamed_ident(i, f);
        let field_attrs = parse_attributes(f);

        match field_attrs.repr {
            Repr::Raw | Repr::Shareable => {
                let write = quote! { payload.push(&#field_ident.into()); };

                if field_attrs.transfer {
                    quote! {
                        transfer.push(#field_ident.clone().into());
                        #write
                    }
                } else {
                    write
                }
            }
            Repr::Serde => quote! {
                payload.push(&serde_wasm_bindgen::to_value(&#field_ident).unwrap());
            },
            // Repr::Shareable => todo!(),
        }
    });

    quote! { #(#write_fields)* }
}

fn read_fields_named(
    structure_ident: &impl quote::ToTokens,
    fields_named: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    let field_count = fields_named.named.len();
    let read_fields = fields_named.named.iter().map(|f| {
        let field_ident = &f.ident;
        let field_attrs = parse_attributes(f);

        match field_attrs.repr {
            Repr::Raw | Repr::Shareable => quote! {
                #field_ident: fields.remove(stringify!(#field_ident)).unwrap().into()
            },
            Repr::Serde => quote! {
                #field_ident: serde_wasm_bindgen::from_value(
                    fields.remove(stringify!(#field_ident)).unwrap()
                ).unwrap()
            },
            // Repr::Shareable => todo!(),
        }
    });

    quote! {std::result::Result::Ok({
        let mut fields = std::collections::HashMap::<String, JsValue>::new();
        for _ in 0..#field_count {
            let field_name = payload.shift().as_string().unwrap();
            fields.insert(field_name, payload.shift());
        }

        #structure_ident { #(#read_fields,)* }
    })}
}

fn read_fields_unnamed(
    structure_ident: &impl quote::ToTokens,
    fields_unnamed: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let read_fields = fields_unnamed.unnamed.iter().map(|f| {
        let field_attrs = parse_attributes(f);

        match field_attrs.repr {
            Repr::Raw | Repr::Shareable => quote! { payload.shift().into() },
            Repr::Serde => quote! { serde_wasm_bindgen::from_value(payload.shift()).unwrap() },
            // Repr::Shareable => todo!(),
        }
    });

    quote! {std::result::Result::Ok(
        #structure_ident(#(#read_fields,)*
    ))}
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
            let list_fields = list_fields(fields);
            let (destructure, write_fields) = match &fields {
                syn::Fields::Named(ref fields_named) => (
                    quote! { let #shareable_ident { #list_fields } = self; },
                    write_fields_named(fields_named),
                ),
                syn::Fields::Unnamed(ref fields_unnamed) => (
                    quote! { let #shareable_ident(#list_fields) = self; },
                    write_fields_unnamed(fields_unnamed),
                ),
                syn::Fields::Unit => (quote! {}, quote! {}),
            };

            let write = quote! {
                let payload = js_sys::Array::new();
                let mut transfer: std::vec::Vec<JsValue> = std::vec::Vec::new();

                #destructure
                #write_fields

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
                syn::Fields::Named(ref fields_named) => {
                    read_fields_named(shareable_ident, fields_named)
                }
                syn::Fields::Unnamed(ref fields_unnamed) => {
                    read_fields_unnamed(shareable_ident, fields_unnamed)
                }
                syn::Fields::Unit => quote! { std::result::Result::Ok(#shareable_ident) },
            };

            let read = quote! {
                let payload: js_sys::Array = value.into();
                #make_struct
            };

            (write, read)
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let write_variants = variants.iter().map(|v| {
                let variant_ident = &v.ident;
                let list_fields = list_fields(&v.fields);

                match &v.fields {
                    syn::Fields::Named(ref fields_named) => {
                        let write_fields = write_fields_named(fields_named);
                        quote! {
                            #shareable_ident::#variant_ident{#list_fields} => {
                                payload.push(&stringify!(#variant_ident).into());
                                #write_fields
                            }
                        }
                    }
                    syn::Fields::Unnamed(ref fields_unnamed) => {
                        let write_fields = write_fields_unnamed(fields_unnamed);
                        quote! {
                            #shareable_ident::#variant_ident(#list_fields) => {
                                payload.push(&stringify!(#variant_ident).into());
                                #write_fields
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
                let read_field = match &v.fields {
                    syn::Fields::Named(ref fields_named) => {
                        let entry_ident = quote! { #shareable_ident::#variant_ident };
                        read_fields_named(&entry_ident, fields_named)
                    }
                    syn::Fields::Unnamed(ref fields_unnamed) => {
                        let entry_ident = quote! { #shareable_ident::#variant_ident };
                        read_fields_unnamed(&entry_ident, fields_unnamed)
                    }
                    syn::Fields::Unit => {
                        quote! { std::result::Result::Ok(#shareable_ident::#variant_ident) }
                    }
                };
                quote! { stringify!(#variant_ident) => #read_field }
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
