use proc_macro2::Span;

use crate::{flag_or_type::FlagOrType, ConvertParsed, PathValue};

/// Represents the 3 cases parsed from the [`meta`](syn::meta::ParseNestedMeta).
#[derive(Clone, Copy, Debug, Default)]
pub enum FlagOrValue<T> {
    /// No value.
    #[default]
    None,
    /// Only the flag.
    Flag {
        /// The path of the meta.
        path: Span,
    },
    /// The path and the value.
    Value {
        /// The path of the meta.
        path: Span,
        /// The value parsed from the meta.
        value: T,
    },
}

impl<T> ConvertParsed for FlagOrValue<T>
where
    T: ConvertParsed,
{
    type Type = FlagOrType<T::Type>;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        let PathValue { path, value } = path_value;

        match value {
            FlagOrType::Flag => Ok(Self::Flag { path }),
            FlagOrType::Type(value) => Ok(Self::Value {
                path,
                value: T::convert(PathValue { path, value })?,
            }),
        }
    }

    fn default() -> Option<Self> {
        Some(Self::None)
    }

    fn flag() -> Option<Self::Type> {
        Some(FlagOrType::Flag)
    }
}
