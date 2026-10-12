use thiserror::Error;

/// Rejects an activation operation identity whose nonce cannot distinguish it from an invalid wire value.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectActivationOperationIdError {
    #[error("project activation operation nonce must not be nil")]
    NilNonce,
}
