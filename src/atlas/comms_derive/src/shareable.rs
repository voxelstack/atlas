use crate::attrs::{parse_attributes, Repr};
use quote::quote;
use std::iter;
use syn::spanned::Spanned;

const UNSUPPORTED_UNION: &str = "unions are not supported by derive(Shareable)";

pub fn expand_derive_shareable(ast: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // Might wanna write this to the payload too so we can check if we're
    // deserializing the right type.
    let shareable_ident = &ast.ident;

    let write = match &ast.data {
        syn::Data::Struct(data_struct) => write_shareable_struct(shareable_ident, data_struct),
        syn::Data::Enum(data_enum) => write_shareable_enum(shareable_ident, data_enum),
        syn::Data::Union(_) => Err(syn::Error::new(ast.span(), UNSUPPORTED_UNION)),
    }?;

    let read = match &ast.data {
        syn::Data::Struct(data_struct) => read_shareable_struct(shareable_ident, data_struct),
        syn::Data::Enum(data_enum) => read_shareable_enum(shareable_ident, data_enum),
        syn::Data::Union(_) => Err(syn::Error::new(ast.span(), UNSUPPORTED_UNION)),
    }?;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics
            core::convert::Into<(
                wasm_bindgen::JsValue,
                std::option::Option<wasm_bindgen::JsValue>
            )> for #shareable_ident #ty_generics
            #where_clause
        {
            fn into(self) -> (wasm_bindgen::JsValue, std::option::Option<wasm_bindgen::JsValue>) {
                let payload = js_sys::Array::new();
                let mut transfer = js_sys::Array::new();

                payload.push(&stringify!(#shareable_ident).into());

                #write

                let transfer = if transfer.length() > 0 {
                    std::option::Option::Some(transfer.into())
                } else {
                    std::option::Option::None
                };

                (payload.into(), transfer)
            }
        }

        impl #impl_generics
            core::convert::TryFrom<wasm_bindgen::JsValue> for #shareable_ident #ty_generics
            #where_clause
        {
            type Error = crate::port::ShareableError;

            fn try_from(value: JsValue) -> Result<Self, Self::Error> {
                let payload: js_sys::Array = value.into();

                let ident = payload
                    .shift()
                    .as_string()
                    .ok_or(crate::port::ShareableError::BadPayload)?;
                if ident != stringify!(#shareable_ident) {
                    return std::result::Result::Err(crate::port::ShareableError::IncompatibleType);
                }

                #read
            }
        }

        impl #impl_generics
            crate::port::Shareable for #shareable_ident #ty_generics
            #where_clause
        {}
    };

    Ok(expanded)
}

fn unnamed_ident(i: usize, f: &syn::Field) -> syn::Ident {
    let span = if let syn::Type::Path(path) = &f.ty {
        path.path.segments.last().unwrap().ident.span()
    } else {
        todo!("might need to support other types that aren't paths");
    };
    syn::Ident::new(&format!("field{}", i), span)
}

fn write_field((index, field): (usize, &syn::Field)) -> syn::Result<proc_macro2::TokenStream> {
    let is_named = field.ident.is_some();
    let field_ident = field.ident.clone().unwrap_or(unnamed_ident(index, field));
    let field_attrs = parse_attributes(field)?;

    let mut statements: Vec<proc_macro2::TokenStream> = Vec::new();

    // Named fields could be out of order so the identifier has to be written to
    // the payload. Since this is only meant to be written/read by
    // derive(Serialize) generated code, it's possible (and probably smart) to
    // remove the extra safety for performance. Have to benchmark the message
    // passing later.
    if is_named {
        statements.push(quote! { payload.push(&stringify!(#field_ident).into()); });
    }

    match field_attrs.repr {
        Repr::Raw => {
            if field_attrs.transfer {
                statements.push(quote! { transfer.push(&#field_ident.clone().into()); });
            }
            statements.push(quote! { payload.push(&#field_ident.into()); });
        }
        Repr::Serde => statements
            .push(quote! { payload.push(&serde_wasm_bindgen::to_value(&#field_ident).unwrap()); }),
        Repr::Shareable => statements.push(quote! {
            let (data, nested_transfer) = #field_ident.into();
            match nested_transfer {
                Some(nested_transfer) => {
                    transfer = transfer.concat(&nested_transfer.into());
                }
                _ => (),
            };
            payload.push(&data);
        }),
    };

    Ok(quote! { #(#statements)* })
}

fn read_field(field: &syn::Field) -> syn::Result<proc_macro2::TokenStream> {
    let field_ident = &field.ident;
    let field_attrs = parse_attributes(field)?;

    let expanded = if field_ident.is_some() {
        match field_attrs.repr {
            Repr::Raw => quote! {
                #field_ident: fields
                    .remove(stringify!(#field_ident))
                    .ok_or(crate::port::ShareableError::BadPayload)?
                    .into()
            },
            Repr::Serde => quote! {
                #field_ident: serde_wasm_bindgen::from_value(fields
                    .remove(stringify!(#field_ident))
                    .ok_or(crate::port::ShareableError::BadPayload)?
                ).map_err(|_| crate::port::ShareableError::BadPayload)?
            },
            Repr::Shareable => quote! {
                #field_ident: fields
                    .remove(stringify!(#field_ident))
                    .ok_or(crate::port::ShareableError::BadPayload)?
                    .try_into()?
            },
        }
    } else {
        match field_attrs.repr {
            Repr::Raw => quote! { payload.shift().into() },
            Repr::Serde => {
                quote! { serde_wasm_bindgen::from_value(payload.shift())
                    .map_err(|_| crate::port::ShareableError::BadPayload)?
                }
            }
            Repr::Shareable => quote! { payload.shift().try_into()? },
        }
    };
    Ok(expanded)
}

fn list_fields(fields: &syn::Fields) -> proc_macro2::TokenStream {
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

fn write_fields_named(fields_named: &syn::FieldsNamed) -> syn::Result<proc_macro2::TokenStream> {
    let write_fields = fields_named
        .named
        .iter()
        .enumerate()
        .map(write_field)
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    Ok(quote! { #(#write_fields)* })
}

fn write_fields_unnamed(
    fields_unnamed: &syn::FieldsUnnamed,
) -> syn::Result<proc_macro2::TokenStream> {
    let write_fields = fields_unnamed
        .unnamed
        .iter()
        .enumerate()
        .map(write_field)
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    Ok(quote! { #(#write_fields)* })
}

fn read_fields_named(
    structure_ident: &impl quote::ToTokens,
    fields_named: &syn::FieldsNamed,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_count = fields_named.named.len();
    let read_fields = fields_named
        .named
        .iter()
        .map(read_field)
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    let read = quote! {std::result::Result::Ok({
        let mut fields = std::collections::HashMap::<String, JsValue>::new();
        for _ in 0..#field_count {
            let field_name = payload
                .shift()
                .as_string()
                .ok_or(crate::port::ShareableError::BadPayload)?;
            fields.insert(field_name, payload.shift());
        }

        #structure_ident { #(#read_fields,)* }
    })};
    Ok(read)
}

fn read_fields_unnamed(
    structure_ident: &impl quote::ToTokens,
    fields_unnamed: &syn::FieldsUnnamed,
) -> syn::Result<proc_macro2::TokenStream> {
    let read_fields = fields_unnamed
        .unnamed
        .iter()
        .map(read_field)
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    let read = quote! {std::result::Result::Ok(
        #structure_ident(#(#read_fields,)*)
    )};
    Ok(read)
}

fn write_shareable_struct(
    shareable_ident: &syn::Ident,
    data_struct: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::DataStruct { fields, .. } = data_struct;

    let list_fields = list_fields(fields);
    let (destructure, write_fields) = match &fields {
        syn::Fields::Named(ref fields_named) => (
            quote! { let #shareable_ident { #list_fields } = self; },
            write_fields_named(fields_named)?,
        ),
        syn::Fields::Unnamed(ref fields_unnamed) => (
            quote! { let #shareable_ident(#list_fields) = self; },
            write_fields_unnamed(fields_unnamed)?,
        ),
        syn::Fields::Unit => (quote! {}, quote! {}),
    };

    let write = quote! {
        #destructure
        #write_fields
    };

    Ok(write)
}

fn read_shareable_struct(
    shareable_ident: &syn::Ident,
    data_struct: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::DataStruct { fields, .. } = data_struct;

    let make_struct = match &fields {
        syn::Fields::Named(ref fields_named) => read_fields_named(shareable_ident, fields_named)?,
        syn::Fields::Unnamed(ref fields_unnamed) => {
            read_fields_unnamed(shareable_ident, fields_unnamed)?
        }
        syn::Fields::Unit => quote! { std::result::Result::Ok(#shareable_ident) },
    };

    Ok(quote! { #make_struct })
}

fn write_shareable_enum(
    shareable_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::DataEnum { variants, .. } = data_enum;

    let write_variants = variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            let list_fields = list_fields(&v.fields);

            let write_variant = match &v.fields {
                syn::Fields::Named(ref fields_named) => {
                    let write_fields = write_fields_named(fields_named)?;
                    quote! {
                        #shareable_ident::#variant_ident{#list_fields} => {
                            payload.push(&stringify!(#variant_ident).into());
                            #write_fields
                        }
                    }
                }
                syn::Fields::Unnamed(ref fields_unnamed) => {
                    let write_fields = write_fields_unnamed(fields_unnamed)?;
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
            };

            Ok(write_variant)
        })
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    let write = quote! {
        match self {
            #(#write_variants,)*
        };
    };

    Ok(write)
}

fn read_shareable_enum(
    shareable_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::DataEnum { variants, .. } = data_enum;

    let read_variants = variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            let read_field = match &v.fields {
                syn::Fields::Named(ref fields_named) => {
                    let entry_ident = quote! { #shareable_ident::#variant_ident };
                    read_fields_named(&entry_ident, fields_named)?
                }
                syn::Fields::Unnamed(ref fields_unnamed) => {
                    let entry_ident = quote! { #shareable_ident::#variant_ident };
                    read_fields_unnamed(&entry_ident, fields_unnamed)?
                }
                syn::Fields::Unit => {
                    quote! { std::result::Result::Ok(#shareable_ident::#variant_ident) }
                }
            };

            let read_variant = quote! { stringify!(#variant_ident) => #read_field };
            Ok(read_variant)
        })
        .collect::<syn::Result<Vec<proc_macro2::TokenStream>>>()?;

    let read = quote! {
        let variant_ident = payload
            .shift()
            .as_string()
            .ok_or(crate::port::ShareableError::BadPayload)?;

        match variant_ident.as_ref() {
            #(#read_variants,)*
            _ => std::result::Result::Err(crate::port::ShareableError::BadPayload)
        }

    };

    Ok(read)
}
