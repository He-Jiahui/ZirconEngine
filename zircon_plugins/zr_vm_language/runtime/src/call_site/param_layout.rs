use std::sync::Arc;

/// Immutable reflected field layout compiled beside a dense call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamLayout {
    /// Shared reflection value type path expected by reads and writes.
    pub value_type_path: Arc<str>,
    /// Whether the field accepts writes through the reflection bridge.
    pub editable: bool,
}

impl ParamLayout {
    pub(crate) fn new(value_type_path: impl Into<Arc<str>>, editable: bool) -> Self {
        Self {
            value_type_path: value_type_path.into(),
            editable,
        }
    }
}
