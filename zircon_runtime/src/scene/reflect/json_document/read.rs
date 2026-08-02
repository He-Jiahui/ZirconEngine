use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::{Format, Loaded, load_versioned};

use super::ReflectedJsonError;
use super::document::ReflectedJsonDocument;

/// Loads versioned reflected JSON; an unwrapped value is the historical v0 form.
pub fn reflected_from_json(json: &str) -> Result<Loaded<ReflectedValue>, ReflectedJsonError> {
    let loaded = load_versioned::<ReflectedJsonDocument>(json.as_bytes(), Format::Text)?;
    Ok(Loaded {
        value: loaded.value.value,
        migrated_from: loaded.migrated_from,
    })
}
