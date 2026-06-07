mod dependency;
mod feature;
mod module;
mod raw;

fn string_from_plugin_toml(value: &str) -> String {
    raw::string_from_plugin_toml(value)
}

pub(in super::super) fn dependency_plugin_id_string_from_plugin_toml(value: &str) -> String {
    dependency::dependency_plugin_id_string_from_plugin_toml(value)
}

pub(in super::super) fn dependency_capability_string_from_plugin_toml(value: &str) -> String {
    dependency::dependency_capability_string_from_plugin_toml(value)
}

pub(in super::super) fn feature_id_string_from_plugin_toml(value: &str) -> String {
    feature::feature_id_string_from_plugin_toml(value)
}

pub(in super::super) fn feature_display_name_string_from_plugin_toml(value: &str) -> String {
    feature::feature_display_name_string_from_plugin_toml(value)
}

pub(in super::super) fn feature_owner_plugin_string_from_plugin_toml(value: &str) -> String {
    feature::feature_owner_plugin_string_from_plugin_toml(value)
}

pub(in super::super) fn module_name_string_from_plugin_toml(value: &str) -> String {
    module::module_name_string_from_plugin_toml(value)
}

pub(in super::super) fn module_crate_name_string_from_plugin_toml(value: &str) -> String {
    module::module_crate_name_string_from_plugin_toml(value)
}
