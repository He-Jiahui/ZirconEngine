use thiserror::Error;

/// Typed failure while encoding a current-version payload.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("schema {schema_id} version {schema_version} contains non-finite float {value}")]
    NonFiniteFloat {
        schema_id: String,
        schema_version: u32,
        value: f64,
    },
    #[error("schema {schema_id} version {schema_version} payload validation failed: {reason}")]
    PayloadValidation {
        schema_id: String,
        schema_version: u32,
        reason: String,
    },
    #[error("schema {schema_id} version {schema_version} payload encode failed: {source}")]
    PayloadEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    #[error("schema {schema_id} version {schema_version} binary encode failed: {source}")]
    BinaryEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: bincode::Error,
    },
    #[error(
        "schema {schema_id} version {schema_version} binary body exceeds the {max}-byte wire limit"
    )]
    BinaryPayloadTooLarge {
        schema_id: String,
        schema_version: u32,
        max: usize,
    },
    #[error("schema {schema_id} version {schema_version} binary value is invalid: {reason}")]
    InvalidBinaryPayload {
        schema_id: String,
        schema_version: u32,
        reason: String,
    },
    #[error("schema {schema_id} version {schema_version} text encode failed: {source}")]
    TextEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
}
