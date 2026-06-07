use super::super::super::super::super::values::module_capability_list_from_plugin_toml;

pub(super) fn module_capabilities_from_plugin_toml(value: &str) -> Vec<String> {
    module_capability_list_from_plugin_toml(value)
}
