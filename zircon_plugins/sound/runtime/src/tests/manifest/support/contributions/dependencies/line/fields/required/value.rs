use super::super::super::super::super::super::values::bool_from_plugin_toml;

pub(super) fn dependency_required_from_plugin_toml(value: &str) -> bool {
    bool_from_plugin_toml(value)
}
