mod raw;

pub(in super::super) fn bool_from_plugin_toml(value: &str) -> bool {
    raw::bool_from_plugin_toml(value)
}
