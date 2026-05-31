use std::collections::BTreeMap;
use std::path::Path;

use super::{for_each_static_plugin_manifest, non_empty_string_value};

#[test]
fn plugin_tomls_declare_option_rows() {
    let mut option_keys = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(options) = table.get("options") else {
            return;
        };
        let options = options.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} options should be an array")
        });
        assert!(
            !options.is_empty(),
            "plugin manifest {relative_path:?} options should not be empty when declared"
        );

        for option in options {
            let option = option.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} option should be a table")
            });
            let key = non_empty_string_value(option, relative_path, "plugin option", "key");
            let option_context = format!("plugin option `{key}`");
            assert_dot_namespaced_option_key(relative_path, &option_context, key);
            if let Some(previous_context) =
                option_keys.insert(key.to_string(), option_context.clone())
            {
                panic!(
                    "plugin option key `{key}` should be globally unique; first declared by {previous_context}, repeated by {option_context} in {}",
                    relative_path.display()
                );
            }

            let display_name =
                non_empty_string_value(option, relative_path, &option_context, "display_name");
            assert_trimmed(relative_path, &option_context, "display_name", display_name);
            let value_type =
                non_empty_string_value(option, relative_path, &option_context, "value_type");
            assert_known_value_type(relative_path, &option_context, value_type);
            let default_value =
                non_empty_string_value(option, relative_path, &option_context, "default_value");
            assert_trimmed(
                relative_path,
                &option_context,
                "default_value",
                default_value,
            );
            assert_default_value_shape(relative_path, &option_context, value_type, default_value);

            if option.get("required_capability").is_some() {
                let capability = non_empty_string_value(
                    option,
                    relative_path,
                    &option_context,
                    "required_capability",
                );
                assert_trimmed(
                    relative_path,
                    &option_context,
                    "required_capability",
                    capability,
                );
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_option_keys_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(options) = table.get("options") else {
            return;
        };
        let options = options.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} options should be an array")
        });

        for option in options {
            let option = option.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} option should be a table")
            });
            let key = non_empty_string_value(option, relative_path, "plugin option", "key");
            let option_context = format!("plugin option `{key}`");
            assert_dot_namespaced_option_key(relative_path, &option_context, key);
        }
    });
}

fn assert_known_value_type(relative_path: &Path, option_context: &str, value_type: &str) {
    assert!(
        matches!(value_type, "bool" | "integer" | "number" | "string" | "enum"),
        "plugin manifest {relative_path:?} {option_context} value_type `{value_type}` should be bool, integer, number, string, or enum"
    );
}

fn assert_default_value_shape(
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
        "string" | "enum" => {}
        _ => unreachable!("value_type should already be validated"),
    }
}

fn assert_trimmed(relative_path: &Path, option_context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {option_context} `{field_name}` should not have leading or trailing whitespace"
    );
}

fn assert_dot_namespaced_option_key(relative_path: &Path, option_context: &str, key: &str) {
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
