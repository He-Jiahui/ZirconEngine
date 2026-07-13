use serde::de::DeserializeOwned;
use serde_json::Value;

use super::text::{document::TextDocument, wire::TEXT_ENVELOPE_KEY};
use super::{Format, LoadError, Loaded, VersionedSchema};

/// Loads a versioned payload, treating an unwrapped text value as schema version zero.
pub fn load_versioned<T>(bytes: &[u8], format: Format) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    match format {
        Format::Text => load_text(bytes),
        Format::Binary => Err(LoadError::UnsupportedFormat { format }),
    }
}

fn load_text<T>(bytes: &[u8]) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    let parsed: Value =
        serde_json::from_slice(bytes).map_err(|source| LoadError::MalformedText { source })?;
    let (payload, source_version) = parse_text_payload::<T>(parsed)?;
    if source_version > T::VERSION {
        return Err(LoadError::FutureVersion {
            schema_id: T::SCHEMA.as_str().to_string(),
            found: source_version,
            supported: T::VERSION,
        });
    }

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

fn parse_text_payload<T>(parsed: Value) -> Result<(Value, u32), LoadError>
where
    T: VersionedSchema,
{
    let Some(object) = parsed.as_object() else {
        return Ok((parsed, 0));
    };
    if !object.contains_key(TEXT_ENVELOPE_KEY) {
        return Ok((parsed, 0));
    }

    let document: TextDocument =
        serde_json::from_value(parsed).map_err(|source| LoadError::InvalidEnvelope { source })?;
    let envelope = document.envelope;
    if envelope.header.schema_id != T::SCHEMA {
        return Err(LoadError::SchemaMismatch {
            expected: T::SCHEMA.as_str().to_string(),
            found: envelope.header.schema_id.as_str().to_string(),
        });
    }
    Ok((envelope.payload, envelope.header.schema_version))
}
