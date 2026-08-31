use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::write_versioned_text;

use super::document::ReflectedJsonDocument;
use super::ReflectedJsonError;
use crate::scene::reflect::RUNTIME_REFLECT_VALUE_BUDGET;

/// Writes the current reflected-JSON schema with canonical text formatting.
pub fn json_from_reflected(value: &ReflectedValue) -> Result<String, ReflectedJsonError> {
    value.validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)?;
    Ok(write_versioned_text(&ReflectedJsonDocument {
        value: value.clone(),
    })?)
}
