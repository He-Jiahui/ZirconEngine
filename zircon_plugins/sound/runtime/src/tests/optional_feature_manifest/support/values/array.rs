mod feature;
mod module;
mod raw;

pub(super) fn string_array_values(value: &str) -> Vec<String> {
    raw::string_array_values(value)
}

fn string_list_from_plugin_toml(value: &str) -> Vec<String> {
    string_array_values(value)
}

pub(in super::super) fn feature_capability_list_from_plugin_toml(value: &str) -> Vec<String> {
    feature::feature_capability_list_from_plugin_toml(value)
}

pub(in super::super) fn module_capability_list_from_plugin_toml(value: &str) -> Vec<String> {
    module::module_capability_list_from_plugin_toml(value)
}
