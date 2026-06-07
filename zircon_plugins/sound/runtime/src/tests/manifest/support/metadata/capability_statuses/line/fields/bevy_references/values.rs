use super::super::super::super::super::super::values::string_array_values;

pub(super) fn capability_status_bevy_references_from_plugin_toml(value: &str) -> Vec<String> {
    string_array_values(value)
}
