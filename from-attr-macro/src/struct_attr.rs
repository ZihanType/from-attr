use from_attr_core::Array;
use syn::{meta::ParseNestedMeta, spanned::Spanned, Attribute, Ident};

use crate::ATTRIBUTE_IDENT;

#[derive(Default)]
struct StructAttrOptions {
    idents: Option<Array<Ident>>,
}

impl StructAttrOptions {
    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let meta_path = &meta.path;

        macro_rules! check_duplicate {
            ($argument:tt) => {
                if self.$argument.is_some() {
                    return Err(meta.error(concat!(
                        "duplicate `",
                        stringify!($argument),
                        "` argument"
                    )));
                }
            };
        }

        if meta_path.is_ident("idents") {
            check_duplicate!(idents);
            self.idents = Some(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("the argument must be one of: `idents`"))
    }

    fn parse_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| self.parse_meta(meta))
    }
}

pub(crate) struct StructAttr {
    pub(crate) idents: Vec<String>,
}

impl StructAttr {
    pub(crate) fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        if attrs.is_empty() {
            return Ok(None);
        }

        let mut options = StructAttrOptions::default();
        let mut errors = Vec::new();
        let mut attr_spans = Vec::new();

        for attr in attrs
            .iter()
            .filter(|attr| attr.path().is_ident(ATTRIBUTE_IDENT))
        {
            attr_spans.push(attr.span());

            if let Err(err) = options.parse_attr(attr) {
                errors.push(err);
            }
        }

        if attr_spans.is_empty() {
            return Ok(None);
        }

        if let Some(e) = errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            return Err(e);
        }

        let StructAttrOptions { idents } = options;

        let idents = idents
            .map(|idents| {
                idents
                    .elems
                    .into_iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if idents.is_empty() {
            let err = attr_spans
                .into_iter()
                .map(|span| syn::Error::new(span, "missing `idents` argument"))
                .reduce(|mut a, b| {
                    a.combine(b);
                    a
                })
                .unwrap_or_else(|| unreachable!("`attr_spans` is not empty"));

            Err(err)
        } else {
            Ok(Some(Self { idents }))
        }
    }
}
