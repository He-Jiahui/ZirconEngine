use super::super::super::super::super::super::values::enabled_by_default_bool_from_plugin_toml;

pub(super) fn enabled_by_default_from_plugin_toml(value: &str) -> bool {
    enabled_by_default_bool_from_plugin_toml(value)
}
