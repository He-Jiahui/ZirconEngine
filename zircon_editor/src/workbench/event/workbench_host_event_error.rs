use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum WorkbenchHostEventError {
    #[error("unsupported workbench binding payload")]
    UnsupportedPayload,
    #[error("unknown menu action id {0}")]
    UnknownMenuAction(String),
}
