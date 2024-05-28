#![doc = include_str!("./docs/lib.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod attribute_ident;
mod attrs_value;
mod convert_parsed;
mod flag_or_type;
mod flag_or_value;
mod from_attr;
mod macros;
mod map;
mod parse_meta;
mod path_value;

#[doc(inline)]
#[doc = include_str!("./docs/from_attr.md")]
pub use from_attr_macro::FromAttr;
#[doc(inline)]
#[doc = include_str!("./docs/from_ident.md")]
pub use from_attr_macro::FromIdent;

pub use self::{
    attribute_ident::AttributeIdent, attrs_value::AttrsValue, convert_parsed::ConvertParsed,
    flag_or_value::FlagOrValue, from_attr::FromAttr, map::Map, parse_meta::ParseMeta,
    path_value::PathValue,
};

#[doc(hidden)]
pub mod __internal {
    pub use proc_macro2;
    pub use syn;
}
