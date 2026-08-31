use std::io;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::binary::{
    decode_binary_current, decode_binary_header, decode_binary_payload, DirectBinaryDecodeError,
};
use super::text::read::{inspect_text, TextInput, TextReadError};
use super::text::wire::MAX_TEXT_DOCUMENT_BYTES;
use super::{Format, LoadError, Loaded, VersionedSchema};

/// Loads a versioned payload and rejects text that lacks the reserved `$zircon` envelope.
pub fn load_versioned<T>(bytes: &[u8], format: Format) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    match format {
        Format::Text => load_text_versioned(bytes, UnversionedTextPolicy::Reject),
        Format::Binary => load_binary_versioned(bytes),
    }
}

/// Explicit alias for profiles whose public contract requires a text envelope.
pub fn load_versioned_envelope<T>(bytes: &[u8], format: Format) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    load_versioned(bytes, format)
}

/// Loads a historical schema-zero text payload when no envelope is present.
///
/// Callers must opt into this only for schemas with an owned v0 migration policy.
pub fn load_versioned_legacy_schema_zero<T>(
    bytes: &[u8],
    format: Format,
) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    match format {
        Format::Text => load_text_versioned(bytes, UnversionedTextPolicy::AcceptSchemaZero),
        Format::Binary => load_binary_versioned(bytes),
    }
}

#[derive(Clone, Copy)]
enum UnversionedTextPolicy {
    AcceptSchemaZero,
    Reject,
}

fn load_text_versioned<T>(
    bytes: &[u8],
    unversioned_policy: UnversionedTextPolicy,
) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    if bytes.len() > MAX_TEXT_DOCUMENT_BYTES {
        return Err(LoadError::TextDocumentTooLarge {
            max: MAX_TEXT_DOCUMENT_BYTES,
            found: bytes.len(),
        });
    }
    match inspect_text(bytes).map_err(text_read_error)? {
        TextInput::Unversioned => match unversioned_policy {
            UnversionedTextPolicy::AcceptSchemaZero => load_schema_zero_text(bytes),
            UnversionedTextPolicy::Reject => Err(LoadError::MissingTextEnvelope {
                schema_id: T::SCHEMA.as_str().to_string(),
            }),
        },
        TextInput::Envelope(envelope) => {
            validate_schema::<T>(&envelope.header.schema_id)?;
            validate_source_version::<T>(envelope.header.schema_version)?;
            if envelope.header.schema_version == T::VERSION {
                validate_migration_chain::<T>()?;
                let value =
                    serde_json::from_str::<T>(envelope.payload.get()).map_err(|source| {
                        LoadError::PayloadDecode {
                            schema_id: T::SCHEMA.as_str().to_string(),
                            schema_version: T::VERSION,
                            source,
                        }
                    })?;
                Ok(Loaded {
                    value,
                    migrated_from: None,
                })
            } else {
                let payload = serde_json::from_str(envelope.payload.get())
                    .map_err(|source| LoadError::InvalidEnvelope { source })?;
                load_migrated_value(payload, envelope.header.schema_version)
            }
        }
    }
}

fn load_schema_zero_text<T>(bytes: &[u8]) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    let payload =
        serde_json::from_slice(bytes).map_err(|source| LoadError::MalformedText { source })?;
    load_migrated_value(payload, 0)
}

fn load_binary_versioned<T>(bytes: &[u8]) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    let (header, payload_body) = decode_binary_header(bytes)?;
    validate_schema::<T>(&header.schema_id)?;
    validate_source_version::<T>(header.schema_version)?;
    if header.schema_version == T::VERSION {
        validate_migration_chain::<T>()?;
        let value = decode_binary_current(payload_body)
            .map_err(|error| binary_current_decode_error::<T>(error))?;
        return Ok(Loaded {
            value,
            migrated_from: None,
        });
    }
    let payload = decode_binary_payload(payload_body)?;
    load_migrated_value(payload, header.schema_version)
}

fn load_migrated_value<T>(payload: Value, source_version: u32) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    validate_source_version::<T>(source_version)?;
    let migrated_from = (source_version < T::VERSION).then_some(source_version);
    let payload = T::migrations().migrate_value(&T::SCHEMA, payload, source_version, T::VERSION)?;
    let value = serde_json::from_value(payload).map_err(|source| LoadError::PayloadDecode {
        schema_id: T::SCHEMA.as_str().to_string(),
        schema_version: T::VERSION,
        source,
    })?;
    Ok(Loaded {
        value,
        migrated_from,
    })
}

fn validate_migration_chain<T>() -> Result<(), LoadError>
where
    T: VersionedSchema + 'static,
{
    T::migrations()
        .validate(&T::SCHEMA, T::VERSION)
        .map_err(LoadError::Migration)
}

fn text_read_error(error: TextReadError) -> LoadError {
    match error {
        TextReadError::Malformed(source) => LoadError::MalformedText { source },
        TextReadError::InvalidEnvelope(source) => LoadError::InvalidEnvelope { source },
    }
}

fn binary_current_decode_error<T>(error: DirectBinaryDecodeError) -> LoadError
where
    T: VersionedSchema,
{
    match error {
        DirectBinaryDecodeError::Malformed(source) => LoadError::MalformedBinary { source },
        DirectBinaryDecodeError::Invalid(source) => LoadError::InvalidBinaryPayload {
            reason: source.to_string(),
        },
        DirectBinaryDecodeError::Payload(reason) => LoadError::PayloadDecode {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            source: serde_json::Error::io(io::Error::new(io::ErrorKind::InvalidData, reason)),
        },
    }
}

fn validate_source_version<T>(source_version: u32) -> Result<(), LoadError>
where
    T: VersionedSchema,
{
    if source_version <= T::VERSION {
        return Ok(());
    }
    Err(LoadError::FutureVersion {
        schema_id: T::SCHEMA.as_str().to_string(),
        found: source_version,
        supported: T::VERSION,
    })
}

fn validate_schema<T>(found: &super::SchemaId) -> Result<(), LoadError>
where
    T: VersionedSchema,
{
    if found == &T::SCHEMA {
        return Ok(());
    }
    Err(LoadError::SchemaMismatch {
        expected: T::SCHEMA.as_str().to_string(),
        found: found.as_str().to_string(),
    })
}
