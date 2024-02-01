#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FlagOrValue<T> {
    #[default]
    None,
    Flag,
    Value(T),
}
