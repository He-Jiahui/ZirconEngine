use super::super::super::super::super::values::dependency_primary_bool_from_plugin_toml;

pub(super) fn dependency_primary_from_plugin_toml(value: &str) -> bool {
    dependency_primary_bool_from_plugin_toml(value)
}
