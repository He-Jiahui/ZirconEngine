use std::collections::BTreeMap;
use std::path::Path;

use super::super::non_empty_string_array_values;

pub(super) fn assert_known_value_type(
    relative_path: &Path,
    option_context: &str,
    value_type: &str,
) {
    assert!(
        matches!(
            value_type,
            "bool" | "integer" | "number" | "string" | "enum"
        ),
        "plugin manifest {relative_path:?} {option_context} value_type `{value_type}` should be bool, integer, number, string, or enum"
    );
}

pub(super) fn assert_default_value_shape(
    relative_path: &Path,
    option_context: &str,
    value_type: &str,
    default_value: &str,
) {
    match value_type {
        "bool" => assert!(
            matches!(default_value, "true" | "false"),
            "plugin manifest {relative_path:?} {option_context} bool default_value `{default_value}` should be true or false"
        ),
        "integer" => {
            default_value.parse::<i64>().unwrap_or_else(|error| {
                panic!(
                    "plugin manifest {relative_path:?} {option_context} integer default_value `{default_value}` should parse as i64: {error}"
                )
            });
        }
        "number" => {
            let number = default_value.parse::<f64>().unwrap_or_else(|error| {
                panic!(
                    "plugin manifest {relative_path:?} {option_context} number default_value `{default_value}` should parse as f64: {error}"
                )
            });
            assert!(
                number.is_finite(),
                "plugin manifest {relative_path:?} {option_context} number default_value `{default_value}` should be finite"
            );
        }
        "enum" => assert_enum_token(
            relative_path,
            option_context,
            "default_value",
            default_value,
        ),
        "string" => {}
        _ => unreachable!("value_type should already be validated"),
    }
}

pub(super) fn assert_enum_values_contract(
    relative_path: &Path,
    option_context: &str,
    option: &toml::Table,
    value_type: &str,
    default_value: &str,
) {
    if value_type != "enum" {
        assert!(
            option.get("enum_values").is_none(),
            "plugin manifest {relative_path:?} {option_context} non-enum option should not declare `enum_values`"
        );
        return;
    }

    let enum_values =
        non_empty_string_array_values(option, relative_path, option_context, "enum_values");
    let mut seen_values = BTreeMap::new();
    for (index, enum_value) in enum_values.iter().enumerate() {
        assert_enum_token(relative_path, option_context, "enum_values", enum_value);
        if let Some(previous_index) = seen_values.insert((*enum_value).to_string(), index) {
            panic!(
                "plugin manifest {relative_path:?} {option_context} enum_values entry `{enum_value}` should be unique; first declared at index {previous_index}, repeated at index {index}"
            );
        }
    }
    assert!(
        enum_values
            .iter()
            .any(|enum_value| *enum_value == default_value),
        "plugin manifest {relative_path:?} {option_context} enum default_value `{default_value}` should be declared in enum_values"
    );
}

pub(super) fn assert_enum_token(
    relative_path: &Path,
    option_context: &str,
    field_name: &str,
    value: &str,
) {
    assert_trimmed(relative_path, option_context, field_name, value);
    assert!(
        value.bytes().all(|byte| byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || byte == b'_'
            || byte == b'-'),
        "plugin manifest {relative_path:?} {option_context} enum `{field_name}` value `{value}` should contain only lowercase ASCII letters, digits, underscores, or hyphens"
    );
}

pub(super) fn assert_trimmed(
    relative_path: &Path,
    option_context: &str,
    field_name: &str,
    value: &str,
) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {option_context} `{field_name}` should not have leading or trailing whitespace"
    );
}

pub(super) fn assert_dot_namespaced_option_key(
    relative_path: &Path,
    option_context: &str,
    key: &str,
) {
    assert_trimmed(relative_path, option_context, "key", key);

    let segments: Vec<_> = key.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {option_context} key `{key}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {option_context} key `{key}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {option_context} key `{key}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}
