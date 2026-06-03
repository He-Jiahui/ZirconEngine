use std::path::Path;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_non_empty_string(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    non_empty_string_value(table, relative_path, context, field_name);
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_non_empty_string_array(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    non_empty_string_array_values(table, relative_path, context, field_name);
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn non_empty_string_value<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> &'a str {
    let value = table
        .get(field_name)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
            )
        });
    assert!(
        !value.is_empty(),
        "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
    );
    value
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn non_empty_string_array_values<
    'a,
>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a str> {
    let values = table
        .get(field_name)
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
            )
        });
    assert!(
        !values.is_empty(),
        "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
    );
    values
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` entries should be strings"
                )
            })
        })
        .inspect(|value| {
            assert!(
                !value.is_empty(),
                "plugin manifest {relative_path:?} {context} `{field_name}` entries should not be empty"
            );
        })
        .collect()
}
