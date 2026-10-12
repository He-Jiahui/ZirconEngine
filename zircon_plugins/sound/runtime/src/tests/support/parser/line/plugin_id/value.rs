use super::super::super::super::super::super::values::dependency_plugin_id_string_from_plugin_toml;

pub(super) fn dependency_plugin_id_from_plugin_toml(value: &str) -> String {
    dependency_plugin_id_string_from_plugin_toml(value)
}
