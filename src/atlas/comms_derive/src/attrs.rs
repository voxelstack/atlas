#[derive(PartialEq, Eq)]
pub enum Repr {
    Raw,
    Serde,
    Shareable,
}

pub struct Attributes {
    pub repr: Repr,
    pub transfer: bool,
}

pub fn parse_attributes(field: &syn::Field) -> Attributes {
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
