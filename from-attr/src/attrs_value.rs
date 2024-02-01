/// Data structure to hold instance implemented [`FromAttr`](trait@crate::FromAttr) trait and parsed attributes.
#[derive(Debug)]
pub struct AttrsValue<A, V> {
    /// The parsed attributes.
    pub attrs: Vec<A>,
    /// The result parsed from the attributes.
    pub value: V,
}
