use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use crate::asset::AssetReference;

use super::super::{material_control, MaterialTextureSlotValue};

pub(super) fn texture_slot_reference(
    slots: &BTreeMap<String, MaterialTextureSlotValue>,
    slot: &str,
) -> Option<AssetReference> {
    slots.get(slot).and_then(|value| value.reference.clone())
}

pub(super) fn override_f32(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<f32> {
    values
        .get(key)
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
}

pub(super) fn override_bool(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<bool> {
    material_control::override_bool(values, key)
}

pub(super) fn override_vec4(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<[f32; 4]> {
    let items = values.get(key)?.as_array()?;
    Some([
        toml_number_as_f32(items.first()?)?,
        toml_number_as_f32(items.get(1)?)?,
        toml_number_as_f32(items.get(2)?)?,
        toml_number_as_f32(items.get(3)?)?,
    ])
}

pub(super) fn override_vec3(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<[f32; 3]> {
    let items = values.get(key)?.as_array()?;
    Some([
        toml_number_as_f32(items.first()?)?,
        toml_number_as_f32(items.get(1)?)?,
        toml_number_as_f32(items.get(2)?)?,
    ])
}

fn toml_number_as_f32(value: &toml::Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

pub(super) fn sync_texture_slot(
    slots: &mut BTreeMap<String, MaterialTextureSlotValue>,
    slot: &str,
    texture: Option<&AssetReference>,
) {
    match texture {
        Some(texture) => match slots.entry(slot.to_string()) {
            Entry::Occupied(mut entry) => {
                let previous = entry.get();
                let mut value = MaterialTextureSlotValue::new(texture.clone());
                value.fallback = previous.fallback.clone();
                value.transform = previous.transform;
                value.uv_channel = previous.texture_uv_channel();
                entry.insert(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(MaterialTextureSlotValue::new(texture.clone()));
            }
        },
        None => {
            let should_remove = if let Some(value) = slots.get_mut(slot) {
                value.reference = None;
                value.fallback.is_none()
                    && value.transform.is_none()
                    && value.texture_uv_channel() == 0
            } else {
                false
            };
            if should_remove {
                slots.remove(slot);
            }
        }
    }
}

pub(super) fn sync_f32_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: f32,
    default: f32,
) {
    if (value - default).abs() > f32::EPSILON {
        values.insert(key.to_string(), toml::Value::Float(value as f64));
    } else {
        values.remove(key);
    }
}

pub(super) fn sync_vec4_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: [f32; 4],
    default: [f32; 4],
) {
    if value != default {
        values.insert(key.to_string(), toml_array(value));
    } else {
        values.remove(key);
    }
}

pub(super) fn sync_vec3_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: [f32; 3],
    default: [f32; 3],
) {
    if value != default {
        values.insert(key.to_string(), toml_array(value));
    } else {
        values.remove(key);
    }
}

fn toml_array<const N: usize>(value: [f32; N]) -> toml::Value {
    toml::Value::Array(
        value
            .into_iter()
            .map(|value| toml::Value::Float(value as f64))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::asset::{AssetReference, AssetUri};
    use crate::core::framework::render::RenderMaterialTextureTransform;

    use super::{sync_texture_slot, MaterialTextureSlotValue};

    fn reference(locator: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(locator).unwrap())
    }

    #[test]
    fn texture_slot_sync_preserves_existing_slot_metadata() {
        let mut previous = MaterialTextureSlotValue::new(reference("res://old.texture"));
        previous.fallback = Some("white".to_string());
        previous.transform = Some(RenderMaterialTextureTransform::default());
        previous.uv_channel = 3;
        let mut slots = BTreeMap::from([("base_color".to_string(), previous)]);
        let replacement = reference("res://new.texture");

        sync_texture_slot(&mut slots, "base_color", Some(&replacement));

        let synchronized = slots.get("base_color").unwrap();
        assert_eq!(synchronized.reference.as_ref(), Some(&replacement));
        assert_eq!(synchronized.fallback.as_deref(), Some("white"));
        assert!(synchronized.transform.is_some());
        assert_eq!(synchronized.uv_channel, 3);
    }

    #[test]
    fn texture_slot_sync_inserts_a_new_slot() {
        let mut slots = BTreeMap::new();
        let reference = reference("res://new.texture");

        sync_texture_slot(&mut slots, "normal", Some(&reference));

        let synchronized = slots.get("normal").unwrap();
        assert_eq!(synchronized.reference.as_ref(), Some(&reference));
        assert_eq!(synchronized.fallback, None);
        assert_eq!(synchronized.transform, None);
        assert_eq!(synchronized.uv_channel, 0);
    }
}
