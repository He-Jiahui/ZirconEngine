use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EditorBindingDispatchError {
    #[error("unsupported editor binding payload")]
    UnsupportedPayload,
    #[error("invalid subject path {0}")]
    InvalidSubjectPath(String),
    #[error("unsupported inspector field {0}")]
    UnsupportedInspectorField(String),
    #[error("invalid inspector field value for {field_id}")]
    InvalidInspectorFieldValue { field_id: String },
    #[error("unknown drawer slot {0}")]
    UnknownDrawerSlot(String),
    #[error("unknown drawer mode {0}")]
    UnknownDrawerMode(String),
    #[error("state mutation failed: {0}")]
    StateMutation(String),
}
