use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;

const MUI_ALERT_DEFAULT_SEVERITY: &str = "success";

pub(super) fn append_variant_token(variant: &mut String, token: &str) {
    if token.is_empty()
        || variant
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case(token))
    {
        return;
    }
    if !variant.is_empty() {
        variant.push(' ');
    }
    variant.push_str(token);
}

pub(super) fn has_non_empty_attribute(
    attributes: &BTreeMap<String, toml::Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes.get(*name).is_some_and(|value| match value {
            toml::Value::String(value) => !value.trim().is_empty(),
            toml::Value::Array(values) => !values.is_empty(),
            toml::Value::Table(values) => !values.is_empty(),
            toml::Value::Integer(_)
            | toml::Value::Float(_)
            | toml::Value::Boolean(_)
            | toml::Value::Datetime(_) => true,
        })
    })
}

pub(super) fn string_from_toml_map(value: Option<&toml::Value>, key: &str) -> Option<String> {
    let toml::Value::Table(map) = value? else {
        return None;
    };
    map.get(key).and_then(value_as_string)
}

pub(super) fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    let mut result = String::with_capacity(value.len());
    result.push(first.to_ascii_uppercase());
    result.push_str(characters.as_str());
    result
}

pub(super) fn alert_color_severity(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|color| !color.is_empty())
        .or_else(|| {
            attributes
                .get("severity")
                .and_then(value_as_string)
                .filter(|severity| !severity.is_empty())
        })
        .unwrap_or_else(|| MUI_ALERT_DEFAULT_SEVERITY.to_string())
}

pub(super) fn dialog_severity(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("severity")
        .and_then(value_as_string)
        .filter(|severity| !severity.is_empty())
        .unwrap_or_else(|| "warning".to_string())
}

pub(super) fn app_bar_color(attributes: &BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|color| !color.is_empty())
        .unwrap_or_else(|| "primary".to_string())
}

pub(super) fn variant_contains(component_variant: &str, expected: &str) -> bool {
    component_variant
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case(expected))
}

#[cfg(test)]
#[path = "shared/single_allocation_pascal_case_tests.rs"]
mod single_allocation_pascal_case_tests;
