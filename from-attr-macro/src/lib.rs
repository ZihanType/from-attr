mod field_attr;
mod flag_or_value;
mod from_attr;
mod from_ident;
mod struct_attr;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

pub(crate) const ATTRIBUTE_IDENT: &str = "attribute";

#[proc_macro_derive(FromAttr, attributes(attribute))]
pub fn from_attr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    from_attr::generate(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(FromIdent)]
pub fn from_ident(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    from_ident::generate(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Debug)]
pub fn debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    from_ident::generate(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
