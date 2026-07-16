use thiserror::Error;

use super::MigrateError;

/// Typed load failure for a versioned payload.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("malformed versioned text: {source}")]
    MalformedText {
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid versioned payload envelope: {source}")]
    InvalidEnvelope {
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "binary payload header is truncated: expected at least {expected} bytes, found {found}"
    )]
    BinaryHeaderTruncated { expected: usize, found: usize },
    #[error("binary payload magic is invalid: found {found:?}")]
    BinaryMagicMismatch { found: [u8; 8] },
    #[error("binary wire version {found} is unsupported; supported version is {supported}")]
    UnsupportedBinaryWireVersion { found: u16, supported: u16 },
    #[error("binary payload body is too large: maximum {max} bytes, found {found}")]
    BinaryPayloadTooLarge { max: usize, found: usize },
    #[error("malformed versioned binary payload: {source}")]
    MalformedBinary {
        #[source]
        source: bincode::Error,
    },
    #[error("invalid binary value-domain payload: {reason}")]
    InvalidBinaryPayload { reason: String },
    #[error("expected schema {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },
    #[error("schema {schema_id} version {found} is newer than supported version {supported}")]
    FutureVersion {
        schema_id: String,
        found: u32,
        supported: u32,
    },
    #[error("schema {schema_id} version {schema_version} payload decode failed: {source}")]
    PayloadDecode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Migration(#[from] MigrateError),
}
