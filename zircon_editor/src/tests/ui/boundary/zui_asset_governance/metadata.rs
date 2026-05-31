use std::collections::BTreeMap;

use toml::Value;

pub(super) fn is_lower_snake_case_identifier(value: &str) -> bool {
    let mut previous_was_underscore = false;
    for (index, character) in value.chars().enumerate() {
        let valid = character.is_ascii_lowercase()
            || (index > 0 && character.is_ascii_digit())
            || (index > 0 && character == '_' && !previous_was_underscore);
        if !valid {
            return false;
        }
        previous_was_underscore = character == '_';
    }
    !value.is_empty() && !previous_was_underscore
}

pub(super) fn is_authoring_route_identifier(value: &str) -> bool {
    let mut segment_count = 0usize;
    for segment in value.split('.') {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return false;
        }
        segment_count += 1;
    }
    segment_count >= 2
}

pub(super) fn is_authoring_path_identifier(value: &str) -> bool {
    let mut segment_count = 0usize;
    for segment in value.split('/') {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return false;
        }
        segment_count += 1;
    }
    segment_count >= 2
}

pub(super) fn is_control_id_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_alphanumeric()
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn is_class_token_identifier(value: &str) -> bool {
    if is_material_class_token_identifier(value) {
        return true;
    }

    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut last = first;
    for character in characters {
        if !(character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '_'
            || character == '-')
        {
            return false;
        }
        last = character;
    }
    last.is_ascii_lowercase() || last.is_ascii_digit()
}

fn is_material_class_token_identifier(value: &str) -> bool {
    let Some(remainder) = value.strip_prefix("Mui") else {
        return false;
    };
    let mut segments = remainder.split('-');
    let Some(root_segment) = segments.next() else {
        return false;
    };
    !root_segment.is_empty()
        && root_segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && segments.all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
}

pub(super) fn class_list_offenders(classes: &[String]) -> Vec<String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    let mut offenders = Vec::new();

    for class in classes {
        if let Some(invalid_class_token) = string_token_metadata_offender(class, "class token") {
            offenders.push(invalid_class_token);
            if class.trim().is_empty() {
                continue;
            }
        } else if !is_class_token_identifier(class) {
            offenders.push(format!(
                "selector-unsafe class token `{class}` outside lowercase selector form"
            ));
        }
        *counts.entry(class.trim()).or_default() += 1;
    }

    offenders.extend(
        counts
            .into_iter()
            .filter_map(|(class, count)| (count > 1).then(|| format!("duplicate `{class}`"))),
    );
    offenders
}

pub(super) fn class_name_prop_offenders(
    attributes: &BTreeMap<String, Value>,
) -> (usize, Vec<String>) {
    let mut checked_class_names = 0usize;
    let mut offenders = Vec::new();

    for (key, value) in attributes {
        collect_class_name_prop_offender_value(
            key,
            value,
            "props",
            &mut checked_class_names,
            &mut offenders,
        );
    }

    (checked_class_names, offenders)
}

pub(super) fn string_metadata_offender(value: &str, label: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(format!("empty {label}"));
    }
    (trimmed != value).then(|| format!("whitespace-padded {label} `{value}`"))
}

pub(super) fn string_token_metadata_offender(value: &str, label: &str) -> Option<String> {
    string_metadata_offender(value, label).or_else(|| {
        value
            .chars()
            .any(char::is_whitespace)
            .then(|| format!("whitespace-containing {label} `{value}`"))
    })
}

pub(super) fn attribute_key_offenders(
    attributes: &BTreeMap<String, Value>,
    label: &str,
) -> (usize, Vec<String>) {
    let mut checked_keys = 0usize;
    let mut offenders = Vec::new();

    for (key, value) in attributes {
        checked_keys += 1;
        if let Some(invalid_key) = string_metadata_offender(key, label) {
            offenders.push(format!("{invalid_key} at `{key}`"));
        }
        collect_nested_attribute_key_offenders(
            value,
            label,
            key,
            &mut checked_keys,
            &mut offenders,
        );
    }

    (checked_keys, offenders)
}

const RESOURCE_LIKE_SUFFIXES: &[&str] = &[
    ".svg", ".png", ".jpg", ".jpeg", ".webp", ".ttf", ".otf", ".ron", ".toml",
];

pub(super) fn resource_path_string_offenders(
    attributes: &BTreeMap<String, Value>,
    label: &str,
) -> (usize, Vec<String>) {
    let mut checked_strings = 0usize;
    let mut offenders = Vec::new();

    for (key, value) in attributes {
        collect_resource_path_string_offenders(
            value,
            label,
            key,
            &mut checked_strings,
            &mut offenders,
        );
    }

    (checked_strings, offenders)
}

fn collect_class_name_prop_offender_value(
    key: &str,
    value: &Value,
    path: &str,
    checked_class_names: &mut usize,
    offenders: &mut Vec<String>,
) {
    let key_path = format!("{path}.{key}");
    if key == "className" || key == "baseClassName" {
        *checked_class_names += 1;
        let Some(class_name) = value.as_str() else {
            offenders.push(format!("{key_path} is not a string"));
            return;
        };
        let class_tokens = class_name
            .split_ascii_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if class_tokens.is_empty() || class_name.trim() != class_name {
            offenders.push(format!("{key_path} is empty or whitespace-padded"));
            return;
        }
        let invalid_classes = class_list_offenders(&class_tokens);
        if !invalid_classes.is_empty() {
            offenders.push(format!("{key_path} contains {invalid_classes:?}"));
        }
        return;
    }

    if key != "slotProps" {
        return;
    }
    let Some(slot_props) = value.as_table() else {
        offenders.push(format!("{key_path} is not a table"));
        return;
    };
    for (slot_name, slot_value) in slot_props {
        let slot_path = format!("{key_path}.{slot_name}");
        let Some(slot_table) = slot_value.as_table() else {
            offenders.push(format!("{slot_path} is not a table"));
            continue;
        };
        for (slot_key, slot_prop_value) in slot_table {
            collect_class_name_prop_offender_value(
                slot_key,
                slot_prop_value,
                &slot_path,
                checked_class_names,
                offenders,
            );
        }
    }
}

fn collect_nested_attribute_key_offenders(
    value: &Value,
    label: &str,
    path: &str,
    checked_keys: &mut usize,
    offenders: &mut Vec<String>,
) {
    match value {
        Value::Table(table) => {
            for (key, nested_value) in table {
                *checked_keys += 1;
                let key_path = format!("{path}.{key}");
                if let Some(invalid_key) = string_metadata_offender(key, label) {
                    offenders.push(format!("{invalid_key} at `{key_path}`"));
                }
                collect_nested_attribute_key_offenders(
                    nested_value,
                    label,
                    &key_path,
                    checked_keys,
                    offenders,
                );
            }
        }
        Value::Array(values) => {
            for (index, nested_value) in values.iter().enumerate() {
                collect_nested_attribute_key_offenders(
                    nested_value,
                    label,
                    &format!("{path}[{}]", index + 1),
                    checked_keys,
                    offenders,
                );
            }
        }
        _ => {}
    }
}

fn collect_resource_path_string_offenders(
    value: &Value,
    label: &str,
    path: &str,
    checked_strings: &mut usize,
    offenders: &mut Vec<String>,
) {
    match value {
        Value::String(value) => {
            if !is_resource_like_string(value) {
                return;
            }
            *checked_strings += 1;
            if let Some(offender) = resource_like_string_offender(value) {
                offenders.push(format!("{offender} at {label}.{path}"));
            }
        }
        Value::Table(table) => {
            for (key, nested_value) in table {
                collect_resource_path_string_offenders(
                    nested_value,
                    label,
                    &format!("{path}.{key}"),
                    checked_strings,
                    offenders,
                );
            }
        }
        Value::Array(values) => {
            for (index, nested_value) in values.iter().enumerate() {
                collect_resource_path_string_offenders(
                    nested_value,
                    label,
                    &format!("{path}[{}]", index + 1),
                    checked_strings,
                    offenders,
                );
            }
        }
        _ => {}
    }
}

fn is_resource_like_string(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    RESOURCE_LIKE_SUFFIXES
        .iter()
        .any(|suffix| lower.contains(suffix))
}

fn resource_like_string_offender(value: &str) -> Option<String> {
    if value.contains('\\') {
        return Some(format!("backslash resource path `{value}`"));
    }
    if value.contains("../") || value.starts_with("..") {
        return Some(format!("relative parent resource path `{value}`"));
    }
    if value
        .get(1..3)
        .is_some_and(|drive_marker| drive_marker == ":/" || drive_marker == ":\\")
    {
        return Some(format!("absolute drive resource path `{value}`"));
    }
    value
        .contains("dev/")
        .then(|| format!("dev-tree resource path `{value}`"))
}
