use serde::Serialize;

use super::text::{canonical::canonicalize_value, document::TextDocument, envelope::TextEnvelope};
use super::{Format, PayloadHeader, VersionedSchema, WriteError};

/// Encodes a payload with the current schema header and canonical text rules.
pub fn write_versioned<T>(value: &T, format: Format) -> Result<Vec<u8>, WriteError>
where
    T: VersionedSchema + Serialize,
{
    match format {
        Format::Text => write_versioned_text(value).map(String::into_bytes),
        Format::Binary => Err(WriteError::UnsupportedFormat { format }),
    }
}

/// Encodes canonical, pretty JSON with a single trailing newline.
pub fn write_versioned_text<T>(value: &T) -> Result<String, WriteError>
where
    T: VersionedSchema + Serialize,
{
    let payload = serde_json::to_value(value).map_err(|source| WriteError::PayloadEncode {
        schema_id: T::SCHEMA.as_str().to_string(),
        schema_version: T::VERSION,
        source,
    })?;
    let document = TextDocument {
        envelope: TextEnvelope {
            header: PayloadHeader {
                schema_id: T::SCHEMA.clone(),
                schema_version: T::VERSION,
            },
            payload,
        },
    };
    let canonical = canonicalize_value(serde_json::to_value(document).map_err(|source| {
        WriteError::TextEncode {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            source,
        }
    })?);
    let mut encoded =
        serde_json::to_string_pretty(&canonical).map_err(|source| WriteError::TextEncode {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            source,
        })?;
    encoded.push('\n');
    Ok(encoded)
}
