/// Implement the [`ConvertParsed<Type = syn::MetaList>`](crate::ConvertParsed) trait for the type.
///
/// Requires that the type has implemented the [`FromAttr`](trait@crate::FromAttr) and [`AttributeIdent`](crate::AttributeIdent) traits.
///
/// Generally used for parsing nested attributes.
///
/// # Example
///
/// ```rust
/// use from_attr::{FromAttr, convert_parsed_from_meta_list};
/// use syn::parse_quote;
///
/// #[derive(FromAttr, PartialEq, Eq, Debug)]
/// #[attribute(idents = [inner])]
/// struct Inner {
///     a: usize,
/// }
///
/// convert_parsed_from_meta_list!(Inner);
///
/// #[derive(FromAttr, PartialEq, Eq, Debug)]
/// #[attribute(idents = [outer])]
/// struct Outer {
///     a: usize,
///     b: Inner,
/// }
///
/// let attrs = [parse_quote!(#[outer(a = 1, b = inner(a = 10))])];
///
/// assert_eq!(
///     Outer::from_attributes(&attrs).unwrap().unwrap().value,
///     Outer {
///         a: 1,
///         b: Inner { a: 10 }
///     }
/// );
/// ```
#[macro_export]
macro_rules! convert_parsed_from_meta_list {
    ($ty:ty) => {
        impl $crate::ConvertParsed for $ty {
            type Type = $crate::__internal::syn::MetaList;

            fn convert(
                path_value: $crate::PathValue<Self::Type>,
            ) -> $crate::__internal::syn::Result<Self> {
                match <Self as $crate::FromAttr>::from_meta_list(&path_value.value)? {
                    Some(a) => Ok(a),
                    None => {
                        let idents = <Self as $crate::AttributeIdent>::IDENTS
                            .iter()
                            .map(|ident| format!("`{}`", *ident))
                            .collect::<::std::vec::Vec<_>>()
                            .join(", ");

                        Err($crate::__internal::syn::Error::new(
                            path_value.path,
                            format!("expected idents: {}", idents),
                        ))
                    }
                }
            }
        }
    };
}
