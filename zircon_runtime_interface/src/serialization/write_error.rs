use std::io;

use thiserror::Error;

/// Typed failure while streaming canonical JSON to an arbitrary byte sink.
#[derive(Debug, Error)]
pub enum CanonicalTextWriteError {
    #[error("canonical text contains non-finite float {value}")]
    NonFinite { value: f64 },
    #[error("canonical text payload validation failed: {reason}")]
    PayloadValidation { reason: String },
    #[error("canonical text payload encode failed: {reason}")]
    PayloadEncode { reason: String },
    #[error("canonical text exceeds the {max}-byte wire limit (found at least {found} bytes)")]
    OutputTooLarge { max: usize, found: usize },
    #[error("canonical text {operation} failed: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
}

impl serde::ser::Error for CanonicalTextWriteError {
    fn custom<T>(message: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::PayloadValidation {
            reason: message.to_string(),
        }
    }
}

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
    #[error(
        "schema {schema_id} version {schema_version} text document exceeds the {max}-byte wire limit (found at least {found} bytes)"
    )]
    TextDocumentTooLarge {
        schema_id: String,
        schema_version: u32,
        max: usize,
        found: usize,
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
    #[error("schema {schema_id} version {schema_version} text sink {operation} failed: {source}")]
    TextWrite {
        schema_id: String,
        schema_version: u32,
        operation: &'static str,
        #[source]
        source: io::Error,
    },
}
