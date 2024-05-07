use proc_macro2::TokenStream;
use syn::{
    meta::{self, ParseNestedMeta},
    parse::Parser,
    Attribute, Meta, MetaList,
};

/// Used to enable parsing of each [`meta`](syn::meta::ParseNestedMeta).
///
/// Generally an helper struct is generated from the derive macro [`FromAttr`](macro@crate::FromAttr),
/// and an implementation of the [`ParseMeta`] trait is generated on this struct.
pub trait ParseMeta {
    /// Whether the type is unit type.
    fn is_unit() -> bool;

    /// Parse one [`meta`](syn::meta::ParseNestedMeta).
    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()>;

    /// Parse one [`Attribute`].
    fn parse_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        if Self::is_unit() {
            attr.meta.require_path_only().map(|_| ())
        } else if matches!(attr.meta, Meta::Path(_)) {
            // #[one(a, b, c)]
            // if `a`, `b`, `c` all have default value,
            // it should be allowed to be written as `#[one]`
            // instead of having to be written as `#[one()]`
            Ok(())
        } else {
            attr.parse_nested_meta(|meta| self.parse_meta(meta))
        }
    }

    /// Parse one [`MetaList`].
    fn parse_meta_list(&mut self, meta_list: &MetaList) -> syn::Result<()> {
        meta_list.parse_nested_meta(|meta| self.parse_meta(meta))
    }

    /// Parse one [`TokenStream`].
    fn parse_tokens(&mut self, tokens: TokenStream) -> syn::Result<()> {
        meta::parser(|meta| self.parse_meta(meta)).parse2(tokens)
    }
}
