use std::collections::BTreeMap;

use super::super::pane_value_conversion::value_as_string;

pub(super) fn value_as_i32(value: &toml::Value) -> Option<i32> {
    value
        .as_integer()
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn first_non_empty_string_attribute(
    attributes: &BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<String> {
    names
        .iter()
        .filter_map(|name| attributes.get(*name))
        .filter_map(value_as_string)
        .find(|value| !value.is_empty())
}

pub(super) fn vec_component(values: &[f32], index: usize, default: f32) -> f32 {
    values.get(index).copied().unwrap_or(default)
}

pub(super) fn humanize_control_id(control_id: &str) -> String {
    let mut text = String::with_capacity(control_id.len());
    for (index, character) in control_id.chars().enumerate() {
        if index > 0 && character.is_uppercase() {
            text.push(' ');
        }
        text.push(character);
    }
    text
}

pub(super) fn should_humanize_control_label(control_id: &str) -> bool {
    control_id.starts_with("Apply")
        || control_id.starts_with("Delete")
        || control_id.ends_with("Button")
        || control_id.ends_with("Action")
}

#[cfg(test)]
#[path = "attribute_values/capacity_tests.rs"]
mod capacity_tests;
