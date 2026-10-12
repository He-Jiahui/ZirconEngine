use super::super::super::super::super::super::values::feature_id_string_from_plugin_toml;

pub(super) fn feature_id_from_plugin_toml(value: &str) -> String {
    feature_id_string_from_plugin_toml(value)
}
