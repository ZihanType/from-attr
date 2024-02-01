use proc_macro2::TokenStream;
use quote::quote;
use quote_use::quote_use;
use syn::{Data, DataEnum, DataStruct, DataUnion, DeriveInput, Fields, Variant};

pub(crate) fn generate(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput { ident, data, .. } = input;

    let (brace_token, variants) = match data {
        Data::Enum(DataEnum {
            brace_token,
            variants,
            ..
        }) => (brace_token, variants),
        Data::Struct(DataStruct { struct_token, .. }) => {
            return Err(syn::Error::new(
                struct_token.span,
                "`FromIdent` can only be derived for enums",
            ))
        }
        Data::Union(DataUnion { union_token, .. }) => {
            return Err(syn::Error::new(
                union_token.span,
                "`FromIdent` can only be derived for enums",
            ))
        }
    };

    if variants.is_empty() {
        return Err(syn::Error::new(
            brace_token.span.join(),
            "`FromIdent` cannot be derived for empty enums",
        ));
    }

    let mut errors = Vec::new();
    let mut match_arms = Vec::new();
    let mut idents = String::new();

    for Variant { ident, fields, .. } in variants {
        match fields {
            Fields::Unit => {}
            Fields::Named(_) | Fields::Unnamed(_) => {
                let e = syn::Error::new_spanned(
                    fields,
                    "`FromIdent` can only be derived for enums with unit fields",
                );

                errors.push(e);
                continue;
            }
        }

        let ident_string = ident.to_string();
        let uppercase = ident_string.to_uppercase();
        let arm = quote! (#uppercase => Self::#ident,);
        match_arms.push(arm);

        idents.push('`');
        idents.push_str(&ident_string);
        idents.push('`');
        idents.push_str(", ");
    }

    if let Some(e) = errors.into_iter().reduce(|mut a, b| {
        a.combine(b);
        a
    }) {
        return Err(e);
    }

    idents.pop(); // remove whitespace
    idents.pop(); // remove comma

    let expand = quote_use! {
        # use std::string::ToString;
        # use from_attr::{ConvertParsed, PathValue};
        # use from_attr::__internal::syn::{self, Ident};

        impl ConvertParsed for #ident {
            type Type = Ident;

            fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
                let ident = path_value.value;

                let this = match ToString::to_string(&ident).to_uppercase().as_str() {
                    #(#match_arms)*
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("invalid ident: `{}`, valid idents: {}", ident, #idents),
                        ))
                    }
                };

                Ok(this)
            }
        }
    };

    Ok(expand)
}
