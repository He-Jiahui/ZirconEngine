mod crate_name;
mod name;

pub(super) fn module_name_string_from_plugin_toml(value: &str) -> String {
    name::module_name_string_from_plugin_toml(value)
}

pub(super) fn module_crate_name_string_from_plugin_toml(value: &str) -> String {
    crate_name::module_crate_name_string_from_plugin_toml(value)
}
