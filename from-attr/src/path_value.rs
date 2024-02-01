use proc_macro2::Span;

use crate::ConvertParsed;

/// Represents the path-value pairs parsed from the [`meta`](syn::meta::ParseNestedMeta).
pub struct PathValue<T> {
    /// The path of the meta.
    pub path: Span,
    /// The value parsed from the meta.
    pub value: T,
}

impl<T> ConvertParsed for PathValue<T>
where
    T: ConvertParsed,
{
    type Type = T::Type;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        Ok(PathValue {
            path: path_value.path,
            value: T::convert(path_value)?,
        })
    }

    fn flag() -> Option<Self::Type> {
        T::flag()
    }
}
