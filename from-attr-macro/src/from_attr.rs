use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::format_ident;
use quote_use::quote_use;
use syn::{
    parse_quote, punctuated::Punctuated, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field,
    Fields, FieldsNamed, Ident, ImplGenerics, Token, TypeGenerics, Visibility, WhereClause,
};

use crate::{field_attr::FieldAttr, flag_or_value::FlagOrValue, struct_attr::StructAttr};

pub(crate) fn generate(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput {
        attrs: input_attrs,
        vis,
        ident: input_ident,
        generics,
        data,
    } = input;

    let Some(StructAttr {
        idents: attr_idents,
    }) = StructAttr::parse_attrs(&input_attrs)?
    else {
        return Err(syn::Error::new(
            input_ident.span(),
            "missing `#[attribute(...)]` attribute",
        ));
    };

    let parser_struct_ident = format_ident!("{input_ident}_Parser");

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                if named.is_empty() {
                    generate_unit(
                        true,
                        vis,
                        parser_struct_ident,
                        input_ident,
                        impl_generics,
                        ty_generics,
                        where_clause,
                        attr_idents,
                    )
                } else {
                    generate_named_fields(
                        named,
                        vis,
                        parser_struct_ident,
                        input_ident,
                        impl_generics,
                        ty_generics,
                        where_clause,
                        attr_idents,
                    )
                }
            }
            Fields::Unit => generate_unit(
                false,
                vis,
                parser_struct_ident,
                input_ident,
                impl_generics,
                ty_generics,
                where_clause,
                attr_idents,
            ),
            Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                fields,
                "expected a struct with named fields or unit struct",
            )),
        },
        Data::Enum(DataEnum { enum_token, .. }) => Err(syn::Error::new(
            enum_token.span,
            "expected a struct with named fields or unit struct",
        )),
        Data::Union(DataUnion { union_token, .. }) => Err(syn::Error::new(
            union_token.span,
            "expected a struct with named fields or unit struct",
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_unit<'a>(
    has_brace_token: bool,
    vis: Visibility,
    parser_struct_ident: Ident,
    struct_ident: Ident,
    impl_generics: ImplGenerics<'a>,
    ty_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
    attr_idents: Vec<String>,
) -> syn::Result<TokenStream> {
    let construct_self = if has_brace_token {
        quote_use! {
            Self {}
        }
    } else {
        quote_use! {
            Self
        }
    };

    let expand = quote_use! {
        # use core::default::Default;
        # use from_attr::__internal::syn::{self, Token, meta::ParseNestedMeta};
        # use from_attr::__internal::proc_macro2::Span;
        # use from_attr::{AttributeIdent, FromAttr, ParseMeta};

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(Default)]
        #vis struct #parser_struct_ident;

        #[automatically_derived]
        impl ParseMeta for #parser_struct_ident {
            fn is_unit() -> bool {
                true
            }

            fn parse_meta(&mut self, _: ParseNestedMeta) -> syn::Result<()> {
                Ok(())
            }
        }

        #[automatically_derived]
        impl #impl_generics AttributeIdent for #struct_ident #ty_generics #where_clause {
            const IDENTS: &'static [&'static str] = &[#(#attr_idents),*];
        }

        #[automatically_derived]
        impl #impl_generics FromAttr for #struct_ident #ty_generics #where_clause {
            type Parser = #parser_struct_ident;

            fn from_parser(parser: Self::Parser, _: &[Span]) -> syn::Result<Self> {
                Ok(#construct_self)
            }
        }
    };

    Ok(expand)
}

#[allow(clippy::too_many_arguments)]
fn generate_named_fields<'a>(
    named: Punctuated<Field, Token![,]>,
    vis: Visibility,
    parser_struct_ident: Ident,
    struct_ident: Ident,
    impl_generics: ImplGenerics<'a>,
    ty_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
    attr_idents: Vec<String>,
) -> syn::Result<TokenStream> {
    let mut conflicts: HashSet<(Ident, Ident)> = HashSet::new();
    let mut fields: Punctuated<TokenStream, Token![,]> = Punctuated::new();
    let mut parse_arguments: Vec<TokenStream> = Vec::new();
    let mut field_values: Punctuated<TokenStream, Token![,]> = Punctuated::new();
    let mut defined_fields: HashMap<Ident, String> = HashMap::new();

    for field in named.into_iter() {
        handle_single_field(
            field,
            &mut conflicts,
            &mut fields,
            &mut parse_arguments,
            &mut field_values,
            &mut defined_fields,
        )?;
    }

    let conflicts_check = conflicts
        .into_iter()
        .flat_map(|(a, b)| -> Option<_> {
            let a_rename = defined_fields.get(&a)?;
            let b_rename = defined_fields.get(&b)?;
            let a_to_b_err_msg =
                format!("`{a_rename}` conflicts with mutually exclusive `{b_rename}`");
            let b_to_a_err_msg =
                format!("`{b_rename}` conflicts with mutually exclusive `{a_rename}`");

            Some(quote_use! {
                # use from_attr::__internal::syn;

                if let (Some(a), Some(b)) = (&parser.#a, &parser.#b) {
                    if let Some(joined_span) = a.path.join(b.path) {
                        return Err(syn::Error::new(joined_span, #a_to_b_err_msg));
                    } else {
                        let mut err = syn::Error::new(a.path, #a_to_b_err_msg);
                        err.combine(syn::Error::new(b.path, #b_to_a_err_msg));
                        return Err(err);
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    let unknown_field_msg = match defined_fields.len() {
        1 => format!(
            "expected field: `{}`",
            defined_fields
                .values()
                .next()
                .expect("expected at least one field")
        ),
        _ => {
            let fields = defined_fields
                .values()
                .map(|a| format!("`{}`", a))
                .collect::<Vec<_>>()
                .join(", ");

            format!("expected fields: {}", fields)
        }
    };

    let expand = quote_use! {
        # use core::default::Default;
        # use from_attr::__internal::syn::{self, Token, meta::ParseNestedMeta, spanned::Spanned};
        # use from_attr::__internal::proc_macro2::Span;
        # use from_attr::{AttributeIdent, FromAttr, ParseMeta};

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(Default)]
        #vis struct #parser_struct_ident {
            #fields
        }

        #[automatically_derived]
        impl ParseMeta for #parser_struct_ident {
            fn is_unit() -> bool {
                false
            }

            fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
                let meta_path = &meta.path;
                let meta_path_span = Spanned::span(meta_path);

                let next_token_is_not_eq = !meta.input.peek(Token![=]);

                #(#parse_arguments)*

                Err(meta.error(#unknown_field_msg))
            }
        }

        #[automatically_derived]
        impl #impl_generics AttributeIdent for #struct_ident #ty_generics #where_clause {
            const IDENTS: &'static [&'static str] = &[#(#attr_idents),*];
        }

        #[automatically_derived]
        impl #impl_generics FromAttr for #struct_ident #ty_generics #where_clause {
            type Parser = #parser_struct_ident;

            fn from_parser(parser: Self::Parser, spans: &[Span]) -> syn::Result<Self> {
                #(#conflicts_check)*
                Ok(Self {#field_values})
            }
        }
    };

    Ok(expand)
}

fn handle_single_field<'a>(
    field: Field,
    conflicts: &'a mut HashSet<(Ident, Ident)>,
    fields: &'a mut Punctuated<TokenStream, Token![,]>,
    parse_arguments: &'a mut Vec<TokenStream>,
    field_values: &'a mut Punctuated<TokenStream, Token![,]>,
    defined_fields: &'a mut HashMap<Ident, String>,
) -> syn::Result<()> {
    let Field {
        attrs, ident, ty, ..
    } = field;

    let FieldAttr {
        rename,
        default,
        conflicts: current_conflicts,
    } = FieldAttr::parse_attrs(&attrs)?.unwrap_or_default();

    let field_ident = ident.expect("expected a named field");
    let rename_field_ident = rename.unwrap_or_else(|| field_ident.to_string());

    for conflict_field_ident in current_conflicts {
        if conflict_field_ident < field_ident {
            conflicts.insert((conflict_field_ident, field_ident.clone()));
        } else {
            conflicts.insert((field_ident.clone(), conflict_field_ident));
        }
    }

    fields.push(quote_use! {
        # use core::option::Option;
        # use from_attr::{ConvertParsed, PathValue};

        #field_ident: Option<PathValue<<#ty as ConvertParsed>::Type>>
    });

    parse_arguments.push(quote_use! {
        # use core::option::Option::Some;
        # use std::string::ToString;
        # use std::format;
        # use from_attr::{ConvertParsed, PathValue};
        # use from_attr::merge;
        # use from_attr::__internal::syn::{self, Token};

        if meta_path.is_ident(#rename_field_ident) {
            if self.#field_ident.is_some() {
                return Err(meta.error(concat!("duplicate `", #rename_field_ident, "` argument")));
            }

            let value = if let Some(Some(value)) = next_token_is_not_eq.then(|| <#ty as ConvertParsed>::flag()) {
                value
            } else {
                meta.value()?.parse()?
            };

            self.#field_ident = Some(PathValue { path: meta_path_span, value });
            return Ok(());
        }
    });

    let missing_field_msg = format!(
        "missing `{i}` field, try `{i} = ...`",
        i = rename_field_ident
    );

    let missing_flag_msg = format!(
        "missing `{i}` flag, try `{i}` or `{i} = ...`",
        i = rename_field_ident
    );

    let default = match default {
        FlagOrValue::None => None,
        FlagOrValue::Flag => Some(parse_quote!(::core::default::Default::default())),
        FlagOrValue::Value(expr) => Some(expr),
    };

    field_values.push(match default {
        None => quote_use! {
            # use core::option::Option::{Some, None};
            # use core::result::Result::{Ok, Err};
            # use from_attr::ConvertParsed;
            # use from_attr::__internal::syn;

            #field_ident: match parser.#field_ident.map(ConvertParsed::convert) {
                Some(Ok(#field_ident)) => #field_ident,
                Some(Err(err)) => return Err(err),
                None => {
                    if let Some(#field_ident) = <#ty as ConvertParsed>::default() {
                        #field_ident
                    } else {
                        let err = spans
                            .into_iter()
                            .map(|span| {
                                if <#ty as ConvertParsed>::flag().is_some() {
                                    syn::Error::new(*span, #missing_flag_msg)
                                } else {
                                    syn::Error::new(*span, #missing_field_msg)
                                }
                            })
                            .reduce(|mut a, b| {
                                a.combine(b);
                                a
                            })
                            .expect("`spans` is not empty");

                        return Err(err);
                    }
                }
            }
        },
        Some(default) => quote_use! {
            # use core::result::Result::Ok;
            # use from_attr::ConvertParsed;

            #field_ident: parser.#field_ident
                .map(ConvertParsed::convert)
                .unwrap_or_else(|| Ok(#default))?
        },
    });

    defined_fields.insert(field_ident, rename_field_ident);

    Ok(())
}
