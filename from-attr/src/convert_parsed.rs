use from_attr_core::Array;
use proc_macro2::{Group, Literal, Punct, TokenTree};
use syn::{
    parse_quote,
    token::{
        Abstract, And, AndAnd, AndEq, As, Async, At, Auto, Await, Become, Break, Caret, CaretEq,
        Colon, Comma, Const, Continue, Crate, Do, Dollar, Dot, DotDot, DotDotDot, DotDotEq, Dyn,
        Else, Enum, Eq, EqEq, Extern, FatArrow, Final, Fn, For, Ge, Gt, If, Impl, In, LArrow, Le,
        Let, Loop, Lt, Match, Minus, MinusEq, Mod, Move, Mut, Ne, Not, Or, OrEq, OrOr, Override,
        PathSep, Percent, PercentEq, Plus, PlusEq, Pound, Priv, Pub, Question, RArrow, Ref, Return,
        SelfType, SelfValue, Semi, Shl, ShlEq, Shr, ShrEq, Slash, SlashEq, Star, StarEq, Static,
        Struct, Super, Tilde, Trait, Try, Typeof, Underscore, Union, Unsafe, Unsized, Use, Virtual,
        Where, While, Yield,
    },
    Abi, AngleBracketedGenericArguments, BareFnArg, BinOp, BoundLifetimes, ConstParam, Constraint,
    DeriveInput, Expr, ExprArray, ExprAssign, ExprAsync, ExprBinary, ExprBlock, ExprBreak,
    ExprCall, ExprCast, ExprClosure, ExprContinue, ExprField, ExprForLoop, ExprIf, ExprIndex,
    ExprLet, ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall, ExprParen, ExprPath,
    ExprRange, ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry, ExprTryBlock, ExprTuple,
    ExprUnary, ExprUnsafe, ExprWhile, ExprYield, FieldValue, FieldsNamed, FieldsUnnamed,
    GenericArgument, GenericParam, Generics, Ident, Index, Lifetime, Lit, LitBool, LitByteStr,
    LitChar, LitFloat, LitInt, LitStr, Member, Meta, MetaList, MetaNameValue,
    ParenthesizedGenericArguments, Path, PathSegment, ReturnType, TraitBound, TraitBoundModifier,
    Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro, TypeNever,
    TypeParam, TypeParamBound, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice,
    TypeTraitObject, TypeTuple, UnOp, Variant, Visibility, WhereClause, WherePredicate,
};

use crate::PathValue;

/// Helper trait to convert syn types implementing [`Parse`](syn::parse::Parse) like
/// [`LitStr`](struct@syn::LitStr) to rust types like [`String`]
///
/// You probably don't need to implement this trait, as most syn types like
/// [`LitStr`](struct@syn::LitStr) and [`Type`] or that have a direct equivalent in
/// those like [`String`], [`char`] or [`f32`] are already implemented. A
/// special treatment have [`Vec`] which are parsed with the syntax `[a, b, c]`.
pub trait ConvertParsed: Sized {
    /// The type this can be converted from
    type Type;

    /// Convert the [`PathValue<Self::Type>`](crate::PathValue) to self.
    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self>;

    /// Returns the value when this type is not specified.
    fn default() -> Option<Self> {
        None
    }

    /// Returns the value when this type is specified as flag.
    fn flag() -> Option<Self::Type> {
        None
    }
}

impl<T> ConvertParsed for Option<T>
where
    T: ConvertParsed,
{
    type Type = T::Type;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        Ok(Some(T::convert(path_value)?))
    }

    fn default() -> Option<Self> {
        Some(None)
    }

    fn flag() -> Option<Self::Type> {
        T::flag()
    }
}

impl<T> ConvertParsed for Vec<T>
where
    T: ConvertParsed,
{
    type Type = Array<T::Type>;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        let PathValue { path, value } = path_value;

        let mut elems = Vec::new();
        let mut errors = Vec::new();

        value
            .elems
            .into_iter()
            .for_each(|value| match T::convert(PathValue { path, value }) {
                Ok(o) => elems.push(o),
                Err(e) => errors.push(e),
            });

        match errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            Some(e) => Err(e),
            None => Ok(elems),
        }
    }

    fn default() -> Option<Self> {
        Some(Vec::new())
    }
}

impl ConvertParsed for bool {
    type Type = LitBool;

    fn convert(path_value: PathValue<Self::Type>) -> syn::Result<Self> {
        Ok(path_value.value.value)
    }

    fn default() -> Option<Self> {
        Some(false)
    }

    fn flag() -> Option<Self::Type> {
        Some(parse_quote!(true))
    }
}

macro_rules! get_value {
    ($from:path => $to:path > $get:path) => {
        impl ConvertParsed for $to {
            type Type = $from;

            fn convert(path_value: PathValue<$from>) -> syn::Result<$to> {
                Ok($get(&path_value.value))
            }
        }
    };
}

get_value!(LitStr => String > LitStr::value);
get_value!(LitChar => char > LitChar::value);

macro_rules! parse_value {
    ($from:path => $($to:path),+ > $parse:path) => {
        $(
            impl ConvertParsed for $to {
                type Type = $from;

                fn convert(path_value: PathValue<$from>) -> syn::Result<$to> {
                    $parse(&path_value.value)
                }
            }
        )*
    };
}

parse_value!(LitInt => u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize > LitInt::base10_parse);
parse_value!(LitFloat => f32, f64 > LitFloat::base10_parse);

macro_rules! convert_parsed {
    ($($type:ty),* $(,)?) => {
        $(
            impl ConvertParsed for $type {
                type Type = $type;

                fn convert(path_value: PathValue<Self>) -> syn::Result<Self> {
                    Ok(path_value.value)
                }
            }
        )*
    };
}

convert_parsed![
    Abi,
    Abstract,
    And,
    AndAnd,
    AndEq,
    AngleBracketedGenericArguments,
    As,
    Async,
    At,
    Auto,
    Await,
    BareFnArg,
    Become,
    BinOp,
    BoundLifetimes,
    Break,
    Caret,
    CaretEq,
    Colon,
    Comma,
    Const,
    ConstParam,
    Constraint,
    Continue,
    Crate,
    DeriveInput,
    Do,
    Dollar,
    Dot,
    DotDot,
    DotDotDot,
    DotDotEq,
    Dyn,
    Else,
    Enum,
    Eq,
    EqEq,
    Expr,
    ExprArray,
    ExprAssign,
    ExprAsync,
    ExprBinary,
    ExprBlock,
    ExprBreak,
    ExprCall,
    ExprCast,
    ExprClosure,
    ExprContinue,
    ExprField,
    ExprForLoop,
    ExprIf,
    ExprIndex,
    ExprLet,
    ExprLit,
    ExprLoop,
    ExprMacro,
    ExprMatch,
    ExprMethodCall,
    ExprParen,
    ExprPath,
    ExprRange,
    ExprReference,
    ExprRepeat,
    ExprReturn,
    ExprStruct,
    ExprTry,
    ExprTryBlock,
    ExprTuple,
    ExprUnary,
    ExprUnsafe,
    ExprWhile,
    ExprYield,
    Extern,
    FatArrow,
    FieldValue,
    FieldsNamed,
    FieldsUnnamed,
    Final,
    Fn,
    For,
    Ge,
    GenericArgument,
    GenericParam,
    Generics,
    Group,
    Gt,
    Ident,
    If,
    Impl,
    In,
    Index,
    LArrow,
    Le,
    Let,
    Lifetime,
    Lit,
    LitBool,
    LitByteStr,
    LitChar,
    LitFloat,
    LitInt,
    LitStr,
    Literal,
    Loop,
    Lt,
    Match,
    Member,
    Meta,
    MetaList,
    MetaNameValue,
    Minus,
    MinusEq,
    Mod,
    Move,
    Mut,
    Ne,
    Not,
    Or,
    OrEq,
    OrOr,
    Override,
    ParenthesizedGenericArguments,
    Path,
    PathSegment,
    PathSep,
    Percent,
    PercentEq,
    Plus,
    PlusEq,
    Pound,
    Priv,
    Pub,
    Punct,
    Question,
    RArrow,
    Ref,
    Return,
    ReturnType,
    SelfType,
    SelfValue,
    Semi,
    Shl,
    ShlEq,
    Shr,
    ShrEq,
    Slash,
    SlashEq,
    Star,
    StarEq,
    Static,
    Struct,
    Super,
    Tilde,
    TokenTree,
    Trait,
    TraitBound,
    TraitBoundModifier,
    Try,
    TypeArray,
    TypeBareFn,
    TypeGroup,
    TypeImplTrait,
    TypeInfer,
    TypeMacro,
    TypeNever,
    TypeParam,
    TypeParamBound,
    TypeParen,
    TypePath,
    TypePtr,
    TypeReference,
    TypeSlice,
    TypeTraitObject,
    TypeTuple,
    Typeof,
    UnOp,
    Underscore,
    Union,
    Unsafe,
    Unsized,
    Use,
    Variant,
    Virtual,
    Visibility,
    Where,
    WhereClause,
    WherePredicate,
    While,
    Yield,
    syn::Macro,
    syn::token::Box,
    syn::token::Default,
    syn::token::Macro,
    syn::token::Type,
    Type,
];

#[cfg_attr(docsrs, doc(cfg(feature = "syn-full")))]
#[cfg(feature = "syn-full")]
mod syn_full {
    use syn::{
        Arm, Block, File, FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic,
        ForeignItemType, ImplItem, ImplItemConst, ImplItemMacro, ImplItemType, Item, ItemConst,
        ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod,
        ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Label,
        Pat, RangeLimits, Receiver, Signature, Stmt, TraitItem, TraitItemConst, TraitItemMacro,
        TraitItemType, UseTree,
    };

    use super::*;

    convert_parsed![
        Block,
        File,
        FnArg,
        ForeignItem,
        ForeignItemFn,
        ForeignItemMacro,
        ForeignItemStatic,
        ForeignItemType,
        ImplItem,
        ImplItemConst,
        ImplItemMacro,
        ImplItemType,
        Item,
        ItemConst,
        ItemEnum,
        ItemExternCrate,
        ItemFn,
        ItemForeignMod,
        ItemImpl,
        ItemMacro,
        ItemMod,
        ItemStatic,
        ItemStruct,
        ItemTrait,
        ItemTraitAlias,
        ItemType,
        ItemUnion,
        ItemUse,
        Label,
        Pat,
        RangeLimits,
        Receiver,
        Signature,
        Stmt,
        TraitItem,
        TraitItemConst,
        TraitItemMacro,
        TraitItemType,
        UseTree,
        Arm,
    ];
}
