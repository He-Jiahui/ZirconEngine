use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTemplateNodeMetadata};

use super::runtime_state::{
    mark_runtime_style_delta_key, retained_runtime_state_attribute,
    RETAINED_RUNTIME_STATE_CANONICAL_KEYS,
};
use super::tokens::style_token_path_is_at_or_below;

/// Borrow the immutable captured baseline and the currently matched rule overlay.
/// Equality still visits nested values, but unchanged values keep their allocations.
pub(super) struct RuntimeStyleLayers<'a> {
    pub(super) base_attributes: &'a BTreeMap<String, Value>,
    pub(super) base_overrides: Option<&'a BTreeMap<String, Value>>,
    pub(super) base_tokens: Option<&'a BTreeMap<String, String>>,
    pub(super) rule_values: &'a BTreeMap<String, Value>,
    pub(super) rule_tokens: &'a BTreeMap<String, String>,
    pub(super) active_states: &'a [String],
}

impl<'a> RuntimeStyleLayers<'a> {
    fn attribute(&self, key: &str) -> Option<&'a Value> {
        if let Some(value) = retained_runtime_state_attribute(key, self.active_states) {
            return value;
        }
        self.rule_values
            .get(key)
            .or_else(|| self.base_attributes.get(key))
    }

    fn style_override(&self, key: &str) -> Option<&'a Value> {
        let baseline = self.base_overrides.and_then(|values| values.get(key));
        if !self.rule_values.contains_key(key)
            || baseline.is_some_and(|value| self.base_attributes.get(key) != Some(value))
        {
            return baseline;
        }
        self.attribute(key).or(baseline)
    }

    fn style_token(&self, path: &str) -> Option<&'a String> {
        if let Some(source) = self.rule_tokens.get(path) {
            return Some(source);
        }
        if self
            .rule_values
            .keys()
            .any(|key| style_token_path_is_at_or_below(path, key))
        {
            return None;
        }
        self.base_tokens.and_then(|values| values.get(path))
    }
}

pub(super) fn patch_runtime_style_attributes(
    metadata: &mut UiTemplateNodeMetadata,
    layers: RuntimeStyleLayers<'_>,
) -> Option<UiDirtyFlags> {
    let mut dirty = UiDirtyFlags::default();
    let attributes_changed = patch_borrowed_map(
        &mut metadata.attributes,
        layers
            .base_attributes
            .keys()
            .chain(layers.rule_values.keys())
            .map(String::as_str)
            .chain(RETAINED_RUNTIME_STATE_CANONICAL_KEYS.iter().copied()),
        |key| layers.attribute(key),
        |key| mark_runtime_style_delta_key(&mut dirty, key),
    );
    let overrides_changed = patch_borrowed_map(
        &mut metadata.style_overrides,
        layers
            .base_overrides
            .into_iter()
            .flat_map(|values| values.keys())
            .chain(layers.rule_values.keys())
            .map(String::as_str),
        |key| layers.style_override(key),
        |_| {},
    );
    let tokens_changed = patch_borrowed_map(
        &mut metadata.style_tokens,
        layers
            .base_tokens
            .into_iter()
            .flat_map(|values| values.keys())
            .chain(layers.rule_tokens.keys())
            .map(String::as_str),
        |key| layers.style_token(key),
        |_| {},
    );
    if attributes_changed || overrides_changed || tokens_changed {
        dirty.render = true;
        Some(dirty)
    } else {
        None
    }
}

fn patch_borrowed_map<'a, V: Clone + PartialEq + 'a>(
    target: &mut BTreeMap<String, V>,
    desired_keys: impl IntoIterator<Item = &'a str>,
    desired: impl Fn(&str) -> Option<&'a V>,
    mut on_change: impl FnMut(&str),
) -> bool {
    let mut changed = false;
    // Visit existing keys too: a rule that stopped matching must restore its baseline
    // value or remove its former property, override, and token source.
    target.retain(|key, current| {
        let Some(value) = desired(key) else {
            changed = true;
            on_change(key);
            return false;
        };
        if current != value {
            *current = value.clone();
            changed = true;
            on_change(key);
        }
        true
    });
    for key in desired_keys {
        if !target.contains_key(key) {
            if let Some(value) = desired(key) {
                target.insert(key.to_owned(), value.clone());
                changed = true;
                on_change(key);
            }
        }
    }
    changed
}

#[cfg(test)]
#[path = "attribute_delta/tests/cases.rs"]
mod tests;
