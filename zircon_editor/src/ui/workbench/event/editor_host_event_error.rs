use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorHostEventError {
    #[error("unsupported editor host binding payload")]
    UnsupportedPayload,
    #[error("unknown menu action id {0}")]
    UnknownMenuAction(String),
}
