use thiserror::Error;

/// Rejection emitted when a stream cannot produce another reproducible draw.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RandomStreamError {
    #[error("random stream draw index is exhausted")]
    DrawIndexExhausted,
}
