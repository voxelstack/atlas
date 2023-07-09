use syn::spanned::Spanned;

pub struct Attributes {
    pub repr: Repr,
    pub transfer: bool,
}

#[derive(PartialEq, Eq)]
pub enum Repr {
    Raw,
    Serde,
    Shareable,
}

struct ParseAttrs {
    repr: Option<Repr>,
    transfer: Option<bool>,
}

const INVALID_FORMAT: &str =
    "unexpected token, expected attribute arguments in parentheses: #[shareable(...)]";
const INVALID_TOKEN: &str =
    "unexpected token, expected comma separated list of ident = lit or ident";
const INVALID_ATTR: &str = "unexpected attribute, expected ident: repr or transfer";
const INVALID_REPR_END: &str = "unexpected end of attribute definition, expected: repr = \"repr\"";
const INVALID_REPR: &str = "invalid repr, expected literal: \"raw\", \"serde\", or \"shareable\"";
const DUPLICATED_ATTR: &str = "unexpected attribute, attribute is already defined";

pub fn parse_attributes(field: &syn::Field) -> syn::Result<Attributes> {
    let mut field_attrs = ParseAttrs {
        repr: None,
        transfer: None,
    };
    let mut transfer_span: Option<proc_macro2::Span> = None;

    field
        .attrs
        .iter()
        .map(|attr| {
            let syn::MetaList { path, tokens, .. } = match &attr.meta {
                syn::Meta::Path(path) => Err(syn::Error::new(path.span(), INVALID_FORMAT)),
                syn::Meta::List(meta_list) => Ok(meta_list),
                syn::Meta::NameValue(name_value) => {
                    Err(syn::Error::new(name_value.span(), INVALID_FORMAT))
                }
            }?;

            if !path
                .get_ident()
                .is_some_and(|ident| ident.to_string() == "shareable")
            {
                return Err(syn::Error::new(path.span(), INVALID_FORMAT));
            }

            let mut token_stream = tokens.clone().into_iter();
            while let Some(ref token) = token_stream.next() {
                match token {
                    proc_macro2::TokenTree::Ident(ident) => match ident.to_string().as_ref() {
                        "transfer" => {
                            if field_attrs.transfer.is_some() {
                                return Err(syn::Error::new(ident.span(), DUPLICATED_ATTR));
                            }
                            transfer_span = Some(ident.span());
                            field_attrs.transfer = Some(true)
                        }
                        "repr" => {
                            if field_attrs.repr.is_some() {
                                return Err(syn::Error::new(ident.span(), DUPLICATED_ATTR));
                            }
                            field_attrs.repr = Some(parse_repr(token, &mut token_stream)?)
                        }
                        _ => return Err(syn::Error::new(ident.span(), INVALID_ATTR)),
                    },
                    proc_macro2::TokenTree::Punct(punct) => {
                        if punct.as_char() != ',' {
                            return Err(syn::Error::new(punct.span(), INVALID_TOKEN));
                        }
                        continue;
                    }
                    _ => return Err(syn::Error::new(token.span(), INVALID_TOKEN)),
                }
            }

            Ok(())
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let field_attrs = Attributes {
        repr: field_attrs.repr.unwrap_or(Repr::Shareable),
        transfer: field_attrs.transfer.unwrap_or(false),
    };

    if field_attrs.transfer && field_attrs.repr != Repr::Raw {
        Err(syn::Error::new(
            transfer_span.unwrap(),
            "invalid attribute, only repr = \"raw\" fields can be transferred",
        ))
    } else {
        Ok(field_attrs)
    }
}

fn parse_repr(
    ident: &proc_macro2::TokenTree,
    token_stream: &mut proc_macro2::token_stream::IntoIter,
) -> syn::Result<Repr> {
    let separator = token_stream
        .next()
        .ok_or(syn::Error::new(ident.span(), INVALID_REPR_END))?;

    let invalid_separator = match &separator {
        proc_macro2::TokenTree::Punct(punct) => punct.as_char() != '=',
        _ => true,
    };

    if invalid_separator {
        return Err(syn::Error::new(
            separator.span(),
            "unexpected token, expected: =",
        ));
    }

    let repr = token_stream
        .next()
        .ok_or(syn::Error::new(ident.span(), INVALID_REPR_END))?;

    match repr {
        proc_macro2::TokenTree::Literal(lit) => match lit.to_string().as_ref() {
            "\"raw\"" => Ok(Repr::Raw),
            "\"serde\"" => Ok(Repr::Serde),
            "\"shareable\"" => Ok(Repr::Shareable),
            _ => Err(syn::Error::new(lit.span(), INVALID_REPR)),
        },
        _ => Err(syn::Error::new(repr.span(), INVALID_REPR)),
    }
}
