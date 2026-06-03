use std::path::Path;

use super::non_empty_string_array_values;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_known_default_packaging_strategies(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
) {
    for packaging in
        non_empty_string_array_values(table, relative_path, context, "default_packaging")
    {
        assert!(
            matches!(packaging, "source_template" | "library_embed" | "native_dynamic"),
            "plugin manifest {relative_path:?} {context} default packaging strategy `{packaging}` should be source_template, library_embed, or native_dynamic"
        );
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn assert_known_runtime_targets(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    for target in non_empty_string_array_values(table, relative_path, context, field_name) {
        assert!(
            matches!(target, "client_runtime" | "server_runtime" | "editor_host"),
            "plugin manifest {relative_path:?} {context} `{field_name}` target `{target}` should be client_runtime, server_runtime, or editor_host"
        );
    }
}
