use from_attr::{convert_parsed_from_meta_list, FlagOrValue, FromAttr, FromIdent};
use quote::quote;
use syn::{parse_quote, Expr, LitStr, Type};

fn basic() {
    #[derive(FromAttr)]
    #[attribute(idents = [test])]
    struct Test {
        a: LitStr,
        b: String,
        c: Option<String>,
        d: Type,
        e: Expr,
        f: Vec<Type>,
        g: bool,
        h: bool,
        i: Option<Type>,
    }

    {
        let attrs = [
            parse_quote!(#[test(a = "hi", b = "ho", c = "xD", d = (), e = if true { "a" } else { "b" }, f = [(), Debug], g)]),
        ];

        assert_eq!(attrs.len(), 1);
        let test = Test::from_attributes(&attrs).unwrap().unwrap().value;
        assert_eq!(attrs.len(), 1);

        assert_eq!(test.a.value(), "hi");
        assert_eq!(test.b, "ho");
        assert_eq!(test.c, Some("xD".to_owned()));
        assert!(matches!(test.d, Type::Tuple(_)));
        assert!(matches!(test.e, Expr::If(_)));
        assert!(test.f.len() == 2);
        assert!(test.g);
        assert!(!test.h);
        assert!(test.i.is_none());
    }

    {
        let tokens = quote!(
            a = "hi",
            b = "ho",
            c = "xD",
            d = (),
            e = if true { "a" } else { "b" },
            f = [(), Debug],
            g
        );

        let test = Test::from_tokens(tokens).unwrap();

        assert_eq!(test.a.value(), "hi");
        assert_eq!(test.b, "ho");
        assert_eq!(test.c, Some("xD".to_owned()));
        assert!(matches!(test.d, Type::Tuple(_)));
        assert!(matches!(test.e, Expr::If(_)));
        assert!(test.f.len() == 2);
        assert!(test.g);
        assert!(!test.h);
        assert!(test.i.is_none());
    }

    {
        let mut attrs = vec![
            parse_quote!(#[something]),
            parse_quote!(#[test(a = "hi", b = "ho", c = "xD", d = (), e = if true { "a" } else { "b" }, f = [(), Debug], g)]),
            parse_quote!(#[another(smth)]),
        ];

        assert_eq!(attrs.len(), 3);
        let test = Test::remove_attributes(&mut attrs).unwrap().unwrap().value;
        assert_eq!(attrs.len(), 2);

        assert_eq!(test.a.value(), "hi");
        assert_eq!(test.b, "ho");
        assert_eq!(test.c, Some("xD".to_owned()));
        assert!(test.i.is_none());
        assert!(matches!(test.d, Type::Tuple(_)));
        assert!(matches!(test.e, Expr::If(_)));
        assert!(test.f.len() == 2);
        assert!(test.g);
        assert!(!test.h);
        assert!(test.i.is_none());
    }
}

fn default() {
    #[derive(FromAttr, Debug, PartialEq)]
    #[attribute(idents = [test])]
    struct Test {
        #[attribute(default)]
        hi: f32,
        #[attribute(default = 10)]
        ho: usize,
    }

    let attrs = [parse_quote!(#[test()])];

    assert_eq!(
        Test::from_attributes(&attrs).unwrap().unwrap().value,
        Test { hi: 0., ho: 10 }
    );
}

fn empty() {
    #[derive(FromAttr, PartialEq, Eq, Debug)]
    #[attribute(idents = [a])]
    struct Test;

    let attrs = [parse_quote!(#[a])];

    assert_eq!(Test::from_attributes(&attrs).unwrap().unwrap().value, Test);
}

fn rename() {
    #[allow(dead_code)]
    #[derive(FromAttr)]
    #[attribute(idents = [test])]
    struct Test {
        #[attribute(rename = "type")]
        ty: Type,
    }

    let tokens = quote!(type = u32);

    assert!(Test::from_tokens(tokens).is_ok());
}

fn nested() {
    #[derive(FromAttr, PartialEq, Eq, Debug)]
    #[attribute(idents = [inner])]
    struct Inner {
        a: usize,
    }

    convert_parsed_from_meta_list!(Inner);

    #[derive(FromAttr, PartialEq, Eq, Debug)]
    #[attribute(idents = [outer])]
    struct Outer {
        a: usize,
        b: Inner,
    }

    let attrs = [parse_quote!(#[outer(a = 1, b = inner(a = 10))])];

    assert_eq!(
        Outer::from_attributes(&attrs).unwrap().unwrap().value,
        Outer {
            a: 1,
            b: Inner { a: 10 }
        }
    );
}

fn conflicts() {
    #[allow(dead_code)]
    #[derive(FromAttr)]
    #[attribute(idents = [test])]
    struct Test {
        #[attribute(conflicts = [b])]
        a: usize,
        b: usize,
    }

    let attrs = [parse_quote!(#[test(a = 1, b = 2)])];

    assert!(Test::from_attributes(&attrs).is_err());
}

fn flag() {
    #[derive(FromAttr)]
    #[attribute(idents = [test])]
    struct Test {
        a: FlagOrValue<usize>,
        b: FlagOrValue<usize>,
        c: FlagOrValue<usize>,
    }

    let attrs = [parse_quote!(#[test(/* not a, */ b /* flag b */, c = 10 /* value c */)])];

    let test = Test::from_attributes(&attrs).unwrap().unwrap().value;
    assert!(matches!(test.a, FlagOrValue::None));
    assert!(matches!(test.b, FlagOrValue::Flag { .. }));
    assert!(matches!(test.c, FlagOrValue::Value { value: 10, .. }));
}

fn r#enum() {
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
}

fn main() {
    basic();
    default();
    empty();
    rename();
    nested();
    conflicts();
    flag();
    r#enum();
}
