use syn::parse::{Parse, ParseStream};

pub enum FlagOrType<T> {
    Flag,
    Type(T),
}

impl<T: Parse> Parse for FlagOrType<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse().map(Self::Type)
    }
}
