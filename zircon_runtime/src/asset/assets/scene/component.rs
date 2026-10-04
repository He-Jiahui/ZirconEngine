//! Versioned, provider-owned scene component rows with stable JSON asset-reference slots.
use crate::asset::AssetReference;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

const ASSET_REFERENCE_SLOT_FIELD: &str = "$zircon_scene_asset_reference";

/// Opaque scene component data carried through the canonical project and cache formats.
/// Identity is persisted per row so a loader can reject a mismatched provider or schema.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneComponentAssetRecord {
    pub type_id: String,
    pub schema_id: String,
    pub schema_version: u32,
    pub provider_id: String,
    pub payload: Value,
    /// References appear as indexed slots in payload and are mapped independently of component type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<AssetReference>,
}

impl SceneComponentAssetRecord {
    pub fn from_typed<T: Serialize>(
        type_id: impl Into<String>,
        schema_id: impl Into<String>,
        schema_version: u32,
        provider_id: impl Into<String>,
        value: &T,
    ) -> Result<Self, String> {
        let value = serde_json::to_value(value)
            .map_err(|error| format!("serialize component payload: {error}"))?;
        let mut references = Vec::new();
        let payload = extract_asset_reference_slots(value, &mut references)?;
        Ok(Self {
            type_id: type_id.into(),
            schema_id: schema_id.into(),
            schema_version,
            provider_id: provider_id.into(),
            payload,
            references,
        })
    }

    /// Validates the payload marker table without requiring a provider's typed payload.
    ///
    /// Dependency extraction and cache admission run this check before publishing any
    /// references.  Keeping it separate from `decode_typed` prevents opaque rows from
    /// bypassing marker validation merely because their provider is not being instantiated.
    pub fn validate_references(&self) -> Result<(), String> {
        self.payload_with_references_restored().map(|_| ())
    }

    fn payload_with_references_restored(&self) -> Result<Value, String> {
        let mut payload = self.payload.clone();
        let mut referenced_slots = vec![false; self.references.len()];
        restore_asset_reference_slots(&mut payload, &self.references, &mut referenced_slots)?;
        if let Some(index) = referenced_slots.iter().position(|seen| !seen) {
            return Err(format!("component row has unreferenced asset slot {index}"));
        }
        Ok(payload)
    }

    pub fn decode_typed<T: DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_value(self.payload_with_references_restored()?)
            .map_err(|error| format!("deserialize component payload: {error}"))
    }
}

fn extract_asset_reference_slots(
    mut value: Value,
    references: &mut Vec<AssetReference>,
) -> Result<Value, String> {
    match &mut value {
        Value::Object(object) => {
            if object.contains_key(ASSET_REFERENCE_SLOT_FIELD) {
                return Err("component payload uses reserved asset-reference slot marker".into());
            }
            if object.len() == 2 && object.contains_key("uuid") && object.contains_key("url") {
                let reference = serde_json::from_value::<AssetReference>(value)
                    .map_err(|error| format!("invalid component asset reference: {error}"))?;
                let index = u32::try_from(references.len())
                    .map_err(|_| "component row has too many asset references".to_owned())?;
                references.push(reference);
                return Ok(serde_json::json!({ ASSET_REFERENCE_SLOT_FIELD: index }));
            }
            for nested in object.values_mut() {
                *nested = extract_asset_reference_slots(std::mem::take(nested), references)?;
            }
        }
        Value::Array(values) => {
            for nested in values {
                *nested = extract_asset_reference_slots(std::mem::take(nested), references)?;
            }
        }
        _ => {}
    }
    Ok(value)
}

fn restore_asset_reference_slots(
    value: &mut Value,
    references: &[AssetReference],
    referenced_slots: &mut [bool],
) -> Result<(), String> {
    match value {
        Value::Object(object) if object.contains_key(ASSET_REFERENCE_SLOT_FIELD) => {
            if object.len() != 1 {
                return Err("component row has malformed asset slot marker object".to_owned());
            }
            let raw_index = object[ASSET_REFERENCE_SLOT_FIELD]
                .as_u64()
                .ok_or_else(|| "component row has a non-integer asset slot".to_owned())?;
            let index = usize::try_from(raw_index)
                .map_err(|_| "component row asset slot does not fit usize".to_owned())?;
            let reference = references
                .get(index)
                .ok_or_else(|| format!("component row asset slot {index} is out of range"))?;
            let seen = referenced_slots
                .get_mut(index)
                .ok_or_else(|| format!("component row asset slot {index} is out of range"))?;
            if *seen {
                return Err(format!("component row repeats asset slot {index}"));
            }
            *seen = true;
            *value = serde_json::to_value(reference)
                .map_err(|error| format!("serialize component asset reference: {error}"))?;
        }
        Value::Object(object) => {
            for nested in object.values_mut() {
                restore_asset_reference_slots(nested, references, referenced_slots)?;
            }
        }
        Value::Array(values) => {
            for nested in values {
                restore_asset_reference_slots(nested, references, referenced_slots)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/component.rs"]
mod tests;
