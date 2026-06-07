mod dependency;
mod feature;
mod raw;

fn bool_from_plugin_toml(value: &str) -> bool {
    raw::bool_from_plugin_toml(value)
}

pub(in super::super) fn dependency_primary_bool_from_plugin_toml(value: &str) -> bool {
    dependency::dependency_primary_bool_from_plugin_toml(value)
}

pub(in super::super) fn enabled_by_default_bool_from_plugin_toml(value: &str) -> bool {
    feature::enabled_by_default_bool_from_plugin_toml(value)
}
