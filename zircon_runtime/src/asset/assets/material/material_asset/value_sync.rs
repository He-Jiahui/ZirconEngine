//! 材质属性与纹理槽的双向同步只更新作者覆写，同时保留已有槽的附加元数据；编辑器写回不能把渲染侧临时默认值持久化成用户输入。

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
#[path = "tests/value_sync.rs"]
mod tests;
