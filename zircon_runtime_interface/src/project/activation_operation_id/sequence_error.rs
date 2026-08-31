use thiserror::Error;

/// Rejects the reserved zero operation sequence.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectActivationOperationSequenceError {
    #[error("project activation operation sequence must not be zero")]
    Zero,
}
