use thiserror::Error;
use zircon_runtime_interface::reflect::ReflectValueValidationError;
use zircon_runtime_interface::serialization::{LoadError, WriteError};

/// Typed persistence failure for versioned reflected JSON.
#[derive(Debug, Error)]
pub enum ReflectedJsonError {
    #[error("reflected JSON value rejected: {0}")]
    Value(#[from] ReflectValueValidationError),
    #[error("reflected JSON load failed: {0}")]
    Load(#[from] LoadError),
    #[error("reflected JSON write failed: {0}")]
    Write(#[from] WriteError),
}
