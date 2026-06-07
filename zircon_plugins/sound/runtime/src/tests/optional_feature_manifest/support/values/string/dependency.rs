mod capability;
mod plugin_id;

pub(super) fn dependency_plugin_id_string_from_plugin_toml(value: &str) -> String {
    plugin_id::dependency_plugin_id_string_from_plugin_toml(value)
}

pub(super) fn dependency_capability_string_from_plugin_toml(value: &str) -> String {
    capability::dependency_capability_string_from_plugin_toml(value)
}
