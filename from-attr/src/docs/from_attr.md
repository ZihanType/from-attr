Generate an implementation of [`FromAttr`](trait@crate::FromAttr) trait.

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

assert_eq!(attrs.len(), 1);
let test = Test::from_attributes(&attrs).unwrap().unwrap().value;
assert_eq!(attrs.len(), 1);

assert_eq!(test.a.value(), "a");
assert_eq!(test.b.unwrap(), "b");
assert!(matches!(test.c, Type::Tuple(_)));
assert!(matches!(test.d, Expr::If(_)));
assert!(test.e.len() == 2);
assert!(test.f);
assert!(!test.g);
```

## Attribute arguments

### `#[attribute]`: used on `struct`

| Name   | Type              | Example           | Optional | Default | Description              |
| ------ | ----------------- | ----------------- | -------- | ------- | ------------------------ |
| idents | `Vec<syn::Ident>` | `idents = [test]` | No       | -       | Idents of the attribute. |

### `#[attribute]`: used on `field`

| Name      | Type                     | Example                             | Optional | Default             | Flag                                | Description          |
| --------- | ------------------------ | ----------------------------------- | -------- | ------------------- | ----------------------------------- | -------------------- |
| rename    | `Option<String>`         | `rename = "type"`                   | Yes      | `None`              | -                                   | Rename the field.    |
| default   | `FlagOrValue<syn::Expr>` | `default` <br><br> `default = true` | Yes      | `FlagOrValue::None` | `core::default::Default::default()` | Default field value. |
| conflicts | `Vec<syn::Ident>`        | `conflicts = [a, b, c]`             | Yes      | `Vec::new()`        | -                                   | Conflicts fields.    |
