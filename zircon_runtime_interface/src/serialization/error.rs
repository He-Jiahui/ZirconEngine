use thiserror::Error;

use super::{Format, MigrateError};

/// Typed load failure for a versioned payload.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("serialization format {format:?} is not available in this milestone")]
    UnsupportedFormat { format: Format },
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
