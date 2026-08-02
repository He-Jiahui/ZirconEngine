use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResourceIoError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("resource io failed: {0}")]
    Io(String),
    #[error("resource scheme is read only: {0}")]
    ReadOnly(String),
}
