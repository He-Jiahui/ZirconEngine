use thiserror::Error;

use super::Format;

/// Typed failure while encoding a current-version payload.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("serialization format {format:?} is not available in this milestone")]
    UnsupportedFormat { format: Format },
    #[error("schema {schema_id} version {schema_version} payload encode failed: {source}")]
    PayloadEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    #[error("schema {schema_id} version {schema_version} text encode failed: {source}")]
    TextEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
}
