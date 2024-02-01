use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, Attribute, MetaList};

use crate::{AttributeIdent, AttrsValue, ParseMeta};

/// Used for conversion from [`Attribute`]s, [`MetaList`], [`TokenStream`] to values.
///
/// Instead of converting directly from [`Attribute`]s, [`MetaList`], [`TokenStream`] to values,
/// we need to use the helper type that implements [`ParseMeta`] and [`Default`] to convert to this type first,
/// and then convert to the value we really need.
pub trait FromAttr: Sized {
    /// The helper type that implements [`ParseMeta`] and [`Default`].
    type Parser: ParseMeta + Default;

    /// Convert from [`Parser`](type@crate::FromAttr::Parser) to values.
    fn from_parser(parser: Self::Parser, spans: &[Span]) -> syn::Result<Self>;

    /// Convert from [`MetaList`] to values.
    fn from_meta_list(list: &MetaList) -> syn::Result<Option<Self>>
    where
        Self: AttributeIdent,
    {
        if <Self as AttributeIdent>::is_ident(&list.path) {
            let mut parser_default = Self::Parser::default();
            let spans = vec![list.span()];
            parser_default.parse_meta_list(list)?;
            Ok(Some(Self::from_parser(parser_default, &spans)?))
        } else {
            Ok(None)
        }
    }

    /// Convert from [`Attribute`]s to values.
    ///
    /// *Does not* remove [`Attribute`]s and is generally used to parse attributes of derive macros.
    fn from_attributes(
        attrs: &[Attribute],
    ) -> Result<Option<AttrsValue<&Attribute, Self>>, AttrsValue<&Attribute, syn::Error>>
    where
        Self: AttributeIdent,
    {
        if attrs.is_empty() {
            return Ok(None);
        }

        let mut matched_attr_spans = Vec::new();
        let mut matched_attrs = Vec::new();
        let mut parser_default = Self::Parser::default();
        let mut errors = Vec::new();

        attrs
            .iter()
            .filter(|attr| <Self as AttributeIdent>::is_ident(attr.path()))
            .for_each(|attr| {
                matched_attr_spans.push(attr.span());
                matched_attrs.push(attr);

                if let Err(err) = parser_default.parse_attr(attr) {
                    errors.push(err);
                };
            });

        if matched_attrs.is_empty() {
            return Ok(None);
        }

        if let Some(e) = errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            return Err(AttrsValue {
                attrs: matched_attrs,
                value: e,
            });
        }

        match Self::from_parser(parser_default, &matched_attr_spans) {
            Ok(o) => Ok(Some(AttrsValue {
                attrs: matched_attrs,
                value: o,
            })),
            Err(err) => Err(AttrsValue {
                attrs: matched_attrs,
                value: err,
            }),
        }
    }

    /// Convert from [`Attribute`]s to values.
    ///
    /// *Does* remove [`Attribute`]s and is generally used to parse attributes of attribute macros.
    fn remove_attributes(
        attrs: &mut Vec<Attribute>,
    ) -> Result<Option<AttrsValue<Attribute, Self>>, AttrsValue<Attribute, syn::Error>>
    where
        Self: AttributeIdent,
    {
        if attrs.is_empty() {
            return Ok(None);
        }

        let mut matched_attr_spans = Vec::new();
        let mut matched_attrs = Vec::new();
        let mut parser_default = Self::Parser::default();
        let mut errors = Vec::new();

        let mut i = 0;

        while i < attrs.len() {
            if !<Self as AttributeIdent>::is_ident(attrs[i].path()) {
                i += 1;
            } else {
                let attr = attrs.remove(i);

                if let Err(err) = parser_default.parse_attr(&attr) {
                    errors.push(err);
                };

                matched_attr_spans.push(attr.span());
                matched_attrs.push(attr);
            }
        }

        if matched_attrs.is_empty() {
            return Ok(None);
        }

        if let Some(e) = errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            return Err(AttrsValue {
                attrs: matched_attrs,
                value: e,
            });
        }

        match Self::from_parser(parser_default, &matched_attr_spans) {
            Ok(o) => Ok(Some(AttrsValue {
                attrs: matched_attrs,
                value: o,
            })),
            Err(err) => Err(AttrsValue {
                attrs: matched_attrs,
                value: err,
            }),
        }
    }

    /// Convert from [`TokenStream`] to values.
    ///
    /// Generally used for parsing [`TokenStream`] for attribute macros.
    fn from_tokens(tokens: TokenStream) -> syn::Result<Self> {
        let mut parser_default = Self::Parser::default();
        let spans = vec![tokens.span()];
        parser_default.parse_tokens(tokens)?;
        Self::from_parser(parser_default, &spans)
    }
}
