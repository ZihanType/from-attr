use syn::Path;

/// Helper trait for holding idents.
pub trait AttributeIdent {
    /// Holds the idents.
    const IDENTS: &'static [&'static str];

    /// Check if path exists in the idents.
    fn is_ident(path: &Path) -> bool {
        Self::IDENTS.iter().any(|ident| path.is_ident(ident))
    }
}
