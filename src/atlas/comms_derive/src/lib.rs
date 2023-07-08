use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod shareable;

#[proc_macro_derive(Shareable, attributes(shareable))]
pub fn derive_shareable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    shareable::expand_derive_shareable(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
