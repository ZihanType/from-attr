# from-attr

[![Crates.io version](https://img.shields.io/crates/v/from-attr.svg?style=flat-square)](https://crates.io/crates/from-attr)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/from-attr)

This crate provides some derive macros for parsing values from attributes.

## Inspired

This crate inspired by [attribute-derive](https://github.com/ModProg/attribute-derive).

## Example

```rust
use from_attr::FromAttr;
use syn::{parse_quote, Expr, LitStr, Type};

#[derive(FromAttr)]
#[attribute(idents = [test])]
struct Test {
    a: LitStr,
    b: Option<String>,
    c: Type,
    d: Expr,
    e: Vec<Type>,
    f: bool,
    g: bool,
}

let attrs = [
    parse_quote!(#[test(a = "a", b = "b", c = (), d = if true { "a" } else { "b" }, e = [(), Debug], f)]),
];

let test = Test::from_attributes(&attrs).unwrap().unwrap().value;

assert_eq!(test.a.value(), "a");
assert_eq!(test.b.unwrap(), "b");
assert!(matches!(test.c, Type::Tuple(_)));
assert!(matches!(test.d, Expr::If(_)));
assert!(test.e.len() == 2);
assert!(test.f);
assert!(!test.g);
```

More examples can be found in the [examples](./examples/) directories.
