use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamLayout {
    pub value_type_path: Arc<str>,
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
