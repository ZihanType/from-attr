use syn::{
    braced, bracketed,
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

pub struct Pair<K, V> {
    pub key: K,
    pub colon_token: Token![:],
    pub value: V,
}

impl<K: Parse, V: Parse> Parse for Pair<K, V> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            colon_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct Map<K, V> {
    pub pairs: Punctuated<Pair<K, V>, Token![,]>,
}

impl<K: Parse, V: Parse> Parse for Map<K, V> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);

        Ok(Self {
            pairs: Punctuated::<Pair<K, V>, Token![,]>::parse_terminated(&content)?,
        })
    }
}
