mod display_name;
mod id;
mod owner_plugin;

pub(super) fn feature_id_string_from_plugin_toml(value: &str) -> String {
    id::feature_id_string_from_plugin_toml(value)
}

pub(super) fn feature_display_name_string_from_plugin_toml(value: &str) -> String {
    display_name::feature_display_name_string_from_plugin_toml(value)
}

pub(super) fn feature_owner_plugin_string_from_plugin_toml(value: &str) -> String {
    owner_plugin::feature_owner_plugin_string_from_plugin_toml(value)
}
