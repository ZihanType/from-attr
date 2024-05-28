use from_attr_core::Pair;

use crate::{ConvertParsed, PathValue};

/// Represents a map parsed from the [`meta`](syn::meta::ParseNestedMeta).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
pub struct Map<K, V>(pub Vec<(K, V)>);

impl<K, V> ConvertParsed for Map<K, V>
where
    K: ConvertParsed,
    V: ConvertParsed,
{
    type Type = from_attr_core::Map<K::Type, V::Type>;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        let PathValue { path, value } = path_value;

        let mut pairs = Vec::new();
        let mut errors = Vec::new();

        value.pairs.into_iter().for_each(|Pair { key, value, .. }| {
            match (
                K::convert(PathValue { path, value: key }),
                #[allow(clippy::redundant_field_names)]
                V::convert(PathValue { path, value: value }),
            ) {
                (Ok(k), Ok(v)) => pairs.push((k, v)),
                (Err(e), _) | (_, Err(e)) => errors.push(e),
            }
        });

        match errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            Some(e) => Err(e),
            None => Ok(Map(pairs)),
        }
    }

    fn default() -> Option<Self> {
        Some(Map(Vec::new()))
    }
}
