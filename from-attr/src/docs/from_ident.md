Generate an implementation of [`ConvertParsed<Type = syn::Ident>`](crate::ConvertParsed) trait.

## Example

```rust
use from_attr::{FromIdent, FromAttr};
use syn::parse_quote;

#[derive(FromIdent, PartialEq, Eq, Debug)]
enum Enum {
    A,
    B,
    C,
}

#[derive(FromAttr, PartialEq, Eq, Debug)]
#[attribute(idents = [test])]
struct Test {
    a: Enum,
}

let attrs = [parse_quote!(#[test(a = A)])];

assert_eq!(
    Test::from_attributes(&attrs).unwrap().unwrap().value,
    Test { a: Enum::A }
);
```
