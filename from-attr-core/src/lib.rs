use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

pub struct Array<T> {
    pub elems: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);

        Ok(Self {
            elems: Punctuated::<T, Token![,]>::parse_terminated(&content)?,
        })
    }
}

impl<T> Array<T> {
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
}
