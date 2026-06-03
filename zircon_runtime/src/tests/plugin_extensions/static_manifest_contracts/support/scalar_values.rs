use std::path::Path;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn bool_value(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> bool {
    table
        .get(field_name)
        .and_then(toml::Value::as_bool)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare boolean `{field_name}`"
            )
        })
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn integer_value(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> i64 {
    table
        .get(field_name)
        .and_then(toml::Value::as_integer)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare integer `{field_name}`"
            )
        })
}
