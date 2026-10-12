use super::super::super::super::super::values::feature_capability_list_from_plugin_toml;

pub(super) fn feature_capabilities_from_plugin_toml(value: &str) -> Vec<String> {
    feature_capability_list_from_plugin_toml(value)
}
