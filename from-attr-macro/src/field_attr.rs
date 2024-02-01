use from_attr_core::Array;
use syn::{meta::ParseNestedMeta, spanned::Spanned, Attribute, Expr, Ident, LitStr, Token};

use crate::{flag_or_value::FlagOrValue, ATTRIBUTE_IDENT};

#[derive(Default)]
struct FieldAttrOptions {
    rename: Option<String>,
    default: FlagOrValue<Expr>,
    conflicts: Option<Array<Ident>>,
}

impl FieldAttrOptions {
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

        if meta_path.is_ident("rename") {
            check_duplicate!(rename);
            self.rename = Some(meta.value()?.parse::<LitStr>()?.value());
            return Ok(());
        }

        if meta_path.is_ident("default") {
            match self.default {
                FlagOrValue::Flag | FlagOrValue::Value(_) => {
                    return Err(meta.error("duplicate `default` argument"))
                }
                FlagOrValue::None => {}
            }

            self.default = if !meta.input.peek(Token![=]) {
                FlagOrValue::Flag
            } else {
                FlagOrValue::Value(meta.value()?.parse()?)
            };
            return Ok(());
        }

        if meta_path.is_ident("conflicts") {
            check_duplicate!(conflicts);
            self.conflicts = Some(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("the argument must be one of: `rename`, `default`, `conflicts`"))
    }

    fn parse_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| self.parse_meta(meta))
    }
}

#[derive(Default)]
pub(crate) struct FieldAttr {
    pub(crate) rename: Option<String>,
    pub(crate) default: FlagOrValue<Expr>,
    pub(crate) conflicts: Vec<Ident>,
}

impl FieldAttr {
    pub(crate) fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        if attrs.is_empty() {
            return Ok(None);
        }

        let mut options = FieldAttrOptions::default();
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

        let FieldAttrOptions {
            rename,
            default,
            conflicts,
        } = options;

        Ok(Some(Self {
            rename,
            default,
            conflicts: conflicts
                .map(|conflicts| conflicts.elems.into_iter().collect())
                .unwrap_or_default(),
        }))
    }
}
