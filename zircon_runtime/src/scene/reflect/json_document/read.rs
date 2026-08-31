use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::{load_versioned_legacy_schema_zero, Format, Loaded};

use super::document::ReflectedJsonDocument;
use super::ReflectedJsonError;
use crate::scene::reflect::RUNTIME_REFLECT_VALUE_BUDGET;

/// Loads versioned reflected JSON; an unwrapped value is the historical v0 form.
pub fn reflected_from_json(json: &str) -> Result<Loaded<ReflectedValue>, ReflectedJsonError> {
    let loaded =
        load_versioned_legacy_schema_zero::<ReflectedJsonDocument>(json.as_bytes(), Format::Text)?;
    loaded
        .value
        .value
        .validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)?;
    Ok(Loaded {
        value: loaded.value.value,
        migrated_from: loaded.migrated_from,
    })
}
